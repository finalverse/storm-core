// File: crates/storm-digital-human/src/lib.rs
// Description: Core digital human system with AI-driven behaviors
// Provides NPCs with personality, memory, and emotional intelligence

pub mod behavior;
pub mod dialogue;
pub mod emotion;
pub mod memory;
pub mod personality;
pub mod relationships;
pub mod animation;

pub use behavior::*;
pub use dialogue::*;
pub use emotion::*;
pub use memory::*;
pub use personality::*;
pub use relationships::*;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum DigitalHumanError {
    #[error("Behavior error: {0}")]
    BehaviorError(String),

    #[error("Dialogue error: {0}")]
    DialogueError(String),

    #[error("Emotion processing error: {0}")]
    EmotionError(String),

    #[error("Memory error: {0}")]
    MemoryError(String),

    #[error("AI processing error: {0}")]
    AIError(String),
}

pub type Result<T> = std::result::Result<T, DigitalHumanError>;

// Re-export commonly used types
pub mod prelude {
    pub use crate::{
        NPCBehavior, BehaviorState, BehaviorTree,
        DialogueEngine, DialogueContext, ConversationState,
        EmotionalState, Emotion, EmotionalResponse,
        NPCMemory, MemoryType, MemoryImportance,
        PersonalityMatrix, PersonalityTrait, TraitValue,
        RelationshipGraph, Relationship, RelationshipType,
        DigitalHumanError, Result,
    };
}