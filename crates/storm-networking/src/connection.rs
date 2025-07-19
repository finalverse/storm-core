// File: crates/storm-networking/src/connection.rs
// Network connection management

use std::net::SocketAddr;
use crate::ConnectionId;

/// Network connection
pub struct Connection {
    pub id: ConnectionId,
    pub remote_addr: SocketAddr,
    pub state: ConnectionState,
    pub last_activity: std::time::Instant,
}

/// Connection states
#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionState {
    Connecting,
    Connected,
    Disconnecting,
    Disconnected,
}

impl Connection {
    pub fn new(id: ConnectionId, remote_addr: SocketAddr) -> Self {
        Self {
            id,
            remote_addr,
            state: ConnectionState::Connecting,
            last_activity: std::time::Instant::now(),
        }
    }
}
