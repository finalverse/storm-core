// File: RealityKit/StormRenderSystem.swift
// Description: Custom render system for Storm's RealityKit integration
// Manages rendering updates, entity registration, and performance optimization

import RealityKit
import Foundation
import os

class StormRenderSystem: ObservableObject {
    @Published var isInitialized = false
    @Published var entityCount = 0
    @Published var renderStatistics = RenderStatistics()
    @Published var isPhysicsDebugEnabled = false
    
    private var registeredEntities: [UInt64: Entity] = [:]
    private var entityUpdateQueue: DispatchQueue
    private let logger = Logger(subsystem: "com.storm.client", category: "RenderSystem")
    
    init() {
        self.entityUpdateQueue = DispatchQueue(label: "com.storm.render.updates", qos: .userInteractive)
        logger.info("Storm render system initialized")
    }
    
    // MARK: - System Management
    
    func initialize() {
        logger.info("Initializing Storm render system")
        isInitialized = true
    }
    
    func shutdown() {
        logger.info("Shutting down Storm render system")
        registeredEntities.removeAll()
        isInitialized = false
    }
    
    // MARK: - Entity Management
    
    func registerEntity(_ entity: Entity, stormId: UInt64) {
        entityUpdateQueue.async { [weak self] in
            self?.registeredEntities[stormId] = entity
            
            DispatchQueue.main.async {
                self?.entityCount = self?.registeredEntities.count ?? 0
            }
        }
        
        logger.debug("Registered entity with Storm ID: \(stormId)")
    }
    
    func unregisterEntity(stormId: UInt64) {
        entityUpdateQueue.async { [weak self] in
            self?.registeredEntities.removeValue(forKey: stormId)
            
            DispatchQueue.main.async {
                self?.entityCount = self?.registeredEntities.count ?? 0
            }
        }
        
        logger.debug("Unregistered entity with Storm ID: \(stormId)")
    }
    
    func getEntity(stormId: UInt64) -> Entity? {
        return registeredEntities[stormId]
    }
    
    func updateEntityTransform(stormId: UInt64, position: simd_float3, rotation: simd_quatf? = nil, scale: simd_float3? = nil) {
        guard let entity = registeredEntities[stormId] else {
            logger.warning("Attempted to update non-registered entity: \(stormId)")
            return
        }
        
        entityUpdateQueue.async {
            entity.position = position
            
            if let rotation = rotation {
                entity.orientation = rotation
            }
            
            if let scale = scale {
                entity.scale = scale
            }
        }
    }
    
    // MARK: - Performance Monitoring
    
    func updateRenderStatistics() {
        renderStatistics.frameCount += 1
        renderStatistics.entityCount = entityCount
        renderStatistics.lastUpdateTime = Date()
        
        // Calculate FPS (simplified)
        let now = CACurrentMediaTime()
        if renderStatistics.lastFrameTime > 0 {
            let deltaTime = now - renderStatistics.lastFrameTime
            renderStatistics.currentFPS = 1.0 / deltaTime
        }
        renderStatistics.lastFrameTime = now
    }
    
    func resetStatistics() {
        renderStatistics = RenderStatistics()
        logger.info("Render statistics reset")
    }
    
    // MARK: - Debug Features
    
    func togglePhysicsDebug() {
        isPhysicsDebugEnabled.toggle()
        logger.info("Physics debug mode: \(isPhysicsDebugEnabled ? "enabled" : "disabled")")
        
        // Apply debug visualization to all entities
        for entity in registeredEntities.values {
            updateEntityDebugVisualization(entity)
        }
    }
    
    private func updateEntityDebugVisualization(_ entity: Entity) {
        if isPhysicsDebugEnabled {
            // Add debug visualization for physics bodies
            if entity.components.has(PhysicsBodyComponent.self) {
                addPhysicsDebugVisualization(to: entity)
            }
        } else {
            // Remove debug visualization
            removePhysicsDebugVisualization(from: entity)
        }
    }
    
    private func addPhysicsDebugVisualization(to entity: Entity) {
        // Remove existing debug visualization
        entity.children.removeAll { $0.name.hasPrefix("debug_") }
        
        // Add wireframe representation of collision shapes
        if let physicsBody = entity.components[PhysicsBodyComponent.self] {
            for (index, shape) in physicsBody.shapes.enumerated() {
                let debugEntity = createDebugVisualization(for: shape, index: index)
                entity.addChild(debugEntity)
            }
        }
    }
    
