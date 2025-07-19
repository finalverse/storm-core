// File: crates/storm-protocol-adapters/src/opensim.rs
// OpenSim/MutSea protocol adapter implementation
// Handles LLUDP protocol for OpenSimulator grids

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::{RwLock, Mutex};
use serde::{Deserialize, Serialize};
use tracing::{info, warn, error, debug};
use anyhow::Result;

use storm_ecs::{World, Entity, Transform, Velocity};
use storm_ai::AIDispatcher;
use storm_networking::{NetworkManager, ConnectionId, ProtocolType, IncomingPacket, OutgoingPacket};
use crate::{ProtocolAdapter, ProtocolMessage, WorldConfig};

/// OpenSim protocol adapter
pub struct OpenSimAdapter {
    ecs_world: Arc<RwLock<World>>,
    ai_dispatcher: Arc<AIDispatcher>,
    connections: Arc<Mutex<HashMap<ConnectionId, OpenSimConnection>>>,
    message_handlers: HashMap<LLUDPMessageType, Box<dyn MessageHandler>>,
}

/// OpenSim connection state
struct OpenSimConnection {
    id: ConnectionId,
    remote_addr: SocketAddr,
    session_id: Option<uuid::Uuid>,
    agent_id: Option<uuid::Uuid>,
    region_id: Option<uuid::Uuid>,
    sequence_number: u32,
    last_ack: u32,
}

/// LLUDP message types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LLUDPMessageType {
    // Login/Session
    UseCircuitCode = 3,
    CompleteAgentMovement = 249,

    // Agent movement
    AgentUpdate = 4,
    AgentAnimation = 20,

    // Object updates
    ObjectUpdate = 12,
    ObjectUpdateCompressed = 13,
    ObjectUpdateCached = 14,

    // Asset requests
    RequestImage = 21,
    ImageData = 22,

    // Chat
    ChatFromViewer = 77,
    ChatFromSimulator = 139,

    // Inventory
    FetchInventory = 73,
    InventoryDescendents = 74,

    // Unknown/Unsupported
    Unknown = 0,
}

/// LLUDP packet structure
#[derive(Debug)]
pub struct LLUDPPacket {
    pub flags: u8,
    pub sequence: u32,
    pub extra_header: Vec<u8>,
    pub message_type: LLUDPMessageType,
    pub payload: Vec<u8>,
}

/// Trait for handling specific LLUDP messages
trait MessageHandler: Send + Sync {
    fn handle_message(
        &self,
        packet: &LLUDPPacket,
        connection: &mut OpenSimConnection,
        world: &mut World,
    ) -> Result<Vec<LLUDPPacket>>;
}

impl OpenSimAdapter {
    pub async fn new(
        ecs_world: Arc<RwLock<World>>,
        ai_dispatcher: Arc<AIDispatcher>,
    ) -> Result<Self> {
        info!("Initializing OpenSim protocol adapter");

        let mut message_handlers: HashMap<LLUDPMessageType, Box<dyn MessageHandler>> = HashMap::new();

        // Register message handlers
        message_handlers.insert(LLUDPMessageType::UseCircuitCode, Box::new(UseCircuitCodeHandler));
        message_handlers.insert(LLUDPMessageType::CompleteAgentMovement, Box::new(CompleteAgentMovementHandler));
        message_handlers.insert(LLUDPMessageType::AgentUpdate, Box::new(AgentUpdateHandler));
        message_handlers.insert(LLUDPMessageType::ObjectUpdate, Box::new(ObjectUpdateHandler));
        message_handlers.insert(LLUDPMessageType::ChatFromViewer, Box::new(ChatHandler));

        Ok(Self {
            ecs_world,
            ai_dispatcher,
            connections: Arc::new(Mutex::new(HashMap::new())),
            message_handlers,
        })
    }

