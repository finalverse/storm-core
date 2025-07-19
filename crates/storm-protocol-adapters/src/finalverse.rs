// File: crates/storm-protocol-adapters/src/finalverse.rs
// Finalverse protocol adapter

use async_trait::async_trait;
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;
use storm_ecs::World;
use storm_ai::AIDispatcher;
use storm_networking::{ConnectionId, ProtocolType as NetworkProtocolType};
use crate::{ProtocolAdapter, ProtocolMessage, WorldConfig};

pub struct FinalverseAdapter {
    ecs_world: Arc<RwLock<World>>,
    ai_dispatcher: Arc<AIDispatcher>,
}

impl FinalverseAdapter {
    pub async fn new(
        ecs_world: Arc<RwLock<World>>,
        ai_dispatcher: Arc<AIDispatcher>,
    ) -> Result<Self> {
        Ok(Self {
            ecs_world,
            ai_dispatcher,
        })
    }
}

#[async_trait]
impl ProtocolAdapter for FinalverseAdapter {
    async fn connect_to_world(&mut self, config: &WorldConfig) -> Result<ConnectionId> {
        // Connect to Finalverse server using WebSocket
        tracing::info!("Connecting to Finalverse world: {}", config.name);
        Ok(uuid::Uuid::new_v4())
    }

    async fn disconnect_from_world(&mut self, connection_id: ConnectionId) -> Result<()> {
        // Disconnect from specific connection
        tracing::info!("Disconnecting from Finalverse connection: {}", connection_id);
        Ok(())
    }

    async fn disconnect_all(&mut self) -> Result<()> {
        // Disconnect from all connections
        tracing::info!("Disconnecting from all Finalverse connections");
        Ok(())
    }

    async fn process_pending_messages(&mut self) -> Result<()> {
        // Process incoming messages from WebSocket connections
        Ok(())
    }

    async fn send_message(&mut self, connection_id: ConnectionId, message: &ProtocolMessage) -> Result<()> {
        // Send message via WebSocket
        tracing::debug!("Sending Finalverse message: {}", message.message_type);
        Ok(())
    }

    fn protocol_type(&self) -> NetworkProtocolType {
        NetworkProtocolType::WebSocket
    }
}