    private func removePhysicsDebugVisualization(from entity: Entity) {
        entity.children.removeAll { $0.name.hasPrefix("debug_") }
    }
    
    private func createDebugVisualization(for shape: ShapeResource, index: Int) -> Entity {
        let debugEntity = Entity()
        debugEntity.name = "debug_physics_\(index)"
        
        // Create wireframe mesh based on shape type
        let debugMesh: MeshResource
        let material = UnlitMaterial(color: .green)
        
        // This is a simplified implementation - in reality, you'd need to extract
        // the actual shape geometry and create wireframe representations
        debugMesh = MeshResource.generateBox(size: [1, 1, 1])
        
        let debugModel = ModelEntity(mesh: debugMesh, materials: [material])
        debugEntity.addChild(debugModel)
        
        return debugEntity
    }
    
    // MARK: - LOD (Level of Detail) Management
    
    func updateLOD(cameraPosition: simd_float3) {
        for (stormId, entity) in registeredEntities {
            let distance = simd_distance(entity.position, cameraPosition)
            let lodLevel = calculateLODLevel(distance: distance)
            
            updateEntityLOD(entity: entity, lodLevel: lodLevel)
        }
    }
    
    private func calculateLODLevel(distance: Float) -> LODLevel {
        if distance < 10 {
            return .high
        } else if distance < 50 {
            return .medium
        } else if distance < 100 {
            return .low
        } else {
            return .minimal
        }
    }
    
    private func updateEntityLOD(entity: Entity, lodLevel: LODLevel) {
        // Update entity detail based on LOD level
        for child in entity.children {
            if let modelEntity = child as? ModelEntity {
                updateModelLOD(modelEntity: modelEntity, lodLevel: lodLevel)
            }
        }
    }
    
    private func updateModelLOD(modelEntity: ModelEntity, lodLevel: LODLevel) {
        // In a real implementation, this would switch between different
        // mesh resolutions or disable certain visual features
        switch lodLevel {
        case .high:
            modelEntity.isEnabled = true
            // Use high-detail mesh and materials
        case .medium:
            modelEntity.isEnabled = true
            // Use medium-detail mesh
        case .low:
            modelEntity.isEnabled = true
            // Use low-detail mesh
        case .minimal:
            modelEntity.isEnabled = false
            // Hide entity or use billboard
        }
    }
    
    // MARK: - Culling
    
    func performFrustumCulling(cameraTransform: Transform, fieldOfView: Float, aspectRatio: Float, nearPlane: Float, farPlane: Float) {
        // Calculate frustum planes
        let frustum = calculateFrustum(
            cameraTransform: cameraTransform,
            fieldOfView: fieldOfView,
            aspectRatio: aspectRatio,
            nearPlane: nearPlane,
            farPlane: farPlane
        )
        
        // Test each entity against frustum
        for (stormId, entity) in registeredEntities {
            let isVisible = isEntityInFrustum(entity: entity, frustum: frustum)
            entity.isEnabled = isVisible
        }
    }
    
    private func calculateFrustum(cameraTransform: Transform, fieldOfView: Float, aspectRatio: Float, nearPlane: Float, farPlane: Float) -> Frustum {
        // Simplified frustum calculation
        // In a real implementation, you'd calculate the 6 frustum planes
        return Frustum(
            near: nearPlane,
            far: farPlane,
            fov: fieldOfView,
            aspect: aspectRatio,
            transform: cameraTransform
        )
    }
    
    private func isEntityInFrustum(entity: Entity, frustum: Frustum) -> Bool {
        // Simplified visibility test
        // In a real implementation, you'd test the entity's bounding box against frustum planes
        let distance = simd_distance(entity.position, frustum.transform.translation)
        return distance <= frustum.far
    }
    
    // MARK: - Material Management
    
    func updateMaterialsForPerformance(targetFPS: Float) {
        let currentFPS = renderStatistics.currentFPS
        
        if currentFPS < targetFPS * 0.8 {
            // Reduce material quality to improve performance
            for entity in registeredEntities.values {
                optimizeMaterials(for: entity)
            }
        } else if currentFPS > targetFPS * 1.2 {
            // Increase material quality
            for entity in registeredEntities.values {
                enhanceMaterials(for: entity)
            }
        }
    }
    
