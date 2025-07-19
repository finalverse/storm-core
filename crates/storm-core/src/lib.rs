// File: crates/storm-core/src/lib.rs
// StormCore - Main library entry point and public API
// Coordinates all major subsystems and exposes core functionality via FFI

use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, span, Level};

// Re-export major modules for internal use
pub mod core;
pub mod error;

// Re-export from workspace crates
pub use storm_ecs as ecs;
pub use storm_ai as ai;
pub use storm_networking as networking;
pub use storm_protocol_adapters as protocol_adapters;
pub use storm_math as math;
pub use storm_assets as assets;

#[cfg(feature = "rendering")]
pub use storm_rendering as rendering;

#[cfg(feature = "audio")]
pub use storm_audio as audio_engine;

#[cfg(feature = "physics")]
pub use storm_physics as physics;

// Public API types
pub use core::{StormConfig, WorldConfig, ProtocolType, PlatformType, RenderBackend};
pub use error::{StormError, StormResult};

/// StormCore - The main engine coordination struct
/// Manages all subsystems and provides unified API for virtual world interactions
pub struct StormCore {
    config: StormConfig,
    ecs_world: Arc<RwLock<ecs::World>>,
    ai_dispatcher: Arc<ai::AIDispatcher>,
    network_manager: Arc<RwLock<networking::NetworkManager>>, // Changed to RwLock for mutable access
    protocol_router: Arc<protocol_adapters::ProtocolRouter>,

    #[cfg(feature = "rendering")]
    render_pipeline: Option<Arc<rendering::RenderPipeline>>,

    #[cfg(feature = "audio")]
    audio_engine: Option<Arc<audio_engine::AudioEngine>>,

    #[cfg(feature = "physics")]
    physics_world: Option<Arc<RwLock<physics::PhysicsWorld>>>, // Changed to RwLock for mutable access
}

impl StormCore {
    /// Initialize new StormCore instance with configuration
    pub async fn new(config: StormConfig) -> StormResult<Self> {
        let _span = span!(Level::INFO, "storm_core_init").entered();
        info!("Initializing StormCore engine v{}", env!("CARGO_PKG_VERSION"));

        // Initialize core ECS world
        let ecs_world = Arc::new(RwLock::new(ecs::World::new()));

        // Initialize AI dispatcher with ML models - convert config properly
        let ai_config: ai::AIConfig = config.ai_config.clone().into();
        let ai_dispatcher = Arc::new(
            ai::AIDispatcher::new(&ai_config).await
                .map_err(|e| StormError::AiError(e.to_string()))?
        );

        // Initialize networking with protocol support - convert config
        let network_config: networking::NetworkConfig = config.network_config.clone().into();
        let network_manager = Arc::new(RwLock::new(
            networking::NetworkManager::new(&network_config).await
                .map_err(|e| StormError::NetworkError(e.to_string()))?
        ));

        // Initialize protocol adapters for OpenSim/MutSea and Finalverse
        let protocol_router = Arc::new(
            protocol_adapters::ProtocolRouter::new(
                ecs_world.clone(),
                ai_dispatcher.clone()
            ).await
                .map_err(|e| StormError::ProtocolError(e.to_string()))?
        );

        // Optional rendering pipeline (platform-dependent)
        #[cfg(feature = "rendering")]
        let render_pipeline = if config.enable_rendering {
            let render_config: rendering::RenderConfig = config.render_config.clone().into();
            Some(Arc::new(
                rendering::RenderPipeline::new(&render_config).await
                    .map_err(|e| StormError::RenderingError(e.to_string()))?
            ))
        } else {
            None
        };

        // Optional audio engine
        #[cfg(feature = "audio")]
        let audio_engine = if config.enable_audio {
            let audio_config: audio_engine::AudioConfig = config.audio_config.clone().into();
            Some(Arc::new(
                audio_engine::AudioEngine::new(&audio_config).await
                    .map_err(|e| StormError::AudioError(e.to_string()))?
            ))
        } else {
            None
        };

        // Optional physics simulation
        #[cfg(feature = "physics")]
        let physics_world = if config.enable_physics {
            let physics_config: physics::PhysicsConfig = config.physics_config.clone().into();
            Some(Arc::new(RwLock::new(
                physics::PhysicsWorld::new(&physics_config)
                    .map_err(|e| StormError::PhysicsError(e.to_string()))?
            )))
        } else {
            None
        };

        info!("StormCore engine initialized successfully");

        Ok(StormCore {
            config,
            ecs_world,
            ai_dispatcher,
            network_manager,
            protocol_router,

            #[cfg(feature = "rendering")]
            render_pipeline,

            #[cfg(feature = "audio")]
            audio_engine,

            #[cfg(feature = "physics")]
            physics_world,
        })
    }

