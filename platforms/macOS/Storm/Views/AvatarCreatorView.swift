// File: Views/AvatarCreatorView.swift
// Description: Quick avatar creation interface for new users
// Simplified version of the full customization system for rapid avatar generation

import SwiftUI
import RealityKit
import os

// MARK: - Storm-Specific Types (to avoid conflicts)

enum StormEchoType: CaseIterable {
    case hope, wisdom, memory, logic, dreams, protection, discovery, creation
}

enum StormEnergyPattern: CaseIterable {
    case flowing, pulsing, stable, chaotic
}

enum StormManifestation: CaseIterable {
    case physical, ethereal, digital, hybrid
}

// MARK: - Storm Echo Component
struct StormEchoComponent: Component {
    let echoType: StormEchoType
    let intensity: Float
    let energyPattern: StormEnergyPattern
    let manifestation: StormManifestation
    
    init(echoType: StormEchoType, intensity: Float = 0.5, energyPattern: StormEnergyPattern = .flowing, manifestation: StormManifestation = .physical) {
        self.echoType = echoType
        self.intensity = intensity
        self.energyPattern = energyPattern
        self.manifestation = manifestation
    }
}

// MARK: - ModelEntity Extensions
extension ModelEntity {
    func safeUpdateMaterials(_ newMaterials: [RealityKit.Material]) {
        if var modelComponent = self.components[ModelComponent.self] {
            modelComponent.materials = newMaterials
            self.components[ModelComponent.self] = modelComponent
        }
    }
}

struct AvatarCreatorView: View {
    @EnvironmentObject var stormEngine: StormEngine
    @Environment(\.presentationMode) var presentationMode
    
    @State private var selectedArchetype: AvatarCreatorArchetype = .human
    @State private var selectedPreset: AvatarPreset = .balanced
    @State private var avatarName = ""
    @State private var isCreating = false
    @State private var creationError: String?
    @State private var previewAvatarId: UInt64?
    
    // Echo-specific properties with Storm prefix to avoid conflicts
    @State private var selectedStormEchoType: StormEchoType = .hope
    @State private var stormEnergyPattern: StormEnergyPattern = .flowing
    @State private var stormManifestation: StormManifestation = .physical
    @State private var stormEchoIntensity: Float = 0.5
    
    private let logger = Logger(subsystem: "com.storm.client", category: "AvatarCreator")
    
    var body: some View {
        NavigationView {
            HStack(spacing: 0) {
                // Configuration panel
                configurationPanel
                
                Divider()
                
                // Preview panel
                previewPanel
            }
            .navigationTitle("Create New Avatar")
            .toolbar {
                ToolbarItem(placement: .cancellationAction) {
                    Button("Cancel") {
                        presentationMode.wrappedValue.dismiss()
                    }
                }
                
                ToolbarItem(placement: .confirmationAction) {
                    Button("Create Avatar") {
                        createAvatar()
                    }
                    .disabled(!isConfigurationValid || isCreating)
                }
            }
        }
        .frame(width: 800, height: 600)
        .alert("Creation Error", isPresented: .constant(creationError != nil)) {
            Button("OK") {
                creationError = nil
            }
        } message: {
            if let error = creationError {
                Text(error)
            }
        }
        .task {
            await createPreviewAvatar()
        }
    }
    
    // MARK: - Configuration Panel
    
