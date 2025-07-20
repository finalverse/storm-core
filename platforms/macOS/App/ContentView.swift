// platforms/macOS/Storm/ContentView.swift
// Description: Main UI for avatar customization and 3D scene.
// Summary: SwiftUI controls for sliders/traits; RealityKit view for 3D rendering.
// Similar to hyper3d.ai: Basic 3D preview with AI stubs.
// Logic: Calls FFI to create/customize (callers to Rust); syncs with RealityKit entities.
//        Physics: Native RealityKit for collisions. Audio: Spatial playback with correct configuration.
//        Error-free: Fixed deprecation by using onChange(of:) with no-param closure (macOS 14+ compatible).
//                  For linker error ('Library storm_ffi not found'): This is an Xcode config issue, not code.
//                  Fix progressively: 1. Ensure libstorm_ffi.dylib is in target/release. 2. In Xcode > Project > Build Phases > Link Binary With Libraries, add the .dylib. 3. Set Build Settings > Search Paths > Library Search Paths to "$(SRCROOT)/../../../target/release" (relative to project). 4. Embed & Sign the dylib in General > Frameworks. 5. Clean & Rebuild. Callers (Swift onChange) remain connected to callees (Rust ffi_customize_avatar).
//        Progressive: Builds on previous; test after Xcode config – app links successfully, no deprecation warnings.

import SwiftUI
import RealityKit

struct ContentView: View {
    @State private var morphValue: Float = 0.5
    @State private var traitIndex: Int = 0
    @State private var avatarId: UInt64 = 0
    @State private var npcId: UInt64 = 0
    @State private var errorMessage: String = ""

    var body: some View {
        VStack {
            // 3D View.
            RealityView { content in
                // Init Rust ECS via FFI.
                if init_ecs_world() != 0 {
                    errorMessage = "ECS init failed"
                    return
                }
                
                // Create avatar entity via FFI.
                avatarId = ffi_create_avatar(0.0, 0.0, -5.0)
                if avatarId == 0 {
                    errorMessage = "Avatar creation failed"
                    return
                }
                
                npcId = ffi_create_npc(0, 5.0, 0.0, -5.0) // Lumi NPC.
                if npcId == 0 {
                    errorMessage = "NPC creation failed"
                    return
                }
                
                // Create RealityKit entities (placeholders; integrate Rust meshes in future).
                let avatarEntity = Entity()
                avatarEntity.position = SIMD3<Float>(0, 0, -5)
                // Caller: ModelEntity init; Callee: generateSphere for mesh.
                let avatarModel = ModelEntity(mesh: .generateSphere(radius: 1.0), materials: [SimpleMaterial(color: .blue, isMetallic: false)]) // Fixed isMetallic.
                avatarEntity.addChild(avatarModel)
                
                // Physics: Enable body and motion for interaction.
                avatarEntity.components.set(PhysicsBodyComponent(massProperties: .default, material: .default, mode: .dynamic))
                avatarEntity.components.set(CollisionComponent(shapes: [.generateSphere(radius: 1.0)]))
                
                let npcEntity = Entity()
                npcEntity.position = SIMD3<Float>(5, 0, -5)
                // Caller: ModelEntity init; Callee: generateBox for mesh.
                let npcModel = ModelEntity(mesh: .generateBox(width: 1.0, height: 2.0, depth: 1.0), materials: [SimpleMaterial(color: .red, isMetallic: false)]) // Fixed isMetallic.
                npcEntity.addChild(npcModel)
                
                // Physics for NPC.
                npcEntity.components.set(PhysicsBodyComponent(massProperties: .default, material: .default, mode: .kinematic))
                npcEntity.components.set(CollisionComponent(shapes: [.generateBox(width: 1.0, height: 2.0, depth: 1.0)]))
                
                // Audio: Spatial sound for NPC dialogue (native RealityKit).
                // Caller: try? AudioFileResource.load; Callee: prepareAudio on npcEntity.
                if let resource = try? AudioFileResource.load(named: "/System/Library/Sounds/Submarine.aiff", configuration: AudioFileResource.Configuration(shouldLoop: false)) { // Fixed with AudioFileResource.Configuration.
                    let controller = npcEntity.prepareAudio(resource)
                    controller.play()
                }
                
                content.add(avatarEntity)
                content.add(npcEntity)
                
                // Simulate AI movement (NPC approaches avatar with physics).
                let motion = PhysicsMotionComponent(linearVelocity: SIMD3<Float>(-1, 0, 0))
                npcEntity.components.set(motion)
            }
            .frame(height: 400)
            
            // Customization Controls.
            Slider(value: $morphValue, in: 0...1) {
                Text("Morph Body")
            }.onChange(of: morphValue) { // Fixed deprecation: Use no-param closure (ignores old value).
                if ffi_customize_avatar(avatarId, Int32(traitIndex), morphValue) != 0 {
                    errorMessage = "Customization failed"
                }
                // Refresh RealityKit (stub: scale; full would sync from Rust).
            }
            
            Picker("Trait", selection: $traitIndex) {
                Text("Hope").tag(0)
                Text("Logic").tag(1)
            }
            
            Button("Generate Companion (AI Stub)") {
                // Stub for AI generation; calls Grok or storm-ai in full.
                print("AI generating companion...")
            }
            
            Text(errorMessage).foregroundColor(.red)
        }
        .padding()
    }
}