    /// Connect to a virtual world using appropriate protocol adapter
    pub async fn connect_to_world(&self, world_config: &WorldConfig) -> StormResult<()> {
        info!("Connecting to world: {} (protocol: {:?})",
              world_config.name, world_config.protocol);

        // Convert storm-core WorldConfig to protocol-adapters WorldConfig
        let adapter_config = protocol_adapters::WorldConfig {
            name: world_config.name.clone(),
            url: world_config.url.clone(),
            protocol: match world_config.protocol {
                ProtocolType::OpenSim => protocol_adapters::ProtocolType::OpenSim,
                ProtocolType::Finalverse => protocol_adapters::ProtocolType::Finalverse,
            },
            credentials: world_config.credentials.as_ref().map(|creds| {
                protocol_adapters::Credentials {
                    username: creds.username.clone(),
                    password: creds.password.clone(),
                    session_token: None,
                    user_agent: None,
                }
            }),
            settings: protocol_adapters::ConnectionSettings::default(),
        };

        // Convert ECS WorldConfig for world initialization
        let ecs_config = ecs::WorldConfig {
            name: world_config.name.clone(),
            protocol: match world_config.protocol {
                ProtocolType::OpenSim => ecs::ProtocolType::OpenSim,
                ProtocolType::Finalverse => ecs::ProtocolType::Finalverse,
            },
        };

        // Initialize world-specific ECS entities and components
        let mut world = self.ecs_world.write().await;
        world.initialize_for_world(&ecs_config)
            .map_err(|e| StormError::EcsError(format!("{:?}", e)))?;

        info!("Successfully connected to world: {}", world_config.name);
        Ok(())
    }

    /// Main engine update loop - should be called each frame
    pub async fn update(&self, delta_time: f32) -> StormResult<()> {
        // Update ECS systems
        {
            let mut world = self.ecs_world.write().await;
            world.update(delta_time)
                .map_err(|e| StormError::EcsError(format!("{:?}", e)))?;
        }

        // Process AI enhancements asynchronously
        self.ai_dispatcher.process_pending_requests().await
            .map_err(|e| StormError::AiError(e.to_string()))?;

        // Update networking - now with proper RwLock access
        {
            let mut network = self.network_manager.write().await;
            network.update().await
                .map_err(|e| StormError::NetworkError(e.to_string()))?;
        }

        // Update rendering if enabled
        #[cfg(feature = "rendering")]
        if let Some(ref renderer) = self.render_pipeline {
            renderer.update(delta_time).await
                .map_err(|e| StormError::RenderingError(e.to_string()))?;
        }

        // Update audio if enabled
        #[cfg(feature = "audio")]
        if let Some(ref audio) = self.audio_engine {
            audio.update(delta_time).await
                .map_err(|e| StormError::AudioError(e.to_string()))?;
        }

        // Update physics if enabled - now with proper RwLock access
        #[cfg(feature = "physics")]
        if let Some(ref physics_arc) = self.physics_world {
            let mut physics = physics_arc.write().await;
            physics.update(delta_time)
                .map_err(|e| StormError::PhysicsError(e.to_string()))?;
        }

        Ok(())
    }

    /// Shutdown engine gracefully
    pub async fn shutdown(&self) -> StormResult<()> {
        info!("Shutting down StormCore engine");

        // Shutdown subsystems
        #[cfg(feature = "rendering")]
        if let Some(ref renderer) = self.render_pipeline {
            renderer.shutdown().await
                .map_err(|e| StormError::RenderingError(e.to_string()))?;
        }

        #[cfg(feature = "audio")]
        if let Some(ref audio) = self.audio_engine {
            audio.shutdown().await
                .map_err(|e| StormError::AudioError(e.to_string()))?;
        }

        // Shutdown network manager with proper RwLock access
        {
            let network = self.network_manager.read().await;
            network.shutdown().await
                .map_err(|e| StormError::NetworkError(e.to_string()))?;
        }

        self.ai_dispatcher.shutdown().await
            .map_err(|e| StormError::AiError(e.to_string()))?;

        info!("StormCore engine shutdown complete");
        Ok(())
    }

    // Getters for subsystem access
    pub fn ecs_world(&self) -> Arc<RwLock<ecs::World>> {
        self.ecs_world.clone()
    }

    pub fn ai_dispatcher(&self) -> Arc<ai::AIDispatcher> {
        self.ai_dispatcher.clone()
    }

    pub fn network_manager(&self) -> Arc<RwLock<networking::NetworkManager>> {
        self.network_manager.clone()
    }

    #[cfg(feature = "physics")]
    pub fn physics_world(&self) -> Option<Arc<RwLock<physics::PhysicsWorld>>> {
        self.physics_world.clone()
    }
}

/// Initialize logging and tracing
pub fn init_logging() {
    use tracing_subscriber::{fmt, EnvFilter};

    fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive(Level::INFO.into()))
        .with_target(false)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .init();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_storm_core_initialization() {
        let config = StormConfig::default();
        let core = StormCore::new(config).await;
        assert!(core.is_ok());
    }
}