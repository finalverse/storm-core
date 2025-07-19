// File: crates/storm-protocol-adapters/src/base.rs
// Base protocol adapter traits and implementations

use async_trait::async_trait;
use anyhow::Result;
use storm_networking::{ConnectionId, ProtocolType as NetworkProtocolType};
use crate::{ProtocolMessage, WorldConfig};

/// Base protocol adapter trait
/// All protocol adapters must implement this trait for consistent interface
#[async_trait]
pub trait ProtocolAdapter: Send + Sync {
    /// Connect to a world using the provided configuration
    async fn connect_to_world(&mut self, config: &WorldConfig) -> Result<ConnectionId>;

    /// Disconnect from a specific world connection
    async fn disconnect_from_world(&mut self, connection_id: ConnectionId) -> Result<()>;

    /// Disconnect from all active connections
    async fn disconnect_all(&mut self) -> Result<()>;

    /// Process pending messages from all connections
    async fn process_pending_messages(&mut self) -> Result<()>;

    /// Send a message to a specific connection
    async fn send_message(&mut self, connection_id: ConnectionId, message: &ProtocolMessage) -> Result<()>;

    /// Get the protocol type this adapter handles (network layer type)
    fn protocol_type(&self) -> NetworkProtocolType;
}