// File: storm-core/crates/storm-avatar/src/customization.rs
// Description: Avatar customization system implementation
// Handles morph targets, textures, and procedural modifications

use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use uuid::Uuid;
use storm_math::{Vec2, Vec3};
use storm_ecs::prelude::*;

/// Main customization data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomizationData {
    pub morph_targets: HashMap<String, MorphTarget>,
    pub texture_layers: Vec<TextureLayer>,
    pub particle_effects: Vec<ParticleEffect>,
    pub accessories: Vec<AccessorySlot>,
    pub procedural_mods: Vec<ProceduralModification>,
    pub appearance_modifiers: Vec<AppearanceModifier>,
}

impl Component for CustomizationData {
    fn type_name() -> &'static str {
        "CustomizationData"
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl Default for CustomizationData {
    fn default() -> Self {
        Self {
            morph_targets: Self::default_morph_targets(),
            texture_layers: vec![],
            particle_effects: vec![],
            accessories: vec![],
            procedural_mods: vec![],
            appearance_modifiers: vec![],
        }
    }
}

impl CustomizationData {
    fn default_morph_targets() -> HashMap<String, MorphTarget> {
        let mut targets = HashMap::new();

        // Face morphs
        targets.insert("face_width".to_string(), MorphTarget::new(0.0, -1.0, 1.0));
        targets.insert("jaw_width".to_string(), MorphTarget::new(0.0, -1.0, 1.0));
        targets.insert("eye_size".to_string(), MorphTarget::new(0.0, -0.5, 0.5));
        targets.insert("nose_length".to_string(), MorphTarget::new(0.0, -0.5, 0.5));
        targets.insert("mouth_width".to_string(), MorphTarget::new(0.0, -0.5, 0.5));

        // Body morphs
        targets.insert("height".to_string(), MorphTarget::new(0.0, -0.3, 0.3));
        targets.insert("muscle_definition".to_string(), MorphTarget::new(0.0, -1.0, 1.0));
        targets.insert("body_weight".to_string(), MorphTarget::new(0.0, -1.0, 1.0));

        targets
    }

    /// Get mutable access to appearance modifiers
    pub fn appearance_modifiers_mut(&mut self) -> &mut Vec<AppearanceModifier> {
        &mut self.appearance_modifiers
    }

    /// Add an appearance modifier
    pub fn add_appearance_modifier(&mut self, modifier: AppearanceModifier) {
        self.appearance_modifiers.push(modifier);
    }
}

// Re-export AppearanceModifier from archetype module
pub use crate::archetype::{AppearanceModifier, ModifierType, BodyRegion};

/// Morph target for shape modifications
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MorphTarget {
    pub value: f32,
    pub min: f32,
    pub max: f32,
    pub affected_bones: Vec<String>,
}

impl MorphTarget {
    pub fn new(value: f32, min: f32, max: f32) -> Self {
        Self {
            value: value.clamp(min, max),
            min,
            max,
            affected_bones: vec![],
        }
    }

    pub fn set_value(&mut self, value: f32) {
        self.value = value.clamp(self.min, self.max);
    }
}

/// Texture layer for appearance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextureLayer {
    pub name: String,
    pub blend_mode: BlendMode,
    pub opacity: f32,
    pub color_tint: [f32; 4], // RGBA
    pub texture_path: String,
}

/// Blend modes for texture layers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BlendMode {
    Normal,
    Add,
    Multiply,
    Screen,
    Overlay,
}

/// Particle effect for visual enhancement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticleEffect {
    pub name: String,
    pub effect_type: ParticleType,
    pub intensity: f32,
    pub color: [f32; 4],
    pub pattern: EnergyPattern,
}

/// Types of particle effects
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParticleType {
    Glow,
    Sparkle,
    Smoke,
    Energy,
    Ethereal,
    Fire,
    Water,
    Light,
}

/// Accessory slot for equipment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessorySlot {
    pub slot_id: String,
    pub item_id: Option<Uuid>,
    pub attachment_point: String,
    pub visible: bool,
}

/// Procedural modification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProceduralModification {
    pub name: String,
    pub mod_type: ModificationType,
    pub parameters: HashMap<String, f32>,
    pub seed: u64,
}

/// Types of procedural modifications
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModificationType {
    NoiseDisplacement,
    FractalPattern,
    ColorVariation,
    TextureBlending,
    GeometrySubdivision,
}

/// Customization engine for applying modifications
pub struct CustomizationEngine {
    initialized: bool,
}

impl CustomizationEngine {
    pub fn new() -> Self {
        Self {
            initialized: false,
        }
    }

    pub fn initialize(&mut self) -> crate::Result<()> {
        self.initialized = true;
        Ok(())
    }

    pub fn apply_customization(&self, _data: &CustomizationData) -> crate::Result<()> {
        if !self.initialized {
            return Err(crate::AvatarError::CustomizationFailed("Engine not initialized".to_string()));
        }

        // Apply customization logic here
        Ok(())
    }
}

impl Default for CustomizationEngine {
    fn default() -> Self {
        Self::new()
    }
}

// Re-export types that might be used in generation.rs
pub use crate::archetype::EnergyPattern;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_customization_data_creation() {
        let data = CustomizationData::default();
        assert!(!data.morph_targets.is_empty());
        assert!(data.morph_targets.contains_key("face_width"));
    }

    #[test]
    fn test_morph_target() {
        let mut morph = MorphTarget::new(0.5, -1.0, 1.0);
        assert_eq!(morph.value, 0.5);

        morph.set_value(2.0); // Should clamp to max
        assert_eq!(morph.value, 1.0);

        morph.set_value(-2.0); // Should clamp to min
        assert_eq!(morph.value, -1.0);
    }

    #[test]
    fn test_customization_engine() {
        let mut engine = CustomizationEngine::new();
        assert!(engine.initialize().is_ok());

        let data = CustomizationData::default();
        assert!(engine.apply_customization(&data).is_ok());
    }
}