// File: platforms/macOS/Storm/Views/WorldRenderView.swift
// Description: Main 3D world rendering view using RealityKit
// Integrates with Storm core for entity management, physics, and AI-driven content

import SwiftUI
import RealityKit
import Combine
import os

struct WorldRenderView: View {
    @EnvironmentObject var stormEngine: StormEngine
    @StateObject private var renderSystem = StormRenderSystem()
    @State private var selectedEntity: Entity?
    @State private var cameraPosition = simd_float3(0, 2, 5)
    @State private var cameraTarget = simd_float3(0, 0, 0)
    @State private var cameraRotation: Float = 0
    
    private let logger = Logger(subsystem: "com.storm.client", category: "WorldRender")
    
    var body: some View {
        ZStack(alignment: .topTrailing) {
            // Main RealityKit view
            RealityView { content in
                await setupScene(content: content)
            } update: { content in
                await updateScene(content: content)
            }
            .gesture(
                DragGesture()
                    .onChanged { value in
                        handleCameraMovement(value)
                    }
            )
            .onTapGesture { location in
                handleWorldTap(at: location)
            }
            
            // Overlay controls
            VStack(alignment: .trailing, spacing: 12) {
                // Camera controls
                CameraControlPanel(
                    position: $cameraPosition,
                    target: $cameraTarget,
                    rotation: $cameraRotation
                )
                
                // Entity inspector
                if let entity = selectedEntity {
                    EntityInspectorPanel(entity: entity)
                        .transition(.opacity)
                }
                
                Spacer()
                
                // World actions
                WorldActionPanel()
                    .environmentObject(stormEngine)
            }
            .padding()
        }
        .navigationTitle("World View")
        .toolbar {
            ToolbarItem(placement: .primaryAction) {
                Menu {
                    Button("Add Avatar") {
                        Task { await createNewAvatar() }
                    }
                    Button("Add NPC") {
                        Task { await createNewNPC() }
                    }
                    Button("Add Object") {
                        Task { await createNewObject() }
                    }
                    Divider()
                    Button("Reset Camera") {
                        resetCamera()
                    }
                    Button("Toggle Physics Debug") {
                        renderSystem.togglePhysicsDebug()
                    }
                } label: {
                    Image(systemName: "plus.circle")
                }
            }
        }
        .onAppear {
            setupRenderSystem()
        }
    }
    
    // MARK: - Scene Setup
    
    @MainActor
    private func setupScene(content: RealityViewContent) async {
        logger.info("Setting up RealityKit scene")
        
        // Initialize ECS via FFI
        let ecsResult = init_ecs_world()
        if ecsResult != 0 {
            logger.error("Failed to initialize ECS world")
            return
        }
        
        // Create world environment
        await createWorldEnvironment(content: content)
        
        // Create default avatar
        await createDefaultAvatar(content: content)
        
        // Create sample NPCs
        await createSampleNPCs(content: content)
        
        // Setup lighting and atmosphere
        setupLighting(content: content)
        
        // Initialize physics world
        setupPhysics(content: content)
        
        logger.info("RealityKit scene setup complete")
    }
    
    @MainActor
    private func updateScene(content: RealityViewContent) async {
        // Update entities based on Storm core state
        await updateEntitiesFromCore(content: content)
        
        // Update AI-driven behaviors
        await updateAIBehaviors(content: content)
        
        // Update physics simulations
        updatePhysics(content: content)
    }
    
    // MARK: - World Environment
    
    private func createWorldEnvironment(content: RealityViewContent) async {
        // Create ground plane
        let groundEntity = Entity()
        groundEntity.name = "Ground"
        
        let groundMesh = MeshResource.generatePlane(width: 100, depth: 100)
        let groundMaterial = SimpleMaterial(color: .init(red: 0.2, green: 0.6, blue: 0.2, alpha: 1.0), isMetallic: false)
        let groundModel = ModelEntity(mesh: groundMesh, materials: [groundMaterial])
        
        groundEntity.addChild(groundModel)
        groundEntity.position = simd_float3(0, 0, 0)
        
        // Add physics body for ground
        let groundShape = ShapeResource.generateStaticMesh(from: groundMesh)
        groundEntity.components.set(CollisionComponent(shapes: [groundShape]))
        groundEntity.components.set(PhysicsBodyComponent(
            massProperties: .default,
            material: .default,
            mode: .static
        ))
        
        content.add(groundEntity)
        
        // Create sky dome
        await createSkyDome(content: content)
        
        // Add procedural trees and objects
        await createProceduralEnvironment(content: content)
    }
    
