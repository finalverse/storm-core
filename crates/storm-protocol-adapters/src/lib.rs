// File: crates/storm-protocol-adapters/src/lib.rs
// Protocol adapters for virtual world protocols
// Bridges between network layer and ECS world state

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use tracing::{info, warn, error};
use anyhow::Result;

use storm_ecs::{World, Entity, Component};
use storm_ai::AIDispatcher;
use storm_networking::{NetworkManager, ConnectionId, ProtocolType as NetworkProtocolType, PacketHandler, IncomingPacket, OutgoingPacket};

pub mod opensim;
pub mod finalverse;
pub mod base;
pub mod core;

pub use base::*;
pub use core::*;

#[cfg(feature = "opensim")]
pub use opensim::*;

#[cfg(feature = "finalverse")]
pub use finalverse::*;

/// Protocol router - manages all protocol adapters
pub struct ProtocolRouter {
    adapters: HashMap<NetworkProtocolType, Box<dyn ProtocolAdapter>>,
    ecs_world: Arc<RwLock<World>>,
    ai_dispatcher: Arc<AIDispatcher>,
    active_connections: HashMap<ConnectionId, NetworkProtocolType>,
}

impl ProtocolRouter {
    pub async fn new(
        ecs_world: Arc<RwLock<World>>,
        ai_dispatcher: Arc<AIDispatcher>,
    ) -> Result<Self> {
        info!("Initializing protocol router");

        let mut adapters: HashMap<NetworkProtocolType, Box<dyn ProtocolAdapter>> = HashMap::new();

        // Initialize OpenSim adapter
        #[cfg(feature = "opensim")]
        {
            let opensim_adapter = opensim::OpenSimAdapter::new(
                ecs_world.clone(),
                ai_dispatcher.clone(),
            ).await?;
            adapters.insert(NetworkProtocolType::LLUDP, Box::new(opensim_adapter));
        }

        // Initialize Finalverse adapter
        #[cfg(feature = "finalverse")]
        {
            let finalverse_adapter = finalverse::FinalverseAdapter::new(
                ecs_world.clone(),
                ai_dispatcher.clone(),
            ).await?;
            adapters.insert(NetworkProtocolType::WebSocket, Box::new(finalverse_adapter));
        }

        Ok(Self {
            adapters,
            ecs_world,
            ai_dispatcher,
            active_connections: HashMap::new(),
        })
    }

    /// Convert core ProtocolType to network ProtocolType
    fn map_protocol_type(protocol: ProtocolType) -> NetworkProtocolType {
        match protocol {
            ProtocolType::OpenSim | ProtocolType::LLUDP => NetworkProtocolType::LLUDP,
            ProtocolType::Finalverse | ProtocolType::WebSocket => NetworkProtocolType::WebSocket,
        }
    }

    /// Connect to a world using the appropriate protocol adapter
    pub async fn connect_world(&mut self, world_config: &WorldConfig) -> Result<ConnectionId> {
        let network_protocol = Self::map_protocol_type(world_config.protocol);

        if let Some(adapter) = self.adapters.get_mut(&network_protocol) {
            let connection_id = adapter.connect_to_world(world_config).await?;
            self.active_connections.insert(connection_id, network_protocol);
            info!("Connected to world {} via {:?}", world_config.name, network_protocol);
            Ok(connection_id)
        } else {
            Err(anyhow::anyhow!("No adapter available for protocol: {:?}", network_protocol))
        }
    }

    /// Process incoming messages from all adapters
    pub async fn process_messages(&mut self) -> Result<()> {
        for adapter in self.adapters.values_mut() {
            adapter.process_pending_messages().await?;
        }
        Ok(())
    }

    /// Disconnect from all worlds
    pub async fn disconnect_all(&mut self) -> Result<()> {
        for adapter in self.adapters.values_mut() {
            adapter.disconnect_all().await?;
        }
        self.active_connections.clear();
        Ok(())
    }

    /// Send a message to a specific connection
    pub async fn send_message(&mut self, connection_id: ConnectionId, message: &ProtocolMessage) -> Result<()> {
        if let Some(&protocol) = self.active_connections.get(&connection_id) {
            if let Some(adapter) = self.adapters.get_mut(&protocol) {
                adapter.send_message(connection_id, message).await?;
            } else {
                return Err(anyhow::anyhow!("No adapter found for connection"));
            }
        } else {
            return Err(anyhow::anyhow!("Connection not found: {}", connection_id));
        }
        Ok(())
    }

    /// Get active connections count
    pub fn active_connections_count(&self) -> usize {
        self.active_connections.len()
    }

    /// Get connections by protocol type
    pub fn get_connections_by_protocol(&self, protocol: NetworkProtocolType) -> Vec<ConnectionId> {
        self.active_connections
            .iter()
            .filter_map(|(id, &proto)| if proto == protocol { Some(*id) } else { None })
            .collect()
    }
}

/// Generic protocol message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolMessage {
    pub message_type: String,
    pub data: serde_json::Value,
    pub timestamp: u64,
}

impl ProtocolMessage {
    /// Create a new protocol message
    pub fn new(message_type: impl Into<String>, data: serde_json::Value) -> Self {
        Self {
            message_type: message_type.into(),
            data,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        }
    }

    /// Create a simple text message
    pub fn text(message_type: impl Into<String>, text: impl Into<String>) -> Self {
        Self::new(message_type, serde_json::Value::String(text.into()))
    }

    /// Create a JSON message
    pub fn json(message_type: impl Into<String>, json: impl serde::Serialize) -> Result<Self> {
        let data = serde_json::to_value(json)?;
        Ok(Self::new(message_type, data))
    }
}