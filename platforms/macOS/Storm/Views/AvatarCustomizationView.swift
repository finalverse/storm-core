// File: platforms/macOS/Storm/Views/AvatarCustomizationView.swift
// Description: Advanced avatar customization interface similar to MetaHuman Creator
// Provides detailed controls for avatar appearance, traits, and digital human features

import SwiftUI
import RealityKit
import Combine
import os

struct AvatarCustomizationView: View {
    @EnvironmentObject var stormEngine: StormEngine
    @StateObject private var customizationEngine = AvatarCustomizationEngine()
    @State private var selectedAvatarId: UInt64?
    @State private var currentCustomization = AvatarCustomizationData()
    @State private var selectedCategory: CustomizationCategory = .general
    @State private var previewMode: PreviewMode = .front
    @State private var isGeneratingAvatar = false
    @State private var showingPresets = false
    
    private let logger = Logger(subsystem: "com.storm.client", category: "AvatarCustomization")
    
    var body: some View {
        HSplitView {
            // Left panel - Controls
            VStack(alignment: .leading, spacing: 0) {
                // Header
                avatarSelectionHeader
                
                Divider()
                
                // Category selection
                categorySelection
                
                Divider()
                
                // Customization controls
                ScrollView {
                    LazyVStack(alignment: .leading, spacing: 16) {
                        customizationControls
                    }
                    .padding()
                }
                
                Divider()
                
                // Action buttons
                actionButtonsPanel
            }
            .frame(minWidth: 300, maxWidth: 400)
            .background(Color(NSColor.controlBackgroundColor))
            
            // Right panel - 3D Preview
            VStack {
                // Preview controls
                previewControls
                
                // 3D Avatar preview
                RealityView { content in
                    await setupAvatarPreview(content: content)
                } update: { content in
                    await updateAvatarPreview(content: content)
                }
                .background(Color.black.opacity(0.1))
                .cornerRadius(8)
                .padding()
            }
        }
        .navigationTitle("Avatar Customization")
        .toolbar {
            ToolbarItem(placement: .primaryAction) {
                Menu {
                    Button("Load Preset...") {
                        showingPresets = true
                    }
                    Button("Save as Preset...") {
                        saveAsPreset()
                    }
                    Divider()
                    Button("Random Generate") {
                        generateRandomAvatar()
                    }
                    Button("AI Enhance") {
                        enhanceWithAI()
                    }
                } label: {
                    Image(systemName: "wand.and.stars")
                }
            }
        }
        .sheet(isPresented: $showingPresets) {
            AvatarPresetLibraryView(customization: $currentCustomization)
        }
        .onAppear {
            loadDefaultAvatar()
        }
    }
    
    // MARK: - Header Section
    
    private var avatarSelectionHeader: some View {
        VStack(alignment: .leading, spacing: 8) {
            HStack {
                Text("Avatar Editor")
                    .font(.title2)
                    .fontWeight(.bold)
                
                Spacer()
                
                Button {
                    createNewAvatar()
                } label: {
                    Image(systemName: "plus.circle")
                }
                .buttonStyle(.borderless)
            }
            
            if let avatarId = selectedAvatarId {
                Text("Editing Avatar: \(avatarId)")
                    .font(.caption)
                    .foregroundColor(.secondary)
            } else {
                Text("No avatar selected")
                    .font(.caption)
                    .foregroundColor(.secondary)
            }
        }
        .padding()
    }
    
    // MARK: - Category Selection
    
    private var categorySelection: some View {
        VStack(spacing: 0) {
            ForEach(CustomizationCategory.allCases, id: \.self) { category in
                Button {
                    selectedCategory = category
                } label: {
                    HStack {
                        Image(systemName: category.icon)
                            .frame(width: 20)
                        Text(category.displayName)
                        Spacer()
                    }
                    .padding(.horizontal)
                    .padding(.vertical, 8)
                    .background(selectedCategory == category ? Color.accentColor.opacity(0.2) : Color.clear)
                }
                .buttonStyle(.borderless)
            }
        }
    }
    
    // MARK: - Customization Controls
    
    @ViewBuilder
    private var customizationControls: some View {
        switch selectedCategory {
        case .general:
            generalCustomizationControls
        case .facial:
            facialCustomizationControls
        case .body:
            bodyCustomizationControls
        case .skin:
            skinCustomizationControls
        case .hair:
            hairCustomizationControls
        case .clothing:
            clothingCustomizationControls
        case .accessories:
            accessoriesCustomizationControls
        case .echoes:
            echoCustomizationControls
        case .advanced:
            advancedCustomizationControls
        }
    }
    
    private var generalCustomizationControls: some View {
        VStack(alignment: .leading, spacing: 16) {
            GroupBox("Basic Information") {
                VStack(alignment: .leading, spacing: 12) {
                    TextField("Avatar Name", text: $currentCustomization.name)
                    
                    Picker("Archetype", selection: $currentCustomization.archetype) {
                        Text("Human").tag(AvatarArchetype.human)
                        Text("Echo").tag(AvatarArchetype.echo)
                        Text("Hybrid").tag(AvatarArchetype.hybrid)
                        Text("Custom").tag(AvatarArchetype.custom)
                    }
                    .pickerStyle(.segmented)
                    
                    if currentCustomization.archetype == .echo {
                        Picker("Echo Type", selection: $currentCustomization.echoType) {
                            Text("Hope").tag(EchoType.hope)
                            Text("Wisdom").tag(EchoType.wisdom)
                            Text("Memory").tag(EchoType.memory)
                            Text("Logic").tag(EchoType.logic)
                            Text("Dreams").tag(EchoType.dreams)
                        }
                    }
                }
                .padding()
            }
            
            GroupBox("Physical Traits") {
                VStack(alignment: .leading, spacing: 12) {
                    SliderControl(
                        title: "Height",
                        value: $currentCustomization.height,
                        range: -0.5...0.5,
                        onChanged: { applyTrait(.height, value: $0) }
                    )
                    
                    SliderControl(
                        title: "Body Mass",
                        value: $currentCustomization.bodyMass,
                        range: -0.5...0.5,
                        onChanged: { applyTrait(.bodyMass, value: $0) }
                    )
                    
                    SliderControl(
                        title: "Muscle Definition",
                        value: $currentCustomization.musculature,
                        range: 0.0...1.0,
                        onChanged: { applyTrait(.musculature, value: $0) }
                    )
                }
                .padding()
            }
        }
    }
    
