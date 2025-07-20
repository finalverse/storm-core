// File: storm-core/crates/storm-avatar/src/lib.rs
// Description: Core avatar system with modular ECS components
// Provides foundation for customizable avatars in virtual worlds

pub mod archetype;
pub mod components;
pub mod customization;
pub mod generation;
pub mod resonance;
pub mod serialization;
pub mod systems;

pub use archetype::*;
pub use components::*;
pub use customization::*;
pub use generation::*;
pub use resonance::*;
pub use serialization::*;
pub use systems::*;

use storm_ecs::prelude::*;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AvatarError {
    #[error("Invalid avatar data: {0}")]
    InvalidData(String),

    #[error("Customization failed: {0}")]
    CustomizationFailed(String),

    #[error("Generation failed: {0}")]
    GenerationFailed(String),

    #[error("Asset not found: {0}")]
    AssetNotFound(String),

    #[error("AI processing failed: {0}")]
    AIProcessingFailed(String),
}

pub type Result<T> = std::result::Result<T, AvatarError>;

// Re-export commonly used types
pub mod prelude {
    pub use crate::{
        AvatarBase, AvatarArchetype, AvatarError, Result,
        CustomizationData, MorphTarget, TextureLayer,
        SongResonance, EchoType, PersonalityAspect,
        AvatarGenerator, CustomizationEngine,
    };
}