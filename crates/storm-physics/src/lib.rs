// File: crates/storm-physics/src/lib.rs
// Physics simulation for StormCore
// Provides collision detection, rigid body dynamics, and spatial queries
// Fixed version with proper mutable update method

use serde::{Deserialize, Serialize};
use tracing::{info, warn};
use anyhow::Result;

/// Physics configuration for the StormCore physics engine
/// Defines simulation parameters including gravity, timestep, and feature flags
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysicsConfig {
    /// Gravity vector in 3D space [x, y, z] (m/s²)
    pub gravity: [f32; 3],
    /// Simulation timestep in seconds (typically 1/60 for 60Hz)
    pub timestep: f32,
    /// Maximum number of physics substeps per frame
    pub max_substeps: u32,
    /// Enable/disable collision detection system
    pub collision_detection_enabled: bool,
}

impl Default for PhysicsConfig {
    fn default() -> Self {
        Self {
            gravity: [0.0, -9.81, 0.0], // Standard Earth gravity
            timestep: 1.0 / 60.0,       // 60Hz simulation
            max_substeps: 4,             // Reasonable substep limit
            collision_detection_enabled: true,
        }
    }
}

/// Main physics world simulation container
/// Manages different physics backend implementations based on feature flags
/// Integrates with ECS for entity-component synchronization
pub struct PhysicsWorld {
    /// Configuration parameters for the physics simulation
    config: PhysicsConfig,

    /// Rapier physics backend (conditional compilation)
    #[cfg(feature = "rapier")]
    rapier_world: Option<RapierWorld>,
    #[cfg(not(feature = "rapier"))]
    rapier_world: Option<()>, // Explicit type annotation for disabled feature

    /// Bullet physics backend (conditional compilation)
    #[cfg(feature = "bullet")]
    bullet_world: Option<BulletWorld>,
    #[cfg(not(feature = "bullet"))]
    bullet_world: Option<()>, // Explicit type annotation for disabled feature
}

impl PhysicsWorld {
    /// Creates a new physics world with the specified configuration
    /// Initializes enabled physics backends based on compile-time features
    /// Returns Result for error handling during initialization
    pub fn new(config: &PhysicsConfig) -> Result<Self> {
        info!("Initializing physics world with config: {:?}", config);

        // Initialize Rapier backend if feature is enabled
        #[cfg(feature = "rapier")]
        let rapier_world: Option<RapierWorld> = Some(RapierWorld::new(config)?);
        #[cfg(not(feature = "rapier"))]
        let rapier_world: Option<()> = None;

        // Initialize Bullet backend if feature is enabled
        #[cfg(feature = "bullet")]
        let bullet_world: Option<BulletWorld> = Some(BulletWorld::new(config)?);
        #[cfg(not(feature = "bullet"))]
        let bullet_world: Option<()> = None;

        Ok(Self {
            config: config.clone(),
            rapier_world,
            bullet_world,
        })
    }

    /// Updates the physics simulation by the specified time delta
    /// Processes rigid body dynamics, collision detection, and constraint solving
    /// Skips processing if collision detection is disabled in config
    /// FIXED: Now takes &mut self for mutable access
    pub fn update(&mut self, delta_time: f32) -> Result<()> {
        // Early return if physics is disabled
        if !self.config.collision_detection_enabled {
            return Ok(());
        }

        // Process Rapier physics backend if available
        #[cfg(feature = "rapier")]
        if let Some(ref mut world) = self.rapier_world {
            world.step(delta_time)?;
        }

        // Process Bullet physics backend if available
        #[cfg(feature = "bullet")]
        if let Some(ref mut world) = self.bullet_world {
            world.step(delta_time)?;
        }

        Ok(())
    }

    /// Retrieves the current physics configuration
    pub fn get_config(&self) -> &PhysicsConfig {
        &self.config
    }

    /// Updates the physics configuration at runtime
    /// Note: Some changes may require world recreation
    pub fn set_config(&mut self, config: PhysicsConfig) -> Result<()> {
        info!("Updating physics configuration");
        self.config = config;

        // TODO: Propagate config changes to active backends
        // This may require backend-specific update methods

        Ok(())
    }
}

// Physics engine implementations with proper conditional compilation