    private var facialCustomizationControls: some View {
        VStack(alignment: .leading, spacing: 16) {
            GroupBox("Face Shape") {
                VStack(alignment: .leading, spacing: 12) {
                    SliderControl(
                        title: "Face Width",
                        value: $currentCustomization.faceWidth,
                        range: -0.5...0.5,
                        onChanged: { applyFacialTrait("face_width", value: $0) }
                    )
                    
                    SliderControl(
                        title: "Cheek Bones",
                        value: $currentCustomization.cheekBones,
                        range: -0.5...0.5,
                        onChanged: { applyFacialTrait("cheek_bones", value: $0) }
                    )
                    
                    SliderControl(
                        title: "Jaw Width",
                        value: $currentCustomization.jawWidth,
                        range: -0.5...0.5,
                        onChanged: { applyFacialTrait("jaw_width", value: $0) }
                    )
                    
                    SliderControl(
                        title: "Chin Prominence",
                        value: $currentCustomization.chinProminence,
                        range: -0.5...0.5,
                        onChanged: { applyFacialTrait("chin_prominence", value: $0) }
                    )
                }
                .padding()
            }
            
            GroupBox("Eyes") {
                VStack(alignment: .leading, spacing: 12) {
                    ColorPicker("Eye Color", selection: $currentCustomization.eyeColor)
                        .onChange(of: currentCustomization.eyeColor) { newColor in
                            applyEyeColor(newColor)
                        }
                    
                    SliderControl(
                        title: "Eye Size",
                        value: $currentCustomization.eyeSize,
                        range: -0.3...0.3,
                        onChanged: { applyFacialTrait("eye_size", value: $0) }
                    )
                    
                    SliderControl(
                        title: "Eye Distance",
                        value: $currentCustomization.eyeDistance,
                        range: -0.2...0.2,
                        onChanged: { applyFacialTrait("eye_distance", value: $0) }
                    )
                }
                .padding()
            }
            
            GroupBox("Nose & Mouth") {
                VStack(alignment: .leading, spacing: 12) {
                    SliderControl(
                        title: "Nose Size",
                        value: $currentCustomization.noseSize,
                        range: -0.3...0.3,
                        onChanged: { applyFacialTrait("nose_size", value: $0) }
                    )
                    
                    SliderControl(
                        title: "Mouth Width",
                        value: $currentCustomization.mouthWidth,
                        range: -0.3...0.3,
                        onChanged: { applyFacialTrait("mouth_width", value: $0) }
                    )
                    
                    SliderControl(
                        title: "Lip Thickness",
                        value: $currentCustomization.lipThickness,
                        range: -0.3...0.3,
                        onChanged: { applyFacialTrait("lip_thickness", value: $0) }
                    )
                }
                .padding()
            }
        }
    }
    
    private var bodyCustomizationControls: some View {
        VStack(alignment: .leading, spacing: 16) {
            GroupBox("Body Proportions") {
                VStack(alignment: .leading, spacing: 12) {
                    SliderControl(
                        title: "Torso Length",
                        value: $currentCustomization.torsoLength,
                        range: -0.2...0.2,
                        onChanged: { applyBodyTrait("torso_length", value: $0) }
                    )
                    
                    SliderControl(
                        title: "Shoulder Width",
                        value: $currentCustomization.shoulderWidth,
                        range: -0.3...0.3,
                        onChanged: { applyBodyTrait("shoulder_width", value: $0) }
                    )
                    
                    SliderControl(
                        title: "Arm Length",
                        value: $currentCustomization.armLength,
                        range: -0.2...0.2,
                        onChanged: { applyBodyTrait("arm_length", value: $0) }
                    )
                    
                    SliderControl(
                        title: "Leg Length",
                        value: $currentCustomization.legLength,
                        range: -0.2...0.2,
                        onChanged: { applyBodyTrait("leg_length", value: $0) }
                    )
                }
                .padding()
            }
            
            GroupBox("Fitness Level") {
                VStack(alignment: .leading, spacing: 12) {
                    SliderControl(
                        title: "Overall Fitness",
                        value: $currentCustomization.fitness,
                        range: 0.0...1.0,
                        onChanged: { applyBodyTrait("fitness", value: $0) }
                    )
                    
                    SliderControl(
                        title: "Muscle Mass",
                        value: $currentCustomization.muscleMass,
                        range: 0.0...1.0,
                        onChanged: { applyBodyTrait("muscle_mass", value: $0) }
                    )
                    
                    SliderControl(
                        title: "Body Fat",
                        value: $currentCustomization.bodyFat,
                        range: 0.0...1.0,
                        onChanged: { applyBodyTrait("body_fat", value: $0) }
                    )
                }
                .padding()
            }
        }
    }
    
    private var skinCustomizationControls: some View {
        VStack(alignment: .leading, spacing: 16) {
            GroupBox("Skin Tone") {
                VStack(alignment: .leading, spacing: 12) {
                    ColorPicker("Base Skin Color", selection: $currentCustomization.skinColor)
                        .onChange(of: currentCustomization.skinColor) { newColor in
                            applySkinColor(newColor)
                        }
                    
                    SliderControl(
                        title: "Skin Warmth",
                        value: $currentCustomization.skinWarmth,
                        range: -0.5...0.5,
                        onChanged: { applySkinTrait("warmth", value: $0) }
                    )
                    
                    SliderControl(
                        title: "Skin Darkness",
                        value: $currentCustomization.skinDarkness,
                        range: 0.0...1.0,
                        onChanged: { applySkinTrait("darkness", value: $0) }
                    )
                }
                .padding()
            }
            
            GroupBox("Skin Properties") {
                VStack(alignment: .leading, spacing: 12) {
                    SliderControl(
                        title: "Skin Smoothness",
                        value: $currentCustomization.skinSmoothness,
                        range: 0.0...1.0,
                        onChanged: { applySkinTrait("smoothness", value: $0) }
                    )
                    
                    SliderControl(
                        title: "Freckles",
                        value: $currentCustomization.freckles,
                        range: 0.0...1.0,
                        onChanged: { applySkinTrait("freckles", value: $0) }
                    )
                    
                    SliderControl(
                        title: "Skin Shine",
                        value: $currentCustomization.skinShine,
                        range: 0.0...1.0,
                        onChanged: { applySkinTrait("shine", value: $0) }
                    )
                    
                    SliderControl(
                        title: "Age Lines",
                        value: $currentCustomization.ageLines,
                        range: 0.0...1.0,
                        onChanged: { applySkinTrait("age_lines", value: $0) }
                    )
                }
                .padding()
            }
        }
    }
    
