// File: storm-core/crates/storm-avatar/src/components.rs
// Description: Core ECS components for avatar system
// Defines the fundamental data structures for avatars

use storm_ecs::prelude::*;
use glam::{Vec3, Quat, Vec4};
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use std::collections::HashMap;

// Import the archetype types
use crate::archetype::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AvatarBase {
    pub id: Uuid,
    pub name: String,
    pub archetype: AvatarArchetype,
    pub level: u32,
    pub experience: u64,
    pub created_at: u64,
    pub last_modified: u64,
}

impl Component for AvatarBase {
    fn type_name() -> &'static str {
        "AvatarBase"
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl Default for AvatarBase {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            name: String::from("Unnamed Avatar"),
            archetype: AvatarArchetype::default(),
            level: 1,
            experience: 0,
            created_at: 0,
            last_modified: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AvatarTransform {
    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

impl Component for AvatarTransform {
    fn type_name() -> &'static str {
        "AvatarTransform"
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl Default for AvatarTransform {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AvatarVisuals {
    pub base_mesh: String,
    pub material_set: String,
    pub color_palette: ColorPalette,
    pub emissive_intensity: f32,
}

impl Component for AvatarVisuals {
    fn type_name() -> &'static str {
        "AvatarVisuals"
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorPalette {
    pub primary: Vec4,
    pub secondary: Vec4,
    pub accent: Vec4,
    pub skin_tone: Vec4,
    pub hair_color: Vec4,
    pub eye_color: Vec4,
}

impl Default for ColorPalette {
    fn default() -> Self {
        Self {
            primary: Vec4::new(0.2, 0.3, 0.8, 1.0),
            secondary: Vec4::new(0.8, 0.8, 0.8, 1.0),
            accent: Vec4::new(0.9, 0.6, 0.2, 1.0),
            skin_tone: Vec4::new(0.9, 0.7, 0.6, 1.0),
            hair_color: Vec4::new(0.2, 0.1, 0.05, 1.0),
            eye_color: Vec4::new(0.3, 0.5, 0.8, 1.0),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AvatarStats {
    pub health: f32,
    pub max_health: f32,
    pub energy: f32,
    pub max_energy: f32,
    pub movement_speed: f32,
    pub experience: u64,
    pub attributes: HashMap<String, f32>,
}

impl Component for AvatarStats {
    fn type_name() -> &'static str {
        "AvatarStats"
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl Default for AvatarStats {
    fn default() -> Self {
        let mut attributes = HashMap::new();
        attributes.insert("strength".to_string(), 10.0);
        attributes.insert("agility".to_string(), 10.0);
        attributes.insert("intelligence".to_string(), 10.0);
        attributes.insert("charisma".to_string(), 10.0);

        Self {
            health: 100.0,
            max_health: 100.0,
            energy: 100.0,
            max_energy: 100.0,
            movement_speed: 5.0,
            experience: 0,
            attributes,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AvatarInventory {
    pub slots: HashMap<EquipmentSlot, Option<ItemId>>,
    pub bag_items: Vec<ItemId>,
    pub max_bag_size: usize,
}

impl Component for AvatarInventory {
    fn type_name() -> &'static str {
        "AvatarInventory"
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum EquipmentSlot {
    Head,
    Chest,
    Legs,
    Feet,
    MainHand,
    OffHand,
    Accessory1,
    Accessory2,
}

pub type ItemId = Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompanionSlot {
    pub companion_id: Option<Uuid>,
    pub bond_level: f32,
    pub abilities: Vec<String>,
}

impl Component for CompanionSlot {
    fn type_name() -> &'static str {
        "CompanionSlot"
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}