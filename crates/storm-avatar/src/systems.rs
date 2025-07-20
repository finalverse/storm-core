// File: storm-core/crates/storm-avatar/src/systems.rs
// Description: ECS systems for avatar updates and behaviors
// Handles runtime avatar logic and state management

use storm_ecs::prelude::*;
use storm_math::{Vec3, Quat}; // Import Vec3 and Quat for transform operations
use crate::*;
use std::collections::HashMap;

/// Avatar update system - handles avatar state changes
pub struct AvatarUpdateSystem {
    last_update: f32,
}

impl AvatarUpdateSystem {
    pub fn new() -> Self {
        Self {
            last_update: 0.0,
        }
    }
}

impl System for AvatarUpdateSystem {
    fn update(&mut self, world: &mut World, delta_time: f32) -> std::result::Result<(), Box<dyn std::error::Error>> {
        self.last_update += delta_time;

        // Update avatar stats based on resonance
        let avatar_updates: Vec<(EntityId, f32, f32)> = {
            // Collect data first to avoid borrowing conflicts
            let mut updates = Vec::new();

            for (entity, avatar_stats) in world.query::<AvatarStats>() {
                if let Some(resonance) = world.get_component::<SongResonance>(entity) {
                    // Calculate energy regeneration based on harmony
                    let regen_rate = 5.0 * resonance.harmony_level;
                    let new_energy = (avatar_stats.energy + regen_rate * delta_time)
                        .min(avatar_stats.max_energy);

                    updates.push((entity.id, new_energy, resonance.harmony_level));
                }
            }

            updates
        };

        // Apply updates
        for (entity_id, new_energy, harmony_level) in avatar_updates {
            // Create entity with proper id
            let entity = Entity { id: entity_id, generation: 0 };
            if let Some(stats) = world.get_component_mut::<AvatarStats>(entity) {
                stats.energy = new_energy;

                // Update experience based on harmony level
                if harmony_level > 0.7 {
                    stats.experience += (delta_time * 10.0) as u64;
                }
            }
        }

        Ok(())
    }
}

/// Resonance update system - handles echo interactions and harmony changes
pub struct ResonanceUpdateSystem {
    interaction_cooldown: HashMap<EntityId, f32>,
}

impl ResonanceUpdateSystem {
    pub fn new() -> Self {
        Self {
            interaction_cooldown: HashMap::new(),
        }
    }
}

impl System for ResonanceUpdateSystem {
    fn update(&mut self, world: &mut World, delta_time: f32) -> std::result::Result<(), Box<dyn std::error::Error>> {
        // Update cooldowns
        for cooldown in self.interaction_cooldown.values_mut() {
            *cooldown -= delta_time;
        }
        self.interaction_cooldown.retain(|_, cooldown| *cooldown > 0.0);

        // Find entities with resonance that can interact
        let resonance_entities: Vec<(EntityId, EchoType, f32)> = {
            let mut entities = Vec::new();

            for (entity, resonance) in world.query::<SongResonance>() {
                // Only process if not on cooldown
                if !self.interaction_cooldown.contains_key(&entity.id) {
                    entities.push((entity.id, resonance.primary_aspect, resonance.harmony_level));
                }
            }

            entities
        };

        // Process potential interactions between entities
        for i in 0..resonance_entities.len() {
            for j in (i + 1)..resonance_entities.len() {
                let (entity1_id, echo1, harmony1) = resonance_entities[i];
                let (entity2_id, echo2, harmony2) = resonance_entities[j];

                // Check if they're close enough to interact (simplified distance check)
                if self.are_entities_close(world, entity1_id, entity2_id) {
                    self.process_echo_interaction(world, entity1_id, entity2_id, echo1, echo2, harmony1, harmony2)?;

                    // Set cooldown to prevent rapid interactions
                    self.interaction_cooldown.insert(entity1_id, 1.0);
                    self.interaction_cooldown.insert(entity2_id, 1.0);
                }
            }
        }

        Ok(())
    }
}