    private var hairCustomizationControls: some View {
        VStack(alignment: .leading, spacing: 16) {
            GroupBox("Hair Style") {
                VStack(alignment: .leading, spacing: 12) {
                    Picker("Hair Style", selection: $currentCustomization.hairStyle) {
                        Text("Short").tag(HairStyle.short)
                        Text("Medium").tag(HairStyle.medium)
                        Text("Long").tag(HairStyle.long)
                        Text("Curly").tag(HairStyle.curly)
                        Text("Bald").tag(HairStyle.bald)
                        Text("Unique").tag(HairStyle.unique)
                    }
                    .pickerStyle(.menu)
                    .onChange(of: currentCustomization.hairStyle) { newStyle in
                        applyHairStyle(newStyle)
                    }
                    
                    ColorPicker("Hair Color", selection: $currentCustomization.hairColor)
                        .onChange(of: currentCustomization.hairColor) { newColor in
                            applyHairColor(newColor)
                        }
                    
                    SliderControl(
                        title: "Hair Length",
                        value: $currentCustomization.hairLength,
                        range: 0.0...1.0,
                        onChanged: { applyHairTrait("length", value: $0) }
                    )
                    
                    SliderControl(
                        title: "Hair Volume",
                        value: $currentCustomization.hairVolume,
                        range: 0.0...1.0,
                        onChanged: { applyHairTrait("volume", value: $0) }
                    )
                    
                    SliderControl(
                        title: "Hair Curl",
                        value: $currentCustomization.hairCurl,
                        range: 0.0...1.0,
                        onChanged: { applyHairTrait("curl", value: $0) }
                    )
                }
                .padding()
            }
            
            GroupBox("Facial Hair") {
                VStack(alignment: .leading, spacing: 12) {
                    Picker("Facial Hair", selection: $currentCustomization.facialHair) {
                        Text("None").tag(FacialHairStyle.none)
                        Text("Mustache").tag(FacialHairStyle.mustache)
                        Text("Goatee").tag(FacialHairStyle.goatee)
                        Text("Full Beard").tag(FacialHairStyle.fullBeard)
                        Text("Stubble").tag(FacialHairStyle.stubble)
                    }
                    .pickerStyle(.menu)
                    .onChange(of: currentCustomization.facialHair) { newStyle in
                        applyFacialHair(newStyle)
                    }
                    
                    if currentCustomization.facialHair != .none {
                        SliderControl(
                            title: "Facial Hair Density",
                            value: $currentCustomization.facialHairDensity,
                            range: 0.0...1.0,
                            onChanged: { applyHairTrait("facial_density", value: $0) }
                        )
                    }
                }
                .padding()
            }
        }
    }
    
    private var clothingCustomizationControls: some View {
        VStack(alignment: .leading, spacing: 16) {
            GroupBox("Outfit Selection") {
                VStack(alignment: .leading, spacing: 12) {
                    Picker("Clothing Style", selection: $currentCustomization.clothingStyle) {
                        Text("Casual").tag(ClothingStyle.casual)
                        Text("Formal").tag(ClothingStyle.formal)
                        Text("Futuristic").tag(ClothingStyle.futuristic)
                        Text("Fantasy").tag(ClothingStyle.fantasy)
                        Text("Minimal").tag(ClothingStyle.minimal)
                    }
                    .pickerStyle(.menu)
                    .onChange(of: currentCustomization.clothingStyle) { newStyle in
                        applyClothingStyle(newStyle)
                    }
                    
                    ColorPicker("Primary Color", selection: $currentCustomization.clothingPrimaryColor)
                        .onChange(of: currentCustomization.clothingPrimaryColor) { newColor in
                            applyClothingColor("primary", color: newColor)
                        }
                    
                    ColorPicker("Secondary Color", selection: $currentCustomization.clothingSecondaryColor)
                        .onChange(of: currentCustomization.clothingSecondaryColor) { newColor in
                            applyClothingColor("secondary", color: newColor)
                        }
                    
                    SliderControl(
                        title: "Material Roughness",
                        value: $currentCustomization.clothingRoughness,
                        range: 0.0...1.0,
                        onChanged: { applyClothingTrait("roughness", value: $0) }
                    )
                    
                    SliderControl(
                        title: "Material Metallic",
                        value: $currentCustomization.clothingMetallic,
                        range: 0.0...1.0,
                        onChanged: { applyClothingTrait("metallic", value: $0) }
                    )
                }
                .padding()
            }
        }
    }
    
