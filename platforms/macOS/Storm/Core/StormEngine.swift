// File: platforms/macOS/Storm/Core/StormEngine.swift
// Description: Main interface to Storm Rust core via FFI
// Manages engine lifecycle, world connections, and provides Swift API

import Foundation
import Combine
import RealityKit
import os

/// Main Storm engine class that interfaces with the Rust core
@MainActor
class StormEngine: ObservableObject {
    // MARK: - Published Properties
    @Published var isInitialized = false
    @Published var connectedWorlds: [ConnectedWorld] = []
    @Published var currentAvatar: AvatarState?
    @Published var networkStatus: NetworkStatus = .disconnected
    @Published var errorMessage: String = ""
    
    // MARK: - Private Properties
    private var stormHandle: OpaquePointer?
    private let logger = Logger(subsystem: "com.storm.client", category: "Engine")
    private var networkingClient: NetworkingClient?
    private var cancellables = Set<AnyCancellable>()
    
    // MARK: - Initialization Progress
    let initializationProgress = PassthroughSubject<InitializationProgress, Never>()
    
    // MARK: - Configuration
    private let defaultConfig = StormConfig(
        aiEnabled: true,
        renderingEnabled: true,
        physicsEnabled: true,
        audioEnabled: true,
        debugMode: false,
        maxConnections: 5
    )
    
    init() {
        self.networkingClient = NetworkingClient.shared
        setupNetworkObservers()
    }
    
    deinit {
        cleanup()
    }
    
    // MARK: - Core Engine Methods
    
    /// Initialize the Storm engine with default configuration
    func initialize() async throws {
        guard !isInitialized else { return }
        
        logger.info("Initializing Storm engine...")
        sendInitProgress(0.1, "Initializing Core Systems...")
        
        // Initialize ECS World via FFI
        let ecsResult = init_ecs_world()
        if ecsResult != 0 {
            throw StormError.initializationFailed("ECS World initialization failed")
        }
        
        sendInitProgress(0.3, "Setting up AI Systems...")
        
        // Initialize AI dispatcher (placeholder for now)
        try await initializeAI()
        
        sendInitProgress(0.5, "Configuring Network...")
        
        // Setup networking
        try await setupNetworking()
        
        sendInitProgress(0.7, "Loading Default World...")
        
        // Create default local world
        try await createDefaultWorld()
        
        sendInitProgress(0.9, "Finalizing...")
        
        // Mark as initialized
        isInitialized = true
        networkStatus = .ready
        
        sendInitProgress(1.0, "Ready!")
        
        logger.info("Storm engine initialized successfully")
    }
    
    /// Connect to a virtual world
    func connectToWorld(_ worldConfig: WorldConfiguration) async throws {
        guard isInitialized else {
            throw StormError.notInitialized("Engine not initialized")
        }
        
        logger.info("Connecting to world: \(worldConfig.name)")
        
        // Create connection via appropriate protocol adapter
        let connectionId = try await createWorldConnection(worldConfig)
        
        // Create avatar in the world
        let avatarId = try await createAvatarInWorld(connectionId)
        
        // Update state
        let connectedWorld = ConnectedWorld(
            id: connectionId,
            config: worldConfig,
            avatarId: avatarId,
            connectionTime: Date()
        )
        
        connectedWorlds.append(connectedWorld)
        networkStatus = .connected(worldConfig.name)
        
        logger.info("Successfully connected to \(worldConfig.name)")
    }
    
    /// Disconnect from a world
    func disconnectFromWorld(_ worldId: UUID) async throws {
        logger.info("Disconnecting from world: \(worldId)")
        
        // Remove from connected worlds
        connectedWorlds.removeAll { $0.id == worldId }
        
        if connectedWorlds.isEmpty {
            networkStatus = .ready
        }
        
        logger.info("Disconnected from world")
    }
    
    /// Create and customize avatar
    func createAvatar(at position: simd_float3, customization: AvatarCustomization? = nil) async throws -> UInt64 {
        let avatarId = ffi_create_avatar(position.x, position.y, position.z)
        
        if avatarId == 0 {
            throw StormError.avatarCreationFailed("Failed to create avatar")
        }
        
        // Apply customization if provided
        if let customization = customization {
            try await applyAvatarCustomization(avatarId, customization)
        }
        
        // Update current avatar state
        currentAvatar = AvatarState(
            id: avatarId,
            position: position,
            customization: customization ?? AvatarCustomization.default
        )
        
        return avatarId
    }
    
    /// Update avatar position and state
    func updateAvatar(id: UInt64, position: simd_float3) async throws {
        let result = ffi_set_entity_position(id, position.x, position.y, position.z)
        if result != 0 {
            throw StormError.updateFailed("Failed to update avatar position")
        }
        
        // Update local state
        if currentAvatar?.id == id {
            currentAvatar?.position = position
        }
    }
    
    /// Customize avatar appearance
    func customizeAvatar(id: UInt64, trait: AvatarTrait, value: Float) async throws {
        let result = ffi_customize_avatar(id, trait.rawValue, value)
        if result != 0 {
            throw StormError.customizationFailed("Failed to customize avatar")
        }
        
        // Update local customization state
        if currentAvatar?.id == id {
            currentAvatar?.customization.updateTrait(trait, value: value)
        }
    }
    
