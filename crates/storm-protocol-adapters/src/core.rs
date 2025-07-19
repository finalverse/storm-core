// File: crates/storm-protocol-adapters/src/core.rs
// Core types and configurations for protocol adapters - NO cyclic dependency

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use anyhow::Result;

/// World configuration for connecting to virtual worlds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldConfig {
    /// Human-readable name of the world
    pub name: String,
    /// Connection URL (login URI for OpenSim, WebSocket endpoint for Finalverse)
    pub url: String,
    /// Protocol type to use for connection
    pub protocol: ProtocolType,
    /// Authentication credentials
    pub credentials: Option<Credentials>,
    /// Connection-specific settings
    pub settings: ConnectionSettings,
}

/// Protocol types supported by StormCore
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ProtocolType {
    /// OpenSim/SecondLife LLUDP protocol
    OpenSim,
    /// Finalverse WebSocket/REST protocol
    Finalverse,
    /// Generic WebSocket for future protocols
    WebSocket,
    /// UDP for real-time protocols
    LLUDP,
}

/// Authentication credentials for world connections
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Credentials {
    /// Username or avatar name
    pub username: String,
    /// Password or authentication token
    pub password: String,
    /// Optional session token for resuming connections
    pub session_token: Option<String>,
    /// User agent string for protocol identification
    pub user_agent: Option<String>,
}

/// Connection-specific settings and preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionSettings {
    /// Maximum connection timeout in seconds
    pub timeout_seconds: u64,
    /// Enable automatic reconnection on disconnect
    pub auto_reconnect: bool,
    /// Maximum number of reconnection attempts
    pub max_reconnect_attempts: u32,
    /// Preferred data compression level (0-9, 0=none, 9=max)
    pub compression_level: u8,
    /// Enable AI-assisted protocol optimization
    pub ai_optimization: bool,
}

impl Default for ConnectionSettings {
    fn default() -> Self {
        Self {
            timeout_seconds: 30,
            auto_reconnect: true,
            max_reconnect_attempts: 3,
            compression_level: 3,
            ai_optimization: true,
        }
    }
}

impl WorldConfig {
    /// Create a new world configuration for OpenSim
    pub fn opensim(name: impl Into<String>, login_uri: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            url: login_uri.into(),
            protocol: ProtocolType::OpenSim,
            credentials: None,
            settings: ConnectionSettings::default(),
        }
    }

    /// Create a new world configuration for Finalverse
    pub fn finalverse(name: impl Into<String>, endpoint: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            url: endpoint.into(),
            protocol: ProtocolType::Finalverse,
            credentials: None,
            settings: ConnectionSettings::default(),
        }
    }

    /// Set authentication credentials
    pub fn with_credentials(mut self, credentials: Credentials) -> Self {
        self.credentials = Some(credentials);
        self
    }

    /// Set connection settings
    pub fn with_settings(mut self, settings: ConnectionSettings) -> Self {
        self.settings = settings;
        self
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<()> {
        if self.name.is_empty() {
            return Err(anyhow::anyhow!("World name cannot be empty"));
        }

        if self.url.is_empty() {
            return Err(anyhow::anyhow!("World URL cannot be empty"));
        }

        // Validate URL format
        url::Url::parse(&self.url)
            .map_err(|e| anyhow::anyhow!("Invalid world URL: {}", e))?;

        Ok(())
    }
}

/// Connection handle for tracking active connections
pub type ConnectionHandle = Uuid;

/// Error types specific to protocol adapters
#[derive(Debug, thiserror::Error)]
pub enum ProtocolError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),

    #[error("Protocol error: {0}")]
    ProtocolError(String),

    #[error("Timeout occurred: {0}")]
    Timeout(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfiguration(String),

    #[error("Unsupported protocol: {0:?}")]
    UnsupportedProtocol(ProtocolType),

    #[error("Network error: {0}")]
    NetworkError(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}

/// Result type for protocol operations
pub type ProtocolResult<T> = Result<T, ProtocolError>;