    private func createSkyDome(content: RealityViewContent) async {
        let skyEntity = Entity()
        skyEntity.name = "Sky"
        
        // Create a large sphere for sky
        let skyMesh = MeshResource.generateSphere(radius: 500)
        var skyMaterial = UnlitMaterial()
        skyMaterial.color = .init(tint: .init(red: 0.5, green: 0.7, blue: 1.0, alpha: 1.0))
        
        let skyModel = ModelEntity(mesh: skyMesh, materials: [skyMaterial])
        skyModel.scale = simd_float3(-1, 1, -1) // Invert to show inner surface
        
        skyEntity.addChild(skyModel)
        content.add(skyEntity)
    }
    
    private func createProceduralEnvironment(content: RealityViewContent) async {
        // Create random trees and rocks using AI-driven placement
        for i in 0..<20 {
            let randomX = Float.random(in: -40...40)
            let randomZ = Float.random(in: -40...40)
            
            if i % 3 == 0 {
                await createTree(at: simd_float3(randomX, 0, randomZ), content: content)
            } else {
                await createRock(at: simd_float3(randomX, 0, randomZ), content: content)
            }
        }
    }
    
    private func createTree(at position: simd_float3, content: RealityViewContent) async {
        let treeEntity = Entity()
        treeEntity.name = "Tree"
        treeEntity.position = position
        
        // Trunk
        let trunkMesh = MeshResource.generateCylinder(height: 3, radius: 0.2)
        let trunkMaterial = SimpleMaterial(color: .init(red: 0.4, green: 0.2, blue: 0.1, alpha: 1.0), isMetallic: false)
        let trunkModel = ModelEntity(mesh: trunkMesh, materials: [trunkMaterial])
        trunkModel.position.y = 1.5
        
        // Leaves
        let leavesMesh = MeshResource.generateSphere(radius: 2)
        let leavesMaterial = SimpleMaterial(color: .init(red: 0.1, green: 0.8, blue: 0.1, alpha: 1.0), isMetallic: false)
        let leavesModel = ModelEntity(mesh: leavesMesh, materials: [leavesMaterial])
        leavesModel.position.y = 4
        
        treeEntity.addChild(trunkModel)
        treeEntity.addChild(leavesModel)
        
        // Add collision for trunk
        treeEntity.components.set(CollisionComponent(shapes: [.generateCylinder(height: 3, radius: 0.2)]))
        treeEntity.components.set(PhysicsBodyComponent(mode: .static))
        
        content.add(treeEntity)
    }
    
    private func createRock(at position: simd_float3, content: RealityViewContent) async {
        let rockEntity = Entity()
        rockEntity.name = "Rock"
        rockEntity.position = position
        
        let size = Float.random(in: 0.5...1.5)
        let rockMesh = MeshResource.generateSphere(radius: size)
        let rockMaterial = SimpleMaterial(color: .init(red: 0.5, green: 0.5, blue: 0.5, alpha: 1.0), isMetallic: false)
        let rockModel = ModelEntity(mesh: rockMesh, materials: [rockMaterial])
        rockModel.position.y = size
        
        rockEntity.addChild(rockModel)
        
        // Add physics
        rockEntity.components.set(CollisionComponent(shapes: [.generateSphere(radius: size)]))
        rockEntity.components.set(PhysicsBodyComponent(
            massProperties: .default,
            material: .default,
            mode: .dynamic
        ))
        
        content.add(rockEntity)
    }
    
    // MARK: - Avatar and NPC Creation
    
    private func createDefaultAvatar(content: RealityViewContent) async {
        do {
            let avatarId = try await stormEngine.createAvatar(at: simd_float3(0, 1, -5))
            await createAvatarEntity(id: avatarId, at: simd_float3(0, 1, -5), content: content)
        } catch {
            logger.error("Failed to create default avatar: \(error)")
        }
    }
    
