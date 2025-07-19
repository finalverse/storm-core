// File: crates/storm-ffi/src/lib.rs
// FFI bindings infrastructure for StormCore
// Provides C-compatible interface for cross-language integration

use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_void};
use std::ptr;
use std::sync::Arc;
use tokio::runtime::Runtime;
use once_cell::sync::Lazy;

use storm_core::{StormCore, StormConfig, StormResult, init_logging};
use storm_ecs::{EntityId, Transform as EcsTransform}; // Use ECS Transform, not math Transform
use storm_math::{Vec3, Quat}; // Import math types for conversion

mod handle;
mod error;
mod callbacks;

pub use handle::*;
pub use error::*;
pub use callbacks::*;

// Global runtime for async operations
static RUNTIME: Lazy<Runtime> = Lazy::new(|| {
    Runtime::new().expect("Failed to create Tokio runtime")
});

/// Opaque handle to StormCore engine instance
#[repr(C)]
pub struct StormHandle {
    ptr: *mut c_void,
}

/// C-compatible configuration structure
#[repr(C)]
pub struct CStormConfig {
    pub enable_rendering: bool,
    pub enable_audio: bool,
    pub enable_physics: bool,
    pub enable_ai: bool,
    pub debug_mode: bool,
    pub platform: u32, // Platform enum as integer
}

/// C-compatible world configuration
#[repr(C)]
pub struct CWorldConfig {
    pub name: *const c_char,
    pub url: *const c_char,
    pub protocol: u32, // Protocol enum as integer
}

/// C-compatible vector3
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CVec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