    private func optimizeMaterials(for entity: Entity) {
        for child in entity.children {
            if let modelEntity = child as? ModelEntity {
                // Replace complex materials with simpler ones
                let optimizedMaterials = modelEntity.materials.map { material in
                    if let physicalMaterial = material as? PhysicallyBasedMaterial {
                        // Convert to simpler unlit material
                        return UnlitMaterial(color: physicalMaterial.baseColor.tint)
                    }
                    return material
                }
                modelEntity.materials = optimizedMaterials
            }
        }
    }
    
    private func enhanceMaterials(for entity: Entity) {
        for child in entity.children {
            if let modelEntity = child as? ModelEntity {
                // Upgrade to higher quality materials if performance allows
                let enhancedMaterials = modelEntity.materials.map { material in
                    if let unlitMaterial = material as? UnlitMaterial {
                        // Convert to physically based material
                        var physicalMaterial = PhysicallyBasedMaterial()
                        physicalMaterial.baseColor = unlitMaterial.color
                        return physicalMaterial
                    }
                    return material
                }
                modelEntity.materials = enhancedMaterials
            }
        }
    }
    
    // MARK: - Batch Operations
    
    func batchUpdateEntities(_ updates: [(UInt64, Transform)]) {
        entityUpdateQueue.async { [weak self] in
            for (stormId, transform) in updates {
                if let entity = self?.registeredEntities[stormId] {
                    entity.transform = transform
                }
            }
        }
    }
    
    func batchSetVisibility(_ visibility: [(UInt64, Bool)]) {
        entityUpdateQueue.async { [weak self] in
            for (stormId, isVisible) in visibility {
                if let entity = self?.registeredEntities[stormId] {
                    entity.isEnabled = isVisible
                }
            }
        }
    }
}

// MARK: - Supporting Types

/// Render statistics for performance monitoring
struct RenderStatistics {
    var frameCount: UInt64 = 0
    var entityCount: Int = 0
    var currentFPS: Double = 0
    var averageFPS: Double = 0
    var lastFrameTime: Double = 0
    var lastUpdateTime: Date = Date()
    var triangleCount: UInt64 = 0
    var drawCalls: UInt32 = 0
    
    var formattedFPS: String {
        return String(format: "%.1f", currentFPS)
    }
    
    var formattedTriangleCount: String {
        if triangleCount > 1_000_000 {
            return String(format: "%.1fM", Double(triangleCount) / 1_000_000.0)
        } else if triangleCount > 1_000 {
            return String(format: "%.1fK", Double(triangleCount) / 1_000.0)
        } else {
            return "\(triangleCount)"
        }
    }
}

/// Level of Detail enumeration
enum LODLevel: Int, CaseIterable {
    case high = 0
    case medium = 1
    case low = 2
    case minimal = 3
    
    var description: String {
        switch self {
        case .high: return "High Detail"
        case .medium: return "Medium Detail"
        case .low: return "Low Detail"
        case .minimal: return "Minimal Detail"
        }
    }
}

/// Simplified frustum for culling
struct Frustum {
    let near: Float
    let far: Float
    let fov: Float
    let aspect: Float
    let transform: Transform
}

// MARK: - Custom Components

/// Component to link RealityKit entities with Storm core entities
struct StormEntityComponent: Component {
    let entityId: UInt64
    let entityType: StormEntityType
    var lastSyncTime: Date = Date()
    var isDirty: Bool = false
}

/// Storm entity types
enum StormEntityType: String, CaseIterable {
    case avatar = "Avatar"
    case npc = "NPC"
    case object = "Object"
    case environment = "Environment"
    case particle = "Particle"
    case light = "Light"
    case audio = "Audio"
    
    var icon: String {
        switch self {
        case .avatar: return "person.fill"
        case .npc: return "brain.head.profile"
        case .object: return "cube.box"
        case .environment: return "tree.fill"
        case .particle: return "sparkles"
        case .light: return "lightbulb.fill"
        case .audio: return "speaker.wave.3.fill"
        }
    }
}

/// AI behavior component for NPCs
struct AIBehaviorComponent: Component {
    let npcType: NPCType
    let entityId: UInt64
    var lastUpdateTime: TimeInterval = 0
    var behaviorState: AIBehaviorState = .idle
    var targetPosition: simd_float3?
    var interactionRadius: Float = 2.0
    var movementSpeed: Float = 1.0
    var personality: PersonalityTraits = PersonalityTraits()
}

