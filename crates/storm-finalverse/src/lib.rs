// File: crates/storm-finalverse/src/lib.rs
// Finalverse protocol implementation
// WebSocket + REST API for next-generation virtual worlds

pub mod websocket;
pub mod rest_api;
pub mod messages;
pub mod auth;

// Re-export main types for easy access
pub use websocket::{FinalverseWebSocket, WebSocketConfig, ConnectionState};
pub use rest_api::{FinalverseRestClient, RestApiConfig};
pub use messages::{InternalMessage, ChatMessage, AssetReference, PhysicsUpdate, AudioEvent, LightingUpdate};
pub use auth::{FinalverseAuth, AuthConfig, Session, UserInfo, Permission, AccountStatus, AuthResponse};

/// Finalverse protocol version
pub const FINALVERSE_PROTOCOL_VERSION: &str = "1.0.0";

/// Finalverse message types
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum FinalverseMessage {
    // Authentication
    Login { username: String, token: String },
    LoginResponse { success: bool, session_id: Option<String> },

    // World state
    WorldUpdate { entities: Vec<EntityUpdate> },
    EntitySpawn { entity: EntityData },
    EntityDespawn { entity_id: String },

    // User actions
    Movement { position: [f32; 3], rotation: [f32; 4] },
    Chat { message: String, channel: String },
    Interaction { target_id: String, action: String },

    // AI integration
    AiRequest { request_id: String, prompt: String },
    AiResponse { request_id: String, response: String },
}

/// Entity update data
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EntityUpdate {
    pub entity_id: String,
    pub position: Option<[f32; 3]>,
    pub rotation: Option<[f32; 4]>,
    pub scale: Option<[f32; 3]>,
    pub properties: std::collections::HashMap<String, serde_json::Value>,
}

/// Complete entity data
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EntityData {
    pub id: String,
    pub name: String,
    pub entity_type: String,
    pub position: [f32; 3],
    pub rotation: [f32; 4],
    pub scale: [f32; 3],
    pub mesh_url: Option<String>,
    pub texture_urls: Vec<String>,
    pub properties: std::collections::HashMap<String, serde_json::Value>,
}

/// Finalverse protocol error types
#[derive(Debug, thiserror::Error)]
pub enum FinalverseError {
    #[error("WebSocket error: {0}")]
    WebSocketError(#[from] tokio_tungstenite::tungstenite::Error),

    #[error("HTTP error: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Authentication error: {0}")]
    AuthenticationError(String),

    #[error("Protocol error: {0}")]
    ProtocolError(String),

    #[error("Connection error: {0}")]
    ConnectionError(String),

    #[error("Timeout error: {0}")]
    TimeoutError(String),
}

/// Result type for Finalverse operations
pub type FinalverseResult<T> = Result<T, FinalverseError>;

/// Finalverse protocol client combining WebSocket and REST functionality
pub struct FinalverseClient {
    /// WebSocket manager for real-time communication
    pub websocket: FinalverseWebSocket,
    /// REST client for API calls
    pub rest_client: FinalverseRestClient,
    /// Authentication manager
    pub auth: FinalverseAuth,
    /// Current session information
    pub current_session: Option<Session>,
}

impl FinalverseClient {
    /// Create a new Finalverse client
    pub fn new(server_url: &str) -> FinalverseResult<Self> {
        let websocket = FinalverseWebSocket::new()
            .map_err(|e| FinalverseError::ConnectionError(e.to_string()))?;

        let rest_client = FinalverseRestClient::new(server_url)
            .map_err(|e| FinalverseError::ConnectionError(e.to_string()))?;

        let auth = FinalverseAuth::new();

        Ok(Self {
            websocket,
            rest_client,
            auth,
            current_session: None,
        })
    }

    /// Authenticate and establish connection
    pub async fn connect(&mut self, username: &str, password: &str) -> FinalverseResult<()> {
        // Authenticate via REST API first
        let auth_response = self.rest_client.authenticate(username, password).await
            .map_err(|e| FinalverseError::AuthenticationError(e.to_string()))?;

        if !auth_response.success {
            return Err(FinalverseError::AuthenticationError(
                auth_response.error.unwrap_or_else(|| "Authentication failed".to_string())
            ));
        }

        // Extract session information
        let session_id = auth_response.session_id
            .ok_or_else(|| FinalverseError::AuthenticationError("No session ID returned".to_string()))?;

        // Connect WebSocket with session
        let ws_url = format!("wss://ws.finalverse.com/connect?session={}", session_id);
        self.websocket.connect(&ws_url, session_id.clone()).await
            .map_err(|e| FinalverseError::ConnectionError(e.to_string()))?;

        // Start WebSocket ping loop
        self.websocket.start_ping_loop().await
            .map_err(|e| FinalverseError::ConnectionError(e.to_string()))?;

        println!("Successfully connected to Finalverse!");
        Ok(())
    }

    /// Disconnect from Finalverse
    pub async fn disconnect(&mut self) -> FinalverseResult<()> {
        self.websocket.disconnect_all().await
            .map_err(|e| FinalverseError::ConnectionError(e.to_string()))?;

        self.rest_client.clear_auth_token();
        self.current_session = None;

        println!("Disconnected from Finalverse");
        Ok(())
    }

