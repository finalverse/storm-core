// File: crates/storm-opensim/src/circuit.rs
// OpenSim circuit management

use uuid::Uuid;

/// Circuit state for LLUDP connection
#[derive(Debug, Clone)]
pub struct Circuit {
    pub code: u32,
    pub session_id: Uuid,
    pub secure_session_id: Uuid,
    pub sequence_in: u32,
    pub sequence_out: u32,
    pub acks_pending: Vec<u32>,
    pub packets_pending: Vec<PendingPacket>,
}

/// Pending packet for resend
#[derive(Debug, Clone)]
pub struct PendingPacket {
    pub sequence: u32,
    pub data: Vec<u8>,
    pub timestamp: std::time::Instant,
    pub resend_count: u32,
}

impl Circuit {
    pub fn new(code: u32, session_id: Uuid, secure_session_id: Uuid) -> Self {
        Self {
            code,
            session_id,
            secure_session_id,
            sequence_in: 0,
            sequence_out: 1,
            acks_pending: Vec::new(),
            packets_pending: Vec::new(),
        }
    }

    pub fn next_sequence(&mut self) -> u32 {
        let seq = self.sequence_out;
        self.sequence_out += 1;
        seq
    }

    pub fn add_ack(&mut self, sequence: u32) {
        self.acks_pending.push(sequence);
    }

    pub fn process_ack(&mut self, sequence: u32) {
        self.packets_pending.retain(|p| p.sequence != sequence);
    }
}