impl ResonanceUpdateSystem {
    /// Check if two entities are close enough to interact
    fn are_entities_close(&self, world: &World, entity1_id: EntityId, entity2_id: EntityId) -> bool {
        // Create entities with proper id
        let entity1 = Entity { id: entity1_id, generation: 0 };
        let entity2 = Entity { id: entity2_id, generation: 0 };

        if let (Some(transform1), Some(transform2)) = (
            world.get_component::<AvatarTransform>(entity1),
            world.get_component::<AvatarTransform>(entity2)
        ) {
            let dx = transform1.position.x - transform2.position.x;
            let dy = transform1.position.y - transform2.position.y;
            let dz = transform1.position.z - transform2.position.z;
            let distance_sq = dx * dx + dy * dy + dz * dz;

            // Interaction range of 5 units
            distance_sq < 25.0
        } else {
            false
        }
    }

    /// Process interaction between two entities with different echo types
    fn process_echo_interaction(
        &self,
        world: &mut World,
        entity1_id: EntityId,
        entity2_id: EntityId,
        echo1: EchoType,
        echo2: EchoType,
        harmony1: f32,
        harmony2: f32,
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let interaction_strength = (harmony1 + harmony2) / 2.0;

        // Update first entity's resonance
        // Create entity with proper id
        let entity1 = Entity { id: entity1_id, generation: 0 };
        if let Some(resonance1) = world.get_component_mut::<SongResonance>(entity1) {
            resonance1.process_echo_interaction(echo2, interaction_strength, None)?;
        }

        // Update second entity's resonance
        // Create entity with proper id
        let entity2 = Entity { id: entity2_id, generation: 0 };
        if let Some(resonance2) = world.get_component_mut::<SongResonance>(entity2) {
            resonance2.process_echo_interaction(echo1, interaction_strength, None)?;
        }

        Ok(())
    }
}

/// Avatar experience and leveling system
pub struct ExperienceSystem;

impl System for ExperienceSystem {
    fn update(&mut self, world: &mut World, _delta_time: f32) -> std::result::Result<(), Box<dyn std::error::Error>> {
        // Collect entities that need level updates
        let level_updates: Vec<(EntityId, u32, u64)> = {
            let mut updates = Vec::new();

            for (entity, avatar_base) in world.query::<AvatarBase>() {
                if let Some(stats) = world.get_component::<AvatarStats>(entity) {
                    let required_exp = Self::experience_for_level(avatar_base.level + 1);
                    if stats.experience >= required_exp {
                        updates.push((entity.id, avatar_base.level + 1, required_exp));
                    }
                }
            }

            updates
        };

        // Apply level updates
        for (entity_id, new_level, exp_used) in level_updates {
            // Create entity with proper id
            let entity = Entity { id: entity_id, generation: 0 };

            // Update avatar level
            if let Some(avatar_base) = world.get_component_mut::<AvatarBase>(entity) {
                avatar_base.level = new_level;
                avatar_base.last_modified = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
            }

            // Deduct experience and increase max stats
            if let Some(stats) = world.get_component_mut::<AvatarStats>(entity) {
                stats.experience -= exp_used;

                // Increase max health and energy on level up
                stats.max_health += 10.0;
                stats.max_energy += 5.0;
                stats.health = stats.max_health; // Full heal on level up
                stats.energy = stats.max_energy; // Full energy restore
            }

            // Improve resonance resistance on level up
            if let Some(resonance) = world.get_component_mut::<SongResonance>(entity) {
                resonance.silence_resistance += 0.05;
                resonance.silence_resistance = resonance.silence_resistance.min(1.0);
            }
        }

        Ok(())
    }
}

impl ExperienceSystem {
    /// Calculate experience required for a given level
    fn experience_for_level(level: u32) -> u64 {
        // Exponential scaling: level^2 * 100
        (level as u64).pow(2) * 100
    }
}

/// Avatar health and status system
pub struct HealthSystem {
    regen_timer: f32,
}

impl HealthSystem {
    pub fn new() -> Self {
        Self {
            regen_timer: 0.0,
        }
    }
}