    /// Parse LLUDP packet from raw bytes
    fn parse_packet(&self, data: &[u8]) -> Result<LLUDPPacket> {
        if data.len() < 6 {
            return Err(anyhow::anyhow!("Packet too short"));
        }

        let flags = data[0];
        let sequence = u32::from_be_bytes([data[1], data[2], data[3], data[4]]);

        let extra_header_len = if flags & 0x40 != 0 { data[5] as usize } else { 0 };
        let header_end = 6 + extra_header_len;

        if data.len() < header_end + 1 {
            return Err(anyhow::anyhow!("Invalid packet format"));
        }

        let extra_header = if extra_header_len > 0 {
            data[6..header_end].to_vec()
        } else {
            Vec::new()
        };

        let message_type_byte = data[header_end];
        let message_type = self.message_type_from_byte(message_type_byte);

        let payload = data[header_end + 1..].to_vec();

        Ok(LLUDPPacket {
            flags,
            sequence,
            extra_header,
            message_type,
            payload,
        })
    }

    /// Convert byte to message type
    fn message_type_from_byte(&self, byte: u8) -> LLUDPMessageType {
        match byte {
            3 => LLUDPMessageType::UseCircuitCode,
            4 => LLUDPMessageType::AgentUpdate,
            12 => LLUDPMessageType::ObjectUpdate,
            13 => LLUDPMessageType::ObjectUpdateCompressed,
            20 => LLUDPMessageType::AgentAnimation,
            21 => LLUDPMessageType::RequestImage,
            22 => LLUDPMessageType::ImageData,
            73 => LLUDPMessageType::FetchInventory,
            74 => LLUDPMessageType::InventoryDescendents,
            77 => LLUDPMessageType::ChatFromViewer,
            139 => LLUDPMessageType::ChatFromSimulator,
            249 => LLUDPMessageType::CompleteAgentMovement,
            _ => LLUDPMessageType::Unknown,
        }
    }

    /// Create LLUDP packet bytes
    fn create_packet(&self, packet: &LLUDPPacket) -> Vec<u8> {
        let mut data = Vec::new();

        data.push(packet.flags);
        data.extend_from_slice(&packet.sequence.to_be_bytes());

        if !packet.extra_header.is_empty() {
            data.push(packet.extra_header.len() as u8);
            data.extend_from_slice(&packet.extra_header);
        }

        data.push(packet.message_type as u8);
        data.extend_from_slice(&packet.payload);

        data
    }
}

#[async_trait::async_trait]
impl ProtocolAdapter for OpenSimAdapter {
    async fn connect_to_world(&mut self, config: &WorldConfig) -> Result<ConnectionId> {
        info!("Connecting to OpenSim world: {}", config.name);

        // Parse grid URL to get login server address
        let login_url = url::Url::parse(&config.url)?;
        let host = login_url.host_str().ok_or_else(|| anyhow::anyhow!("Invalid URL"))?;
        let port = login_url.port().unwrap_or(9000);
        let addr: SocketAddr = format!("{}:{}", host, port).parse()?;

        // Create connection (this would normally involve login sequence)
        let connection_id = ConnectionId::new_v4();
        let connection = OpenSimConnection {
            id: connection_id,
            remote_addr: addr,
            session_id: None,
            agent_id: None,
            region_id: None,
            sequence_number: 1,
            last_ack: 0,
        };

        let mut connections = self.connections.lock().await;
        connections.insert(connection_id, connection);

        info!("Connected to OpenSim world with ID: {}", connection_id);
        Ok(connection_id)
    }

    async fn disconnect_from_world(&mut self, connection_id: ConnectionId) -> Result<()> {
        let mut connections = self.connections.lock().await;
        connections.remove(&connection_id);
        info!("Disconnected from OpenSim world: {}", connection_id);
        Ok(())
    }

    async fn disconnect_all(&mut self) -> Result<()> {
        let mut connections = self.connections.lock().await;
        connections.clear();
        info!("Disconnected from all OpenSim worlds");
        Ok(())
    }