    private func createAvatarEntity(id: UInt64, at position: simd_float3, content: RealityViewContent) async {
        let avatarEntity = Entity()
        avatarEntity.name = "Avatar_\(id)"
        avatarEntity.position = position
        
        // Create avatar model (simplified humanoid)
        let bodyMesh = MeshResource.generateCylinder(height: 1.8, radius: 0.3)
        let bodyMaterial = SimpleMaterial(color: .init(red: 0.8, green: 0.6, blue: 0.4, alpha: 1.0), isMetallic: false)
        let bodyModel = ModelEntity(mesh: bodyMesh, materials: [bodyMaterial])
        bodyModel.position.y = 0.9
        
        // Head
        let headMesh = MeshResource.generateSphere(radius: 0.15)
        let headMaterial = SimpleMaterial(color: .init(red: 0.9, green: 0.7, blue: 0.5, alpha: 1.0), isMetallic: false)
        let headModel = ModelEntity(mesh: headMesh, materials: [headMaterial])
        headModel.position.y = 2.0
        
        avatarEntity.addChild(bodyModel)
        avatarEntity.addChild(headModel)
        
        // Add physics and collision
        avatarEntity.components.set(CollisionComponent(shapes: [.generateCapsule(height: 1.8, radius: 0.3)]))
        avatarEntity.components.set(PhysicsBodyComponent(
            massProperties: .default,
            material: .default,
            mode: .dynamic
        ))
        
        // Add custom component to track Storm entity ID
        avatarEntity.components.set(StormEntityComponent(entityId: id, entityType: .avatar))
        
        content.add(avatarEntity)
        renderSystem.registerEntity(avatarEntity, stormId: id)
    }
    
    private func createSampleNPCs(content: RealityViewContent) async {
        // Create Lumi NPC (Echo of Hope)
        do {
            let lumiId = try await stormEngine.createNPC(type: .lumi, at: simd_float3(5, 1, -5))
            await createNPCEntity(id: lumiId, type: .lumi, at: simd_float3(5, 1, -5), content: content)
        } catch {
            logger.error("Failed to create Lumi NPC: \(error)")
        }
        
        // Create Sage NPC (Echo of Wisdom)
        do {
            let sageId = try await stormEngine.createNPC(type: .sage, at: simd_float3(-5, 1, -5))
            await createNPCEntity(id: sageId, type: .sage, at: simd_float3(-5, 1, -5), content: content)
        } catch {
            logger.error("Failed to create Sage NPC: \(error)")
        }
    }
    
    private func createNPCEntity(id: UInt64, type: NPCType, at position: simd_float3, content: RealityViewContent) async {
        let npcEntity = Entity()
        npcEntity.name = "NPC_\(type)_\(id)"
        npcEntity.position = position
        
        // Create NPC model based on type
        let npcModel = createNPCModel(for: type)
        npcEntity.addChild(npcModel)
        
        // Add physics
        npcEntity.components.set(CollisionComponent(shapes: [.generateBox(width: 1.0, height: 2.0, depth: 1.0)]))
        npcEntity.components.set(PhysicsBodyComponent(
            massProperties: .default,
            material: .default,
            mode: .kinematic
        ))
        
        // Add AI behavior component
        npcEntity.components.set(AIBehaviorComponent(npcType: type, entityId: id))
        npcEntity.components.set(StormEntityComponent(entityId: id, entityType: .npc))
        
        // Add particle effects for Echoes
        await addEchoEffects(to: npcEntity, type: type)
        
        // Add spatial audio
        await addNPCAudio(to: npcEntity, type: type)
        
        content.add(npcEntity)
        renderSystem.registerEntity(npcEntity, stormId: id)
        
        // Start AI behavior
        await startNPCAIBehavior(npcEntity, type: type)
    }
    
    private func createNPCModel(for type: NPCType) -> ModelEntity {
        switch type {
        case .lumi: // Echo of Hope - Glowing blue humanoid
            let mesh = MeshResource.generateBox(width: 0.8, height: 2.0, depth: 0.8)
            let material = SimpleMaterial(color: .init(red: 0.2, green: 0.6, blue: 1.0, alpha: 0.8), isMetallic: false)
            return ModelEntity(mesh: mesh, materials: [material])
            
        case .sage: // Echo of Wisdom - Golden crystalline form
            let mesh = MeshResource.generateBox(width: 1.0, height: 2.2, depth: 1.0)
            let material = SimpleMaterial(color: .init(red: 1.0, green: 0.8, blue: 0.2, alpha: 0.9), isMetallic: true)
            return ModelEntity(mesh: mesh, materials: [material])
            
        case .guardian: // Echo of Protection - Robust stone-like
            let mesh = MeshResource.generateBox(width: 1.2, height: 2.5, depth: 1.2)
            let material = SimpleMaterial(color: .init(red: 0.4, green: 0.4, blue: 0.4, alpha: 1.0), isMetallic: false)
            return ModelEntity(mesh: mesh, materials: [material])
            
        case .explorer: // Echo of Discovery - Sleek green form
            let mesh = MeshResource.generateBox(width: 0.7, height: 1.9, depth: 0.7)
            let material = SimpleMaterial(color: .init(red: 0.2, green: 0.8, blue: 0.3, alpha: 0.85), isMetallic: false)
            return ModelEntity(mesh: mesh, materials: [material])
        }
    }
    
