// File: crates/storm-ecs/src/lib.rs
// Entity-Component-System implementation for StormCore
// High-performance world simulation with AI integration

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::any::{Any, TypeId};
use std::sync::atomic::{AtomicU64, Ordering};

/// Entity identifier - simple u64 for performance
pub type EntityId = u64;

/// Component trait - all components must implement this
pub trait Component: Any + Send + Sync + 'static {
    fn type_name() -> &'static str where Self: Sized;
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

/// Entity handle with type-safe component access
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Entity {
    pub id: EntityId,
    pub generation: u32,
}

impl Entity {
    pub fn new(id: EntityId, generation: u32) -> Self {
        Self { id, generation }
    }
}

/// ECS World - manages all entities and components
pub struct World {
    next_entity_id: AtomicU64,
    entities: HashMap<EntityId, Entity>,
    components: HashMap<TypeId, ComponentStorage>,
    systems: Vec<Box<dyn System>>,
}

impl World {
    pub fn new() -> Self {
        Self {
            next_entity_id: AtomicU64::new(1),
            entities: HashMap::new(),
            components: HashMap::new(),
            systems: Vec::new(),
        }
    }

    /// Create a new entity
    pub fn create_entity(&mut self) -> Entity {
        let id = self.next_entity_id.fetch_add(1, Ordering::SeqCst);
        let entity = Entity::new(id, 0);
        self.entities.insert(id, entity);
        entity
    }

    /// Remove an entity and all its components
    pub fn remove_entity(&mut self, entity: Entity) -> bool {
        if self.entities.remove(&entity.id).is_some() {
            // Remove from all component storages
            for storage in self.components.values_mut() {
                storage.remove(entity.id);
            }
            true
        } else {
            false
        }
    }

    /// Add a component to an entity - fixed borrowing issue
    pub fn add_component<T: Component>(&mut self, entity: Entity, component: T) {
        let type_id = TypeId::of::<T>();

        // Check if storage exists, if not create it
        if !self.components.contains_key(&type_id) {
            self.components.insert(type_id, ComponentStorage::new::<T>());
        }

        // Now we can safely get the storage and insert
        if let Some(storage) = self.components.get_mut(&type_id) {
            storage.insert(entity.id, Box::new(component));
        }
    }

    /// Get a component from an entity
    pub fn get_component<T: Component>(&self, entity: Entity) -> Option<&T> {
        let type_id = TypeId::of::<T>();
        self.components.get(&type_id)?.get(entity.id)?.downcast_ref::<T>()
    }

    /// Get a mutable component from an entity
    pub fn get_component_mut<T: Component>(&mut self, entity: Entity) -> Option<&mut T> {
        let type_id = TypeId::of::<T>();
        self.components.get_mut(&type_id)?.get_mut(entity.id)?.downcast_mut::<T>()
    }

    /// Check if entity has component
    pub fn has_component<T: Component>(&self, entity: Entity) -> bool {
        let type_id = TypeId::of::<T>();
        self.components.get(&type_id)
            .map_or(false, |storage| storage.contains(entity.id))
    }

    /// Query entities with specific components
    pub fn query<T: Component>(&self) -> impl Iterator<Item = (Entity, &T)> {
        let type_id = TypeId::of::<T>();
        self.components.get(&type_id)
            .into_iter()
            .flat_map(|storage| {
                storage.iter().filter_map(|(entity_id, component)| {
                    let entity = self.entities.get(entity_id)?;
                    let component = component.downcast_ref::<T>()?;
                    Some((*entity, component))
                })
            })
    }

    /// Add a system to the world
    pub fn add_system<T: System + 'static>(&mut self, system: T) {
        self.systems.push(Box::new(system));
    }

    /// Update all systems - better fix using mem::take
    pub fn update(&mut self, delta_time: f32) -> Result<(), Box<dyn std::error::Error>> {
        // Temporarily take the systems out of self
        let mut systems = std::mem::take(&mut self.systems);

        // Update each system
        for system in &mut systems {
            system.update(self, delta_time)?;
        }

        // Put the systems back
        self.systems = systems;

        Ok(())
    }

    /// Initialize world for a specific virtual world - Fixed to use local WorldConfig
    pub fn initialize_for_world(&mut self, world_config: &WorldConfig) -> Result<(), Box<dyn std::error::Error>> {
        // Create basic world entities
        let world_entity = self.create_entity();
        self.add_component(world_entity, WorldInfo {
            name: world_config.name.clone(),
            protocol: world_config.protocol,
        });

        // Add core systems
        self.add_system(TransformSystem);
        self.add_system(MovementSystem);

        Ok(())
    }
}

/// Component storage for type-erased components
struct ComponentStorage {
    components: HashMap<EntityId, Box<dyn Any + Send + Sync>>,
}

impl ComponentStorage {
    fn new<T: Component>() -> Self {
        Self {
            components: HashMap::new(),
        }
    }

    fn insert(&mut self, entity_id: EntityId, component: Box<dyn Any + Send + Sync>) {
        self.components.insert(entity_id, component);
    }

