// File: Views/AvatarCreatorView.swift
// Description: Quick avatar creation interface for new users
// Simplified version of the full customization system for rapid avatar generation

import SwiftUI
import RealityKit
import os

struct AvatarCreatorView: View {
    @EnvironmentObject var stormEngine: StormEngine
    @Environment(\.presentationMode) var presentationMode
    
    @State private var selectedArchetype: AvatarArchetype = .human
    @State private var selectedPreset: AvatarPreset = .balanced
    @State private var avatarName = ""
    @State private var isCreating = false
    @State private var creationError: String?
    @State private var previewAvatarId: UInt64?
    
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
                    ForEach(AvatarArchetype.allCases, id: \.self) { archetype in
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
            
            // 3D Preview
            RealityView { content in
                await setupPreview(content: content)
            } update: { content in
                await updatePreview(content: content)
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
    
    private func createPreviewAvatar() async {
        do {
            let avatarId = try await stormEngine.createAvatar(at: simd_float3(0, 0, -2))
            await MainActor.run {
                previewAvatarId = avatarId
            }
            await updatePreviewAvatar()
        } catch {
            logger.error("Failed to create preview avatar: \(error)")
        }
    }
    
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
        case .hybrid:
            traits[.height] = 0.05
            traits[.bodyMass] = -0.1
            traits[.skinTone] = 0.7
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
    
    // MARK: - Preview Rendering
    
    private func setupPreview(content: RealityViewContent) async {
        // Create preview environment
        let backgroundEntity = Entity()
        backgroundEntity.name = "PreviewBackground"
        
        let backgroundMesh = MeshResource.generateSphere(radius: 10)
        var backgroundMaterial = UnlitMaterial()
        backgroundMaterial.color = .init(tint: .init(red: 0.15, green: 0.15, blue: 0.2, alpha: 1.0))
        
        let backgroundModel = ModelEntity(mesh: backgroundMesh, materials: [backgroundMaterial])
        backgroundModel.scale = simd_float3(-1, 1, -1)
        
        backgroundEntity.addChild(backgroundModel)
        content.add(backgroundEntity)
        
        // Add lighting
        let lightEntity = Entity()
        lightEntity.name = "PreviewLight"
        
        let light = DirectionalLightComponent(color: .white, intensity: 2000)
        lightEntity.components.set(light)
        lightEntity.orientation = simd_quatf(angle: .pi/4, axis: [1, -0.2, 0])
        
        content.add(lightEntity)
        
        // Create preview avatar if we have one
        if let avatarId = previewAvatarId {
            await createPreviewAvatarEntity(id: avatarId, content: content)
        }
    }
    
    private func updatePreview(content: RealityViewContent) async {
        // Update avatar appearance
        if let avatarEntity = content.entities.first(where: { $0.name == "PreviewAvatar" }) {
            await updatePreviewAvatarAppearance(avatarEntity)
        }
    }
    
    private func createPreviewAvatarEntity(id: UInt64, content: RealityViewContent) async {
        let avatarEntity = Entity()
        avatarEntity.name = "PreviewAvatar"
        avatarEntity.position = simd_float3(0, -0.5, -2)
        
        // Create simplified avatar representation
        await createAvatarMesh(entity: avatarEntity)
        
        content.add(avatarEntity)
    }
    
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
    
    private func updatePreviewAvatarAppearance(_ avatarEntity: Entity) async {
        // Update avatar based on current archetype and preset
        let customization = generateCustomization()
        
        // Apply height scaling
        if let heightTrait = customization.traits[.height] {
            let scale = 1.0 + heightTrait * 0.2
            avatarEntity.scale = simd_float3(scale, scale, scale)
        }
        
        // Apply Echo effects for Echo archetype
        if selectedArchetype == .echo {
            await applyEchoEffects(to: avatarEntity)
        }
    }
    
    private func applyEchoEffects(to entity: Entity) async {
        // Add glowing effect for Echo avatars
        for child in entity.children {
            if let bodyParts = child.children.first as? ModelEntity {
                if var material = bodyParts.materials.first as? SimpleMaterial {
                    // Add glow based on preset
                    switch selectedPreset {
                    case .luminous:
                        material.color = .init(tint: .init(red: 1.0, green: 1.0, blue: 0.8, alpha: 0.9))
                    case .ethereal:
                        material.color = .init(tint: .init(red: 0.8, green: 0.9, blue: 1.0, alpha: 0.8))
                    case .powerful:
                        material.color = .init(tint: .init(red: 1.0, green: 0.6, blue: 0.2, alpha: 0.9))
                    case .mysterious:
                        material.color = .init(tint: .init(red: 0.4, green: 0.2, blue: 0.8, alpha: 0.85))
                    default:
                        break
                    }
                    
                    bodyParts.materials = [material]
                }
            }
        }
    }
    
    // MARK: - Actions
    
    private func createAvatar() {
        guard !isCreating else { return }
        
        isCreating = true
        creationError = nil
        
        Task {
            do {
                let customization = generateCustomization()
                let avatarId = try await stormEngine.createAvatar(
                    at: simd_float3(0, 1, -5),
                    customization: customization
                )
                
                logger.info("Created new avatar with ID: \(avatarId)")
                
                await MainActor.run {
                    isCreating = false
                    presentationMode.wrappedValue.dismiss()
                }
            } catch {
                await MainActor.run {
                    isCreating = false
                    creationError = error.localizedDescription
                }
                
                logger.error("Failed to create avatar: \(error)")
            }
        }
    }
    
    private func randomizeAppearance() {
        selectedPreset = presetsForArchetype.randomElement() ?? .balanced
        Task { await updatePreviewAvatar() }
    }
    
    private func generateAIAvatar() {
        // TODO: Implement AI avatar generation
        logger.info("AI avatar generation requested")
    }
    
    private func rotatePreview(_ degrees: Float) {
        // TODO: Implement preview rotation
    }
    
    private func resetPreviewCamera() {
        // TODO: Implement camera reset
    }
}

// MARK: - Supporting Types

enum AvatarArchetype: String, CaseIterable {
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
    let archetype: AvatarArchetype
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