    private var configurationPanel: some View {
        VStack(alignment: .leading, spacing: 24) {
            // Header
            VStack(alignment: .leading, spacing: 8) {
                Text("Avatar Creator")
                    .font(.title2)
                    .fontWeight(.bold)
                
                Text("Create your virtual identity in seconds")
                    .font(.subheadline)
                    .foregroundColor(.secondary)
            }
            
            // Avatar Name
            VStack(alignment: .leading, spacing: 8) {
                Text("Avatar Name")
                    .font(.headline)
                
                TextField("Enter your avatar name", text: $avatarName)
                    .textFieldStyle(.roundedBorder)
                
                Text("This name will be visible to other users in virtual worlds")
                    .font(.caption)
                    .foregroundColor(.secondary)
            }
            
            // Archetype Selection
            VStack(alignment: .leading, spacing: 12) {
                Text("Archetype")
                    .font(.headline)
                
                Text("Choose the fundamental nature of your avatar")
                    .font(.caption)
                    .foregroundColor(.secondary)
                
                LazyVGrid(columns: Array(repeating: GridItem(.flexible()), count: 2), spacing: 12) {
                    ForEach(AvatarCreatorArchetype.allCases, id: \.self) { archetype in
                        ArchetypeCard(
                            archetype: archetype,
                            isSelected: selectedArchetype == archetype
                        ) {
                            selectedArchetype = archetype
                            Task { await updatePreviewAvatar() }
                        }
                    }
                }
            }
            
            // Echo Configuration (shown only for Echo and Hybrid types)
            if selectedArchetype == AvatarCreatorArchetype.echo || selectedArchetype == AvatarCreatorArchetype.hybrid {
                stormEchoConfigurationSection
            }
            
            // Preset Selection
            VStack(alignment: .leading, spacing: 12) {
                Text("Appearance Preset")
                    .font(.headline)
                
                Text("Quick appearance configurations based on your archetype")
                    .font(.caption)
                    .foregroundColor(.secondary)
                
                VStack(spacing: 8) {
                    ForEach(presetsForArchetype, id: \.self) { preset in
                        PresetRow(
                            preset: preset,
                            isSelected: selectedPreset == preset
                        ) {
                            selectedPreset = preset
                            Task { await updatePreviewAvatar() }
                        }
                    }
                }
            }
            
            Spacer()
            
            // Quick Actions
            VStack(spacing: 12) {
                Button("Randomize Appearance") {
                    randomizeAppearance()
                }
                .buttonStyle(.bordered)
                .frame(maxWidth: .infinity)
                
                Button("AI Generate Avatar") {
                    generateAIAvatar()
                }
                .buttonStyle(.bordered)
                .frame(maxWidth: .infinity)
            }
        }
        .padding()
        .frame(width: 350)
        .background(Color(NSColor.controlBackgroundColor))
    }
    
    // MARK: - Storm Echo Configuration Section
    private var stormEchoConfigurationSection: some View {
        VStack(alignment: .leading, spacing: 16) {
            Text("Echo Properties")
                .font(.headline)
            
            VStack(alignment: .leading, spacing: 12) {
                Text("Echo Type")
                    .font(.subheadline)
                
                Picker("Echo Type", selection: $selectedStormEchoType) {
                    Text("Hope").tag(StormEchoType.hope)
                    Text("Wisdom").tag(StormEchoType.wisdom)
                    Text("Memory").tag(StormEchoType.memory)
                    Text("Logic").tag(StormEchoType.logic)
                    Text("Dreams").tag(StormEchoType.dreams)
                    Text("Protection").tag(StormEchoType.protection)
                    Text("Discovery").tag(StormEchoType.discovery)
                    Text("Creation").tag(StormEchoType.creation)
                }
                .pickerStyle(MenuPickerStyle())
                .onChange(of: selectedStormEchoType) {
                    Task { await updatePreviewAvatar() }
                }
            }
            
            VStack(alignment: .leading, spacing: 12) {
                Text("Energy Pattern")
                    .font(.subheadline)
                
                Picker("Energy Pattern", selection: $stormEnergyPattern) {
                    Text("Flowing").tag(StormEnergyPattern.flowing)
                    Text("Pulsing").tag(StormEnergyPattern.pulsing)
                    Text("Stable").tag(StormEnergyPattern.stable)
                    Text("Chaotic").tag(StormEnergyPattern.chaotic)
                }
                .pickerStyle(MenuPickerStyle())
                .onChange(of: stormEnergyPattern) {
                    Task { await updatePreviewAvatar() }
                }
            }
            
            VStack(alignment: .leading, spacing: 12) {
                Text("Manifestation")
                    .font(.subheadline)
                
                Picker("Manifestation", selection: $stormManifestation) {
                    Text("Physical").tag(StormManifestation.physical)
                    Text("Ethereal").tag(StormManifestation.ethereal)
                    Text("Digital").tag(StormManifestation.digital)
                    Text("Hybrid").tag(StormManifestation.hybrid)
                }
                .pickerStyle(MenuPickerStyle())
                .onChange(of: stormManifestation) {
                    Task { await updatePreviewAvatar() }
                }
            }
            
            VStack(alignment: .leading, spacing: 12) {
                HStack {
                    Text("Echo Intensity")
                        .font(.subheadline)
                    Spacer()
                    Text(String(format: "%.2f", stormEchoIntensity))
                        .font(.caption)
                        .foregroundColor(.secondary)
                }
                
                Slider(value: $stormEchoIntensity, in: 0...1)
                    .onChange(of: stormEchoIntensity) { _ in
                        Task { await updatePreviewAvatar() }
                    }
            }
        }
    }
    
