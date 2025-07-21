// File: BuildFixes.swift
// Description: Common build fixes and compatibility helpers for Storm macOS
// Add this file to your project to resolve common compilation issues

import Foundation
import SwiftUI

// MARK: - Missing Types Compatibility

// If you get errors about missing NetworkStatus, add this:
enum NetworkStatus: Equatable {
    case disconnected
    case connecting
    case ready
    case connected(String)
    case error(String)
    
    var color: Color {
        switch self {
        case .disconnected: return .gray
        case .connecting: return .yellow
        case .ready: return .blue
        case .connected: return .green
        case .error: return .red
        }
    }
    
    var displayText: String {
        switch self {
        case .disconnected: return "Disconnected"
        case .connecting: return "Connecting..."
        case .ready: return "Ready"
        case .connected(let worldName): return "Connected to \(worldName)"
        case .error(let message): return "Error: \(message)"
        }
    }
}

// If you get errors about missing AvatarTrait, add this:
enum AvatarTrait: Int32, CaseIterable {
    case height = 0
    case bodyMass = 1
    case skinTone = 2
    case eyeColor = 3
    case hairColor = 4
    case faceShape = 5
    case musculature = 6
    case accessories = 7
}

// If you get errors about missing NPCType, add this:
enum NPCType: Int32, CaseIterable {
    case lumi = 0      // Echo of Hope
    case sage = 1      // Echo of Wisdom
    case guardian = 2  // Echo of Protection
    case explorer = 3  // Echo of Discovery
}

// If you get errors about missing AvatarCustomization, add this:
struct AvatarCustomization {
    var traits: [AvatarTrait: Float] = [:]
    
    static let `default` = AvatarCustomization(traits: [
        .height: 0.0,
        .bodyMass: 0.0,
        .skinTone: 0.5,
        .eyeColor: 0.5
    ])
    
    mutating func updateTrait(_ trait: AvatarTrait, value: Float) {
        traits[trait] = value
    }
}

// MARK: - FFI Function Stubs

// If you get linker errors about missing FFI functions, add these stubs temporarily:

@_silgen_name("init_ecs_world")
func init_ecs_world() -> Int32 {
    // Stub implementation - replace with actual FFI when Rust library is ready
    print("⚠️ Using stub: init_ecs_world")
    return 0 // Success
}

@_silgen_name("ffi_create_avatar")
func ffi_create_avatar(_ x: Float, _ y: Float, _ z: Float) -> UInt64 {
    print("⚠️ Using stub: ffi_create_avatar at (\(x), \(y), \(z))")
    return UInt64.random(in: 1...1000000) // Return fake ID
}

@_silgen_name("ffi_customize_avatar")
func ffi_customize_avatar(_ entityId: UInt64, _ traitIndex: Int32, _ morph: Float) -> Int32 {
    print("⚠️ Using stub: ffi_customize_avatar entity:\(entityId) trait:\(traitIndex) value:\(morph)")
    return 0 // Success
}

@_silgen_name("ffi_create_npc")
func ffi_create_npc(_ echoIndex: Int32, _ x: Float, _ y: Float, _ z: Float) -> UInt64 {
    print("⚠️ Using stub: ffi_create_npc type:\(echoIndex) at (\(x), \(y), \(z))")
    return UInt64.random(in: 1000001...2000000) // Return fake ID
}

@_silgen_name("ffi_set_entity_position")
func ffi_set_entity_position(_ entityId: UInt64, _ x: Float, _ y: Float, _ z: Float, _ usePhysics: Int32) -> Int32 {
    print("⚠️ Using stub: ffi_set_entity_position entity:\(entityId) to (\(x), \(y), \(z))")
    return 0 // Success
}

@_silgen_name("storm_core_get_version")
func storm_core_get_version() -> UnsafePointer<Int8>? {
    print("⚠️ Using stub: storm_core_get_version")
    return "1.0.0-stub".withCString { $0 }
}

// MARK: - Build Configuration Help

struct BuildConfiguration {
    static func printSetupInstructions() {
        print("""
        
        🌪️ STORM BUILD CONFIGURATION
        
        If you're seeing compilation errors, follow these steps:
        
        1. REMOVE ARKit references:
           - ARKit is iOS-only, not needed for macOS
           - Remove 'import ARKit' from WorldRenderView.swift
           
        2. ENSURE FFI stubs are active:
           - Add BuildFixes.swift to your project
           - These provide placeholder functions until Rust library is ready
           
        3. CHECK framework linking:
           - SwiftUI.framework ✓
           - RealityKit.framework ✓  
           - Metal.framework ✓
           - Combine.framework ✓
           
        4. VERIFY build settings:
           - Deployment Target: macOS 14.0+
           - Swift Language Version: 5.9+
           - Enable RealityKit: Yes
           
        5. RUST LIBRARY (when ready):
           - Build: cargo build --release --target aarch64-apple-darwin
           - Copy: libstorm_ffi.dylib to Frameworks/
           - Link: Add to Build Phases -> Link Binary With Libraries
           
        Remove the stub functions in BuildFixes.swift when your Rust library is ready!
        
        """)
    }
}

// MARK: - Compatibility Extensions

extension Color {
    static func systemColor(_ name: String) -> Color {
        switch name {
        case "gray": return .gray
        case "blue": return .blue
        case "green": return .green
        case "yellow": return .yellow
        case "red": return .red
        case "orange": return .orange
        case "purple": return .purple
        default: return .primary
        }
    }
}

// Call this in your App's init to see setup instructions
extension App {
    static func printStormSetup() {
        BuildConfiguration.printSetupInstructions()
    }
}
