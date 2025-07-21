// File: platforms/macOS/Storm/Core/StormFFIBridge.swift
// Description: Complete FFI bridge to Storm Rust core with performance optimization
// Handles all communication between Swift frontend and Rust backend systems

import Foundation
import os

/// Main FFI bridge class for Storm Rust core integration
class StormFFIBridge {
    private static let logger = Logger(subsystem: "com.storm.client", category: "FFI")
    
    // MARK: - Core Engine Functions
    
    /// Initialize the Storm core engine
    static func initializeCore() -> StormResult<OpaquePointer> {
        logger.info("Initializing Storm core via FFI")
        
        let handle = storm_core_init()
        if handle == nil {
            return .failure(.initializationFailed("Core initialization returned null handle"))
        }
        
        return .success(handle!)
    }
    
    /// Shutdown the Storm core engine
    static func shutdownCore(_ handle: OpaquePointer) -> StormResult<Void> {
        logger.info("Shutting down Storm core")
        
        let result = storm_core_shutdown(handle)
        if result != 0 {
            return .failure(.shutdownFailed("Core shutdown failed with code: \(result)"))
        }
        
        return .success(())
    }
    
    /// Get Storm core version information
    static func getCoreVersion() -> String {
        guard let versionPtr = storm_core_get_version() else {
            return "Unknown"
        }
        
        return String(cString: versionPtr)
    }
    
    // MARK: - ECS World Management
    
    /// Initialize the ECS world
    static func initializeECSWorld() -> StormResult<Void> {
        logger.info("Initializing ECS world")
        
        let result = init_ecs_world()
        if result != 0 {
            return .failure(.ecsInitializationFailed("ECS initialization failed with code: \(result)"))
        }
        
        return .success(())
    }
    
    /// Query entity data from ECS
    static func queryEntity(_ entityId: UInt64) -> StormResult<EntityData> {
        var dataSize: Int32 = 0
        guard let dataPtr = storm_query_entity(entityId, &dataSize) else {
            return .failure(.entityNotFound("Entity \(entityId) not found"))
        }
        
        defer { storm_free_entity_data(dataPtr) }
        
        let data = Data(bytes: dataPtr, count: Int(dataSize))
        
        do {
            let entityData = try JSONDecoder().decode(EntityData.self, from: data)
            return .success(entityData)
        } catch {
            return .failure(.deserializationFailed("Failed to deserialize entity data: \(error)"))
        }
    }
    
    /// Update entity components via ECS
    static func updateEntityComponents(_ entityId: UInt64, _ components: [String: Any]) -> StormResult<Void> {
        do {
            let jsonData = try JSONSerialization.data(withJSONObject: components)
            let result = jsonData.withUnsafeBytes { bytes in
                storm_update_entity_components(entityId, bytes.bindMemory(to: UInt8.self).baseAddress, Int32(jsonData.count))
            }
            
            if result != 0 {
                return .failure(.updateFailed("Failed to update entity components with code: \(result)"))
            }
            
            return .success(())
        } catch {
            return .failure(.serializationFailed("Failed to serialize components: \(error)"))
        }
    }
    
    // MARK: - Avatar Management
    
    /// Create a new avatar entity
    static func createAvatar(at position: simd_float3, customization: AvatarCustomizationFFI? = nil) -> StormResult<UInt64> {
        logger.info("Creating avatar at position: \(position)")
        
        let avatarId: UInt64
        
        if let customization = customization {
            // Create avatar with customization
            var customizationData = customization.toCStruct()
            avatarId = withUnsafePointer(to: &customizationData) { ptr in
                ffi_create_avatar_with_customization(position.x, position.y, position.z, ptr)
            }
        } else {
            // Create basic avatar
            avatarId = ffi_create_avatar(position.x, position.y, position.z)
        }
        
        if avatarId == 0 {
            return .failure(.avatarCreationFailed("Avatar creation returned ID 0"))
        }
        
        logger.info("Created avatar with ID: \(avatarId)")
        return .success(avatarId)
    }
    
    /// Customize avatar appearance
    static func customizeAvatar(_ entityId: UInt64, trait: AvatarTraitFFI, value: Float) -> StormResult<Void> {
        logger.debug("Customizing avatar \(entityId) trait \(trait.rawValue) to \(value)")
        
        let result = ffi_customize_avatar(entityId, trait.rawValue, value)
        if result != 0 {
            return .failure(.customizationFailed("Avatar customization failed with code: \(result)"))
        }
        
        return .success(())
    }
    
    /// Apply complex avatar customization
    static func applyAvatarCustomization(_ entityId: UInt64, _ customization: AvatarCustomizationFFI) -> StormResult<Void> {
        logger.info("Applying full customization to avatar \(entityId)")
        
        var customizationData = customization.toCStruct()
        let result = withUnsafePointer(to: &customizationData) { ptr in
            ffi_apply_avatar_customization(entityId, ptr)
        }
        
        if result != 0 {
            return .failure(.customizationFailed("Full customization failed with code: \(result)"))
        }
        
        return .success(())
    }
    