    private var accessoriesCustomizationControls: some View {
        VStack(alignment: .leading, spacing: 16) {
            GroupBox("Accessories") {
                VStack(alignment: .leading, spacing: 12) {
                    Toggle("Glasses", isOn: $currentCustomization.hasGlasses)
                        .onChange(of: currentCustomization.hasGlasses) { hasGlasses in
                            applyAccessory("glasses", enabled: hasGlasses)
                        }
                    
                    if currentCustomization.hasGlasses {
                        Picker("Glasses Style", selection: $currentCustomization.glassesStyle) {
                            Text("Round").tag(GlassesStyle.round)
                            Text("Square").tag(GlassesStyle.square)
                            Text("Aviator").tag(GlassesStyle.aviator)
                            Text("Cat Eye").tag(GlassesStyle.catEye)
                        }
                        .pickerStyle(.menu)
                    }
                    
                    Toggle("Jewelry", isOn: $currentCustomization.hasJewelry)
                        .onChange(of: currentCustomization.hasJewelry) { hasJewelry in
                            applyAccessory("jewelry", enabled: hasJewelry)
                        }
                    
                    Toggle("Hat/Headwear", isOn: $currentCustomization.hasHat)
                        .onChange(of: currentCustomization.hasHat) { hasHat in
                            applyAccessory("hat", enabled: hasHat)
                        }
                    
                    Toggle("Tattoos", isOn: $currentCustomization.hasTattoos)
                        .onChange(of: currentCustomization.hasTattoos) { hasTattoos in
                            applyAccessory("tattoos", enabled: hasTattoos)
                        }
                    
                    if currentCustomization.hasTattoos {
                        SliderControl(
                            title: "Tattoo Coverage",
                            value: $currentCustomization.tattooCoverage,
                            range: 0.0...1.0,
                            onChanged: { applyAccessoryTrait("tattoo_coverage", value: $0) }
                        )
                    }
                }
                .padding()
            }
        }
    }
    
    private var echoCustomizationControls: some View {
        VStack(alignment: .leading, spacing: 16) {
            GroupBox("Echo Properties") {
                VStack(alignment: .leading, spacing: 12) {
                    if currentCustomization.archetype == .echo || currentCustomization.archetype == .hybrid {
                        SliderControl(
                            title: "Echo Intensity",
                            value: $currentCustomization.echoIntensity,
                            range: 0.0...1.0,
                            onChanged: { applyEchoTrait("intensity", value: $0) }
                        )
                        
                        ColorPicker("Echo Glow Color", selection: $currentCustomization.echoGlowColor)
                            .onChange(of: currentCustomization.echoGlowColor) { newColor in
                                applyEchoColor(newColor)
                            }
                        
                        SliderControl(
                            title: "Transparency",
                            value: $currentCustomization.echoTransparency,
                            range: 0.0...0.8,
                            onChanged: { applyEchoTrait("transparency", value: $0) }
                        )
                        
                        SliderControl(
                            title: "Particle Density",
                            value: $currentCustomization.echoParticleDensity,
                            range: 0.0...1.0,
                            onChanged: { applyEchoTrait("particle_density", value: $0) }
                        )
                        
                        Picker("Energy Pattern", selection: $currentCustomization.energyPattern) {
                            Text("Static").tag(EnergyPattern.static)
                            Text("Flowing").tag(EnergyPattern.flowing)
                            Text("Pulsing").tag(EnergyPattern.pulsing)
                            Text("Swirling").tag(EnergyPattern.swirling)
                        }
                        .pickerStyle(.menu)
                        .onChange(of: currentCustomization.energyPattern) { pattern in
                            applyEnergyPattern(pattern)
                        }
                    } else {
                        Text("Echo properties only available for Echo and Hybrid archetypes")
                            .foregroundColor(.secondary)
                            .font(.caption)
                    }
                }
                .padding()
            }
        }
    }
    
    private var advancedCustomizationControls: some View {
        VStack(alignment: .leading, spacing: 16) {
            GroupBox("AI Enhancement") {
                VStack(alignment: .leading, spacing: 12) {
                    Button("Auto-Generate Features") {
                        generateRandomFeatures()
                    }
                    .buttonStyle(.bordered)
                    
                    Button("AI Optimize Proportions") {
                        optimizeProportionsWithAI()
                    }
                    .buttonStyle(.bordered)
                    
                    Button("Apply Style Transfer") {
                        applyStyleTransfer()
                    }
                    .buttonStyle(.bordered)
                    
                    Text("AI enhancement uses machine learning to improve avatar realism and coherence")
                        .font(.caption)
                        .foregroundColor(.secondary)
                }
                .padding()
            }
            
            GroupBox("Export/Import") {
                VStack(alignment: .leading, spacing: 12) {
                    Button("Export Avatar Data") {
                        exportAvatarData()
                    }
                    .buttonStyle(.bordered)
                    
                    Button("Import Avatar Data") {
                        importAvatarData()
                    }
                    .buttonStyle(.bordered)
                    
                    Button("Generate QR Code") {
                        generateQRCode()
                    }
                    .buttonStyle(.bordered)
                }
                .padding()
            }
        }
    }
    
    // MARK: - Preview Controls
    
    private var previewControls: some View {
        HStack {
            Text("Preview Mode:")
            
            Picker("", selection: $previewMode) {
                Text("Front").tag(PreviewMode.front)
                Text("Side").tag(PreviewMode.side)
                Text("Back").tag(PreviewMode.back)
                Text("Full Body").tag(PreviewMode.fullBody)
            }
            .pickerStyle(.segmented)
            
            Spacer()
            
            Button("Reset View") {
                resetPreviewCamera()
            }
            .buttonStyle(.bordered)
        }
        .padding()
    }
    
    // MARK: - Action Buttons
    
    private var actionButtonsPanel: some View {
        VStack(spacing: 8) {
            HStack {
                Button("Reset") {
                    resetCustomization()
                }
                .buttonStyle(.bordered)
                
                Button("Randomize") {
                    generateRandomAvatar()
                }
                .buttonStyle(.bordered)
            }
            
            Button("Apply Changes") {
                applyAllChanges()
            }
            .buttonStyle(.borderedProminent)
            .disabled(selectedAvatarId == nil)
        }
        .padding()
    }
    
    // MARK: - 3D Preview Setup
    
    private func setupAvatarPreview(content: RealityViewContent) async {
        // Create preview environment
        await createPreviewEnvironment(content: content)
        
        // Create avatar for preview
        if let avatarId = selectedAvatarId {
            await createPreviewAvatar(id: avatarId, content: content)
        }
    }
    
    private func updateAvatarPreview(content: RealityViewContent) async {
        // Update avatar appearance based on current customization
        await updatePreviewAvatar(content: content)
    }
    