impl System for HealthSystem {
    fn update(&mut self, world: &mut World, delta_time: f32) -> std::result::Result<(), Box<dyn std::error::Error>> {
        self.regen_timer += delta_time;

        // Health regeneration every second
        if self.regen_timer >= 1.0 {
            self.regen_timer = 0.0;

            // Collect entities that need health updates
            let health_updates: Vec<(EntityId, f32)> = {
                let mut updates = Vec::new();

                for (entity, stats) in world.query::<AvatarStats>() {
                    if stats.health < stats.max_health {
                        // Regenerate based on constitution and harmony
                        let mut regen_rate = 2.0; // Base regen

                        if let Some(resonance) = world.get_component::<SongResonance>(entity) {
                            regen_rate *= 1.0 + resonance.harmony_level * 0.5;
                        }

                        let new_health = (stats.health + regen_rate).min(stats.max_health);
                        updates.push((entity.id, new_health));
                    }
                }

                updates
            };

            // Apply health updates
            for (entity_id, new_health) in health_updates {
                // Create entity with proper id
                let entity = Entity { id: entity_id, generation: 0 };
                if let Some(stats) = world.get_component_mut::<AvatarStats>(entity) {
                    stats.health = new_health;
                }
            }
        }

        Ok(())
    }
}

/// Avatar customization update system
pub struct CustomizationSystem;

impl System for CustomizationSystem {
    fn update(&mut self, world: &mut World, _delta_time: f32) -> std::result::Result<(), Box<dyn std::error::Error>> {
        // Update dynamic customization effects (like particle systems)
        for (entity, _customization) in world.query::<CustomizationData>() {
            // Update particle effects based on current state
            if let Some(resonance) = world.get_component::<SongResonance>(entity) {
                // This would update particle intensity based on harmony level
                // For now, we'll just track that it should be updated
                let _harmony_intensity = resonance.harmony_level;

                // In a real implementation, this would update visual effects
                // based on the avatar's current resonance state
            }
        }

        Ok(())
    }
}

/// System for managing avatar inventory and equipment
pub struct InventorySystem;

