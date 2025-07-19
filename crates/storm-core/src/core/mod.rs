// File: crates/storm-core/src/core/mod.rs
// Core types, configuration, and common utilities for StormCore
// Defines fundamental data structures used throughout the engine

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub mod config;
pub mod handle;

pub use config::*;
pub use handle::*;

/// Core engine configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StormConfig {
    pub ai_config: AIConfig,
    pub network_config: NetworkConfig,
    pub render_config: RenderConfig,
    pub audio_config: AudioConfig,
    pub physics_config: PhysicsConfig,

    // Feature flags
    pub enable_rendering: bool,
    pub enable_audio: bool,
    pub enable_physics: bool,
    pub enable_ai_enhanced: bool,

    // Platform-specific settings
    pub platform: PlatformType,
    pub debug_mode: bool,
}

impl Default for StormConfig {
    fn default() -> Self {
        Self {
            ai_config: AIConfig::default(),
            network_config: NetworkConfig::default(),
            render_config: RenderConfig::default(),
            audio_config: AudioConfig::default(),
            physics_config: PhysicsConfig::default(),
            enable_rendering: true,
            enable_audio: true,
            enable_physics: true,
            enable_ai_enhanced: true,
            platform: PlatformType::detect(),
            debug_mode: cfg!(debug_assertions),
        }
    }
}

/// AI system configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIConfig {
    pub grok_api_key: Option<String>,
    pub grok_api_endpoint: String,
    pub local_ml_enabled: bool,
    pub model_cache_dir: String,
    pub max_concurrent_requests: usize,
    pub ai_enhancement_timeout_ms: u64,
}

impl Default for AIConfig {
    fn default() -> Self {
        Self {
            grok_api_key: std::env::var("GROK_API_KEY").ok(),
            grok_api_endpoint: "https://api.x.ai/v1".to_string(),
            local_ml_enabled: true,
            model_cache_dir: "./models".to_string(),
            max_concurrent_requests: 10,
            ai_enhancement_timeout_ms: 200,
        }
    }
}

/// Network configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub max_connections: usize,
    pub connection_timeout_ms: u64,
    pub packet_buffer_size: usize,
    pub compression_enabled: bool,
    pub encryption_enabled: bool,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            max_connections: 100,
            connection_timeout_ms: 5000,
            packet_buffer_size: 8192,
            compression_enabled: true,
            encryption_enabled: true,
        }
    }
}

/// Rendering pipeline configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderConfig {
    pub backend: RenderBackend,
    pub vsync_enabled: bool,
    pub max_fps: u32,
    pub shadow_quality: ShadowQuality,
    pub texture_quality: TextureQuality,
}

impl Default for RenderConfig {
    fn default() -> Self {
        let platform = PlatformType::detect();
        let backend = RenderBackend::for_platform(platform);

        Self {
            backend,
            vsync_enabled: true,
            max_fps: 60,
            shadow_quality: ShadowQuality::Medium,
            texture_quality: TextureQuality::High,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShadowQuality {
    Low,
    Medium,
    High,
    Ultra,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TextureQuality {
    Low,
    Medium,
    High,
    Ultra,
}

/// Audio engine configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioConfig {
    pub sample_rate: u32,
    pub buffer_size: u32,
    pub spatial_audio_enabled: bool,
    pub max_audio_sources: usize,
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            sample_rate: 44100,
            buffer_size: 512,
            spatial_audio_enabled: true,
            max_audio_sources: 64,
        }
    }
}

/// Physics simulation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysicsConfig {
    pub gravity: [f32; 3],
    pub timestep: f32,
    pub max_substeps: u32,
    pub collision_detection_enabled: bool,
}

impl Default for PhysicsConfig {
    fn default() -> Self {
        Self {
            gravity: [0.0, -9.81, 0.0],
            timestep: 1.0 / 60.0,
            max_substeps: 4,
            collision_detection_enabled: true,
        }
    }
}

/// Configuration for connecting to a virtual world
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldConfig {
    pub name: String,
    pub url: String,
    pub protocol: ProtocolType,
    pub credentials: Option<WorldCredentials>,
}

/// Supported virtual world protocols
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProtocolType {
    OpenSim,
    Finalverse,
}

