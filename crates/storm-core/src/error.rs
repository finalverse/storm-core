// File: crates/storm-core/src/error.rs
// Comprehensive error handling for StormCore engine

use thiserror::Error;

/// StormCore result type alias
pub type StormResult<T> = Result<T, StormError>;

/// Comprehensive error types for StormCore engine
#[derive(Error, Debug)]
pub enum StormError {
    #[error("Initialization failed: {0}")]
    InitializationError(String),

    #[error("Configuration error: {0}")]
    ConfigurationError(String),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Protocol error: {0}")]
    ProtocolError(String),

    #[error("ECS error: {0}")]
    EcsError(String),

    #[error("AI dispatcher error: {0}")]
    AiError(String),

    #[error("Rendering error: {0}")]
    RenderingError(String),

    #[error("Audio error: {0}")]
    AudioError(String),

    #[error("Physics error: {0}")]
    PhysicsError(String),

    #[error("Asset loading error: {0}")]
    AssetError(String),

    #[error("Platform not supported: {0}")]
    PlatformNotSupported(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Task join error: {0}")]
    TaskJoinError(#[from] tokio::task::JoinError),

    #[error("Generic error: {0}")]
    Generic(String),
}

impl From<anyhow::Error> for StormError {
    fn from(err: anyhow::Error) -> Self {
        StormError::Generic(err.to_string())
    }
}

impl From<&str> for StormError {
    fn from(err: &str) -> Self {
        StormError::Generic(err.to_string())
    }
}

impl From<String> for StormError {
    fn from(err: String) -> Self {
        StormError::Generic(err)
    }
}