    // MARK: - Preview Panel
    
    private var previewPanel: some View {
        VStack {
            // Preview header
            VStack(spacing: 8) {
                Text("Preview")
                    .font(.title2)
                    .fontWeight(.semibold)
                
                if !avatarName.isEmpty {
                    Text(avatarName)
                        .font(.headline)
                        .foregroundColor(.secondary)
                }
            }
            .padding(.top)
            
            // 3D Preview - use RealityView scene for setup and updates
            RealityView { scene in
                Task { await setupPreview(scene: scene) }
            } update: { scene in
                Task { await updatePreview(scene: scene) }
            }
            .background(Color.black.opacity(0.05))
            .cornerRadius(12)
            .padding()
            
            // Preview controls
            VStack(spacing: 12) {
                HStack {
                    Button("Rotate Left") {
                        rotatePreview(-45)
                    }
                    .buttonStyle(.bordered)
                    .controlSize(.small)
                    
                    Spacer()
                    
                    Button("Reset View") {
                        resetPreviewCamera()
                    }
                    .buttonStyle(.bordered)
                    .controlSize(.small)
                    
                    Spacer()
                    
                    Button("Rotate Right") {
                        rotatePreview(45)
                    }
                    .buttonStyle(.bordered)
                    .controlSize(.small)
                }
                
                if isCreating {
                    ProgressView("Creating avatar...")
                        .progressViewStyle(CircularProgressViewStyle())
                }
            }
            .padding()
        }
    }
    
    // MARK: - Helper Properties
    
    private var isConfigurationValid: Bool {
        !avatarName.isEmpty && avatarName.count >= 3
    }
    
    private var presetsForArchetype: [AvatarPreset] {
        switch selectedArchetype {
        case .human:
            return [.balanced, .athletic, .scholarly, .charismatic]
        case .echo:
            return [.luminous, .ethereal, .powerful, .mysterious]
        case .hybrid:
            return [.balanced, .harmonious, .dynamic, .unique]
        case .custom:
            return [.experimental, .artistic, .technical, .expressive]
        }
    }
    
    // MARK: - Preview Management
    
    @MainActor
    private func createPreviewAvatar() async {
        do {
            let avatarId = try await stormEngine.createAvatar(at: simd_float3(0, 0, -2))
            previewAvatarId = avatarId
            await updatePreviewAvatar()
        } catch {
            logger.error("Failed to create preview avatar: \(error)")
        }
    }
    
    @MainActor
    private func updatePreviewAvatar() async {
        guard let avatarId = previewAvatarId else { return }
        
        // Apply archetype and preset customizations
        let customization = generateCustomization()
        
        do {
            for (trait, value) in customization.traits {
                try await stormEngine.customizeAvatar(id: avatarId, trait: trait, value: value)
            }
        } catch {
            logger.error("Failed to update preview avatar: \(error)")
        }
    }
    
    private func generateCustomization() -> AvatarCustomization {
        var traits: [AvatarTrait: Float] = [:]
        
        // Apply archetype-based modifications
        switch selectedArchetype {
        case .human:
            traits[.height] = 0.0
            traits[.bodyMass] = 0.0
            traits[.skinTone] = 0.6
        case .echo:
            traits[.height] = 0.1
            traits[.bodyMass] = -0.2
            traits[.skinTone] = 0.8
            traits[.accessories] = stormEchoIntensity // Use accessories slot for echo intensity
        case .hybrid:
            traits[.height] = 0.05
            traits[.bodyMass] = -0.1
            traits[.skinTone] = 0.7
            traits[.accessories] = stormEchoIntensity * 0.5 // Reduced echo intensity for hybrids
        case .custom:
            // Will be randomized
            break
        }
        
        // Apply preset modifications
        switch selectedPreset {
        case .balanced:
            // No additional modifications
            break
        case .athletic:
            traits[.bodyMass] = (traits[.bodyMass] ?? 0) + 0.2
            traits[.musculature] = 0.7
        case .scholarly:
            traits[.bodyMass] = (traits[.bodyMass] ?? 0) - 0.1
            traits[.height] = (traits[.height] ?? 0) + 0.1
        case .charismatic:
            traits[.faceShape] = 0.3
        case .luminous:
            traits[.skinTone] = 0.9
        case .ethereal:
            traits[.bodyMass] = (traits[.bodyMass] ?? 0) - 0.3
        case .powerful:
            traits[.height] = (traits[.height] ?? 0) + 0.2
            traits[.musculature] = 0.8
        case .mysterious:
            traits[.skinTone] = 0.3
        case .harmonious:
            // Balanced hybrid traits
            break
        case .dynamic:
            traits[.musculature] = 0.6
        case .unique:
            // Special hybrid features
            break
        case .experimental:
            // Randomized traits
            break
        case .artistic:
            traits[.faceShape] = 0.4
        case .technical:
            // Precise, geometric features
            break
        case .expressive:
            traits[.eyeColor] = 0.8
        }
        
        return AvatarCustomization(traits: traits)
    }
    
