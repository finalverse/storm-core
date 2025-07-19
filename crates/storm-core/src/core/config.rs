// File: crates/storm-core/src/core/config.rs

use serde::{Deserialize, Serialize};

/// Extended platform detection including WebAssembly
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlatformType {
    MacOS,
    iOS,
    Windows,
    Linux,
    Android,
    WASM,  // Added WebAssembly support
}

impl PlatformType {
    pub fn detect() -> Self {
        #[cfg(target_arch = "wasm32")]
        return PlatformType::WASM;

        #[cfg(target_os = "macos")]
        return PlatformType::MacOS;

        #[cfg(target_os = "ios")]
        return PlatformType::iOS;

        #[cfg(target_os = "windows")]
        return PlatformType::Windows;

        #[cfg(target_os = "linux")]
        return PlatformType::Linux;

        #[cfg(target_os = "android")]
        return PlatformType::Android;
    }

    pub fn supports_metal(&self) -> bool {
        matches!(self, PlatformType::MacOS | PlatformType::iOS)
    }

    pub fn supports_vulkan(&self) -> bool {
        matches!(
            self,
            PlatformType::Windows | PlatformType::Linux | PlatformType::Android
        )
    }

    pub fn supports_webgl(&self) -> bool {
        matches!(self, PlatformType::WASM)
    }

    pub fn is_mobile(&self) -> bool {
        matches!(self, PlatformType::iOS | PlatformType::Android)
    }

    pub fn is_desktop(&self) -> bool {
        matches!(self, PlatformType::MacOS | PlatformType::Windows | PlatformType::Linux)
    }

    pub fn is_web(&self) -> bool {
        matches!(self, PlatformType::WASM)
    }
}

/// Extended render backend support
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RenderBackend {
    Metal,
    Vulkan,
    WebGL,     // Added WebGL support
    Software,
}

impl RenderBackend {
    pub fn for_platform(platform: PlatformType) -> Self {
        match platform {
            PlatformType::MacOS | PlatformType::iOS => RenderBackend::Metal,
            PlatformType::Windows | PlatformType::Linux | PlatformType::Android => RenderBackend::Vulkan,
            PlatformType::WASM => RenderBackend::WebGL,
        }
    }

    pub fn supports_compute_shaders(&self) -> bool {
        matches!(self, RenderBackend::Metal | RenderBackend::Vulkan)
    }

    pub fn supports_ray_tracing(&self) -> bool {
        matches!(self, RenderBackend::Vulkan) // Modern Vulkan implementations
    }
}