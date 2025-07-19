// File: crates/storm-ai/src/models.rs
// AI models and data structures

use serde::{Deserialize, Serialize};

/// AI model configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    pub name: String,
    pub version: String,
    pub model_type: ModelType,
    pub capabilities: Vec<String>,
}

/// Types of AI models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModelType {
    LanguageModel,
    VisionModel,
    AudioModel,
    PathfindingModel,
    BehaviorModel,
}

impl Default for ModelConfig {
    fn default() -> Self {
        Self {
            name: "default".to_string(),
            version: "1.0".to_string(),
            model_type: ModelType::LanguageModel,
            capabilities: vec!["basic".to_string()],
        }
    }
}