/// Rapier physics backend implementation
/// Provides high-performance rigid body dynamics and collision detection
#[cfg(feature = "rapier")]
struct RapierWorld {
    /// Configuration reference for simulation parameters
    config: PhysicsConfig,
    // TODO: Add actual rapier data structures
    // - RigidBodySet for managing rigid bodies
    // - ColliderSet for collision shapes
    // - PhysicsPipeline for simulation stepping
    // - IntegrationParameters for solver settings
}

#[cfg(feature = "rapier")]
impl RapierWorld {
    /// Creates a new Rapier physics world with the given configuration
    fn new(config: &PhysicsConfig) -> Result<Self> {
        info!("Initializing Rapier physics backend");

        // TODO: Initialize rapier data structures
        // let mut bodies = RigidBodySet::new();
        // let mut colliders = ColliderSet::new();
        // let gravity = Vector::new(config.gravity[0], config.gravity[1], config.gravity[2]);

        Ok(Self {
            config: config.clone(),
        })
    }

    /// Steps the Rapier physics simulation forward by delta_time
    /// Handles integration, collision detection, and constraint solving
    fn step(&mut self, delta_time: f32) -> Result<()> {
        // TODO: Implement actual Rapier stepping
        // self.pipeline.step(
        //     &gravity,
        //     &integration_parameters,
        //     &mut island_manager,
        //     &mut broad_phase,
        //     &mut narrow_phase,
        //     &mut bodies,
        //     &mut colliders,
        //     &mut joints,
        //     &mut solver,
        // );

        Ok(())
    }
}

/// Bullet physics backend implementation
/// Alternative physics engine for specialized use cases
#[cfg(feature = "bullet")]
struct BulletWorld {
    /// Configuration reference for simulation parameters
    config: PhysicsConfig,
    // TODO: Add actual bullet data structures
}

#[cfg(feature = "bullet")]
impl BulletWorld {
    /// Creates a new Bullet physics world with the given configuration
    fn new(config: &PhysicsConfig) -> Result<Self> {
        info!("Initializing Bullet physics backend");

        // TODO: Initialize bullet data structures

        Ok(Self {
            config: config.clone(),
        })
    }

    /// Steps the Bullet physics simulation forward by delta_time
    fn step(&mut self, delta_time: f32) -> Result<()> {
        // TODO: Implement actual Bullet stepping

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_physics_world_creation() {
        let config = PhysicsConfig::default();
        let world = PhysicsWorld::new(&config);
        assert!(world.is_ok(), "PhysicsWorld creation should succeed");

        let world = world.unwrap();
        assert_eq!(world.get_config().gravity, [0.0, -9.81, 0.0]);
        assert_eq!(world.get_config().timestep, 1.0 / 60.0);
    }

    #[test]
    fn test_physics_config_default() {
        let config = PhysicsConfig::default();
        assert_eq!(config.gravity, [0.0, -9.81, 0.0]);
        assert_eq!(config.timestep, 1.0 / 60.0);
        assert_eq!(config.max_substeps, 4);
        assert!(config.collision_detection_enabled);
    }

    #[test]
    fn test_physics_update_disabled() {
        let mut config = PhysicsConfig::default();
        config.collision_detection_enabled = false;

        let mut world = PhysicsWorld::new(&config).unwrap();
        let result = world.update(1.0 / 60.0);

        assert!(result.is_ok(), "Update should succeed even when disabled");
    }

    #[test]
    fn test_physics_config_update() {
        let config = PhysicsConfig::default();
        let mut world = PhysicsWorld::new(&config).unwrap();

        let new_config = PhysicsConfig {
            gravity: [0.0, -3.71, 0.0], // Mars gravity
            timestep: 1.0 / 120.0,      // 120Hz
            max_substeps: 8,
            collision_detection_enabled: true,
        };

        let result = world.set_config(new_config.clone());
        assert!(result.is_ok());
        assert_eq!(world.get_config().gravity, [0.0, -3.71, 0.0]);
    }

    #[test]
    fn test_physics_update_mutable() {
        let config = PhysicsConfig::default();
        let mut world = PhysicsWorld::new(&config).unwrap();

        // Test that update requires mutable reference
        let result = world.update(1.0 / 60.0);
        assert!(result.is_ok());
    }
}