/// Authentication credentials for world connection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldCredentials {
    pub username: String,
    pub password: String,
    pub additional_fields: HashMap<String, String>,
}

impl WorldConfig {
    pub fn new_opensim(name: &str, grid_url: &str, username: &str, password: &str) -> Self {
        Self {
            name: name.to_string(),
            url: grid_url.to_string(),
            protocol: ProtocolType::OpenSim,
            credentials: Some(WorldCredentials {
                username: username.to_string(),
                password: password.to_string(),
                additional_fields: HashMap::new(),
            }),
        }
    }

    pub fn new_finalverse(name: &str, server_url: &str) -> Self {
        Self {
            name: name.to_string(),
            url: server_url.to_string(),
            protocol: ProtocolType::Finalverse,
            credentials: None,
        }
    }
}

// Conversion implementations for subsystem configs
impl From<AIConfig> for storm_ai::AIConfig {
    fn from(config: AIConfig) -> Self {
        storm_ai::AIConfig {
            grok_api_key: config.grok_api_key,
            grok_api_endpoint: config.grok_api_endpoint,
            local_ml_enabled: config.local_ml_enabled,
            model_cache_dir: config.model_cache_dir,
            max_concurrent_requests: config.max_concurrent_requests,
            ai_enhancement_timeout_ms: config.ai_enhancement_timeout_ms,
        }
    }
}

impl From<NetworkConfig> for storm_networking::NetworkConfig {
    fn from(config: NetworkConfig) -> Self {
        storm_networking::NetworkConfig {
            max_connections: config.max_connections,
            connection_timeout_ms: config.connection_timeout_ms,
            packet_buffer_size: config.packet_buffer_size,
            compression_enabled: config.compression_enabled,
            encryption_enabled: config.encryption_enabled,
        }
    }
}

#[cfg(feature = "rendering")]
impl From<RenderConfig> for storm_rendering::RenderConfig {
    fn from(config: RenderConfig) -> Self {
        storm_rendering::RenderConfig {
            backend: config.backend.into(),
            vsync_enabled: config.vsync_enabled,
            max_fps: config.max_fps,
            shadow_quality: match config.shadow_quality {
                ShadowQuality::Low => storm_rendering::ShadowQuality::Low,
                ShadowQuality::Medium => storm_rendering::ShadowQuality::Medium,
                ShadowQuality::High => storm_rendering::ShadowQuality::High,
                ShadowQuality::Ultra => storm_rendering::ShadowQuality::Ultra,
            },
            texture_quality: match config.texture_quality {
                TextureQuality::Low => storm_rendering::TextureQuality::Low,
                TextureQuality::Medium => storm_rendering::TextureQuality::Medium,
                TextureQuality::High => storm_rendering::TextureQuality::High,
                TextureQuality::Ultra => storm_rendering::TextureQuality::Ultra,
            },
        }
    }
}

#[cfg(feature = "audio")]
impl From<AudioConfig> for storm_audio::AudioConfig {
    fn from(config: AudioConfig) -> Self {
        storm_audio::AudioConfig {
            sample_rate: config.sample_rate,
            buffer_size: config.buffer_size,
            spatial_audio_enabled: config.spatial_audio_enabled,
            max_audio_sources: config.max_audio_sources,
        }
    }
}

#[cfg(feature = "physics")]
impl From<PhysicsConfig> for storm_physics::PhysicsConfig {
    fn from(config: PhysicsConfig) -> Self {
        storm_physics::PhysicsConfig {
            gravity: config.gravity,
            timestep: config.timestep,
            max_substeps: config.max_substeps,
            collision_detection_enabled: config.collision_detection_enabled,
        }
    }
}

#[cfg(feature = "rendering")]
impl From<RenderBackend> for storm_rendering::RenderBackend {
    fn from(backend: RenderBackend) -> Self {
        match backend {
            RenderBackend::Metal => storm_rendering::RenderBackend::Metal,
            RenderBackend::Vulkan => storm_rendering::RenderBackend::Vulkan,
            RenderBackend::WebGL => storm_rendering::RenderBackend::WebGL,
            RenderBackend::Software => storm_rendering::RenderBackend::Software,
        }
    }
}