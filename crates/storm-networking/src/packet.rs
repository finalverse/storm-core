// File: crates/storm-networking/src/packet.rs
// Network packet definitions

use serde::{Deserialize, Serialize};

/// Network packet structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Packet {
    pub header: PacketHeader,
    pub payload: Vec<u8>,
}

/// Packet header
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PacketHeader {
    pub packet_type: PacketType,
    pub sequence: u32,
    pub timestamp: u64,
    pub checksum: u32,
}

/// Packet types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PacketType {
    Handshake,
    Data,
    Ack,
    Heartbeat,
    Disconnect,
}
