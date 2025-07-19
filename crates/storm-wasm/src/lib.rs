// File: crates/storm-wasm/src/lib.rs
// WebAssembly bindings for StormCore engine
// Provides browser-compatible API for 3D virtual world clients

use wasm_bindgen::prelude::*;
use web_sys::{console, window, WebGlRenderingContext};
use js_sys::{Promise, Object, Reflect};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use once_cell::sync::Lazy;

use storm_core::{StormCore, StormConfig};
use storm_ecs::{Entity, Component};
use storm_math::{Vec3, Quat, Transform};

// Global allocator for WASM
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

// Global engine instance
static ENGINE: Lazy<Arc<Mutex<Option<StormCore>>>> =
    Lazy::new(|| Arc::new(Mutex::new(None)));

/// Initialize panic hook for better error messages in browser console
#[wasm_bindgen(start)]
pub fn init() {
    console_error_panic_hook::set_once();

    // Initialize tracing for browser console
    tracing_wasm::set_as_global_default();

    console::log_1(&"StormCore WASM initialized".into());
}

/// WASM-compatible configuration structure
#[wasm_bindgen]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmStormConfig {
    enable_rendering: bool,
    enable_audio: bool,
    enable_physics: bool,
    enable_ai: bool,
    debug_mode: bool,
}

#[wasm_bindgen]
impl WasmStormConfig {
    #[wasm_bindgen(constructor)]
    pub fn new() -> WasmStormConfig {
        WasmStormConfig {
            enable_rendering: true,
            enable_audio: true,
            enable_physics: true,
            enable_ai: true,
            debug_mode: true,
        }
    }

    #[wasm_bindgen(getter)]
    pub fn enable_rendering(&self) -> bool { self.enable_rendering }

    #[wasm_bindgen(setter)]
    pub fn set_enable_rendering(&mut self, value: bool) { self.enable_rendering = value; }

    #[wasm_bindgen(getter)]
    pub fn enable_audio(&self) -> bool { self.enable_audio }

    #[wasm_bindgen(setter)]
    pub fn set_enable_audio(&mut self, value: bool) { self.enable_audio = value; }

    #[wasm_bindgen(getter)]
    pub fn enable_physics(&self) -> bool { self.enable_physics }

    #[wasm_bindgen(setter)]
    pub fn set_enable_physics(&mut self, value: bool) { self.enable_physics = value; }

    #[wasm_bindgen(getter)]
    pub fn enable_ai(&self) -> bool { self.enable_ai }

    #[wasm_bindgen(setter)]
    pub fn set_enable_ai(&mut self, value: bool) { self.enable_ai = value; }
}

impl From<WasmStormConfig> for StormConfig {
    fn from(wasm_config: WasmStormConfig) -> Self {
        let mut config = StormConfig::default();
        config.enable_rendering = wasm_config.enable_rendering;
        config.enable_audio = wasm_config.enable_audio;
        config.enable_physics = wasm_config.enable_physics;
        config.enable_ai_enhanced = wasm_config.enable_ai;
        config.debug_mode = wasm_config.debug_mode;

        // WASM-specific overrides
        config.platform = storm_core::PlatformType::WASM;
        config.render_config.backend = storm_core::RenderBackend::WebGL;

        config
    }
}

/// Main WASM engine wrapper
#[wasm_bindgen]
pub struct WasmStormEngine {
    initialized: bool,
}

#[wasm_bindgen]
impl WasmStormEngine {
    #[wasm_bindgen(constructor)]
    pub fn new() -> WasmStormEngine {
        WasmStormEngine {
            initialized: false,
        }
    }

    /// Initialize the engine with configuration
    #[wasm_bindgen]
    pub async fn initialize(&mut self, config: WasmStormConfig) -> Result<(), JsValue> {
        console::log_1(&"Initializing StormCore engine for WASM".into());

        let storm_config: StormConfig = config.into();

        // Create the engine instance
        let engine = StormCore::new(storm_config)
            .await
            .map_err(|e| JsValue::from_str(&format!("Failed to initialize StormCore: {}", e)))?;

        // Store in global state
        let mut global_engine = ENGINE.lock().unwrap();
        *global_engine = Some(engine);

        self.initialized = true;
        console::log_1(&"StormCore engine initialized successfully".into());

        Ok(())
    }

    /// Connect to a virtual world
    #[wasm_bindgen]
    pub async fn connect_to_world(&self, world_url: &str, protocol: &str) -> Result<(), JsValue> {
        if !self.initialized {
            return Err(JsValue::from_str("Engine not initialized"));
        }

        console::log_2(&"Connecting to world:".into(), &world_url.into());

        // Parse protocol type
        let protocol_type = match protocol.to_lowercase().as_str() {
            "opensim" | "mutsea" => storm_core::ProtocolType::OpenSim,
            "finalverse" => storm_core::ProtocolType::Finalverse,
            _ => return Err(JsValue::from_str("Unsupported protocol")),
        };

        let world_config = storm_core::WorldConfig {
            name: "WASM World".to_string(),
            url: world_url.to_string(),
            protocol: protocol_type,
            credentials: None,
        };

        // Connect using global engine
        let engine = ENGINE.lock().unwrap();
        if let Some(ref engine) = *engine {
            engine.connect_to_world(&world_config)
                .await
                .map_err(|e| JsValue::from_str(&format!("Failed to connect: {}", e)))?;
        } else {
            return Err(JsValue::from_str("Engine not available"));
        }

        console::log_1(&"Successfully connected to world".into());
        Ok(())
    }