impl System for InventorySystem {
    fn update(&mut self, world: &mut World, _delta_time: f32) -> std::result::Result<(), Box<dyn std::error::Error>> {
        // Process equipment changes and their effects on stats
        for (entity, inventory) in world.query::<AvatarInventory>() {
            if let Some(_stats) = world.get_component::<AvatarStats>(entity) {
                // Calculate equipment bonuses
                let mut _equipment_bonus = 0.0;

                // Count equipped items
                for slot in inventory.slots.values() {
                    if slot.is_some() {
                        _equipment_bonus += 5.0; // Simple bonus per equipped item
                    }
                }

                // Note: In real implementation, apply the equipment_bonus to stats
                // We use underscore prefix to suppress unused variable warnings
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_avatar_update_system() {
        let mut world = World::new();
        let entity = world.create_entity();

        // Add required components
        world.add_component(entity, AvatarStats {
            health: 50.0,
            max_health: 100.0,
            energy: 30.0,
            max_energy: 50.0,
            ..Default::default()
        });

        world.add_component(entity, SongResonance {
            harmony_level: 0.8,
            primary_aspect: EchoType::Hope,
            secondary_aspects: vec![],
            silence_resistance: 0.5,
            resonance_history: ResonanceHistory::default(),
        });

        let mut system = AvatarUpdateSystem::new();
        assert!(system.update(&mut world, 1.0).is_ok());

        // Check that energy was regenerated
        let stats = world.get_component::<AvatarStats>(entity).unwrap();
        assert!(stats.energy > 30.0);
    }

    #[test]
    fn test_experience_system() {
        let mut world = World::new();
        let entity = world.create_entity();

        world.add_component(entity, AvatarBase {
            id: uuid::Uuid::new_v4(),
            name: "Test Avatar".to_string(),
            archetype: AvatarArchetype::default(),
            level: 1,
            experience: 0,
            created_at: 0,
            last_modified: 0,
        });

        world.add_component(entity, AvatarStats {
            health: 100.0,
            max_health: 100.0,
            energy: 50.0,
            max_energy: 50.0,
            experience: 200, // Enough to level up
            ..Default::default()
        });

        let mut system = ExperienceSystem;
        assert!(system.update(&mut world, 1.0).is_ok());

        // Check that avatar leveled up
        let avatar_base = world.get_component::<AvatarBase>(entity).unwrap();
        assert_eq!(avatar_base.level, 2);
    }

    #[test]
    fn test_health_system() {
        let mut world = World::new();
        let entity = world.create_entity();

        world.add_component(entity, AvatarStats {
            health: 50.0,
            max_health: 100.0,
            energy: 50.0,
            max_energy: 50.0,
            ..Default::default()
        });

        let mut system = HealthSystem::new();
        system.regen_timer = 1.0; // Force immediate regeneration

        assert!(system.update(&mut world, 1.0).is_ok());

        // Check that health was regenerated
        let stats = world.get_component::<AvatarStats>(entity).unwrap();
        assert!(stats.health > 50.0);
    }

    #[test]
    fn test_resonance_system() {
        let mut world = World::new();

        // Create two entities for interaction
        let entity1 = world.create_entity();
        let entity2 = world.create_entity();

        // Add transforms so they're close to each other
        world.add_component(entity1, AvatarTransform {
            position: Vec3::new(0.0, 0.0, 0.0),
            rotation: Quat::IDENTITY, // Use IDENTITY constant instead
            scale: Vec3::new(1.0, 1.0, 1.0),
        });

        world.add_component(entity2, AvatarTransform {
            position: Vec3::new(2.0, 0.0, 0.0), // Within interaction range
            rotation: Quat::IDENTITY, // Use IDENTITY constant instead
            scale: Vec3::new(1.0, 1.0, 1.0),
        });

        // Add different resonances
        world.add_component(entity1, SongResonance {
            harmony_level: 0.7,
            primary_aspect: EchoType::Hope,
            secondary_aspects: vec![],
            silence_resistance: 0.5,
            resonance_history: ResonanceHistory::default(),
        });

        world.add_component(entity2, SongResonance {
            harmony_level: 0.6,
            primary_aspect: EchoType::Logic,
            secondary_aspects: vec![],
            silence_resistance: 0.4,
            resonance_history: ResonanceHistory::default(),
        });

        let mut system = ResonanceUpdateSystem::new();
        assert!(system.update(&mut world, 1.0).is_ok());

        // Check that entities interacted (should have recorded encounters)
        let resonance1 = world.get_component::<SongResonance>(entity1).unwrap();
        let resonance2 = world.get_component::<SongResonance>(entity2).unwrap();

        // Both should have recorded the interaction
        assert!(!resonance1.resonance_history.events.is_empty() || !resonance2.resonance_history.events.is_empty());
    }

    #[test]
    fn test_experience_calculation() {
        assert_eq!(ExperienceSystem::experience_for_level(1), 100);
        assert_eq!(ExperienceSystem::experience_for_level(2), 400);
        assert_eq!(ExperienceSystem::experience_for_level(3), 900);
        assert_eq!(ExperienceSystem::experience_for_level(10), 10000);
    }

    #[test]
    fn test_customization_system() {
        let mut world = World::new();
        let entity = world.create_entity();

        world.add_component(entity, CustomizationData::default());
        world.add_component(entity, SongResonance::default());

        let mut system = CustomizationSystem;
        assert!(system.update(&mut world, 1.0).is_ok());
    }

    #[test]
    fn test_inventory_system() {
        let mut world = World::new();
        let entity = world.create_entity();

        let mut inventory = AvatarInventory {
            slots: HashMap::new(),
            bag_items: vec![],
            max_bag_size: 20,
        };

        // Add some equipped items
        inventory.slots.insert(EquipmentSlot::Head, Some(uuid::Uuid::new_v4()));
        inventory.slots.insert(EquipmentSlot::Chest, Some(uuid::Uuid::new_v4()));

        world.add_component(entity, inventory);
        world.add_component(entity, AvatarStats::default());

        let mut system = InventorySystem;
        assert!(system.update(&mut world, 1.0).is_ok());
    }

    #[test]
    fn test_system_integration() {
        let mut world = World::new();
        let entity = world.create_entity();

        // Add all components
        world.add_component(entity, AvatarBase {
            id: uuid::Uuid::new_v4(),
            name: "Integration Test Avatar".to_string(),
            archetype: AvatarArchetype::default(),
            level: 1,
            experience: 0,
            created_at: 0,
            last_modified: 0,
        });

        world.add_component(entity, AvatarStats {
            health: 80.0,
            max_health: 100.0,
            energy: 40.0,
            max_energy: 50.0,
            experience: 150,
            ..Default::default()
        });

        world.add_component(entity, SongResonance::default());
        world.add_component(entity, CustomizationData::default());

        // Run all systems
        let mut avatar_system = AvatarUpdateSystem::new();
        let mut health_system = HealthSystem::new();
        let mut experience_system = ExperienceSystem;
        let mut customization_system = CustomizationSystem;

        assert!(avatar_system.update(&mut world, 1.0).is_ok());
        assert!(health_system.update(&mut world, 1.0).is_ok());
        assert!(experience_system.update(&mut world, 1.0).is_ok());
        assert!(customization_system.update(&mut world, 1.0).is_ok());

        // Verify avatar was updated
        let stats = world.get_component::<AvatarStats>(entity).unwrap();
        assert!(stats.energy > 40.0); // Should have regenerated energy
    }
}

/// Helper functions for system registration
pub fn add_avatar_systems(world: &mut World) {
    world.add_system(AvatarUpdateSystem::new());
    world.add_system(ResonanceUpdateSystem::new());
    world.add_system(ExperienceSystem);
    world.add_system(HealthSystem::new());
    world.add_system(CustomizationSystem);
    world.add_system(InventorySystem);
}

/// Avatar system coordinator - manages all avatar-related systems
pub struct AvatarSystemCoordinator {
    systems_initialized: bool,
}

impl AvatarSystemCoordinator {
    pub fn new() -> Self {
        Self {
            systems_initialized: false,
        }
    }

    /// Initialize all avatar systems in the world
    pub fn initialize_systems(&mut self, world: &mut World) -> std::result::Result<(), Box<dyn std::error::Error>> {
        if self.systems_initialized {
            return Ok(());
        }

        add_avatar_systems(world);
        self.systems_initialized = true;

        Ok(())
    }

    /// Check if a world has all required avatar components
    pub fn validate_avatar_entity(&self, world: &World, entity: Entity) -> bool {
        world.has_component::<AvatarBase>(entity) &&
            world.has_component::<AvatarStats>(entity) &&
            world.has_component::<SongResonance>(entity)
    }

    /// Create a complete avatar entity with all required components
    pub fn create_avatar_entity(
        &self,
        world: &mut World,
        base: AvatarBase,
        stats: AvatarStats,
        resonance: SongResonance,
        customization: Option<CustomizationData>,
    ) -> Entity {
        let entity = world.create_entity();

        // Add core components
        world.add_component(entity, base);
        world.add_component(entity, stats);
        world.add_component(entity, resonance);

        // Add transform
        world.add_component(entity, AvatarTransform::default());

        // Add customization if provided
        if let Some(custom) = customization {
            world.add_component(entity, custom);
        } else {
            world.add_component(entity, CustomizationData::default());
        }

        // Add inventory
        world.add_component(entity, AvatarInventory {
            slots: HashMap::new(),
            bag_items: vec![],
            max_bag_size: 20,
        });

        entity
    }
}

impl Default for AvatarSystemCoordinator {
    fn default() -> Self {
        Self::new()
    }
}

/// Performance metrics for avatar systems
#[derive(Debug, Clone)]
pub struct AvatarSystemMetrics {
    pub entities_processed: usize,
    pub average_update_time: f32,
    pub resonance_interactions: usize,
    pub level_ups: usize,
}

impl Default for AvatarSystemMetrics {
    fn default() -> Self {
        Self {
            entities_processed: 0,
            average_update_time: 0.0,
            resonance_interactions: 0,
            level_ups: 0,
        }
    }
}

/// System performance monitor
pub struct SystemMonitor {
    metrics: AvatarSystemMetrics,
    update_times: Vec<f32>,
}

impl SystemMonitor {
    pub fn new() -> Self {
        Self {
            metrics: AvatarSystemMetrics::default(),
            update_times: Vec::new(),
        }
    }

    pub fn record_update_time(&mut self, time: f32) {
        self.update_times.push(time);

        // Keep only last 100 measurements
        if self.update_times.len() > 100 {
            self.update_times.remove(0);
        }

        // Calculate average
        self.metrics.average_update_time =
            self.update_times.iter().sum::<f32>() / self.update_times.len() as f32;
    }

    pub fn get_metrics(&self) -> &AvatarSystemMetrics {
        &self.metrics
    }

    pub fn reset_metrics(&mut self) {
        self.metrics = AvatarSystemMetrics::default();
        self.update_times.clear();
    }
}

impl Default for SystemMonitor {
    fn default() -> Self {
        Self::new()
    }
}