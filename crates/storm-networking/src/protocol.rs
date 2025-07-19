// File: crates/storm-networking/src/protocol.rs
// Protocol definitions

/// Network protocols
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Protocol {
    TCP,
    UDP,
    WebSocket,
    QUIC,
}

/// Protocol handler trait
pub trait ProtocolHandler: Send + Sync {
    fn handle_data(&self, data: &[u8]) -> anyhow::Result<Vec<u8>>;
    fn protocol_type(&self) -> Protocol;
}