    /// Update engine (call this from requestAnimationFrame)
    #[wasm_bindgen]
    pub async fn update(&self, delta_time: f32) -> Result<(), JsValue> {
        if !self.initialized {
            return Ok(()); // Silently skip if not initialized
        }

        let engine = ENGINE.lock().unwrap();
        if let Some(ref engine) = *engine {
            engine.update(delta_time)
                .await
                .map_err(|e| JsValue::from_str(&format!("Update failed: {}", e)))?;
        }

        Ok(())
    }

    /// Get engine statistics as JSON
    #[wasm_bindgen]
    pub fn get_stats(&self) -> Result<JsValue, JsValue> {
        if !self.initialized {
            return Err(JsValue::from_str("Engine not initialized"));
        }

        // Create stats object
        let stats = js_sys::Object::new();
        Reflect::set(&stats, &"fps".into(), &60.0.into())?;
        Reflect::set(&stats, &"entities".into(), &0.into())?;
        Reflect::set(&stats, &"memory_mb".into(), &0.0.into())?;

        Ok(stats.into())
    }

    /// Shutdown engine
    #[wasm_bindgen]
    pub async fn shutdown(&mut self) -> Result<(), JsValue> {
        if !self.initialized {
            return Ok(());
        }

        console::log_1(&"Shutting down StormCore engine".into());

        let mut engine = ENGINE.lock().unwrap();
        if let Some(engine) = engine.take() {
            engine.shutdown()
                .await
                .map_err(|e| JsValue::from_str(&format!("Shutdown failed: {}", e)))?;
        }

        self.initialized = false;
        console::log_1(&"StormCore engine shutdown complete".into());

        Ok(())
    }
}

/// WASM-compatible entity representation
#[wasm_bindgen]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmEntity {
    id: u64,
    position: [f32; 3],
    rotation: [f32; 4], // Quaternion
    scale: [f32; 3],
    name: String,
}

#[wasm_bindgen]
impl WasmEntity {
    #[wasm_bindgen(constructor)]
    pub fn new(name: &str) -> WasmEntity {
        WasmEntity {
            id: 0, // Will be set by ECS
            position: [0.0, 0.0, 0.0],
            rotation: [0.0, 0.0, 0.0, 1.0], // Identity quaternion
            scale: [1.0, 1.0, 1.0],
            name: name.to_string(),
        }
    }

    // Position getters/setters
    #[wasm_bindgen(getter)]
    pub fn position(&self) -> Vec<f32> {
        self.position.to_vec()
    }

    #[wasm_bindgen(setter)]
    pub fn set_position(&mut self, pos: &[f32]) {
        if pos.len() >= 3 {
            self.position = [pos[0], pos[1], pos[2]];
        }
    }

    // Rotation getters/setters  
    #[wasm_bindgen(getter)]
    pub fn rotation(&self) -> Vec<f32> {
        self.rotation.to_vec()
    }

    #[wasm_bindgen(setter)]
    pub fn set_rotation(&mut self, rot: &[f32]) {
        if rot.len() >= 4 {
            self.rotation = [rot[0], rot[1], rot[2], rot[3]];
        }
    }

    // Scale getters/setters
    #[wasm_bindgen(getter)]
    pub fn scale(&self) -> Vec<f32> {
        self.scale.to_vec()
    }

    #[wasm_bindgen(setter)]
    pub fn set_scale(&mut self, scale: &[f32]) {
        if scale.len() >= 3 {
            self.scale = [scale[0], scale[1], scale[2]];
        }
    }

    #[wasm_bindgen(getter)]
    pub fn name(&self) -> String {
        self.name.clone()
    }

    #[wasm_bindgen(setter)]
    pub fn set_name(&mut self, name: &str) {
        self.name = name.to_string();
    }
}

/// Utility functions for WASM environment
#[wasm_bindgen]
pub fn get_performance_now() -> f64 {
    window()
        .unwrap()
        .performance()
        .unwrap()
        .now()
}

#[wasm_bindgen]
pub fn log_to_console(message: &str) {
    console::log_1(&message.into());
}

#[wasm_bindgen]
pub fn get_canvas_context(canvas_id: &str) -> Result<WebGlRenderingContext, JsValue> {
    let document = window().unwrap().document().unwrap();
    let canvas = document
        .get_element_by_id(canvas_id)
        .unwrap()
        .dyn_into::<web_sys::HtmlCanvasElement>()?;

    canvas
        .get_context("webgl")?
        .unwrap()
        .dyn_into::<WebGlRenderingContext>()
        .map_err(|_| JsValue::from_str("Failed to get WebGL context"))
}

/// Export version information
#[wasm_bindgen]
pub fn get_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}