    /// Get avatar customization data
    static func getAvatarCustomization(_ entityId: UInt64) -> StormResult<AvatarCustomizationFFI> {
        var customizationData = AvatarCustomizationCStruct()
        let result = withUnsafeMutablePointer(to: &customizationData) { ptr in
            ffi_get_avatar_customization(entityId, ptr)
        }
        
        if result != 0 {
            return .failure(.queryFailed("Failed to get avatar customization with code: \(result)"))
        }
        
        return .success(AvatarCustomizationFFI.fromCStruct(customizationData))
    }
    
    // MARK: - NPC Management
    
    /// Create an NPC with AI behavior
    static func createNPC(type: NPCTypeFFI, at position: simd_float3, behavior: NPCBehaviorConfig? = nil) -> StormResult<UInt64> {
        logger.info("Creating NPC of type \(type) at position: \(position)")
        
        let npcId: UInt64
        
        if let behavior = behavior {
            var behaviorData = behavior.toCStruct()
            npcId = withUnsafePointer(to: &behaviorData) { ptr in
                ffi_create_npc_with_behavior(type.rawValue, position.x, position.y, position.z, ptr)
            }
        } else {
            npcId = ffi_create_npc(type.rawValue, position.x, position.y, position.z)
        }
        
        if npcId == 0 {
            return .failure(.npcCreationFailed("NPC creation returned ID 0"))
        }
        
        logger.info("Created NPC with ID: \(npcId)")
        return .success(npcId)
    }
    
    /// Update NPC AI behavior
    static func updateNPCBehavior(_ entityId: UInt64, _ behavior: NPCBehaviorConfig) -> StormResult<Void> {
        var behaviorData = behavior.toCStruct()
        let result = withUnsafePointer(to: &behaviorData) { ptr in
            ffi_update_npc_behavior(entityId, ptr)
        }
        
        if result != 0 {
            return .failure(.updateFailed("Failed to update NPC behavior with code: \(result)"))
        }
        
        return .success(())
    }
    
    /// Get NPC dialogue options
    static func getNPCDialogue(_ entityId: UInt64, context: String) -> StormResult<[DialogueOption]> {
        let contextCString = context.cString(using: .utf8)!
        var optionsCount: Int32 = 0
        
        guard let optionsPtr = contextCString.withUnsafeBytes({ bytes in
            ffi_get_npc_dialogue(entityId, bytes.bindMemory(to: Int8.self).baseAddress, &optionsCount)
        }) else {
            return .failure(.queryFailed("Failed to get NPC dialogue"))
        }
        
        defer { ffi_free_dialogue_options(optionsPtr, optionsCount) }
        
        let optionsArray = Array(UnsafeBufferPointer(start: optionsPtr, count: Int(optionsCount)))
        let dialogueOptions = optionsArray.map { DialogueOption.fromCStruct($0) }
        
        return .success(dialogueOptions)
    }
    
    // MARK: - World and Protocol Management
    
    /// Connect to a virtual world
    static func connectToWorld(_ config: WorldConfigurationFFI) -> StormResult<UInt64> {
        logger.info("Connecting to world: \(config.name)")
        
        var configData = config.toCStruct()
        let connectionId = withUnsafePointer(to: &configData) { ptr in
            ffi_connect_to_world(ptr)
        }
        
        if connectionId == 0 {
            return .failure(.connectionFailed("World connection failed"))
        }
        
        logger.info("Connected to world with connection ID: \(connectionId)")
        return .success(connectionId)
    }
    
    /// Disconnect from a world
    static func disconnectFromWorld(_ connectionId: UInt64) -> StormResult<Void> {
        logger.info("Disconnecting from world: \(connectionId)")
        
        let result = ffi_disconnect_from_world(connectionId)
        if result != 0 {
            return .failure(.disconnectionFailed("World disconnection failed with code: \(result)"))
        }
        
        return .success(())
    }
    
    /// Send protocol message
    static func sendProtocolMessage(_ connectionId: UInt64, _ message: ProtocolMessageFFI) -> StormResult<Void> {
        var messageData = message.toCStruct()
        let result = withUnsafePointer(to: &messageData) { ptr in
            ffi_send_protocol_message(connectionId, ptr)
        }
        
        if result != 0 {
            return .failure(.messageSendFailed("Failed to send protocol message with code: \(result)"))
        }
        
        return .success(())
    }
    
    // MARK: - Physics Integration
    
    /// Set entity position with physics
    static func setEntityPosition(_ entityId: UInt64, position: simd_float3, usePhysics: Bool = true) -> StormResult<Void> {
        let result = ffi_set_entity_position(entityId, position.x, position.y, position.z, usePhysics ? 1 : 0)
        if result != 0 {
            return .failure(.updateFailed("Failed to set entity position with code: \(result)"))
        }
        
        return .success(())
    }
    