    private func createPreviewEnvironment(content: RealityViewContent) async {
        // Create neutral background
        let backgroundEntity = Entity()
        backgroundEntity.name = "PreviewBackground"
        
        let backgroundMesh = MeshResource.generateSphere(radius: 10)
        var backgroundMaterial = UnlitMaterial()
        backgroundMaterial.color = .init(tint: .init(red: 0.2, green: 0.2, blue: 0.25, alpha: 1.0))
        
        let backgroundModel = ModelEntity(mesh: backgroundMesh, materials: [backgroundMaterial])
        backgroundModel.scale = simd_float3(-1, 1, -1) // Invert to show inner surface
        
        backgroundEntity.addChild(backgroundModel)
        content.add(backgroundEntity)
        
        // Add preview lighting
        let lightEntity = Entity()
        lightEntity.name = "PreviewLight"
        
        let light = DirectionalLightComponent(color: .white, intensity: 1500)
        lightEntity.components.set(light)
        lightEntity.orientation = simd_quatf(angle: .pi/4, axis: [1, -0.5, 0])
        
        content.add(lightEntity)
        
        // Add fill light
        let fillLightEntity = Entity()
        fillLightEntity.name = "FillLight"
        
        let fillLight = DirectionalLightComponent(color: .init(red: 0.8, green: 0.9, blue: 1.0, alpha: 1.0), intensity: 500)
        fillLightEntity.components.set(fillLight)
        fillLightEntity.orientation = simd_quatf(angle: -.pi/6, axis: [-1, 0.3, 0])
        
        content.add(fillLightEntity)
    }
    
    private func createPreviewAvatar(id: UInt64, content: RealityViewContent) async {
        // Create detailed avatar model for preview
        let avatarEntity = Entity()
        avatarEntity.name = "PreviewAvatar"
        avatarEntity.position = simd_float3(0, 0, -2)
        
        // Create high-detail avatar mesh (simplified for demo)
        await createDetailedAvatarMesh(entity: avatarEntity)
        
        content.add(avatarEntity)
    }
    
    private func createDetailedAvatarMesh(entity: Entity) async {
        // Head
        let headEntity = Entity()
        headEntity.name = "Head"
        let headMesh = MeshResource.generateSphere(radius: 0.12)
        let headMaterial = createSkinMaterial()
        let headModel = ModelEntity(mesh: headMesh, materials: [headMaterial])
        headModel.position.y = 1.7
        headEntity.addChild(headModel)
        
        // Body
        let bodyEntity = Entity()
        bodyEntity.name = "Body"
        let bodyMesh = MeshResource.generateCylinder(height: 1.2, radius: 0.2)
        let bodyMaterial = createSkinMaterial()
        let bodyModel = ModelEntity(mesh: bodyMesh, materials: [bodyMaterial])
        bodyModel.position.y = 1.0
        bodyEntity.addChild(bodyModel)
        
        // Arms
        let leftArmEntity = Entity()
        leftArmEntity.name = "LeftArm"
        let armMesh = MeshResource.generateCylinder(height: 0.8, radius: 0.08)
        let armMaterial = createSkinMaterial()
        let leftArmModel = ModelEntity(mesh: armMesh, materials: [armMaterial])
        leftArmModel.position = simd_float3(-0.3, 1.3, 0)
        leftArmModel.orientation = simd_quatf(angle: .pi/12, axis: [0, 0, 1])
        leftArmEntity.addChild(leftArmModel)
        
        let rightArmEntity = Entity()
        rightArmEntity.name = "RightArm"
        let rightArmModel = ModelEntity(mesh: armMesh, materials: [armMaterial])
        rightArmModel.position = simd_float3(0.3, 1.3, 0)
        rightArmModel.orientation = simd_quatf(angle: -.pi/12, axis: [0, 0, 1])
        rightArmEntity.addChild(rightArmModel)
        
        // Legs
        let leftLegEntity = Entity()
        leftLegEntity.name = "LeftLeg"
        let legMesh = MeshResource.generateCylinder(height: 1.0, radius: 0.1)
        let legMaterial = createSkinMaterial()
        let leftLegModel = ModelEntity(mesh: legMesh, materials: [legMaterial])
        leftLegModel.position = simd_float3(-0.1, 0.5, 0)
        leftLegEntity.addChild(leftLegModel)
        
        let rightLegEntity = Entity()
        rightLegEntity.name = "RightLeg"
        let rightLegModel = ModelEntity(mesh: legMesh, materials: [legMaterial])
        rightLegModel.position = simd_float3(0.1, 0.5, 0)
        rightLegEntity.addChild(rightLegModel)
        
        entity.addChild(headEntity)
        entity.addChild(bodyEntity)
        entity.addChild(leftArmEntity)
        entity.addChild(rightArmEntity)
        entity.addChild(leftLegEntity)
        entity.addChild(rightLegEntity)
    }
    
    private func createSkinMaterial() -> SimpleMaterial {
        let skinColor = currentCustomization.skinColor
        return SimpleMaterial(
            color: .init(
                red: Float(skinColor.components.red),
                green: Float(skinColor.components.green),
                blue: Float(skinColor.components.blue),
                alpha: 1.0
            ),
            isMetallic: false
        )
    }
    
    private func updatePreviewAvatar(content: RealityViewContent) async {
        // Update avatar appearance based on current customization settings
        guard let avatarEntity = content.entities.first(where: { $0.name == "PreviewAvatar" }) else { return }
        
        // Update materials and transforms based on customization
        await applyCustomizationToPreview(avatarEntity)
    }
    
    private func applyCustomizationToPreview(_ avatarEntity: Entity) async {
        // Apply current customization to preview avatar
        // This would include updating materials, scales, positions, etc.
        
        // Update head scale based on facial customization
        if let headEntity = avatarEntity.findEntity(named: "Head") {
            let scaleX = 1.0 + currentCustomization.faceWidth * 0.3
            let scaleY = 1.0 + currentCustomization.height * 0.1
            headEntity.scale = simd_float3(scaleX, scaleY, 1.0)
        }
        
        // Update body proportions
        if let bodyEntity = avatarEntity.findEntity(named: "Body") {
            let scaleX = 1.0 + currentCustomization.bodyMass * 0.5
            let scaleY = 1.0 + currentCustomization.height * 0.2
            bodyEntity.scale = simd_float3(scaleX, scaleY, scaleX)
        }
        
        // Apply Echo effects if applicable
        if currentCustomization.archetype == .echo || currentCustomization.archetype == .hybrid {
            await applyEchoEffectsToPreview(avatarEntity)
        }
    }
    