    /// Create NPC with AI behavior
    func createNPC(type: NPCType, at position: simd_float3) async throws -> UInt64 {
        let npcId = ffi_create_npc(type.rawValue, position.x, position.y, position.z)
        
        if npcId == 0 {
            throw StormError.npcCreationFailed("Failed to create NPC")
        }
        
        logger.info("Created NPC of type \(type) with ID: \(npcId)")
        return npcId
    }
    
    // MARK: - Private Helper Methods
    
    private func sendInitProgress(_ progress: Double, _ step: String) {
        initializationProgress.send(InitializationProgress(progress: progress, step: step))
    }
    
    private func initializeAI() async throws {
        // Placeholder for AI initialization
        // In full implementation, this would load ML models, connect to Grok API, etc.
        try await Task.sleep(nanoseconds: 500_000_000) // Simulate AI loading
    }
    
    private func setupNetworking() async throws {
        // Initialize network client
        guard let client = networkingClient else {
            throw StormError.networkingFailed("Network client unavailable")
        }
        
        // Test network connectivity
        try await withCheckedThrowingContinuation { continuation in
            client.testConnection { success in
                if success {
                    continuation.resume()
                } else {
                    continuation.resume(throwing: StormError.networkingFailed("Network test failed"))
                }
            }
        }
    }
    
    private func createDefaultWorld() async throws {
        // Create a local default world for immediate use
        let defaultWorld = WorldConfiguration(
            name: "Local World",
            url: "local://default",
            protocol: .local,
            credentials: nil
        )
        
        // Initialize default entities (ground plane, lighting, etc.)
        let avatarId = try await createAvatar(at: simd_float3(0, 1, -5))
        
        logger.info("Default world created with avatar ID: \(avatarId)")
    }
    
    private func createWorldConnection(_ config: WorldConfiguration) async throws -> UUID {
        // Implementation depends on protocol type
        switch config.protocol {
        case .opensim:
            return try await connectToOpenSim(config)
        case .finalverse:
            return try await connectToFinalverse(config)
        case .local:
            return UUID() // Local connection
        }
    }
    
    private func connectToOpenSim(_ config: WorldConfiguration) async throws -> UUID {
        // Implement OpenSim connection logic
        logger.info("Connecting to OpenSim grid: \(config.url)")
        // This would use the Rust protocol adapter for OpenSim
        return UUID()
    }
    
    private func connectToFinalverse(_ config: WorldConfiguration) async throws -> UUID {
        // Implement Finalverse connection logic
        logger.info("Connecting to Finalverse server: \(config.url)")
        // This would use the WebSocket protocol adapter
        return UUID()
    }
    
    private func createAvatarInWorld(_ connectionId: UUID) async throws -> UInt64 {
        // Create avatar in the connected world
        return try await createAvatar(at: simd_float3(128, 25, 128)) // Default spawn position
    }
    
    private func applyAvatarCustomization(_ avatarId: UInt64, _ customization: AvatarCustomization) async throws {
        // Apply each customization trait
        for (trait, value) in customization.traits {
            try await customizeAvatar(id: avatarId, trait: trait, value: value)
        }
    }
    
    private func setupNetworkObservers() {
        // Observe network status changes
        NotificationCenter.default.publisher(for: .networkStatusChanged)
            .receive(on: DispatchQueue.main)
            .sink { [weak self] notification in
                if let status = notification.object as? NetworkStatus {
                    self?.networkStatus = status
                }
            }
            .store(in: &cancellables)
    }
    
    private func cleanup() {
        if let handle = stormHandle {
            // Cleanup FFI handle when available
            stormHandle = nil
        }
        
        cancellables.removeAll()
        logger.info("Storm engine cleaned up")
    }
}

// MARK: - Supporting Types

struct StormConfig {
    let aiEnabled: Bool
    let renderingEnabled: Bool
    let physicsEnabled: Bool
    let audioEnabled: Bool
    let debugMode: Bool
    let maxConnections: Int
}

struct ConnectedWorld {
    let id: UUID
    let config: WorldConfiguration
    let avatarId: UInt64
    let connectionTime: Date
}

struct AvatarState {
    let id: UInt64
    var position: simd_float3
    var customization: AvatarCustomization
}

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

enum NPCType: Int32, CaseIterable {
    case lumi = 0      // Echo of Hope
    case sage = 1      // Echo of Wisdom
    case guardian = 2  // Echo of Protection
    case explorer = 3  // Echo of Discovery
}

enum NetworkStatus: Equatable {
    case disconnected
    case connecting
    case ready
    case connected(String) // World name
    case error(String)
}

enum StormError: LocalizedError {
    case notInitialized(String)
    case initializationFailed(String)
    case networkingFailed(String)
    case avatarCreationFailed(String)
    case npcCreationFailed(String)
    case customizationFailed(String)
    case updateFailed(String)
    case connectionFailed(String)
    
    var errorDescription: String? {
        switch self {
        case .notInitialized(let message),
             .initializationFailed(let message),
             .networkingFailed(let message),
             .avatarCreationFailed(let message),
             .npcCreationFailed(let message),
             .customizationFailed(let message),
             .updateFailed(let message),
             .connectionFailed(let message):
            return message
        }
    }
}

// MARK: - Notification Extensions

extension Notification.Name {
    static let networkStatusChanged = Notification.Name("networkStatusChanged")
}
