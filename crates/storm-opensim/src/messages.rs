// File: crates/storm-opensim/src/messages.rs
// OpenSim LLUDP message definitions

use serde::{Deserialize, Serialize};

/// LLUDP message types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LLUDPMessageType {
    UseCircuitCode = 3,
    CompleteAgentMovement = 249,
    AgentUpdate = 4,
    AgentAnimation = 20,
    ObjectUpdate = 12,
    ObjectUpdateCompressed = 13,
    ObjectUpdateCached = 14,
    RequestImage = 21,
    ImageData = 22,
    ChatFromViewer = 77,
    ChatFromSimulator = 139,
    FetchInventory = 73,
    InventoryDescendents = 74,
    Unknown = 0,
}

/// LLUDP packet structure
#[derive(Debug, Clone)]
pub struct LLUDPPacket {
    pub flags: u8,
    pub sequence: u32,
    pub extra_header: Vec<u8>,
    pub message_type: LLUDPMessageType,
    pub payload: Vec<u8>,
}

/// Grid information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GridInfo {
    pub name: String,
    pub login_uri: String,
    pub welcome_message: String,
    pub economy: String,
}

/// Agent state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentState {
    pub agent_id: uuid::Uuid,
    pub session_id: uuid::Uuid,
    pub position: [f32; 3],
    pub look_at: [f32; 3],
    pub region_id: uuid::Uuid,
}

/// Asset information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetInfo {
    pub id: uuid::Uuid,
    pub asset_type: u8,
    pub name: String,
    pub description: String,
    pub data: Vec<u8>,
}

/// Inventory item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryItem {
    pub id: uuid::Uuid,
    pub parent_id: uuid::Uuid,
    pub name: String,
    pub description: String,
    pub asset_id: uuid::Uuid,
    pub item_type: u32,
}