    // MARK: - Preview Rendering using RealityView scene

    @MainActor
    private func setupPreview(scene: RealityViewScene) async {
        // Create preview environment
        let backgroundEntity = AnchorEntity()
        backgroundEntity.name = "PreviewBackground"
        
        let backgroundMesh = MeshResource.generateSphere(radius: 10)
        var backgroundMaterial = UnlitMaterial()
        backgroundMaterial.color = .init(red: 0.15, green: 0.15, blue: 0.2, alpha: 1.0)
        
        let backgroundModel = ModelEntity(mesh: backgroundMesh, materials: [backgroundMaterial])
        backgroundModel.scale = simd_float3(-1, 1, -1)
        
        backgroundEntity.addChild(backgroundModel)
        scene.addAnchor(backgroundEntity)
        
        // Add lighting
        let lightEntity = AnchorEntity()
        lightEntity.name = "PreviewLight"
        
        let light = DirectionalLightComponent(color: .white, intensity: 2000)
        lightEntity.components.set(light)
        lightEntity.orientation = simd_quatf(angle: .pi/4, axis: [1, -0.2, 0])
        
        scene.addAnchor(lightEntity)
        
        // Create preview avatar if we have one
        if let avatarId = previewAvatarId {
            await createPreviewAvatarEntity(id: avatarId, scene: scene)
        }
    }

    @MainActor
    private func updatePreview(scene: RealityViewScene) async {
        // Update avatar appearance
        if let avatarEntity = scene.anchors.first(where: { $0.name == "PreviewAvatar" }) {
            await updatePreviewAvatarAppearance(avatarEntity)
        }
    }

    @MainActor
    private func createPreviewAvatarEntity(id: UInt64, scene: RealityViewScene) async {
        let avatarEntity = AnchorEntity()
        avatarEntity.name = "PreviewAvatar"
        avatarEntity.position = simd_float3(0, -0.5, -2)
        
        // Create simplified avatar representation
        await createAvatarMesh(entity: avatarEntity)
        
        // Add storm echo component if needed
        if selectedArchetype == AvatarCreatorArchetype.echo || selectedArchetype == AvatarCreatorArchetype.hybrid {
            let stormEchoComponent = StormEchoComponent(
                echoType: selectedStormEchoType,
                intensity: stormEchoIntensity,
                energyPattern: stormEnergyPattern,
                manifestation: stormManifestation
            )
            avatarEntity.components.set(stormEchoComponent)
        }
        
        scene.addAnchor(avatarEntity)
    }
    