    private func applyEchoEffectsToPreview(_ avatarEntity: Entity) async {
        // Add glowing effects and transparency for Echo avatars
        for child in avatarEntity.children {
            if let modelEntity = child.children.first as? ModelEntity {
                // Apply transparency
                if var material = modelEntity.materials.first as? SimpleMaterial {
                    let alpha = 1.0 - currentCustomization.echoTransparency
                    material.color = .init(
                        red: material.color.tint.red,
                        green: material.color.tint.green,
                        blue: material.color.tint.blue,
                        alpha: alpha
                    )
                    
                    // Add glow color tint
                    let glowColor = currentCustomization.echoGlowColor
                    material.color = .init(
                        red: material.color.tint.red * Float(glowColor.components.red),
                        green: material.color.tint.green * Float(glowColor.components.green),
                        blue: material.color.tint.blue * Float(glowColor.components.blue),
                        alpha: alpha
                    )
                    
                    modelEntity.materials = [material]
                }
            }
        }
    }
    
    // MARK: - Trait Application Methods
    
    private func applyTrait(_ trait: AvatarTrait, value: Float) {
        guard let avatarId = selectedAvatarId else { return }
        
        Task {
            do {
                try await stormEngine.customizeAvatar(id: avatarId, trait: trait, value: value)
                logger.info("Applied trait \(trait) with value \(value)")
            } catch {
                logger.error("Failed to apply trait: \(error)")
            }
        }
    }
    
    private func applyFacialTrait(_ traitName: String, value: Float) {
        // Apply facial customization via FFI
        guard let avatarId = selectedAvatarId else { return }
        
        // Map trait name to appropriate FFI call
        let result = ffi_customize_avatar(avatarId, mapFacialTraitToIndex(traitName), value)
        if result != 0 {
            logger.error("Failed to apply facial trait: \(traitName)")
        }
    }
    
    private func applyBodyTrait(_ traitName: String, value: Float) {
        // Apply body customization
        logger.info("Applying body trait: \(traitName) = \(value)")
    }
    
    private func applySkinTrait(_ traitName: String, value: Float) {
        // Apply skin customization
        logger.info("Applying skin trait: \(traitName) = \(value)")
    }
    
    private func applyHairTrait(_ traitName: String, value: Float) {
        // Apply hair customization
        logger.info("Applying hair trait: \(traitName) = \(value)")
    }
    
    private func applyClothingTrait(_ traitName: String, value: Float) {
        // Apply clothing customization
        logger.info("Applying clothing trait: \(traitName) = \(value)")
    }
    
    private func applyAccessoryTrait(_ traitName: String, value: Float) {
        // Apply accessory customization
        logger.info("Applying accessory trait: \(traitName) = \(value)")
    }
    
    private func applyEchoTrait(_ traitName: String, value: Float) {
        // Apply Echo-specific customization
        logger.info("Applying echo trait: \(traitName) = \(value)")
    }
    
    // MARK: - Color Application Methods
    
    private func applyEyeColor(_ color: Color) {
        logger.info("Applying eye color: \(color)")
    }
    
    private func applySkinColor(_ color: Color) {
        logger.info("Applying skin color: \(color)")
    }
    
    private func applyHairColor(_ color: Color) {
        logger.info("Applying hair color: \(color)")
    }
    
    private func applyClothingColor(_ type: String, color: Color) {
        logger.info("Applying \(type) clothing color: \(color)")
    }
    
    private func applyEchoColor(_ color: Color) {
        logger.info("Applying echo glow color: \(color)")
    }
    
    // MARK: - Style Application Methods
    
    private func applyHairStyle(_ style: HairStyle) {
        logger.info("Applying hair style: \(style)")
    }
    
    private func applyFacialHair(_ style: FacialHairStyle) {
        logger.info("Applying facial hair: \(style)")
    }
    
    private func applyClothingStyle(_ style: ClothingStyle) {
        logger.info("Applying clothing style: \(style)")
    }
    
    private func applyAccessory(_ accessory: String, enabled: Bool) {
        logger.info("Setting \(accessory): \(enabled)")
    }
    
    private func applyEnergyPattern(_ pattern: EnergyPattern) {
        logger.info("Applying energy pattern: \(pattern)")
    }
    
    // MARK: - Action Methods
    
    private func loadDefaultAvatar() {
        Task {
            do {
                if let currentAvatar = stormEngine.currentAvatar {
                    selectedAvatarId = currentAvatar.id
                    currentCustomization = AvatarCustomizationData.from(currentAvatar.customization)
                }
            }
        }
    }
    
    private func createNewAvatar() {
        Task {
            do {
                let newAvatarId = try await stormEngine.createAvatar(at: simd_float3(0, 1, -2))
                await MainActor.run {
                    selectedAvatarId = newAvatarId
                    currentCustomization = AvatarCustomizationData()
                }
            } catch {
                logger.error("Failed to create new avatar: \(error)")
            }
        }
    }
    
    private func resetCustomization() {
        currentCustomization = AvatarCustomizationData()
    }
    