    /// Send a chat message
    pub async fn send_chat(&self, message: &str, channel: &str) -> FinalverseResult<()> {
        let chat_msg = FinalverseMessage::Chat {
            message: message.to_string(),
            channel: channel.to_string(),
        };

        // Validate message
        chat_msg.validate()
            .map_err(|e| FinalverseError::ProtocolError(e.to_string()))?;

        // Send via WebSocket if available, otherwise queue
        let session_ids = self.websocket.get_session_ids().await;
        if let Some(session_id) = session_ids.first() {
            self.websocket.send_message(session_id, &chat_msg).await
                .map_err(|e| FinalverseError::ConnectionError(e.to_string()))?;
        } else {
            return Err(FinalverseError::ConnectionError("No active WebSocket connection".to_string()));
        }

        Ok(())
    }

    /// Update avatar position
    pub async fn update_position(&self, position: [f32; 3], rotation: [f32; 4]) -> FinalverseResult<()> {
        let movement_msg = FinalverseMessage::Movement { position, rotation };

        // Validate movement data
        movement_msg.validate()
            .map_err(|e| FinalverseError::ProtocolError(e.to_string()))?;

        // Send via WebSocket
        let session_ids = self.websocket.get_session_ids().await;
        if let Some(session_id) = session_ids.first() {
            self.websocket.send_message(session_id, &movement_msg).await
                .map_err(|e| FinalverseError::ConnectionError(e.to_string()))?;
        } else {
            return Err(FinalverseError::ConnectionError("No active WebSocket connection".to_string()));
        }

        Ok(())
    }

    /// Get world information
    pub async fn get_world_info(&self, world_id: &str) -> FinalverseResult<rest_api::WorldInfo> {
        self.rest_client.get_world_info(world_id).await
            .map_err(|e| FinalverseError::ProtocolError(e.to_string()))
    }

    /// Check connection status
    pub async fn is_connected(&self) -> bool {
        self.websocket.active_connections_count().await > 0 &&
            self.rest_client.is_authenticated()
    }
}

// Implementation for message validation
impl FinalverseMessage {
    pub fn validate(&self) -> anyhow::Result<()> {
        match self {
            FinalverseMessage::Login { username, token } => {
                if username.is_empty() {
                    return Err(anyhow::anyhow!("Username cannot be empty"));
                }
                if token.is_empty() {
                    return Err(anyhow::anyhow!("Token cannot be empty"));
                }
            }
            FinalverseMessage::Movement { position, rotation } => {
                if position.iter().any(|&x| x.is_nan() || x.is_infinite()) {
                    return Err(anyhow::anyhow!("Invalid position values"));
                }
                if rotation.iter().any(|&x| x.is_nan() || x.is_infinite()) {
                    return Err(anyhow::anyhow!("Invalid rotation values"));
                }
            }
            FinalverseMessage::Chat { message, channel } => {
                if message.is_empty() {
                    return Err(anyhow::anyhow!("Chat message cannot be empty"));
                }
                if channel.is_empty() {
                    return Err(anyhow::anyhow!("Chat channel cannot be empty"));
                }
            }
            _ => {} // Other variants are valid by default
        }
        Ok(())
    }
}

// Implementation for EntityUpdate
impl EntityUpdate {
    pub fn validate(&self) -> anyhow::Result<()> {
        if self.entity_id.is_empty() {
            return Err(anyhow::anyhow!("Entity ID cannot be empty"));
        }

        if let Some(position) = &self.position {
            if position.iter().any(|&x| x.is_nan() || x.is_infinite()) {
                return Err(anyhow::anyhow!("Invalid position values"));
            }
        }

        Ok(())
    }

    pub fn has_changes(&self) -> bool {
        self.position.is_some() ||
            self.rotation.is_some() ||
            self.scale.is_some() ||
            !self.properties.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_finalverse_message_serialization() {
        let msg = FinalverseMessage::Chat {
            message: "Hello, world!".to_string(),
            channel: "general".to_string(),
        };

        let json = serde_json::to_string(&msg).unwrap();
        let deserialized: FinalverseMessage = serde_json::from_str(&json).unwrap();

        match deserialized {
            FinalverseMessage::Chat { message, channel } => {
                assert_eq!(message, "Hello, world!");
                assert_eq!(channel, "general");
            }
            _ => panic!("Wrong message type"),
        }
    }

    #[test]
    fn test_entity_update_validation() {
        let mut update = EntityUpdate {
            entity_id: "test".to_string(),
            position: Some([1.0, 2.0, 3.0]),
            rotation: None,
            scale: None,
            properties: std::collections::HashMap::new(),
        };

        assert!(update.validate().is_ok());
        assert!(update.has_changes());

        // Test invalid position
        update.position = Some([f32::NAN, 2.0, 3.0]);
        assert!(update.validate().is_err());
    }

    #[tokio::test]
    async fn test_client_creation() {
        let result = FinalverseClient::new("https://api.finalverse.example.com");
        assert!(result.is_ok());

        let client = result.unwrap();
        assert!(!client.is_connected().await);
    }
}