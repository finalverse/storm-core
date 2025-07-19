// File: crates/storm-finalverse/src/messages.rs
// Message types and serialization for Finalverse protocol
// Defines all message structures and their validation logic

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use anyhow::Result;

// Re-export main message types from lib.rs for convenience
pub use crate::{FinalverseMessage, EntityUpdate, EntityData};

/// Extended message types for internal protocol handling
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum InternalMessage {
    // Connection management
    ConnectionEstablished { session_id: String, timestamp: i64 },
    ConnectionLost { session_id: String, reason: String },
    Heartbeat { session_id: String, timestamp: i64 },

    // User presence
    UserJoined { user_id: String, avatar_data: AvatarData },
    UserLeft { user_id: String },
    UserStatusUpdate { user_id: String, status: UserStatus },

    // World events
    WeatherChange { weather_type: String, intensity: f32 },
    TimeOfDayUpdate { time: f32, date: String },
    EventNotification { event_id: String, event_type: String, data: serde_json::Value },

    // AI-generated content
    NarrativeEvent { narrative_id: String, content: String, characters: Vec<String> },
    ProceduralContent { content_type: String, content_data: serde_json::Value },

    // System messages
    ServerMaintenance { scheduled_time: i64, duration_minutes: u32 },
    VersionUpdate { required_version: String, download_url: String },
    Error { error_code: i32, message: String, details: Option<String> },
}

/// Avatar data for user representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AvatarData {
    pub avatar_id: String,
    pub display_name: String,
    pub appearance: AppearanceData,
    pub position: [f32; 3],
    pub rotation: [f32; 4],
    pub animation_state: String,
    pub attachments: Vec<AttachmentData>,
}

/// Appearance customization data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppearanceData {
    pub mesh_url: String,
    pub texture_urls: HashMap<String, String>, // slot -> texture URL
    pub material_properties: HashMap<String, f32>,
    pub scale_factors: [f32; 3],
    pub color_tints: HashMap<String, [f32; 4]>, // RGBA color values
}

/// Attachment data for avatar accessories
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttachmentData {
    pub attachment_id: String,
    pub attachment_point: String,
    pub mesh_url: String,
    pub texture_url: Option<String>,
    pub relative_position: [f32; 3],
    pub relative_rotation: [f32; 4],
    pub scale: [f32; 3],
}

/// User status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserStatus {
    pub online: bool,
    pub activity: String,
    pub mood: Option<String>,
    pub custom_status: Option<String>,
    pub last_seen: i64,
}

/// Chat message with extended metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub message_id: String,
    pub sender_id: String,
    pub sender_name: String,
    pub channel: String,
    pub content: String,
    pub timestamp: i64,
    pub message_type: ChatMessageType,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Types of chat messages
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChatMessageType {
    /// Regular user message
    Normal,
    /// System announcement
    System,
    /// Whisper/private message
    Whisper,
    /// Emote/action message
    Emote,
    /// AI-generated narrative text
    Narrative,
    /// Out-of-character message
    Ooc,
}

/// Asset reference with caching information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetReference {
    pub asset_id: String,
    pub asset_type: AssetType,
    pub url: String,
    pub checksum: String,
    pub size_bytes: u64,
    pub cache_policy: CachePolicy,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Types of assets in Finalverse
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssetType {
    /// 3D mesh/model file
    Mesh,
    /// Texture image
    Texture,
    /// Audio file
    Audio,
    /// Animation data
    Animation,
    /// Script/behavior code
    Script,
    /// Particle effect definition
    ParticleEffect,
    /// Material definition
    Material,
}

/// Cache policy for assets
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CachePolicy {
    /// Never cache, always download
    NoCache,
    /// Cache for session only
    SessionOnly,
    /// Cache temporarily with TTL
    Temporary { ttl_seconds: u32 },
    /// Cache permanently until version changes
    Persistent,
}

/// Physics update message for real-time synchronization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysicsUpdate {
    pub entity_id: String,
    pub timestamp: i64,
    pub position: [f32; 3],
    pub velocity: [f32; 3],
    pub angular_velocity: [f32; 3],
    pub rotation: [f32; 4], // Quaternion
    pub collision_data: Option<CollisionData>,
}

/// Collision event data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollisionData {
    pub collision_id: String,
    pub other_entity_id: String,
    pub contact_point: [f32; 3],
    pub contact_normal: [f32; 3],
    pub impact_force: f32,
}