    /// Apply force to physics body
    static func applyForceToEntity(_ entityId: UInt64, force: simd_float3) -> StormResult<Void> {
        let result = ffi_apply_force_to_entity(entityId, force.x, force.y, force.z)
        if result != 0 {
            return .failure(.physicsOperationFailed("Failed to apply force with code: \(result)"))
        }
        
        return .success(())
    }
    
    /// Simulate physics step
    static func simulatePhysicsStep(_ deltaTime: Float) -> StormResult<Void> {
        let result = ffi_simulate_physics_step(deltaTime)
        if result != 0 {
            return .failure(.physicsOperationFailed("Physics simulation failed with code: \(result)"))
        }
        
        return .success(())
    }
    
    // MARK: - AI Integration
    
    /// Process AI enhancement
    static func processAIEnhancement(_ entityId: UInt64, enhancementType: AIEnhancementType, parameters: [String: Any]) -> StormResult<AIEnhancementResult> {
        do {
            let paramData = try JSONSerialization.data(withJSONObject: parameters)
            var resultSize: Int32 = 0
            
            let resultPtr = paramData.withUnsafeBytes { bytes in
                ffi_process_ai_enhancement(
                    entityId,
                    enhancementType.rawValue,
                    bytes.bindMemory(to: UInt8.self).baseAddress,
                    Int32(paramData.count),
                    &resultSize
                )
            }
            
            guard let resultPtr = resultPtr else {
                return .failure(.aiProcessingFailed("AI enhancement returned null result"))
            }
            
            defer { ffi_free_ai_result(resultPtr) }
            
            let resultData = Data(bytes: resultPtr, count: Int(resultSize))
            let result = try JSONDecoder().decode(AIEnhancementResult.self, from: resultData)
            
            return .success(result)
        } catch {
            return .failure(.aiProcessingFailed("AI enhancement failed: \(error)"))
        }
    }
    
    /// Get AI behavioral prediction
    static func getAIBehaviorPrediction(_ entityId: UInt64, timeHorizon: Float) -> StormResult<BehaviorPrediction> {
        var predictionSize: Int32 = 0
        guard let predictionPtr = ffi_get_ai_behavior_prediction(entityId, timeHorizon, &predictionSize) else {
            return .failure(.aiProcessingFailed("Failed to get AI behavior prediction"))
        }
        
        defer { ffi_free_behavior_prediction(predictionPtr) }
        
        do {
            let predictionData = Data(bytes: predictionPtr, count: Int(predictionSize))
            let prediction = try JSONDecoder().decode(BehaviorPrediction.self, from: predictionData)
            return .success(prediction)
        } catch {
            return .failure(.deserializationFailed("Failed to deserialize behavior prediction: \(error)"))
        }
    }
    
    // MARK: - Asset Management
    
    /// Load asset from file
    static func loadAsset(_ assetPath: String, assetType: AssetType) -> StormResult<UInt64> {
        let pathCString = assetPath.cString(using: .utf8)!
        let assetId = pathCString.withUnsafeBytes { bytes in
            ffi_load_asset(bytes.bindMemory(to: Int8.self).baseAddress, assetType.rawValue)
        }
        
        if assetId == 0 {
            return .failure(.assetLoadFailed("Failed to load asset: \(assetPath)"))
        }
        
        return .success(assetId)
    }
    
    /// Get asset data
    static func getAssetData(_ assetId: UInt64) -> StormResult<AssetData> {
        var dataSize: Int32 = 0
        guard let dataPtr = ffi_get_asset_data(assetId, &dataSize) else {
            return .failure(.assetNotFound("Asset \(assetId) not found"))
        }
        
        defer { ffi_free_asset_data(dataPtr) }
        
        do {
            let assetData = Data(bytes: dataPtr, count: Int(dataSize))
            let asset = try JSONDecoder().decode(AssetData.self, from: assetData)
            return .success(asset)
        } catch {
            return .failure(.deserializationFailed("Failed to deserialize asset data: \(error)"))
        }
    }
    
    /// Verify asset authenticity (blockchain/crypto)
    static func verifyAsset(_ assetId: UInt64) -> StormResult<AssetVerification> {
        var verificationSize: Int32 = 0
        guard let verificationPtr = ffi_verify_asset(assetId, &verificationSize) else {
            return .failure(.verificationFailed("Asset verification failed"))
        }
        
        defer { ffi_free_verification_data(verificationPtr) }
        
        do {
            let verificationData = Data(bytes: verificationPtr, count: Int(verificationSize))
            let verification = try JSONDecoder().decode(AssetVerification.self, from: verificationData)
            return .success(verification)
        } catch {
            return .failure(.deserializationFailed("Failed to deserialize verification data: \(error)"))
        }
    }
    