    @MainActor
    private func createAvatarMesh(entity: Entity) async {
        // Head
        let headEntity = Entity()
        headEntity.name = "Head"
        let headMesh = MeshResource.generateSphere(radius: 0.15)
        let headMaterial = SimpleMaterial(color: .init(red: 0.8, green: 0.6, blue: 0.4, alpha: 1.0), isMetallic: false)
        let headModel = ModelEntity(mesh: headMesh, materials: [headMaterial])
        headModel.position.y = 1.75
        headEntity.addChild(headModel)
        
        // Body
        let bodyEntity = Entity()
        bodyEntity.name = "Body"
        let bodyMesh = MeshResource.generateCylinder(height: 1.2, radius: 0.25)
        let bodyMaterial = SimpleMaterial(color: .init(red: 0.7, green: 0.5, blue: 0.3, alpha: 1.0), isMetallic: false)
        let bodyModel = ModelEntity(mesh: bodyMesh, materials: [bodyMaterial])
        bodyModel.position.y = 1.0
        bodyEntity.addChild(bodyModel)
        
        // Arms
        let leftArmEntity = Entity()
        leftArmEntity.name = "LeftArm"
        let armMesh = MeshResource.generateCylinder(height: 0.8, radius: 0.08)
        let armMaterial = SimpleMaterial(color: .init(red: 0.8, green: 0.6, blue: 0.4, alpha: 1.0), isMetallic: false)
        let leftArmModel = ModelEntity(mesh: armMesh, materials: [armMaterial])
        leftArmModel.position = simd_float3(-0.35, 1.3, 0)
        leftArmModel.orientation = simd_quatf(angle: .pi/8, axis: [0, 0, 1])
        leftArmEntity.addChild(leftArmModel)
        
        let rightArmEntity = Entity()
        rightArmEntity.name = "RightArm"
        let rightArmModel = ModelEntity(mesh: armMesh, materials: [armMaterial])
        rightArmModel.position = simd_float3(0.35, 1.3, 0)
        rightArmModel.orientation = simd_quatf(angle: -.pi/8, axis: [0, 0, 1])
        rightArmEntity.addChild(rightArmModel)
        
        // Legs
        let leftLegEntity = Entity()
        leftLegEntity.name = "LeftLeg"
        let legMesh = MeshResource.generateCylinder(height: 1.0, radius: 0.1)
        let legMaterial = SimpleMaterial(color: .init(red: 0.8, green: 0.6, blue: 0.4, alpha: 1.0), isMetallic: false)
        let leftLegModel = ModelEntity(mesh: legMesh, materials: [legMaterial])
        leftLegModel.position = simd_float3(-0.12, 0.4, 0)
        leftLegEntity.addChild(leftLegModel)
        
        let rightLegEntity = Entity()
        rightLegEntity.name = "RightLeg"
        let rightLegModel = ModelEntity(mesh: legMesh, materials: [legMaterial])
        rightLegModel.position = simd_float3(0.12, 0.4, 0)
        rightLegEntity.addChild(rightLegModel)
        
        entity.addChild(headEntity)
        entity.addChild(bodyEntity)
        entity.addChild(leftArmEntity)
        entity.addChild(rightArmEntity)
        entity.addChild(leftLegEntity)
        entity.addChild(rightLegEntity)
    }
    
    @MainActor
    private func updatePreviewAvatarAppearance(_ avatarEntity: Entity) async {
        // Update avatar based on current archetype and preset
        let customization = generateCustomization()
        
        // Apply height scaling
        if let heightTrait = customization.traits[.height] {
            let scale = 1.0 + heightTrait * 0.2
            avatarEntity.scale = simd_float3(scale, scale, scale)
        }
        
        // Apply Echo effects for Echo archetype
        if selectedArchetype == .echo || selectedArchetype == .hybrid {
            await applyStormEchoEffects(to: avatarEntity)
        }
    }
    
    @MainActor
    private func applyStormEchoEffects(to entity: Entity) async {
        // Apply glow and effects based on echo type and intensity
        let glowIntensity = selectedArchetype == .echo ? stormEchoIntensity : stormEchoIntensity * 0.5
        
        for child in entity.children {
            if let modelEntity = child.children.first as? ModelEntity {
                var newMaterial: SimpleMaterial
                
                switch selectedStormEchoType {
                case .hope:
                    let alpha = Double(0.7 + glowIntensity * 0.3)
                    newMaterial = SimpleMaterial(
                        color: .init(red: 1.0, green: 1.0, blue: 0.8, alpha: alpha),
                        isMetallic: false
                    )
                case .wisdom:
                    let alpha = Double(0.7 + glowIntensity * 0.3)
                    newMaterial = SimpleMaterial(
                        color: .init(red: 0.6, green: 0.8, blue: 1.0, alpha: alpha),
                        isMetallic: false
                    )
                case .memory:
                    let alpha = Double(0.6 + glowIntensity * 0.4)
                    newMaterial = SimpleMaterial(
                        color: .init(red: 0.8, green: 0.6, blue: 1.0, alpha: alpha),
                        isMetallic: false
                    )
                case .logic:
                    let alpha = Double(0.8 + glowIntensity * 0.2)
                    newMaterial = SimpleMaterial(
                        color: .init(red: 0.4, green: 1.0, blue: 0.8, alpha: alpha),
                        isMetallic: true
                    )
                case .dreams:
                    let alpha = Double(0.5 + glowIntensity * 0.5)
                    newMaterial = SimpleMaterial(
                        color: .init(red: 1.0, green: 0.7, blue: 0.9, alpha: alpha),
                        isMetallic: false
                    )
                case .protection:
                    let alpha = Double(0.8 + glowIntensity * 0.2)
                    newMaterial = SimpleMaterial(
                        color: .init(red: 0.8, green: 0.8, blue: 0.8, alpha: alpha),
                        isMetallic: true
                    )
                case .discovery:
                    let alpha = Double(0.7 + glowIntensity * 0.3)
                    newMaterial = SimpleMaterial(
                        color: .init(red: 1.0, green: 0.6, blue: 0.2, alpha: alpha),
                        isMetallic: false
                    )
                case .creation:
                    let alpha = Double(0.7 + glowIntensity * 0.3)
                    newMaterial = SimpleMaterial(
                        color: .init(red: 0.6, green: 1.0, blue: 0.6, alpha: alpha),
                        isMetallic: false
                    )
                }
                
                modelEntity.safeUpdateMaterials([newMaterial])
            }
        }
        
        // Add floating animation for ethereal manifestation
        if stormManifestation == .ethereal {
            addFloatingAnimation(to: entity)
        }
    }
    