    private func generateRandomAvatar() {
        isGeneratingAvatar = true
        
        Task {
            // Generate random values for all traits
            await MainActor.run {
                currentCustomization.height = Float.random(in: -0.3...0.3)
                currentCustomization.bodyMass = Float.random(in: -0.3...0.3)
                currentCustomization.faceWidth = Float.random(in: -0.2...0.2)
                currentCustomization.eyeSize = Float.random(in: -0.2...0.2)
                currentCustomization.noseSize = Float.random(in: -0.2...0.2)
                currentCustomization.mouthWidth = Float.random(in: -0.2...0.2)
                
                // Random colors
                currentCustomization.skinColor = Color(
                    red: Double.random(in: 0.3...0.9),
                    green: Double.random(in: 0.2...0.8),
                    blue: Double.random(in: 0.1...0.7)
                )
                
                currentCustomization.hairColor = Color(
                    red: Double.random(in: 0.1...0.9),
                    green: Double.random(in: 0.1...0.7),
                    blue: Double.random(in: 0.1...0.5)
                )
                
                currentCustomization.eyeColor = Color(
                    red: Double.random(in: 0.2...0.8),
                    green: Double.random(in: 0.3...0.9),
                    blue: Double.random(in: 0.4...1.0)
                )
                
                // Random styles
                currentCustomization.hairStyle = HairStyle.allCases.randomElement() ?? .medium
                currentCustomization.clothingStyle = ClothingStyle.allCases.randomElement() ?? .casual
                
                isGeneratingAvatar = false
            }
            
            // Apply the random customization
            applyAllChanges()
        }
    }
    
    private func generateRandomFeatures() {
        // AI-enhanced random generation with more coherent results
        generateRandomAvatar()
    }
    
    private func optimizeProportionsWithAI() {
        // Use AI to optimize avatar proportions for realism
        logger.info("Optimizing proportions with AI")
    }
    
    private func applyStyleTransfer() {
        // Apply style transfer from reference images
        logger.info("Applying style transfer")
    }
    
    private func enhanceWithAI() {
        logger.info("Enhancing avatar with AI")
    }
    
    private func applyAllChanges() {
        guard let avatarId = selectedAvatarId else { return }
        
        Task {
            do {
                // Apply all current customization settings
                try await stormEngine.customizeAvatar(id: avatarId, trait: .height, value: currentCustomization.height)
                try await stormEngine.customizeAvatar(id: avatarId, trait: .bodyMass, value: currentCustomization.bodyMass)
                try await stormEngine.customizeAvatar(id: avatarId, trait: .skinTone, value: Float(currentCustomization.skinColor.components.red))
                try await stormEngine.customizeAvatar(id: avatarId, trait: .eyeColor, value: Float(currentCustomization.eyeColor.components.red))
                
                logger.info("Applied all customization changes to avatar \(avatarId)")
            } catch {
                logger.error("Failed to apply customization changes: \(error)")
            }
        }
    }
    
    private func resetPreviewCamera() {
        // Reset camera to default position
        previewMode = .front
    }
    
    private func saveAsPreset() {
        // Save current customization as a preset
        logger.info("Saving customization as preset")
    }
    
    private func exportAvatarData() {
        // Export avatar data to file
        logger.info("Exporting avatar data")
    }
    
    private func importAvatarData() {
        // Import avatar data from file
        logger.info("Importing avatar data")
    }
    
    private func generateQRCode() {
        // Generate QR code for avatar sharing
        logger.info("Generating QR code for avatar")
    }
    
    // MARK: - Helper Methods
    
    private func mapFacialTraitToIndex(_ traitName: String) -> Int32 {
        switch traitName {
        case "face_width": return 10
        case "cheek_bones": return 11
        case "jaw_width": return 12
        case "chin_prominence": return 13
        case "eye_size": return 14
        case "eye_distance": return 15
        case "nose_size": return 16
        case "mouth_width": return 17
        case "lip_thickness": return 18
        default: return 0
        }
    }
}

// MARK: - Supporting Types and Data Structures

struct AvatarCustomizationData {
    // General
    var name: String = "New Avatar"
    var archetype: AvatarArchetype = .human
    var echoType: EchoType = .hope
    
    // Physical traits
    var height: Float = 0.0
    var bodyMass: Float = 0.0
    var musculature: Float = 0.5
    
    // Facial features
    var faceWidth: Float = 0.0
    var cheekBones: Float = 0.0
    var jawWidth: Float = 0.0
    var chinProminence: Float = 0.0
    var eyeSize: Float = 0.0
    var eyeDistance: Float = 0.0
    var noseSize: Float = 0.0
    var mouthWidth: Float = 0.0
    var lipThickness: Float = 0.0
    var eyeColor: Color = .blue
    
    // Body proportions
    var torsoLength: Float = 0.0
    var shoulderWidth: Float = 0.0
    var armLength: Float = 0.0
    var legLength: Float = 0.0
    var fitness: Float = 0.5
    var muscleMass: Float = 0.5
    var bodyFat: Float = 0.3
    
    // Skin
    var skinColor: Color = Color(red: 0.8, green: 0.6, blue: 0.4)
    var skinWarmth: Float = 0.0
    var skinDarkness: Float = 0.5
    var skinSmoothness: Float = 0.7
    var freckles: Float = 0.0
    var skinShine: Float = 0.3
    var ageLines: Float = 0.0
    
    // Hair
    var hairStyle: HairStyle = .medium
    var hairColor: Color = .brown
    var hairLength: Float = 0.5
    var hairVolume: Float = 0.5
    var hairCurl: Float = 0.3
    var facialHair: FacialHairStyle = .none
    var facialHairDensity: Float = 0.5
    
    // Clothing
    var clothingStyle: ClothingStyle = .casual
    var clothingPrimaryColor: Color = .blue
    var clothingSecondaryColor: Color = .white
    var clothingRoughness: Float = 0.5
    var clothingMetallic: Float = 0.1
    
    // Accessories
    var hasGlasses: Bool = false
    var glassesStyle: GlassesStyle = .round
    var hasJewelry: Bool = false
    var hasHat: Bool = false
    var hasTattoos: Bool = false
    var tattooCoverage: Float = 0.2
    
    // Echo properties
    var echoIntensity: Float = 0.5
    var echoGlowColor: Color = .cyan
    var echoTransparency: Float = 0.2
    var echoParticleDensity: Float = 0.3
    var energyPattern: EnergyPattern = .flowing
    