    // MARK: - Performance and Monitoring
    
    /// Get performance metrics
    static func getPerformanceMetrics() -> StormResult<PerformanceMetrics> {
        var metricsSize: Int32 = 0
        guard let metricsPtr = ffi_get_performance_metrics(&metricsSize) else {
            return .failure(.queryFailed("Failed to get performance metrics"))
        }
        
        defer { ffi_free_performance_metrics(metricsPtr) }
        
        do {
            let metricsData = Data(bytes: metricsPtr, count: Int(metricsSize))
            let metrics = try JSONDecoder().decode(PerformanceMetrics.self, from: metricsData)
            return .success(metrics)
        } catch {
            return .failure(.deserializationFailed("Failed to deserialize performance metrics: \(error)"))
        }
    }
    
    /// Set performance target
    static func setPerformanceTarget(_ target: PerformanceTarget) -> StormResult<Void> {
        var targetData = target.toCStruct()
        let result = withUnsafePointer(to: &targetData) { ptr in
            ffi_set_performance_target(ptr)
        }
        
        if result != 0 {
            return .failure(.configurationFailed("Failed to set performance target with code: \(result)"))
        }
        
        return .success(())
    }
    
    // MARK: - Error Handling and Utilities
    
    /// Get last error message from Rust core
    static func getLastError() -> String? {
        guard let errorPtr = ffi_get_last_error() else {
            return nil
        }
        
        return String(cString: errorPtr)
    }
    
    /// Clear last error
    static func clearLastError() {
        ffi_clear_last_error()
    }
    
    /// Enable debug logging in Rust core
    static func setDebugLogging(_ enabled: Bool) {
        ffi_set_debug_logging(enabled ? 1 : 0)
    }
}

// MARK: - FFI Data Structures

/// C-compatible avatar customization structure
struct AvatarCustomizationCStruct {
    var height: Float
    var bodyMass: Float
    var skinTone: Float
    var eyeColor: Float
    var hairColor: Float
    var faceShape: Float
    var musculature: Float
    var accessories: UInt32
    var echoIntensity: Float
    var echoType: Int32
    var clothingStyle: Int32
    var reserved: (UInt8, UInt8, UInt8, UInt8) // For future expansion
}

/// C-compatible world configuration structure
struct WorldConfigurationCStruct {
    var name: (Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8,
               Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8,
               Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8,
               Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8) // 64 chars
    var url: (Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8,
              Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8,
              Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8,
              Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8,
              Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8,
              Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8,
              Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8,
              Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8) // 128 chars
    var protocolType: Int32
    var port: UInt16
    var useSSL: UInt8
    var reserved: UInt8
}

/// C-compatible NPC behavior configuration
struct NPCBehaviorCStruct {
    var aggressiveness: Float
    var curiosity: Float
    var sociability: Float
    var intelligence: Float
    var movementSpeed: Float
    var interactionRadius: Float
    var behaviorType: Int32
    var personalityTraits: UInt32 // Bitfield for personality traits
}

/// C-compatible performance target
struct PerformanceTargetCStruct {
    var targetFPS: Float
    var maxMemoryMB: UInt32
    var maxCPUPercent: Float
    var qualityLevel: Int32
    var enableOptimizations: UInt8
    var reserved: (UInt8, UInt8, UInt8) // Padding
}

// MARK: - Swift Wrapper Types

/// Avatar customization data for FFI
struct AvatarCustomizationFFI {
    let height: Float
    let bodyMass: Float
    let skinTone: Float
    let eyeColor: Float
    let hairColor: Float
    let faceShape: Float
    let musculature: Float
    let accessories: UInt32
    let echoIntensity: Float
    let echoType: EchoTypeFFI
    let clothingStyle: ClothingStyleFFI
    
    func toCStruct() -> AvatarCustomizationCStruct {
        return AvatarCustomizationCStruct(
            height: height,
            bodyMass: bodyMass,
            skinTone: skinTone,
            eyeColor: eyeColor,
            hairColor: hairColor,
            faceShape: faceShape,
            musculature: musculature,
            accessories: accessories,
            echoIntensity: echoIntensity,
            echoType: echoType.rawValue,
            clothingStyle: clothingStyle.rawValue,
            reserved: (0, 0, 0, 0)
        )
    }
    
    static func fromCStruct(_ cStruct: AvatarCustomizationCStruct) -> AvatarCustomizationFFI {
        return AvatarCustomizationFFI(
            height: cStruct.height,
            bodyMass: cStruct.bodyMass,
            skinTone: cStruct.skinTone,
            eyeColor: cStruct.eyeColor,
            hairColor: cStruct.hairColor,
            faceShape: cStruct.faceShape,
            musculature: cStruct.musculature,
            accessories: cStruct.accessories,
            echoIntensity: cStruct.echoIntensity,
            echoType: EchoTypeFFI(rawValue: cStruct.echoType) ?? .hope,
            clothingStyle: ClothingStyleFFI(rawValue: cStruct.clothingStyle) ?? .casual
        )
    }
}