    fn get(&self, entity_id: EntityId) -> Option<&Box<dyn Any + Send + Sync>> {
        self.components.get(&entity_id)
    }

    fn get_mut(&mut self, entity_id: EntityId) -> Option<&mut Box<dyn Any + Send + Sync>> {
        self.components.get_mut(&entity_id)
    }

    fn remove(&mut self, entity_id: EntityId) -> Option<Box<dyn Any + Send + Sync>> {
        self.components.remove(&entity_id)
    }

    fn contains(&self, entity_id: EntityId) -> bool {
        self.components.contains_key(&entity_id)
    }

    fn iter(&self) -> impl Iterator<Item = (&EntityId, &Box<dyn Any + Send + Sync>)> {
        self.components.iter()
    }
}

/// System trait for ECS processing
pub trait System: Send + Sync {
    fn update(&mut self, world: &mut World, delta_time: f32) -> Result<(), Box<dyn std::error::Error>>;
}

// Core Components

/// 3D Transform component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transform {
    pub position: [f32; 3],
    pub rotation: [f32; 4], // Quaternion
    pub scale: [f32; 3],
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            position: [0.0, 0.0, 0.0],
            rotation: [0.0, 0.0, 0.0, 1.0], // Identity quaternion
            scale: [1.0, 1.0, 1.0],
        }
    }
}

impl Component for Transform {
    fn type_name() -> &'static str {
        "Transform"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

/// Velocity component for movement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Velocity {
    pub linear: [f32; 3],
    pub angular: [f32; 3],
}

impl Default for Velocity {
    fn default() -> Self {
        Self {
            linear: [0.0, 0.0, 0.0],
            angular: [0.0, 0.0, 0.0],
        }
    }
}

impl Component for Velocity {
    fn type_name() -> &'static str {
        "Velocity"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

/// Simple protocol type for this crate
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProtocolType {
    OpenSim,
    Finalverse,
}

/// Simple world config for this crate - matching the one in storm-core
#[derive(Debug, Clone)]
pub struct WorldConfig {
    pub name: String,
    pub protocol: ProtocolType,
}

/// World information component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldInfo {
    pub name: String,
    pub protocol: ProtocolType,
}

impl Component for WorldInfo {
    fn type_name() -> &'static str {
        "WorldInfo"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

// Core Systems

/// Transform system for basic transform operations
pub struct TransformSystem;

impl System for TransformSystem {
    fn update(&mut self, _world: &mut World, _delta_time: f32) -> Result<(), Box<dyn std::error::Error>> {
        // Transform system logic here
        Ok(())
    }
}

/// Movement system for applying velocity to transforms
pub struct MovementSystem;

impl System for MovementSystem {
    fn update(&mut self, world: &mut World, delta_time: f32) -> Result<(), Box<dyn std::error::Error>> {
        // Collect entity IDs and velocities first (immutable borrow)
        let velocity_data: Vec<(EntityId, [f32; 3])> = {
            world.query::<Velocity>()
                .filter(|(entity, _)| world.has_component::<Transform>(*entity))
                .map(|(entity, velocity)| (entity.id, velocity.linear))
                .collect()
        };

        // Now apply updates (mutable borrow)
        for (entity_id, linear_velocity) in velocity_data {
            let entity = Entity::new(entity_id, 0);
            if let Some(transform) = world.get_component_mut::<Transform>(entity) {
                transform.position[0] += linear_velocity[0] * delta_time;
                transform.position[1] += linear_velocity[1] * delta_time;
                transform.position[2] += linear_velocity[2] * delta_time;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_world_creation() {
        let mut world = World::new();
        let entity = world.create_entity();

        world.add_component(entity, Transform::default());
        assert!(world.has_component::<Transform>(entity));
    }

    #[test]
    fn test_component_query() {
        let mut world = World::new();
        let entity = world.create_entity();

        let transform = Transform {
            position: [1.0, 2.0, 3.0],
            ..Default::default()
        };

        world.add_component(entity, transform);

        let results: Vec<_> = world.query::<Transform>().collect();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].1.position, [1.0, 2.0, 3.0]);
    }

    #[test]
    fn test_world_initialization() {
        let mut world = World::new();
        let config = WorldConfig {
            name: "Test World".to_string(),
            protocol: ProtocolType::OpenSim,
        };

        assert!(world.initialize_for_world(&config).is_ok());
    }

    #[test]
    fn test_movement_system() {
        let mut world = World::new();
        let entity = world.create_entity();

        // Add transform and velocity
        world.add_component(entity, Transform::default());
        world.add_component(entity, Velocity {
            linear: [1.0, 0.0, 0.0],
            angular: [0.0, 0.0, 0.0],
        });

        // Add movement system and update
        world.add_system(MovementSystem);
        assert!(world.update(1.0).is_ok());

        // Check that position was updated
        let transform = world.get_component::<Transform>(entity).unwrap();
        assert_eq!(transform.position[0], 1.0);
    }
}