/// C-compatible quaternion
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CQuat {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

/// C-compatible transform
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CTransform {
    pub position: CVec3,
    pub rotation: CQuat,
    pub scale: CVec3,
}

/// Error codes for C interface
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StormErrorCode {
    Success = 0,
    InitializationFailed = 1,
    InvalidHandle = 2,
    InvalidParameter = 3,
    NetworkError = 4,
    ProtocolError = 5,
    EcsError = 6,
    AiError = 7,
    RenderingError = 8,
    AudioError = 9,
    PhysicsError = 10,
    AssetError = 11,
    PlatformNotSupported = 12,
    GenericError = 99,
}

/// Callback function types
pub type ErrorCallback = extern "C" fn(error_code: StormErrorCode, message: *const c_char, user_data: *mut c_void);
pub type EntityCallback = extern "C" fn(entity_id: u64, user_data: *mut c_void);

/// Initialize the StormCore engine
///
/// # Safety
/// This function is safe to call multiple times, but only the first call will have effect.
#[no_mangle]
pub unsafe extern "C" fn storm_init_logging() {
    init_logging();
}

/// Create a new StormCore engine instance
///
/// # Safety
/// The returned handle must be freed with `storm_free_handle`.
/// The config pointer must be valid for the duration of this call.
#[no_mangle]
pub unsafe extern "C" fn storm_create_engine(config: *const CStormConfig) -> *mut StormHandle {
    if config.is_null() {
        return ptr::null_mut();
    }

    let c_config = &*config;
    let storm_config = StormConfig {
        enable_rendering: c_config.enable_rendering,
        enable_audio: c_config.enable_audio,
        enable_physics: c_config.enable_physics,
        enable_ai_enhanced: c_config.enable_ai,
        debug_mode: c_config.debug_mode,
        platform: platform_from_u32(c_config.platform),
        ..Default::default()
    };

    match RUNTIME.block_on(StormCore::new(storm_config)) {
        Ok(core) => {
            let handle = Box::new(StormHandle {
                ptr: Box::into_raw(Box::new(core)) as *mut c_void,
            });
            Box::into_raw(handle)
        }
        Err(_) => ptr::null_mut(),
    }
}

/// Free a StormCore engine handle
///
/// # Safety
/// The handle must be a valid pointer returned by `storm_create_engine`.
/// After calling this function, the handle must not be used again.
#[no_mangle]
pub unsafe extern "C" fn storm_free_handle(handle: *mut StormHandle) {
    if !handle.is_null() {
        let handle_box = Box::from_raw(handle);
        if !handle_box.ptr.is_null() {
            let _core_box = Box::from_raw(handle_box.ptr as *mut StormCore);
        }
    }
}

/// Connect to a virtual world
///
/// # Safety
/// Handle must be valid. World config strings must be null-terminated and valid UTF-8.
#[no_mangle]
pub unsafe extern "C" fn storm_connect_to_world(
    handle: *mut StormHandle,
    world_config: *const CWorldConfig,
) -> StormErrorCode {
    if handle.is_null() || world_config.is_null() {
        return StormErrorCode::InvalidParameter;
    }

    let handle_ref = &*handle;
    if handle_ref.ptr.is_null() {
        return StormErrorCode::InvalidHandle;
    }

    let core = &*(handle_ref.ptr as *const StormCore);
    let c_world_config = &*world_config;

    // Convert C strings to Rust strings
    let name = match CStr::from_ptr(c_world_config.name).to_str() {
        Ok(s) => s.to_string(),
        Err(_) => return StormErrorCode::InvalidParameter,
    };

    let url = match CStr::from_ptr(c_world_config.url).to_str() {
        Ok(s) => s.to_string(),
        Err(_) => return StormErrorCode::InvalidParameter,
    };

    let protocol = protocol_from_u32(c_world_config.protocol);

    let world_config = storm_core::core::WorldConfig {
        name,
        url,
        protocol,
        credentials: None,
    };

    match RUNTIME.block_on(core.connect_to_world(&world_config)) {
        Ok(_) => StormErrorCode::Success,
        Err(e) => error_from_storm_error(e),
    }
}

/// Update the engine (call once per frame)
///
/// # Safety
/// Handle must be valid.
#[no_mangle]
pub unsafe extern "C" fn storm_update(handle: *mut StormHandle, delta_time: f32) -> StormErrorCode {
    if handle.is_null() {
        return StormErrorCode::InvalidHandle;
    }

    let handle_ref = &*handle;
    if handle_ref.ptr.is_null() {
        return StormErrorCode::InvalidHandle;
    }

    let core = &*(handle_ref.ptr as *const StormCore);

    match RUNTIME.block_on(core.update(delta_time)) {
        Ok(_) => StormErrorCode::Success,
        Err(e) => error_from_storm_error(e),
    }
}

/// Shutdown the engine gracefully
///
/// # Safety
/// Handle must be valid.
#[no_mangle]
pub unsafe extern "C" fn storm_shutdown(handle: *mut StormHandle) -> StormErrorCode {
    if handle.is_null() {
        return StormErrorCode::InvalidHandle;
    }

    let handle_ref = &*handle;
    if handle_ref.ptr.is_null() {
        return StormErrorCode::InvalidHandle;
    }

    let core = &*(handle_ref.ptr as *const StormCore);

    match RUNTIME.block_on(core.shutdown()) {
        Ok(_) => StormErrorCode::Success,
        Err(e) => error_from_storm_error(e),
    }
}

/// Create a new entity in the ECS world
///
/// # Safety
/// Handle must be valid.
#[no_mangle]
pub unsafe extern "C" fn storm_create_entity(handle: *mut StormHandle) -> u64 {
    if handle.is_null() {
        return 0;
    }

    let handle_ref = &*handle;
    if handle_ref.ptr.is_null() {
        return 0;
    }

    let core = &*(handle_ref.ptr as *const StormCore);
    let world_arc = core.ecs_world();

    match RUNTIME.block_on(async {
        let mut world = world_arc.write().await;
        let entity = world.create_entity();
        entity.id
    }) {
        id => id,
    }
}

/// Set entity transform
///
/// # Safety
/// Handle must be valid, entity_id must be valid.
#[no_mangle]
pub unsafe extern "C" fn storm_set_entity_transform(
    handle: *mut StormHandle,
    entity_id: u64,
    transform: *const CTransform,
) -> StormErrorCode {
    if handle.is_null() || transform.is_null() {
        return StormErrorCode::InvalidParameter;
    }

    let handle_ref = &*handle;
    if handle_ref.ptr.is_null() {
        return StormErrorCode::InvalidHandle;
    }

    let core = &*(handle_ref.ptr as *const StormCore);
    let world_arc = core.ecs_world();
    let c_transform = &*transform;

    // Convert CTransform to ECS Transform
    let ecs_transform = EcsTransform {
        position: [c_transform.position.x, c_transform.position.y, c_transform.position.z],
        rotation: [c_transform.rotation.x, c_transform.rotation.y, c_transform.rotation.z, c_transform.rotation.w],
        scale: [c_transform.scale.x, c_transform.scale.y, c_transform.scale.z],
    };

    RUNTIME.block_on(async {
        let mut world = world_arc.write().await;
        let entity = storm_ecs::Entity::new(entity_id, 0);
        world.add_component(entity, ecs_transform);
    });

    StormErrorCode::Success
}

/// Get entity transform
///
/// # Safety
/// Handle must be valid, entity_id must be valid, out_transform must be a valid pointer.
#[no_mangle]
pub unsafe extern "C" fn storm_get_entity_transform(
    handle: *mut StormHandle,
    entity_id: u64,
    out_transform: *mut CTransform,
) -> StormErrorCode {
    if handle.is_null() || out_transform.is_null() {
        return StormErrorCode::InvalidParameter;
    }

    let handle_ref = &*handle;
    if handle_ref.ptr.is_null() {
        return StormErrorCode::InvalidHandle;
    }

    let core = &*(handle_ref.ptr as *const StormCore);
    let world_arc = core.ecs_world();

    let result = RUNTIME.block_on(async {
        let world = world_arc.read().await;
        let entity = storm_ecs::Entity::new(entity_id, 0);
        world.get_component::<EcsTransform>(entity).cloned() // Clone to avoid borrowing issues
    });

    if let Some(transform) = result {
        let c_transform = &mut *out_transform;
        c_transform.position = CVec3 {
            x: transform.position[0],
            y: transform.position[1],
            z: transform.position[2],
        };
        c_transform.rotation = CQuat {
            x: transform.rotation[0],
            y: transform.rotation[1],
            z: transform.rotation[2],
            w: transform.rotation[3],
        };
        c_transform.scale = CVec3 {
            x: transform.scale[0],
            y: transform.scale[1],
            z: transform.scale[2],
        };
        StormErrorCode::Success
    } else {
        StormErrorCode::EcsError
    }
}

/// Get engine version string
///
/// # Safety
/// Returns a static string that doesn't need to be freed.
#[no_mangle]
pub unsafe extern "C" fn storm_get_version() -> *const c_char {
    static VERSION: &[u8] = concat!(env!("CARGO_PKG_VERSION"), "\0").as_bytes();
    VERSION.as_ptr() as *const c_char
}

/// Get last error message
///
/// # Safety
/// Returns a pointer to internal storage that may be invalidated by subsequent calls.
#[no_mangle]
pub unsafe extern "C" fn storm_get_last_error() -> *const c_char {
    // In a real implementation, this would return the last error message
    static ERROR_MSG: &[u8] = b"No error\0";
    ERROR_MSG.as_ptr() as *const c_char
}

// Helper functions for conversions

fn platform_from_u32(value: u32) -> storm_core::PlatformType {
    match value {
        0 => storm_core::PlatformType::MacOS,
        1 => storm_core::PlatformType::iOS,
        2 => storm_core::PlatformType::Windows,
        3 => storm_core::PlatformType::Linux,
        4 => storm_core::PlatformType::Android,
        5 => storm_core::PlatformType::WASM,
        _ => storm_core::PlatformType::detect(),
    }
}

fn protocol_from_u32(value: u32) -> storm_core::ProtocolType {
    match value {
        0 => storm_core::ProtocolType::OpenSim,
        1 => storm_core::ProtocolType::Finalverse,
        _ => storm_core::ProtocolType::OpenSim,
    }
}

fn error_from_storm_error(error: storm_core::StormError) -> StormErrorCode {
    use storm_core::StormError;

    match error {
        StormError::InitializationError(_) => StormErrorCode::InitializationFailed,
        StormError::NetworkError(_) => StormErrorCode::NetworkError,
        StormError::ProtocolError(_) => StormErrorCode::ProtocolError,
        StormError::EcsError(_) => StormErrorCode::EcsError,
        StormError::AiError(_) => StormErrorCode::AiError,
        StormError::RenderingError(_) => StormErrorCode::RenderingError,
        StormError::AudioError(_) => StormErrorCode::AudioError,
        StormError::PhysicsError(_) => StormErrorCode::PhysicsError,
        StormError::AssetError(_) => StormErrorCode::AssetError,
        StormError::PlatformNotSupported(_) => StormErrorCode::PlatformNotSupported,
        _ => StormErrorCode::GenericError,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_string() {
        unsafe {
            let version = storm_get_version();
            assert!(!version.is_null());

            let version_str = CStr::from_ptr(version);
            assert!(version_str.to_str().is_ok());
        }
    }

    #[test]
    fn test_platform_conversion() {
        assert_eq!(platform_from_u32(0), storm_core::PlatformType::MacOS);
        assert_eq!(platform_from_u32(1), storm_core::PlatformType::iOS);
        assert_eq!(platform_from_u32(999), storm_core::PlatformType::detect());
    }
}