    static func from(_ customization: AvatarCustomization) -> AvatarCustomizationData {
        var data = AvatarCustomizationData()
        
        // Map traits from StormEngine customization to UI data
        if let height = customization.traits[.height] {
            data.height = height
        }
        if let bodyMass = customization.traits[.bodyMass] {
            data.bodyMass = bodyMass
        }
        
        return data
    }
}

enum CustomizationCategory: String, CaseIterable {
    case general = "General"
    case facial = "Facial"
    case body = "Body"
    case skin = "Skin"
    case hair = "Hair"
    case clothing = "Clothing"
    case accessories = "Accessories"
    case echoes = "Echoes"
    case advanced = "Advanced"
    
    var displayName: String {
        return rawValue
    }
    
    var icon: String {
        switch self {
        case .general: return "person.crop.circle"
        case .facial: return "face.smiling"
        case .body: return "figure.stand"
        case .skin: return "paintpalette"
        case .hair: return "scissors"
        case .clothing: return "tshirt"
        case .accessories: return "eyeglasses"
        case .echoes: return "sparkles"
        case .advanced: return "gearshape.2"
        }
    }
}

enum AvatarArchetype: String, CaseIterable {
    case human = "Human"
    case echo = "Echo"
    case hybrid = "Hybrid"
    case custom = "Custom"
}

enum EchoType: String, CaseIterable {
    case hope = "Hope"
    case wisdom = "Wisdom"
    case memory = "Memory"
    case logic = "Logic"
    case dreams = "Dreams"
}

enum HairStyle: String, CaseIterable {
    case short = "Short"
    case medium = "Medium"
    case long = "Long"
    case curly = "Curly"
    case bald = "Bald"
    case unique = "Unique"
}

enum FacialHairStyle: String, CaseIterable {
    case none = "None"
    case mustache = "Mustache"
    case goatee = "Goatee"
    case fullBeard = "Full Beard"
    case stubble = "Stubble"
}

enum ClothingStyle: String, CaseIterable {
    case casual = "Casual"
    case formal = "Formal"
    case futuristic = "Futuristic"
    case fantasy = "Fantasy"
    case minimal = "Minimal"
}

enum GlassesStyle: String, CaseIterable {
    case round = "Round"
    case square = "Square"
    case aviator = "Aviator"
    case catEye = "Cat Eye"
}

enum EnergyPattern: String, CaseIterable {
    case static = "Static"
    case flowing = "Flowing"
    case pulsing = "Pulsing"
    case swirling = "Swirling"
}

enum PreviewMode: String, CaseIterable {
    case front = "Front"
    case side = "Side"
    case back = "Back"
    case fullBody = "Full Body"
}

// MARK: - Custom Controls

struct SliderControl: View {
    let title: String
    @Binding var value: Float
    let range: ClosedRange<Float>
    let onChanged: (Float) -> Void
    
    var body: some View {
        VStack(alignment: .leading, spacing: 4) {
            HStack {
                Text(title)
                    .font(.caption)
                Spacer()
                Text(String(format: "%.2f", value))
                    .font(.caption)
                    .foregroundColor(.secondary)
            }
            
            Slider(value: $value, in: range) { editing in
                if !editing {
                    onChanged(value)
                }
            }
        }
    }
}

// MARK: - Avatar Customization Engine

class AvatarCustomizationEngine: ObservableObject {
    private let logger = Logger(subsystem: "com.storm.client", category: "CustomizationEngine")
    
    func initialize() {
        logger.info("Avatar customization engine initialized")
    }
    
    func applyCustomization(_ customization: AvatarCustomizationData, to avatarId: UInt64) async throws {
        // Apply all customization parameters to the avatar
        logger.info("Applying full customization to avatar \(avatarId)")
    }
    
    func generateRandomCustomization() -> AvatarCustomizationData {
        var customization = AvatarCustomizationData()
        
        // Generate random but coherent values
        customization.height = Float.random(in: -0.3...0.3)
        customization.bodyMass = Float.random(in: -0.3...0.3)
        
        return customization
    }
}

// MARK: - Preset Library View

struct AvatarPresetLibraryView: View {
    @Binding var customization: AvatarCustomizationData
    @Environment(\.presentationMode) var presentationMode
    
    var body: some View {
        NavigationView {
            VStack {
                Text("Avatar Preset Library")
                    .font(.title2)
                    .padding()
                
                ScrollView {
                    LazyVGrid(columns: Array(repeating: GridItem(.flexible()), count: 3)) {
                        ForEach(samplePresets, id: \.name) { preset in
                            PresetCard(preset: preset) {
                                customization = preset
                                presentationMode.wrappedValue.dismiss()
                            }
                        }
                    }
                    .padding()
                }
                
                Spacer()
            }
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .confirmationAction) {
                    Button("Done") {
                        presentationMode.wrappedValue.dismiss()
                    }
                }
            }
        }
        .frame(width: 600, height: 500)
    }
    
    private var samplePresets: [AvatarCustomizationData] {
        [
            // Create sample presets
            AvatarCustomizationData(), // Default human
            // Add more presets here
        ]
    }
}

struct PresetCard: View {
    let preset: AvatarCustomizationData
    let onSelect: () -> Void
    
    var body: some View {
        VStack {
            RoundedRectangle(cornerRadius: 8)
                .fill(Color.gray.opacity(0.3))
                .frame(height: 120)
                .overlay(
                    Text("Preview")
                        .foregroundColor(.secondary)
                )
            
            Text(preset.name)
                .font(.caption)
                .fontWeight(.medium)
        }
        .background(Color(NSColor.controlBackgroundColor))
        .cornerRadius(8)
        .onTapGesture {
            onSelect()
        }
    }
}

// MARK: - Color Extensions

extension Color {
    var components: (red: Double, green: Double, blue: Double, alpha: Double) {
        let uiColor = NSColor(self)
        var red: CGFloat = 0
        var green: CGFloat = 0
        var blue: CGFloat = 0
        var alpha: CGFloat = 0
        
        uiColor.getRed(&red, green: &green, blue: &blue, alpha: &alpha)
        
        return (
            red: Double(red),
            green: Double(green),
            blue: Double(blue),
            alpha: Double(alpha)
        )
    }
}