    private func addFloatingAnimation(to entity: Entity) {
        let floatAnimation = FromToByAnimation<Transform>(
            name: "ethereal_float",
            from: .init(scale: entity.scale, rotation: entity.orientation, translation: entity.position),
            to: .init(scale: entity.scale, rotation: entity.orientation, translation: entity.position + simd_float3(0, 0.1, 0)),
            duration: 2.0,
            timing: .easeInOut,
            isAdditive: false,
            bindTarget: .transform
        )

        if let resource = try? AnimationResource.generate(with: floatAnimation) {
            entity.playAnimation(resource.repeat())
        }
    }
    
    // MARK: - Actions
    
    private func createAvatar() {
        guard !isCreating else { return }
        
        isCreating = true
        creationError = nil
        
        Task { @MainActor in
            do {
                let customization = generateCustomization()
                let avatarId = try await stormEngine.createAvatar(
                    at: simd_float3(0, 1, -5),
                    customization: customization
                )
                
                logger.info("Created new avatar with ID: \(avatarId)")
                
                isCreating = false
                presentationMode.wrappedValue.dismiss()
            } catch {
                isCreating = false
                creationError = error.localizedDescription
                logger.error("Failed to create avatar: \(error)")
            }
        }
    }
    
    private func randomizeAppearance() {
        selectedPreset = presetsForArchetype.randomElement() ?? .balanced
        
        // Randomize echo properties if applicable
        if selectedArchetype == .echo || selectedArchetype == .hybrid {
            selectedStormEchoType = StormEchoType.allCases.randomElement() ?? .hope
            stormEnergyPattern = StormEnergyPattern.allCases.randomElement() ?? .flowing
            stormManifestation = StormManifestation.allCases.randomElement() ?? .physical
            stormEchoIntensity = Float.random(in: 0.3...0.9)
        }
        
        Task { await updatePreviewAvatar() }
    }
    
    private func generateAIAvatar() {
        // AI avatar generation with procedural selection
        selectedArchetype = AvatarCreatorArchetype.allCases.randomElement() ?? .human
        selectedPreset = presetsForArchetype.randomElement() ?? .balanced
        
        if selectedArchetype == .echo || selectedArchetype == .hybrid {
            selectedStormEchoType = StormEchoType.allCases.randomElement() ?? .hope
            stormEnergyPattern = StormEnergyPattern.allCases.randomElement() ?? .flowing
            stormManifestation = StormManifestation.allCases.randomElement() ?? .physical
            stormEchoIntensity = Float.random(in: 0.4...0.8)
        }
        
        // Generate creative name
        let names = ["Astra", "Echo", "Nova", "Zephyr", "Lumina", "Cypher", "Vortex", "Phoenix"]
        let suffixes = ["walker", "weaver", "seeker", "guardian", "whisper", "storm", "light", "dream"]
        avatarName = "\(names.randomElement() ?? "Echo")\(suffixes.randomElement() ?? "walker")"
        
        logger.info("AI avatar generation completed")
        Task { await updatePreviewAvatar() }
    }
    
    private func rotatePreview(_ degrees: Float) {
        // Rotate the preview avatar
        if let avatarId = previewAvatarId {
            // Implementation would rotate the entity in the RealityView
            logger.debug("Rotating preview by \(degrees) degrees")
        }
    }
    