/// World configuration for FFI
struct WorldConfigurationFFI {
    let name: String
    let url: String
    let protocolType: ProtocolTypeFFI
    let port: UInt16
    let useSSL: Bool
    
    func toCStruct() -> WorldConfigurationCStruct {
        var config = WorldConfigurationCStruct(
            name: (0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,
                   0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0),
            url: (0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,
                  0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,
                  0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,
                  0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0),
            protocolType: protocolType.rawValue,
            port: port,
            useSSL: useSSL ? 1 : 0,
            reserved: 0
        )
        
        // Copy name string (safely)
        let nameBytes = name.utf8.prefix(63) // Leave room for null terminator
        withUnsafeMutableBytes(of: &config.name) { buffer in
            nameBytes.enumerated().forEach { index, byte in
                buffer[index] = Int8(bitPattern: byte)
            }
        }
        
        // Copy URL string (safely)
        let urlBytes = url.utf8.prefix(127) // Leave room for null terminator
        withUnsafeMutableBytes(of: &config.url) { buffer in
            urlBytes.enumerated().forEach { index, byte in
                buffer[index] = Int8(bitPattern: byte)
            }
        }
        
        return config
    }
}

/// NPC behavior configuration for FFI
struct NPCBehaviorConfig {
    let aggressiveness: Float
    let curiosity: Float
    let sociability: Float
    let intelligence: Float
    let movementSpeed: Float
    let interactionRadius: Float
    let behaviorType: NPCBehaviorTypeFFI
    let personalityTraits: Set<PersonalityTraitFFI>
    
    func toCStruct() -> NPCBehaviorCStruct {
        let traitsBitfield = personalityTraits.reduce(0) { result, trait in
            result | (1 << trait.rawValue)
        }
        
        return NPCBehaviorCStruct(
            aggressiveness: aggressiveness,
            curiosity: curiosity,
            sociability: sociability,
            intelligence: intelligence,
            movementSpeed: movementSpeed,
            interactionRadius: interactionRadius,
            behaviorType: behaviorType.rawValue,
            personalityTraits: UInt32(traitsBitfield)
        )
    }
}

/// Performance target configuration
struct PerformanceTarget {
    let targetFPS: Float
    let maxMemoryMB: UInt32
    let maxCPUPercent: Float
    let qualityLevel: QualityLevel
    let enableOptimizations: Bool
    
    func toCStruct() -> PerformanceTargetCStruct {
        return PerformanceTargetCStruct(
            targetFPS: targetFPS,
            maxMemoryMB: maxMemoryMB,
            maxCPUPercent: maxCPUPercent,
            qualityLevel: qualityLevel.rawValue,
            enableOptimizations: enableOptimizations ? 1 : 0,
            reserved: (0, 0, 0)
        )
    }
}

// MARK: - Enums for FFI

enum AvatarTraitFFI: Int32, CaseIterable {
    case height = 0
    case bodyMass = 1
    case skinTone = 2
    case eyeColor = 3
    case hairColor = 4
    case faceShape = 5
    case musculature = 6
    case accessories = 7
    case echoIntensity = 8
    case transparency = 9
    case facialWidth = 10
    case cheekBones = 11
    case jawWidth = 12
    case chinProminence = 13
    case eyeSize = 14
    case eyeDistance = 15
    case noseSize = 16
    case mouthWidth = 17
    case lipThickness = 18
}

enum NPCTypeFFI: Int32, CaseIterable {
    case lumi = 0      // Echo of Hope
    case sage = 1      // Echo of Wisdom
    case guardian = 2  // Echo of Protection
    case explorer = 3  // Echo of Discovery
    case merchant = 4  // Commercial NPC
    case guide = 5     // Tutorial/Help NPC
    case companion = 6 // Player companion
    case adversary = 7 // Challenging opponent
}

enum EchoTypeFFI: Int32, CaseIterable {
    case hope = 0
    case wisdom = 1
    case memory = 2
    case logic = 3
    case dreams = 4
    case protection = 5
    case discovery = 6
    case creation = 7
}

enum ClothingStyleFFI: Int32, CaseIterable {
    case casual = 0
    case formal = 1
    case futuristic = 2
    case fantasy = 3
    case minimal = 4
    case traditional = 5
    case punk = 6
    case elegant = 7
}

enum ProtocolTypeFFI: Int32, CaseIterable {
    case opensim = 0
    case finalverse = 1
    case webSocket = 2
    case local = 3
    case custom = 4
}

enum AssetType: Int32, CaseIterable {
    case mesh = 0
    case texture = 1
    case audio = 2
    case animation = 3
    case script = 4
    case material = 5
    case world = 6
    case avatar = 7
}