    async fn process_pending_messages(&mut self) -> Result<()> {
        // In a real implementation, this would process incoming packets
        // from the network layer and route them to appropriate handlers
        Ok(())
    }

    async fn send_message(&mut self, connection_id: ConnectionId, message: &ProtocolMessage) -> Result<()> {
        debug!("Sending message to OpenSim connection {}: {}", connection_id, message.message_type);
        // Implementation would serialize the message and send via network layer
        Ok(())
    }

    fn protocol_type(&self) -> storm_networking::ProtocolType {
        storm_networking::ProtocolType::LLUDP
    }
}

// Message Handlers

struct UseCircuitCodeHandler;

impl MessageHandler for UseCircuitCodeHandler {
    fn handle_message(
        &self,
        packet: &LLUDPPacket,
        connection: &mut OpenSimConnection,
        _world: &mut World,
    ) -> Result<Vec<LLUDPPacket>> {
        info!("Handling UseCircuitCode message");

        // Parse circuit code from payload
        if packet.payload.len() >= 16 {
            // Extract session and agent IDs from payload
            // This is a simplified implementation
            connection.session_id = Some(uuid::Uuid::new_v4());
            connection.agent_id = Some(uuid::Uuid::new_v4());
        }

        Ok(vec![])
    }
}

struct CompleteAgentMovementHandler;

impl MessageHandler for CompleteAgentMovementHandler {
    fn handle_message(
        &self,
        _packet: &LLUDPPacket,
        connection: &mut OpenSimConnection,
        world: &mut World,
    ) -> Result<Vec<LLUDPPacket>> {
        info!("Agent movement completed for connection: {}", connection.id);

        // Create agent entity in ECS
        if let Some(agent_id) = connection.agent_id {
            let entity = world.create_entity();
            world.add_component(entity, Transform::default());
            world.add_component(entity, Velocity::default());
            world.add_component(entity, OpenSimAgent {
                agent_id,
                session_id: connection.session_id,
                connection_id: connection.id,
            });
        }

        Ok(vec![])
    }
}

struct AgentUpdateHandler;

impl MessageHandler for AgentUpdateHandler {
    fn handle_message(
        &self,
        packet: &LLUDPPacket,
        connection: &mut OpenSimConnection,
        _world: &mut World,
    ) -> Result<Vec<LLUDPPacket>> {
        // Parse agent position/rotation from packet payload
        // Update corresponding entity in ECS

        // This is a simplified implementation
        debug!("Handling agent update for connection: {}", connection.id);

        Ok(vec![])
    }
}

struct ObjectUpdateHandler;

impl MessageHandler for ObjectUpdateHandler {
    fn handle_message(
        &self,
        packet: &LLUDPPacket,
        _connection: &mut OpenSimConnection,
        _world: &mut World,
    ) -> Result<Vec<LLUDPPacket>> {
        debug!("Handling object update, payload size: {}", packet.payload.len());

        // Parse object data and create/update entities in ECS
        // This would involve complex parsing of the ObjectUpdate message format

        Ok(vec![])
    }
}

struct ChatHandler;

impl MessageHandler for ChatHandler {
    fn handle_message(
        &self,
        packet: &LLUDPPacket,
        connection: &mut OpenSimConnection,
        _world: &mut World,
    ) -> Result<Vec<LLUDPPacket>> {
        info!("Handling chat message from connection: {}", connection.id);

        // Parse chat message and potentially process with AI
        // Create response if needed

        Ok(vec![])
    }
}

/// OpenSim agent component
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct OpenSimAgent {
    pub agent_id: uuid::Uuid,
    pub session_id: Option<uuid::Uuid>,
    pub connection_id: ConnectionId,
}

impl storm_ecs::Component for OpenSimAgent {
    fn type_name() -> &'static str {
        "OpenSimAgent"
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}