/// AI behavior states
enum AIBehaviorState: String, CaseIterable {
    case idle = "Idle"
    case moving = "Moving"
    case interacting = "Interacting"
    case thinking = "Thinking"
    case patrolling = "Patrolling"
    case following = "Following"
    case fleeing = "Fleeing"
    
    var color: String {
        switch self {
        case .idle: return "gray"
        case .moving: return "blue"
        case .interacting: return "green"
        case .thinking: return "purple"
        case .patrolling: return "orange"
        case .following: return "yellow"
        case .fleeing: return "red"
        }
    }
}

/// NPC personality traits
struct PersonalityTraits {
    var curiosity: Float = 0.5
    var sociability: Float = 0.5
    var aggressiveness: Float = 0.3
    var intelligence: Float = 0.7
    var loyalty: Float = 0.6
    var creativity: Float = 0.5
    
    func getTraitValue(_ trait: String) -> Float {
        switch trait.lowercased() {
        case "curiosity": return curiosity
        case "sociability": return sociability
        case "aggressiveness": return aggressiveness
        case "intelligence": return intelligence
        case "loyalty": return loyalty
        case "creativity": return creativity
        default: return 0.5
        }
    }
}

/// Enhanced spin component for dynamic objects
struct SpinComponent: Component {
    var speed: Float
    var axis: simd_float3
    var isActive: Bool = true
    var randomVariation: Float = 0.0
    
    init(speed: Float = 1.0, axis: simd_float3 = [0, 1, 0]) {
        self.speed = speed
        self.axis = normalize(axis)
    }
}

/// Audio source component for spatial audio
struct AudioSourceComponent: Component {
    let audioResourceName: String
    var volume: Float = 1.0
    var pitch: Float = 1.0
    var isLooping: Bool = false
    var is3D: Bool = true
    var maxDistance: Float = 100.0
    var rolloffFactor: Float = 1.0
    var isPlaying: Bool = false
    
    init(audioResourceName: String, volume: Float = 1.0, is3D: Bool = true) {
        self.audioResourceName = audioResourceName
        self.volume = volume
        self.is3D = is3D
    }
}

/// Performance optimization component
struct PerformanceComponent: Component {
    var lodLevel: LODLevel = .high
    var cullingRadius: Float = 100.0
    var isStaticObject: Bool = false
    var lastOptimizationTime: Date = Date()
    var triangleCount: UInt32 = 0
    var memoryUsage: UInt64 = 0
    
    var shouldCull: Bool {
        return cullingRadius > 0
    }
}

/// Network sync component for multiplayer
struct NetworkSyncComponent: Component {
    var isNetworked: Bool = true
    var syncPosition: Bool = true
    var syncRotation: Bool = true
    var syncScale: Bool = false
    var syncAnimations: Bool = true
    var updateRate: Float = 10.0 // Updates per second
    var lastSyncTime: Date = Date()
    var authority: NetworkAuthority = .server
    
    var shouldSync: Bool {
        let timeSinceLastSync = Date().timeIntervalSince(lastSyncTime)
        return timeSinceLastSync >= 1.0 / updateRate
    }
}

/// Network authority types
enum NetworkAuthority: String, CaseIterable {
    case server = "Server"
    case client = "Client"
    case shared = "Shared"
    case none = "None"
}

/// Interactive component for clickable objects
struct InteractiveComponent: Component {
    var canInteract: Bool = true
    var interactionType: InteractionType = .examine
    var interactionRadius: Float = 2.0
    var requiresLineOfSight: Bool = true
    var cooldownTime: TimeInterval = 0.0
    var lastInteractionTime: Date = Date.distantPast
    var interactionData: [String: String] = [:]
    
    var isOnCooldown: Bool {
        let timeSinceLastInteraction = Date().timeIntervalSince(lastInteractionTime)
        return timeSinceLastInteraction < cooldownTime
    }
}

/// Interaction types
enum InteractionType: String, CaseIterable {
    case examine = "Examine"
    case pickup = "Pickup"
    case use = "Use"
    case talk = "Talk"
    case attack = "Attack"
    case trade = "Trade"
    case follow = "Follow"
    case sit = "Sit"
    case dance = "Dance"
    case teleport = "Teleport"
    
    var icon: String {
        switch self {
        case .examine: return "eye"
        case .pickup: return "hand.raised"
        case .use: return "hand.point.up"
        case .talk: return "bubble.left"
        case .attack: return "sword"
        case .trade: return "arrow.left.arrow.right"
        case .follow: return "figure.walk"
        case .sit: return "chair"
        case .dance: return "figure.dance"
        case .teleport: return "location.circle"
        }
    }
}