enum AIEnhancementType: Int32, CaseIterable {
    case procedural = 0
    case optimization = 1
    case prediction = 2
    case generation = 3
    case analysis = 4
    case enhancement = 5
}

enum NPCBehaviorTypeFFI: Int32, CaseIterable {
    case passive = 0
    case interactive = 1
    case aggressive = 2
    case helpful = 3
    case mysterious = 4
    case playful = 5
}

enum PersonalityTraitFFI: Int32, CaseIterable {
    case openness = 0
    case conscientiousness = 1
    case extraversion = 2
    case agreeableness = 3
    case neuroticism = 4
    case courage = 5
    case compassion = 6
    case curiosity = 7
    case creativity = 8
    case loyalty = 9
}

enum QualityLevel: Int32, CaseIterable {
    case low = 0
    case medium = 1
    case high = 2
    case ultra = 3
    case adaptive = 4
}

// MARK: - Result and Data Types

/// Result type for FFI operations
enum StormResult<T> {
    case success(T)
    case failure(StormFFIError)
    
    var isSuccess: Bool {
        switch self {
        case .success: return true
        case .failure: return false
        }
    }
    
    var value: T? {
        switch self {
        case .success(let value): return value
        case .failure: return nil
        }
    }
    
    var error: StormFFIError? {
        switch self {
        case .success: return nil
        case .failure(let error): return error
        }
    }
}

/// FFI Error types
enum StormFFIError: LocalizedError {
    case initializationFailed(String)
    case shutdownFailed(String)
    case ecsInitializationFailed(String)
    case entityNotFound(String)
    case avatarCreationFailed(String)
    case npcCreationFailed(String)
    case customizationFailed(String)
    case updateFailed(String)
    case connectionFailed(String)
    case disconnectionFailed(String)
    case messageSendFailed(String)
    case physicsOperationFailed(String)
    case aiProcessingFailed(String)
    case assetLoadFailed(String)
    case assetNotFound(String)
    case verificationFailed(String)
    case queryFailed(String)
    case configurationFailed(String)
    case serializationFailed(String)
    case deserializationFailed(String)
    
    var errorDescription: String? {
        switch self {
        case .initializationFailed(let msg),
             .shutdownFailed(let msg),
             .ecsInitializationFailed(let msg),
             .entityNotFound(let msg),
             .avatarCreationFailed(let msg),
             .npcCreationFailed(let msg),
             .customizationFailed(let msg),
             .updateFailed(let msg),
             .connectionFailed(let msg),
             .disconnectionFailed(let msg),
             .messageSendFailed(let msg),
             .physicsOperationFailed(let msg),
             .aiProcessingFailed(let msg),
             .assetLoadFailed(let msg),
             .assetNotFound(let msg),
             .verificationFailed(let msg),
             .queryFailed(let msg),
             .configurationFailed(let msg),
             .serializationFailed(let msg),
             .deserializationFailed(let msg):
            return msg
        }
    }
}

// MARK: - Data Transfer Objects

struct EntityData: Codable {
    let id: UInt64
    let position: [Float] // [x, y, z]
    let rotation: [Float] // [x, y, z, w] quaternion
    let scale: [Float]    // [x, y, z]
    let components: [String: AnyCodable]
    let metadata: [String: String]
}

struct AIEnhancementResult: Codable {
    let success: Bool
    let enhancementId: UInt64
    let parameters: [String: AnyCodable]
    let confidence: Float
    let processingTime: Float
    let recommendations: [String]
}

struct BehaviorPrediction: Codable {
    let entityId: UInt64
    let timeHorizon: Float
    let predictedActions: [PredictedAction]
    let confidence: Float
    let alternativeScenarios: [BehaviorScenario]
}

struct PredictedAction: Codable {
    let actionType: String
    let parameters: [String: AnyCodable]
    let probability: Float
    let timing: Float
}

struct BehaviorScenario: Codable {
    let name: String
    let probability: Float
    let actions: [PredictedAction]
}

struct AssetData: Codable {
    let id: UInt64
    let name: String
    let assetType: String
    let size: UInt64
    let checksum: String
    let metadata: [String: AnyCodable]
    let dependencies: [UInt64]
}

struct AssetVerification: Codable {
    let verified: Bool
    let signature: String
    let certificate: String
    let blockchainHash: String?
    let trustScore: Float
    let verificationTime: String
}

struct PerformanceMetrics: Codable {
    let fps: Float
    let frameTime: Float
    let memoryUsage: UInt64
    let cpuUsage: Float
    let gpuUsage: Float
    let networkLatency: Float
    let entitiesCount: UInt32
    let drawCalls: UInt32
    let triangles: UInt64
}

struct DialogueOption: Codable {
    let id: UInt32
    let text: String
    let responseType: String
    let emotionalTone: String
    let consequences: [String]
    