    private func addEchoEffects(to entity: Entity, type: NPCType) async {
        // Add particle effects based on Echo type
        // Note: In a real implementation, this would use custom particle systems
        // For now, we'll simulate with animated materials and scale changes
        
        let glowAnimation = AnimationResource.generateOrbitAnimation(
            duration: 3.0,
            axis: [0, 1, 0],
            startTransform: entity.transform,
            spinCountPerOrbit: 1
        )
        
        entity.playAnimation(glowAnimation.repeat())
    }
    
    private func addNPCAudio(to entity: Entity, type: NPCType) async {
        // Add spatial audio for NPC dialogue and ambient sounds
        do {
            let audioResource: AudioFileResource
            
            switch type {
            case .lumi:
                // Gentle, hopeful ambient sound
                audioResource = try AudioFileResource.load(
                    named: "/System/Library/Sounds/Glass.aiff",
                    configuration: AudioFileResource.Configuration(shouldLoop: true)
                )
            case .sage:
                // Wise, deep resonance
                audioResource = try AudioFileResource.load(
                    named: "/System/Library/Sounds/Bottle.aiff",
                    configuration: AudioFileResource.Configuration(shouldLoop: true)
                )
            case .guardian:
                // Strong, protective rumble
                audioResource = try AudioFileResource.load(
                    named: "/System/Library/Sounds/Sosumi.aiff",
                    configuration: AudioFileResource.Configuration(shouldLoop: true)
                )
            case .explorer:
                // Curious, energetic sound
                audioResource = try AudioFileResource.load(
                    named: "/System/Library/Sounds/Ping.aiff",
                    configuration: AudioFileResource.Configuration(shouldLoop: true)
                )
            }
            
            let audioController = entity.prepareAudio(audioResource)
            audioController.gain = -20 // Quiet ambient sound
            audioController.play()
            
        } catch {
            logger.error("Failed to add audio to NPC: \(error)")
        }
    }
    
    // MARK: - Scene Updates
    
    private func updateEntitiesFromCore(content: RealityViewContent) async {
        // Sync entity positions and states from Storm core
        for entity in content.entities {
            if let stormComponent = entity.components[StormEntityComponent.self] {
                // Update position from core state
                await updateEntityFromCore(entity, stormId: stormComponent.entityId)
            }
        }
    }
    
    private func updateEntityFromCore(_ entity: Entity, stormId: UInt64) async {
        // Query Storm core for entity state
        // This would be implemented with actual FFI calls to get entity data
        // For now, we'll simulate with basic movement patterns
        
        if let aiComponent = entity.components[AIBehaviorComponent.self] {
            await updateAIBehavior(entity, aiComponent: aiComponent)
        }
    }
    
    private func updateAIBehaviors(content: RealityViewContent) async {
        // Update AI-driven entity behaviors
        for entity in content.entities {
            if let aiComponent = entity.components[AIBehaviorComponent.self] {
                await updateAIBehavior(entity, aiComponent: aiComponent)
            }
        }
    }
    
    private func updateAIBehavior(_ entity: Entity, aiComponent: AIBehaviorComponent) async {
        switch aiComponent.npcType {
        case .lumi:
            // Lumi moves in gentle, hopeful patterns
            await animateLumiMovement(entity)
        case .sage:
            // Sage remains mostly stationary but occasionally turns
            await animateSageMovement(entity)
        case .guardian:
            // Guardian patrols in protective patterns
            await animateGuardianMovement(entity)
        case .explorer:
            // Explorer moves curiously around the environment
            await animateExplorerMovement(entity)
        }
    }
    
    private func animateLumiMovement(_ entity: Entity) async {
        // Gentle floating movement
        let time = CACurrentMediaTime()
        let offsetY = sin(time * 0.5) * 0.2
        entity.position.y = 1.0 + Float(offsetY)
        
        // Gentle rotation
        let rotation = simd_quatf(angle: Float(time * 0.1), axis: [0, 1, 0])
        entity.orientation = rotation
    }
    