/// Particle effect component
struct ParticleEffectComponent: Component {
    let effectType: ParticleEffectType
    var intensity: Float = 1.0
    var color: simd_float4 = [1, 1, 1, 1]
    var isActive: Bool = true
    var duration: TimeInterval = -1 // -1 for infinite
    var startTime: Date = Date()
    
    var shouldExpire: Bool {
        guard duration > 0 else { return false }
        return Date().timeIntervalSince(startTime) >= duration
    }
}

/// Particle effect types
enum ParticleEffectType: String, CaseIterable {
    case glow = "Glow"
    case sparkles = "Sparkles"
    case fire = "Fire"
    case smoke = "Smoke"
    case water = "Water"
    case energy = "Energy"
    case magic = "Magic"
    case teleport = "Teleport"
    
    var description: String {
        switch self {
        case .glow: return "Soft ambient glow effect"
        case .sparkles: return "Sparkling particle effect"
        case .fire: return "Flame particle system"
        case .smoke: return "Smoke particle system"
        case .water: return "Water droplet effect"
        case .energy: return "Energy beam effect"
        case .magic: return "Magical aura effect"
        case .teleport: return "Teleportation effect"
        }
    }
}

/// Animation component for complex animations
struct AnimationComponent: Component {
    var currentAnimation: String = "idle"
    var isPlaying: Bool = false
    var loop: Bool = true
    var speed: Float = 1.0
    var weight: Float = 1.0
    var crossfadeDuration: Float = 0.3
    var availableAnimations: [String] = []
    var lastAnimationTime: Date = Date()
    
    func hasAnimation(_ name: String) -> Bool {
        return availableAnimations.contains(name)
    }
}

/// Echo-specific component for supernatural avatars
struct EchoComponent: Component {
    let echoType: EchoType
    var echoIntensity: Float = 1.0
    var glowColor: simd_float4 = [0.5, 0.8, 1.0, 1.0]
    var transparency: Float = 0.0
    var energyPattern: EnergyPattern = .flowing
    var manifestationLevel: Float = 1.0
    var echoAbilities: [EchoAbility] = []
    
    var isFullyManifested: Bool {
        return manifestationLevel >= 1.0
    }
}

/// Echo types from Storm lore
enum EchoType: String, CaseIterable {
    case hope = "Hope"
    case wisdom = "Wisdom"
    case memory = "Memory"
    case logic = "Logic"
    case dreams = "Dreams"
    case protection = "Protection"
    case discovery = "Discovery"
    case creation = "Creation"
    
    var color: simd_float4 {
        switch self {
        case .hope: return [1.0, 1.0, 0.5, 1.0] // Golden
        case .wisdom: return [0.8, 0.6, 1.0, 1.0] // Purple
        case .memory: return [0.7, 0.9, 1.0, 1.0] // Light Blue
        case .logic: return [0.9, 0.9, 0.9, 1.0] // Silver
        case .dreams: return [1.0, 0.7, 0.9, 1.0] // Pink
        case .protection: return [0.6, 1.0, 0.6, 1.0] // Green
        case .discovery: return [1.0, 0.8, 0.4, 1.0] // Orange
        case .creation: return [1.0, 0.5, 0.5, 1.0] // Red
        }
    }
    
    var description: String {
        switch self {
        case .hope: return "Radiant beings of optimism and inspiration"
        case .wisdom: return "Ancient guardians of knowledge and truth"
        case .memory: return "Keepers of history and experience"
        case .logic: return "Masters of reason and analysis"
        case .dreams: return "Weavers of imagination and possibility"
        case .protection: return "Shields against darkness and harm"
        case .discovery: return "Explorers of the unknown"
        case .creation: return "Architects of new realities"
        }
    }
}

/// Energy patterns for Echo manifestations
enum EnergyPattern: String, CaseIterable {
    case static = "Static"
    case flowing = "Flowing"
    case pulsing = "Pulsing"
    case swirling = "Swirling"
    case cascading = "Cascading"
    case spiraling = "Spiraling"
    
    var animationDuration: Float {
        switch self {
        case .static: return 0.0
        case .flowing: return 3.0
        case .pulsing: return 2.0
        case .swirling: return 4.0
        case .cascading: return 2.5
        case .spiraling: return 5.0
        }
    }
}