    static func fromCStruct(_ cStruct: DialogueOptionCStruct) -> DialogueOption {
        return DialogueOption(
            id: cStruct.id,
            text: String(cString: &cStruct.text.0),
            responseType: String(cString: &cStruct.responseType.0),
            emotionalTone: String(cString: &cStruct.emotionalTone.0),
            consequences: [] // Would parse consequences array
        )
    }
}

struct DialogueOptionCStruct {
    let id: UInt32
    let text: (Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8,
               Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8,
               Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8,
               Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8) // 64 chars
    let responseType: (Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8) // 16 chars
    let emotionalTone: (Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8, Int8) // 16 chars
    let consequencesCount: UInt8
    let reserved: (UInt8, UInt8, UInt8)
}

struct ProtocolMessageFFI {
    let messageType: UInt32
    let data: Data
    let priority: UInt8
    let reliable: Bool
    
    func toCStruct() -> ProtocolMessageCStruct {
        return ProtocolMessageCStruct(
            messageType: messageType,
            dataSize: UInt32(data.count),
            priority: priority,
            reliable: reliable ? 1 : 0,
            reserved: (0, 0)
        )
    }
}

struct ProtocolMessageCStruct {
    let messageType: UInt32
    let dataSize: UInt32
    let priority: UInt8
    let reliable: UInt8
    let reserved: (UInt8, UInt8)
}

// MARK: - Helper for Any Codable

struct AnyCodable: Codable {
    let value: Any
    
    init<T>(_ value: T?) {
        self.value = value ?? ()
    }
    
    init(from decoder: Decoder) throws {
        let container = try decoder.singleValueContainer()
        
        if container.decodeNil() {
            value = ()
        } else if let bool = try? container.decode(Bool.self) {
            value = bool
        } else if let int = try? container.decode(Int.self) {
            value = int
        } else if let double = try? container.decode(Double.self) {
            value = double
        } else if let string = try? container.decode(String.self) {
            value = string
        } else if let array = try? container.decode([AnyCodable].self) {
            value = array.map { $0.value }
        } else if let dictionary = try? container.decode([String: AnyCodable].self) {
            value = dictionary.mapValues { $0.value }
        } else {
            throw DecodingError.dataCorruptedError(in: container, debugDescription: "Cannot decode AnyCodable")
        }
    }
    
    func encode(to encoder: Encoder) throws {
        var container = encoder.singleValueContainer()
        
        switch value {
        case is Void:
            try container.encodeNil()
        case let bool as Bool:
            try container.encode(bool)
        case let int as Int:
            try container.encode(int)
        case let double as Double:
            try container.encode(double)
        case let string as String:
            try container.encode(string)
        case let array as [Any]:
            try container.encode(array.map(AnyCodable.init))
        case let dictionary as [String: Any]:
            try container.encode(dictionary.mapValues(AnyCodable.init))
        default:
            throw EncodingError.invalidValue(value, EncodingError.Context(codingPath: encoder.codingPath, debugDescription: "Cannot encode AnyCodable"))
        }
    }
}

// MARK: - C Function Declarations (to be linked with Rust library)

@_silgen_name("storm_core_init")
func storm_core_init() -> OpaquePointer?

@_silgen_name("storm_core_shutdown")
func storm_core_shutdown(_ handle: OpaquePointer) -> Int32

@_silgen_name("storm_core_get_version")
func storm_core_get_version() -> UnsafePointer<Int8>?

@_silgen_name("init_ecs_world")
func init_ecs_world() -> Int32

@_silgen_name("ffi_create_avatar")
func ffi_create_avatar(_ x: Float, _ y: Float, _ z: Float) -> UInt64

@_silgen_name("ffi_create_avatar_with_customization")
func ffi_create_avatar_with_customization(_ x: Float, _ y: Float, _ z: Float, _ customization: UnsafePointer<AvatarCustomizationCStruct>) -> UInt64

@_silgen_name("ffi_customize_avatar")
func ffi_customize_avatar(_ entityId: UInt64, _ traitIndex: Int32, _ morph: Float) -> Int32

@_silgen_name("ffi_apply_avatar_customization")
func ffi_apply_avatar_customization(_ entityId: UInt64, _ customization: UnsafePointer<AvatarCustomizationCStruct>) -> Int32

@_silgen_name("ffi_get_avatar_customization")
func ffi_get_avatar_customization(_ entityId: UInt64, _ customization: UnsafeMutablePointer<AvatarCustomizationCStruct>) -> Int32

@_silgen_name("ffi_create_npc")
func ffi_create_npc(_ echoIndex: Int32, _ x: Float, _ y: Float, _ z: Float) -> UInt64

@_silgen_name("ffi_create_npc_with_behavior")
func ffi_create_npc_with_behavior(_ npcType: Int32, _ x: Float, _ y: Float, _ z: Float, _ behavior: UnsafePointer<NPCBehaviorCStruct>) -> UInt64