    private func animateSageMovement(_ entity: Entity) async {
        // Slow, contemplative rotation
        let time = CACurrentMediaTime()
        let rotation = simd_quatf(angle: Float(time * 0.05), axis: [0, 1, 0])
        entity.orientation = rotation
    }
    
    private func animateGuardianMovement(_ entity: Entity) async {
        // Patrol movement in a small area
        let time = CACurrentMediaTime()
        let patrolRadius: Float = 2.0
        let x = cos(time * 0.2) * patrolRadius
        let z = sin(time * 0.2) * patrolRadius
        
        entity.position.x = -5 + x
        entity.position.z = -5 + z
        
        // Face movement direction
        let direction = simd_normalize(simd_float3(x, 0, z))
        let rotation = simd_quatf(from: [0, 0, 1], to: direction)
        entity.orientation = rotation
    }
    
    private func animateExplorerMovement(_ entity: Entity) async {
        // Curious wandering movement
        let time = CACurrentMediaTime()
        let wanderRadius: Float = 8.0
        let x = cos(time * 0.3 + 1.5) * wanderRadius
        let z = sin(time * 0.3 + 1.5) * wanderRadius
        
        entity.position.x = x
        entity.position.z = z
        
        // Quick, inquisitive turns
        let rotation = simd_quatf(angle: Float(time * 0.4), axis: [0, 1, 0])
        entity.orientation = rotation
    }
    
    private func startNPCAIBehavior(_ entity: Entity, type: NPCType) async {
        // Initialize AI behavior patterns
        // This would integrate with the Storm AI system for complex behaviors
        logger.info("Starting AI behavior for NPC type: \(type)")
    }
    
    // MARK: - Lighting and Physics
    
    private func setupLighting(content: RealityViewContent) {
        // Add directional light (sun)
        let lightEntity = Entity()
        lightEntity.name = "Sun"
        
        let light = DirectionalLightComponent(color: .white, intensity: 1000)
        lightEntity.components.set(light)
        lightEntity.orientation = simd_quatf(angle: .pi/4, axis: [1, 0, 0])
        
        content.add(lightEntity)
        
        // Add ambient light
        let ambientEntity = Entity()
        ambientEntity.name = "Ambient"
        
        let ambientLight = DirectionalLightComponent(color: .init(red: 0.3, green: 0.3, blue: 0.5, alpha: 1.0), intensity: 200)
        ambientEntity.components.set(ambientLight)
        
        content.add(ambientEntity)
    }
    
    private func setupPhysics(content: RealityViewContent) {
        // Physics is automatically handled by RealityKit
        // Additional physics configuration can be added here
        logger.info("Physics system initialized")
    }
    
    private func updatePhysics(content: RealityViewContent) {
        // Update physics simulations
        // RealityKit handles most physics automatically
        // Custom physics logic can be added here
    }
    
    // MARK: - User Interaction
    
    private func handleCameraMovement(_ value: DragGesture.Value) {
        // Simple camera orbit controls
        let sensitivity: Float = 0.01
        let deltaX = Float(value.translation.x) * sensitivity
        let deltaY = Float(value.translation.y) * sensitivity
        
        // Update camera position (simplified)
        cameraPosition.x += deltaX
        cameraPosition.y -= deltaY
    }
    
    private func handleWorldTap(at location: CGPoint) {
        // Handle taps on the 3D world
        logger.info("World tapped at location: \(location)")
        
        // In a full implementation, this would perform ray casting
        // to select entities or place new objects
    }
    
    private func resetCamera() {
        withAnimation(.easeInOut(duration: 1.0)) {
            cameraPosition = simd_float3(0, 2, 5)
            cameraTarget = simd_float3(0, 0, 0)
            cameraRotation = 0
        }
    }
    
    // MARK: - Entity Creation Actions
    
    private func createNewAvatar() async {
        do {
            let position = simd_float3(
                Float.random(in: -5...5),
                1,
                Float.random(in: -5...5)
            )
            let _ = try await stormEngine.createAvatar(at: position)
            logger.info("New avatar created at position: \(position)")
        } catch {
            logger.error("Failed to create new avatar: \(error)")
        }
    }
    