/// Echo abilities
enum EchoAbility: String, CaseIterable {
    case illuminate = "Illuminate"
    case phase = "Phase"
    case teleport = "Teleport"
    case heal = "Heal"
    case shield = "Shield"
    case enhance = "Enhance"
    case reveal = "Reveal"
    case transform = "Transform"
    
    var cooldown: TimeInterval {
        switch self {
        case .illuminate: return 5.0
        case .phase: return 10.0
        case .teleport: return 30.0
        case .heal: return 20.0
        case .shield: return 15.0
        case .enhance: return 25.0
        case .reveal: return 8.0
        case .transform: return 60.0
        }
    }
}

/// Component registration helper
extension Entity {
    func addStormComponent<T: Component>(_ component: T) {
        self.components.set(component)
    }
    
    func getStormComponent<T: Component>(_ type: T.Type) -> T? {
        return self.components[type]
    }
    
    func hasStormComponent<T: Component>(_ type: T.Type) -> Bool {
        return self.components.has(type)
    }
    
    func removeStormComponent<T: Component>(_ type: T.Type) {
        self.components.remove(type)
    }
}

// MARK: - System Updates

extension StormRenderSystem {
    /// Update all AI behaviors
    func updateAIBehaviors(deltaTime: Float) {
        for entity in registeredEntities.values {
            if let aiComponent = entity.components[AIBehaviorComponent.self] {
                updateEntityAIBehavior(entity: entity, aiComponent: aiComponent, deltaTime: deltaTime)
            }
        }
    }
    
    private func updateEntityAIBehavior(entity: Entity, aiComponent: AIBehaviorComponent, deltaTime: Float) {
        switch aiComponent.behaviorState {
        case .idle:
            // Random chance to start moving
            if Float.random(in: 0...1) < 0.01 { // 1% chance per frame
                let randomDirection = simd_float3(
                    Float.random(in: -1...1),
                    0,
                    Float.random(in: -1...1)
                )
                entity.components[AIBehaviorComponent.self]?.targetPosition = entity.position + randomDirection * 5
                entity.components[AIBehaviorComponent.self]?.behaviorState = .moving
            }
            
        case .moving:
            if let targetPosition = aiComponent.targetPosition {
                let direction = targetPosition - entity.position
                let distance = simd_length(direction)
                
                if distance > 0.1 {
                    let normalizedDirection = direction / distance
                    let moveDistance = min(distance, aiComponent.movementSpeed * deltaTime)
                    entity.position += normalizedDirection * moveDistance
                } else {
                    entity.components[AIBehaviorComponent.self]?.behaviorState = .idle
                    entity.components[AIBehaviorComponent.self]?.targetPosition = nil
                }
            }
            
        case .interacting, .thinking, .patrolling, .following, .fleeing:
            // Additional behavior implementations would go here
            break
        }
    }
    
    /// Update all spinning objects
    func updateSpinComponents(deltaTime: Float) {
        for entity in registeredEntities.values {
            if let spinComponent = entity.components[SpinComponent.self], spinComponent.isActive {
                let rotationAmount = spinComponent.speed * deltaTime
                let rotation = simd_quatf(angle: rotationAmount, axis: spinComponent.axis)
                entity.orientation = entity.orientation * rotation
            }
        }
    }
    
    /// Update particle effects
    func updateParticleEffects() {
        for entity in registeredEntities.values {
            if let particleComponent = entity.components[ParticleEffectComponent.self] {
                if particleComponent.shouldExpire {
                    entity.components.remove(ParticleEffectComponent.self)
                }
            }
        }
    }
    
    /// Update network synchronization
    func updateNetworkSync() {
        for entity in registeredEntities.values {
            if let networkComponent = entity.components[NetworkSyncComponent.self],
               networkComponent.shouldSync {
                // Sync entity state over network
                syncEntityOverNetwork(entity: entity, networkComponent: networkComponent)
            }
        }
    }
    
    private func syncEntityOverNetwork(entity: Entity, networkComponent: NetworkSyncComponent) {
        // Implementation would send entity state to network layer
        entity.components[NetworkSyncComponent.self]?.lastSyncTime = Date()
    }
}

#Preview("StormRenderSystem") {
    VStack {
        Text("Storm Render System")
            .font(.title)
        Text("Custom RealityKit integration for Storm")
            .font(.subheadline)
            .foregroundColor(.secondary)
    }
    .padding()
}