/// Audio event for spatial audio system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioEvent {
    pub audio_id: String,
    pub source_entity: Option<String>,
    pub audio_type: AudioType,
    pub position: [f32; 3],
    pub volume: f32,
    pub pitch: f32,
    pub loop_count: i32, // -1 for infinite loop
    pub fade_in_duration: f32,
    pub fade_out_duration: f32,
}

/// Types of audio events
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AudioType {
    /// Environmental background sound
    Ambient,
    /// Sound effect triggered by action
    Effect,
    /// Music track
    Music,
    /// Voice/speech audio
    Voice,
    /// User interface sound
    Ui,
}

/// Lighting update for dynamic environment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LightingUpdate {
    pub light_id: String,
    pub light_type: LightType,
    pub position: [f32; 3],
    pub direction: [f32; 3],
    pub color: [f32; 3], // RGB
    pub intensity: f32,
    pub range: f32,
    pub spotlight_angle: Option<f32>,
    pub shadows_enabled: bool,
}

/// Types of lights in the scene
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LightType {
    /// Directional light (like sun)
    Directional,
    /// Point light (omnidirectional)
    Point,
    /// Spot light with cone
    Spot,
    /// Area light with defined shape
    Area,
}

// Extended implementations unique to messages.rs (not duplicating lib.rs)

impl EntityUpdate {
    /// Create a position-only update
    pub fn position_update(entity_id: String, position: [f32; 3]) -> Self {
        Self {
            entity_id,
            position: Some(position),
            rotation: None,
            scale: None,
            properties: HashMap::new(),
        }
    }

    /// Create a transform update (position, rotation, scale)
    pub fn transform_update(
        entity_id: String,
        position: [f32; 3],
        rotation: [f32; 4],
        scale: [f32; 3],
    ) -> Self {
        Self {
            entity_id,
            position: Some(position),
            rotation: Some(rotation),
            scale: Some(scale),
            properties: HashMap::new(),
        }
    }

    /// Create a property-only update
    pub fn property_update(
        entity_id: String,
        properties: HashMap<String, serde_json::Value>,
    ) -> Self {
        Self {
            entity_id,
            position: None,
            rotation: None,
            scale: None,
            properties,
        }
    }
}

impl EntityData {
    /// Create a basic entity with minimal data
    pub fn basic(id: String, name: String, position: [f32; 3]) -> Self {
        Self {
            id,
            name,
            entity_type: "basic".to_string(),
            position,
            rotation: [0.0, 0.0, 0.0, 1.0], // Identity quaternion
            scale: [1.0, 1.0, 1.0],
            mesh_url: None,
            texture_urls: Vec::new(),
            properties: HashMap::new(),
        }
    }

    /// Convert to EntityUpdate for sending changes
    pub fn to_update(&self) -> EntityUpdate {
        EntityUpdate {
            entity_id: self.id.clone(),
            position: Some(self.position),
            rotation: Some(self.rotation),
            scale: Some(self.scale),
            properties: self.properties.clone(),
        }
    }
}

impl FinalverseMessage {
    /// Get message type as string for logging/debugging
    pub fn message_type_str(&self) -> &'static str {
        match self {
            FinalverseMessage::Login { .. } => "Login",
            FinalverseMessage::LoginResponse { .. } => "LoginResponse",
            FinalverseMessage::WorldUpdate { .. } => "WorldUpdate",
            FinalverseMessage::EntitySpawn { .. } => "EntitySpawn",
            FinalverseMessage::EntityDespawn { .. } => "EntityDespawn",
            FinalverseMessage::Movement { .. } => "Movement",
            FinalverseMessage::Chat { .. } => "Chat",
            FinalverseMessage::Interaction { .. } => "Interaction",
            FinalverseMessage::AiRequest { .. } => "AiRequest",
            FinalverseMessage::AiResponse { .. } => "AiResponse",
        }
    }

    /// Check if message requires authentication
    pub fn requires_auth(&self) -> bool {
        match self {
            FinalverseMessage::Login { .. } => false,
            FinalverseMessage::LoginResponse { .. } => false,
            _ => true,
        }
    }

    /// Get priority level for message processing (higher = more important)
    pub fn priority(&self) -> u8 {
        match self {
            FinalverseMessage::Login { .. } => 10,
            FinalverseMessage::LoginResponse { .. } => 10,
            FinalverseMessage::Movement { .. } => 8,
            FinalverseMessage::Chat { .. } => 6,
            FinalverseMessage::WorldUpdate { .. } => 7,
            FinalverseMessage::EntitySpawn { .. } => 7,
            FinalverseMessage::EntityDespawn { .. } => 7,
            FinalverseMessage::Interaction { .. } => 5,
            FinalverseMessage::AiRequest { .. } => 4,
            FinalverseMessage::AiResponse { .. } => 4,
        }
    }
}

