// File: crates/storm-opensim/src/serialization.rs
// OpenSim message serialization

use anyhow::Result;
use crate::messages::{LLUDPPacket, LLUDPMessageType};

/// Serialize LLUDP packet to bytes
pub fn serialize_packet(packet: &LLUDPPacket) -> Result<Vec<u8>> {
    let mut data = Vec::new();

    data.push(packet.flags);
    data.extend_from_slice(&packet.sequence.to_be_bytes());

    if !packet.extra_header.is_empty() {
        data.push(packet.extra_header.len() as u8);
        data.extend_from_slice(&packet.extra_header);
    }

    data.push(packet.message_type as u8);
    data.extend_from_slice(&packet.payload);

    Ok(data)
}

/// Deserialize bytes to LLUDP packet
pub fn deserialize_packet(data: &[u8]) -> Result<LLUDPPacket> {
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
    let message_type = match message_type_byte {
        3 => LLUDPMessageType::UseCircuitCode,
        4 => LLUDPMessageType::AgentUpdate,
        12 => LLUDPMessageType::ObjectUpdate,
        _ => LLUDPMessageType::Unknown,
    };

    let payload = data[header_end + 1..].to_vec();

    Ok(LLUDPPacket {
        flags,
        sequence,
        extra_header,
        message_type,
        payload,
    })
}
