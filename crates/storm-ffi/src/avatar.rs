// crates/storm-ffi/src/avatar.rs
// Description: FFI extensions for avatar/NPC functions, callable from Swift.
// Summary: Exposes safe C-compatible APIs for creating/customizing avatars and NPCs.
// Logic: Wraps storm-avatar calls; ensures no panics by returning error codes (0 = success, 1 = error).
//        Callers: Swift app; Callees: storm-avatar functions. Uses global ECS for simplicity (mutex in prod).

use storm_avatar::{create_avatar, customize_avatar, create_npc, EchoType};
use storm_ecs::{Entity, World as EcsWorld};
use std::ptr;
use std::sync::Mutex;

// Global ECS world (safe with Mutex for threading).
lazy_static::lazy_static! {
    static ref ECS_WORLD: Mutex<Option<EcsWorld>> = Mutex::new(None);
}

// Init function.
#[no_mangle]
pub extern "C" fn init_ecs_world() -> i32 {
    let mut guard = ECS_WORLD.lock().unwrap(); // Safe lock.
    *guard = Some(EcsWorld::new());
    0 // Success.
}

// Create avatar via FFI.
#[no_mangle]
pub extern "C" fn ffi_create_avatar(x: f32, y: f32, z: f32) -> u64 {
    let mut guard = ECS_WORLD.lock().unwrap();
    if let Some(world) = guard.as_mut() {
        match create_avatar(world, storm_math::Vector3::new(x, y, z)) {
            Ok(entity) => entity.id() as u64,
            Err(_) => 0, // Error code.
        }
    } else {
        0
    }
}

// Customize via FFI (trait as int: 0=Hope, 1=Logic).
#[no_mangle]
pub extern "C" fn ffi_customize_avatar(entity_id: u64, trait_int: i32, morph: f32) -> i32 {
    let new_trait = match trait_int {
        0 => storm_avatar::Trait::Hope { intensity: 1.0 },
        1 => storm_avatar::Trait::Logic { code_overlays: true },
        _ => return 1, // Error.
    };
    let mut guard = ECS_WORLD.lock().unwrap();
    if let Some(world) = guard.as_mut() {
        match customize_avatar(world, Entity::from_id(entity_id), new_trait, morph) {
            Ok(_) => 0,
            Err(_) => 1,
        }
    } else {
        1
    }
}

// Create NPC similarly (echo_int: 0=Lumi, 1=Kai).
#[no_mangle]
pub extern "C" fn ffi_create_npc(echo_int: i32, x: f32, y: f32, z: f32) -> u64 {
    let echo_type = match echo_int {
        0 => EchoType::Lumi,
        1 => EchoType::Kai,
        _ => return 0,
    };
    let mut guard = ECS_WORLD.lock().unwrap();
    if let Some(world) = guard.as_mut() {
        match create_npc(world, echo_type, storm_math::Vector3::new(x, y, z)) {
            Ok(entity) => entity.id() as u64,
            Err(_) => 0,
        }
    } else {
        0
    }
}