    private func createNewNPC() async {
        do {
            let position = simd_float3(
                Float.random(in: -10...10),
                1,
                Float.random(in: -10...10)
            )
            let npcType = NPCType.allCases.randomElement() ?? .lumi
            let _ = try await stormEngine.createNPC(type: npcType, at: position)
            logger.info("New NPC of type \(npcType) created at position: \(position)")
        } catch {
            logger.error("Failed to create new NPC: \(error)")
        }
    }
    
    private func createNewObject() async {
        // Create interactive objects in the world
        logger.info("Creating new interactive object")
        // Implementation would add custom objects to the world
    }
    
    private func setupRenderSystem() {
        renderSystem.initialize()
    }
}

// MARK: - Custom Components

struct StormEntityComponent: Component {
    let entityId: UInt64
    let entityType: StormEntityType
}

enum StormEntityType {
    case avatar
    case npc
    case object
    case environment
}

struct AIBehaviorComponent: Component {
    let npcType: NPCType
    let entityId: UInt64
    var lastUpdateTime: TimeInterval = 0
    var behaviorState: AIBehaviorState = .idle
}

enum AIBehaviorState {
    case idle
    case moving
    case interacting
    case thinking
}

// MARK: - Control Panels

struct CameraControlPanel: View {
    @Binding var position: simd_float3
    @Binding var target: simd_float3
    @Binding var rotation: Float
    
    var body: some View {
        VStack(alignment: .trailing, spacing: 8) {
            Text("Camera")
                .font(.headline)
            
            VStack(spacing: 4) {
                HStack {
                    Text("Zoom")
                        .font(.caption)
                    Slider(value: Binding(
                        get: { simd_distance(position, target) },
                        set: { distance in
                            let direction = simd_normalize(position - target)
                            position = target + direction * distance
                        }
                    ), in: 2...20)
                    .frame(width: 100)
                }
                
                HStack {
                    Text("Height")
                        .font(.caption)
                    Slider(value: $position.y, in: 0.5...10)
                        .frame(width: 100)
                }
                
                HStack {
                    Text("Rotation")
                        .font(.caption)
                    Slider(value: $rotation, in: 0...(2 * .pi))
                        .frame(width: 100)
                        .onChange(of: rotation) { newRotation in
                            let distance = simd_distance(position, target)
                            position.x = target.x + cos(newRotation) * distance
                            position.z = target.z + sin(newRotation) * distance
                        }
                }
            }
            
            Button("Reset View") {
                withAnimation {
                    position = simd_float3(0, 2, 5)
                    target = simd_float3(0, 0, 0)
                    rotation = 0
                }
            }
            .buttonStyle(.bordered)
        }
        .padding()
        .background(Color.black.opacity(0.7))
        .cornerRadius(8)
    }
}

struct EntityInspectorPanel: View {
    let entity: Entity
    
    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            Text("Entity Inspector")
                .font(.headline)
            
            Text("Name: \(entity.name)")
            Text("Position: \(formatPosition(entity.position))")
            
            if let stormComponent = entity.components[StormEntityComponent.self] {
                Text("Storm ID: \(stormComponent.entityId)")
                Text("Type: \(stormComponent.entityType)")
            }
        }
        .padding()
        .background(Color.black.opacity(0.7))
        .cornerRadius(8)
        .frame(maxWidth: 200)
    }
    
    private func formatPosition(_ position: simd_float3) -> String {
        return String(format: "%.1f, %.1f, %.1f", position.x, position.y, position.z)
    }
}

struct WorldActionPanel: View {
    @EnvironmentObject var stormEngine: StormEngine
    
    var body: some View {
        VStack(spacing: 8) {
            Text("World Actions")
                .font(.headline)
            
            Button("Spawn Avatar") {
                Task {
                    do {
                        let _ = try await stormEngine.createAvatar(
                            at: simd_float3(Float.random(in: -5...5), 1, Float.random(in: -5...5))
                        )
                    } catch {
                        print("Failed to spawn avatar: \(error)")
                    }
                }
            }
            .buttonStyle(.bordered)
            
            Button("Add Lumi") {
                Task {
                    do {
                        let _ = try await stormEngine.createNPC(
                            type: .lumi,
                            at: simd_float3(Float.random(in: -8...8), 1, Float.random(in: -8...8))
                        )
                    } catch {
                        print("Failed to create Lumi: \(error)")
                    }
                }
            }
            .buttonStyle(.bordered)
        }
        .padding()
        .background(Color.black.opacity(0.7))
        .cornerRadius(8)
    }
}
