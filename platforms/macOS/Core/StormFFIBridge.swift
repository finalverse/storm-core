// platforms/macos/Storm/Core/StormFFIBridge.swift
// Description: Bridge to Rust FFI for avatar and NPC operations.
// Summary: Defines Swift wrappers for Rust FFI functions, handling creation and customization. Acts as intermediary between Swift callers (e.g., StormEngine) and Rust callees.
// Logic: Uses @convention(c) for C-compatible closures if needed; direct calls to bridged functions. Error-free: Checks return codes; assumes bridging header is set. Compiles with proper header.
// Progressive: Add after bridging header; test with dummy calls to ensure linking.

import Foundation

class StormFFIBridge {
    static func initializeECS() -> Bool {
        // Caller: StormEngine init; callee: Rust init_ecs_world.
        return init_ecs_world() == 0
    }
    
    static func createAvatar(x: Float, y: Float, z: Float) -> UInt64 {
        // Caller: StormEngine; callee: Rust ffi_create_avatar.
        return ffi_create_avatar(x, y, z)
    }
    
    static func customizeAvatar(entityId: UInt64, traitIndex: Int32, morph: Float) -> Bool {
        // Caller: AvatarCustomizationView; callee: Rust ffi_customize_avatar.
        return ffi_customize_avatar(entityId, traitIndex, morph) == 0
    }
    
    static func createNPC(echoIndex: Int32, x: Float, y: Float, z: Float) -> UInt64 {
        // Caller: StormEngine; callee: Rust ffi_create_npc.
        return ffi_create_npc(echoIndex, x, y, z)
    }
}