/// Message serialization utilities
pub mod serialization {
    use super::*;

    /// Serialize a message to JSON bytes
    pub fn serialize_message(message: &FinalverseMessage) -> Result<Vec<u8>> {
        serde_json::to_vec(message)
            .map_err(|e| anyhow::anyhow!("Failed to serialize message: {}", e))
    }

    /// Deserialize a message from JSON bytes
    pub fn deserialize_message(data: &[u8]) -> Result<FinalverseMessage> {
        serde_json::from_slice(data)
            .map_err(|e| anyhow::anyhow!("Failed to deserialize message: {}", e))
    }

    /// Serialize a message to JSON string
    pub fn serialize_message_string(message: &FinalverseMessage) -> Result<String> {
        serde_json::to_string(message)
            .map_err(|e| anyhow::anyhow!("Failed to serialize message to string: {}", e))
    }

    /// Deserialize a message from JSON string
    pub fn deserialize_message_string(data: &str) -> Result<FinalverseMessage> {
        serde_json::from_str(data)
            .map_err(|e| anyhow::anyhow!("Failed to deserialize message from string: {}", e))
    }

    /// Pretty-print a message for debugging
    pub fn pretty_print_message(message: &FinalverseMessage) -> Result<String> {
        serde_json::to_string_pretty(message)
            .map_err(|e| anyhow::anyhow!("Failed to pretty-print message: {}", e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entity_update_creation() {
        let update = EntityUpdate::position_update("entity1".to_string(), [1.0, 2.0, 3.0]);
        assert_eq!(update.entity_id, "entity1");
        assert_eq!(update.position, Some([1.0, 2.0, 3.0]));
        assert_eq!(update.rotation, None);
        // Note: has_changes() is tested in lib.rs to avoid duplication
    }

    #[test]
    fn test_message_priority() {
        let login = FinalverseMessage::Login {
            username: "test".to_string(),
            token: "token".to_string(),
        };
        let chat = FinalverseMessage::Chat {
            message: "hello".to_string(),
            channel: "general".to_string(),
        };

        assert!(login.priority() > chat.priority());
    }

    #[test]
    fn test_serialization() {
        let message = FinalverseMessage::Chat {
            message: "Hello, world!".to_string(),
            channel: "general".to_string(),
        };

        let serialized = serialization::serialize_message(&message).unwrap();
        let deserialized = serialization::deserialize_message(&serialized).unwrap();

        match deserialized {
            FinalverseMessage::Chat { message: msg, channel: ch } => {
                assert_eq!(msg, "Hello, world!");
                assert_eq!(ch, "general");
            }
            _ => panic!("Wrong message type after deserialization"),
        }
    }

    #[test]
    fn test_entity_data_creation() {
        let entity = EntityData::basic("test_id".to_string(), "Test Entity".to_string(), [1.0, 2.0, 3.0]);
        assert_eq!(entity.id, "test_id");
        assert_eq!(entity.name, "Test Entity");
        assert_eq!(entity.position, [1.0, 2.0, 3.0]);
        assert_eq!(entity.rotation, [0.0, 0.0, 0.0, 1.0]); // Identity quaternion
    }

    #[test]
    fn test_message_type_str() {
        let chat = FinalverseMessage::Chat {
            message: "test".to_string(),
            channel: "general".to_string(),
        };
        assert_eq!(chat.message_type_str(), "Chat");

        let login = FinalverseMessage::Login {
            username: "user".to_string(),
            token: "token".to_string(),
        };
        assert_eq!(login.message_type_str(), "Login");
    }

    #[test]
    fn test_message_auth_requirements() {
        let login = FinalverseMessage::Login {
            username: "user".to_string(),
            token: "token".to_string(),
        };
        assert!(!login.requires_auth());

        let chat = FinalverseMessage::Chat {
            message: "test".to_string(),
            channel: "general".to_string(),
        };
        assert!(chat.requires_auth());
    }
}