@_silgen_name("ffi_update_npc_behavior")
func ffi_update_npc_behavior(_ entityId: UInt64, _ behavior: UnsafePointer<NPCBehaviorCStruct>) -> Int32

@_silgen_name("ffi_get_npc_dialogue")
func ffi_get_npc_dialogue(_ entityId: UInt64, _ context: UnsafePointer<Int8>, _ optionsCount: UnsafeMutablePointer<Int32>) -> UnsafePointer<DialogueOptionCStruct>?

@_silgen_name("ffi_free_dialogue_options")
func ffi_free_dialogue_options(_ options: UnsafePointer<DialogueOptionCStruct>, _ count: Int32)

@_silgen_name("ffi_set_entity_position")
func ffi_set_entity_position(_ entityId: UInt64, _ x: Float, _ y: Float, _ z: Float, _ usePhysics: Int32) -> Int32

@_silgen_name("ffi_apply_force_to_entity")
func ffi_apply_force_to_entity(_ entityId: UInt64, _ fx: Float, _ fy: Float, _ fz: Float) -> Int32

@_silgen_name("ffi_simulate_physics_step")
func ffi_simulate_physics_step(_ deltaTime: Float) -> Int32

@_silgen_name("ffi_connect_to_world")
func ffi_connect_to_world(_ config: UnsafePointer<WorldConfigurationCStruct>) -> UInt64

@_silgen_name("ffi_disconnect_from_world")
func ffi_disconnect_from_world(_ connectionId: UInt64) -> Int32

@_silgen_name("ffi_send_protocol_message")
func ffi_send_protocol_message(_ connectionId: UInt64, _ message: UnsafePointer<ProtocolMessageCStruct>) -> Int32

@_silgen_name("storm_query_entity")
func storm_query_entity(_ entityId: UInt64, _ dataSize: UnsafeMutablePointer<Int32>) -> UnsafePointer<UInt8>?

@_silgen_name("storm_free_entity_data")
func storm_free_entity_data(_ data: UnsafePointer<UInt8>)

@_silgen_name("storm_update_entity_components")
func storm_update_entity_components(_ entityId: UInt64, _ data: UnsafePointer<UInt8>, _ dataSize: Int32) -> Int32

@_silgen_name("ffi_process_ai_enhancement")
func ffi_process_ai_enhancement(_ entityId: UInt64, _ enhancementType: Int32, _ parameters: UnsafePointer<UInt8>, _ paramSize: Int32, _ resultSize: UnsafeMutablePointer<Int32>) -> UnsafePointer<UInt8>?

@_silgen_name("ffi_free_ai_result")
func ffi_free_ai_result(_ result: UnsafePointer<UInt8>)

@_silgen_name("ffi_get_ai_behavior_prediction")
func ffi_get_ai_behavior_prediction(_ entityId: UInt64, _ timeHorizon: Float, _ predictionSize: UnsafeMutablePointer<Int32>) -> UnsafePointer<UInt8>?

@_silgen_name("ffi_free_behavior_prediction")
func ffi_free_behavior_prediction(_ prediction: UnsafePointer<UInt8>)

@_silgen_name("ffi_load_asset")
func ffi_load_asset(_ assetPath: UnsafePointer<Int8>, _ assetType: Int32) -> UInt64

@_silgen_name("ffi_get_asset_data")
func ffi_get_asset_data(_ assetId: UInt64, _ dataSize: UnsafeMutablePointer<Int32>) -> UnsafePointer<UInt8>?

@_silgen_name("ffi_free_asset_data")
func ffi_free_asset_data(_ data: UnsafePointer<UInt8>)

@_silgen_name("ffi_verify_asset")
func ffi_verify_asset(_ assetId: UInt64, _ verificationSize: UnsafeMutablePointer<Int32>) -> UnsafePointer<UInt8>?

@_silgen_name("ffi_free_verification_data")
func ffi_free_verification_data(_ data: UnsafePointer<UInt8>)

@_silgen_name("ffi_get_performance_metrics")
func ffi_get_performance_metrics(_ metricsSize: UnsafeMutablePointer<Int32>) -> UnsafePointer<UInt8>?

@_silgen_name("ffi_free_performance_metrics")
func ffi_free_performance_metrics(_ metrics: UnsafePointer<UInt8>)

@_silgen_name("ffi_set_performance_target")
func ffi_set_performance_target(_ target: UnsafePointer<PerformanceTargetCStruct>) -> Int32

@_silgen_name("ffi_get_last_error")
func ffi_get_last_error() -> UnsafePointer<Int8>?

@_silgen_name("ffi_clear_last_error")
func ffi_clear_last_error()

@_silgen_name("ffi_set_debug_logging")
func ffi_set_debug_logging(_ enabled: Int32)