    private func resetPreviewCamera() {
        // Reset camera to default position
        logger.debug("Resetting preview camera")
    }
}

// MARK: - Supporting Types

enum AvatarCreatorArchetype: String, CaseIterable {
    case human = "Human"
    case echo = "Echo"
    case hybrid = "Hybrid"
    case custom = "Custom"
    
    var description: String {
        switch self {
        case .human:
            return "Traditional human form with realistic proportions and appearance"
        case .echo:
            return "Mystical beings with supernatural abilities and ethereal presence"
        case .hybrid:
            return "Unique blend combining human and Echo characteristics"
        case .custom:
            return "Completely customizable with unlimited creative possibilities"
        }
    }
    
    var icon: String {
        switch self {
        case .human: return "person.fill"
        case .echo: return "sparkles"
        case .hybrid: return "person.2.badge.gearshape"
        case .custom: return "paintbrush.fill"
        }
    }
}

enum AvatarPreset: String, CaseIterable {
    // Human presets
    case balanced = "Balanced"
    case athletic = "Athletic"
    case scholarly = "Scholarly"
    case charismatic = "Charismatic"
    
    // Echo presets
    case luminous = "Luminous"
    case ethereal = "Ethereal"
    case powerful = "Powerful"
    case mysterious = "Mysterious"
    
    // Hybrid presets
    case harmonious = "Harmonious"
    case dynamic = "Dynamic"
    case unique = "Unique"
    
    // Custom presets
    case experimental = "Experimental"
    case artistic = "Artistic"
    case technical = "Technical"
    case expressive = "Expressive"
    
    var description: String {
        switch self {
        case .balanced: return "Well-proportioned with average build"
        case .athletic: return "Strong and fit with muscular definition"
        case .scholarly: return "Tall and lean with intellectual features"
        case .charismatic: return "Attractive features with strong presence"
        case .luminous: return "Bright and radiant with golden glow"
        case .ethereal: return "Translucent and otherworldly"
        case .powerful: return "Imposing presence with intense energy"
        case .mysterious: return "Dark and enigmatic appearance"
        case .harmonious: return "Perfect balance of human and Echo traits"
        case .dynamic: return "Changeable features that shift over time"
        case .unique: return "One-of-a-kind appearance"
        case .experimental: return "Cutting-edge features and designs"
        case .artistic: return "Creative and expressive styling"
        case .technical: return "Precise geometric features"
        case .expressive: return "Highly animated and emotional"
        }
    }
}

// MARK: - UI Components

struct ArchetypeCard: View {
    let archetype: AvatarCreatorArchetype
    let isSelected: Bool
    let onSelect: () -> Void
    
    var body: some View {
        Button {
            onSelect()
        } label: {
            VStack(spacing: 8) {
                Image(systemName: archetype.icon)
                    .font(.title2)
                    .foregroundColor(isSelected ? .white : .primary)
                
                Text(archetype.rawValue)
                    .font(.caption)
                    .fontWeight(.medium)
                    .foregroundColor(isSelected ? .white : .primary)
            }
            .frame(height: 80)
            .frame(maxWidth: .infinity)
            .background(isSelected ? Color.accentColor : Color(NSColor.controlBackgroundColor))
            .cornerRadius(8)
            .overlay(
                RoundedRectangle(cornerRadius: 8)
                    .stroke(isSelected ? Color.accentColor : Color.primary.opacity(0.2), lineWidth: 1)
            )
        }
        .buttonStyle(.plain)
        .help(archetype.description)
    }
}

struct PresetRow: View {
    let preset: AvatarPreset
    let isSelected: Bool
    let onSelect: () -> Void
    
    var body: some View {
        Button {
            onSelect()
        } label: {
            HStack {
                Text(preset.rawValue)
                    .font(.subheadline)
                    .fontWeight(isSelected ? .semibold : .regular)
                
                Spacer()
                
                if isSelected {
                    Image(systemName: "checkmark.circle.fill")
                        .foregroundColor(.accentColor)
                }
            }
            .padding(.vertical, 8)
            .padding(.horizontal, 12)
            .background(isSelected ? Color.accentColor.opacity(0.1) : Color.clear)
            .cornerRadius(6)
        }
        .buttonStyle(.plain)
        .help(preset.description)
    }
}

#Preview {
    AvatarCreatorView()
        .environmentObject(StormEngine())
}
