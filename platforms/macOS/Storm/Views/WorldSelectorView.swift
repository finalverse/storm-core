// File: Views/WorldSelectorView.swift
// Description: World selection and connection interface
// Allows users to discover, configure, and connect to virtual worlds

import SwiftUI
import os

struct WorldSelectorView: View {
    @EnvironmentObject var worldDiscoveryService: WorldDiscoveryService
    @EnvironmentObject var stormEngine: StormEngine
    @Environment(\.presentationMode) var presentationMode
    
    @State private var selectedTab: WorldTab = .discover
    @State private var searchText = ""
    @State private var selectedProtocol: ProtocolType?
    @State private var showingCustomWorld = false
    @State private var isConnecting = false
    @State private var connectionError: String?
    
    private let logger = Logger(subsystem: "com.storm.client", category: "WorldSelector")
    
    var body: some View {
        NavigationView {
            VStack(spacing: 0) {
                // Header
                worldSelectorHeader
                
                // Search and filters
                searchAndFilters
                
                Divider()
                
                // Tab selection
                tabSelection
                
                // Content area
                contentArea
                
                // Bottom actions
                bottomActions
            }
            .navigationTitle("Connect to World")
            .toolbar {
                ToolbarItem(placement: .cancellationAction) {
                    Button("Cancel") {
                        presentationMode.wrappedValue.dismiss()
                    }
                }
                
                ToolbarItem(placement: .primaryAction) {
                    Button("Add Custom") {
                        showingCustomWorld = true
                    }
                }
            }
        }
        .frame(width: 800, height: 600)
        .alert("Connection Error", isPresented: .constant(connectionError != nil)) {
            Button("OK") {
                connectionError = nil
            }
        } message: {
            if let error = connectionError {
                Text(error)
            }
        }
        .sheet(isPresented: $showingCustomWorld) {
            CustomWorldView()
        }
        .task {
            if worldDiscoveryService.discoveredWorlds.isEmpty {
                await worldDiscoveryService.discoverWorlds()
            }
        }
    }
    
    // MARK: - Header
    
    private var worldSelectorHeader: some View {
        VStack(alignment: .leading, spacing: 8) {
            Text("Select a Virtual World")
                .font(.title2)
                .fontWeight(.bold)
            
            Text("Connect to OpenSim grids, Finalverse servers, or create a local world")
                .font(.subheadline)
                .foregroundColor(.secondary)
        }
        .frame(maxWidth: .infinity, alignment: .leading)
        .padding()
    }
    
    // MARK: - Search and Filters
    
    private var searchAndFilters: some View {
        HStack {
            // Search field
            HStack {
                Image(systemName: "magnifyingglass")
                    .foregroundColor(.secondary)
                
                TextField("Search worlds...", text: $searchText)
                    .textFieldStyle(.plain)
                
                if !searchText.isEmpty {
                    Button {
                        searchText = ""
                    } label: {
                        Image(systemName: "xmark.circle.fill")
                            .foregroundColor(.secondary)
                    }
                    .buttonStyle(.plain)
                }
            }
            .padding(.horizontal, 8)
            .padding(.vertical, 6)
            .background(Color(NSColor.controlBackgroundColor))
            .cornerRadius(6)
            
            // Protocol filter
            Picker("Protocol", selection: $selectedProtocol) {
                Text("All Protocols").tag(nil as ProtocolType?)
                ForEach(ProtocolType.allCases, id: \.self) { protocol in
                    Text(protocol.displayName).tag(protocol as ProtocolType?)
                }
            }
            .pickerStyle(.menu)
            .frame(width: 150)
            
            // Refresh button
            Button {
                Task {
                    await worldDiscoveryService.discoverWorlds()
                }
            } label: {
                Image(systemName: "arrow.clockwise")
            }
            .buttonStyle(.bordered)
            .disabled(worldDiscoveryService.isDiscovering)
        }
        .padding(.horizontal)
    }
    
    // MARK: - Tab Selection
    
    private var tabSelection: some View {
        HStack(spacing: 0) {
            ForEach(WorldTab.allCases, id: \.self) { tab in
                Button {
                    selectedTab = tab
                } label: {
                    VStack(spacing: 4) {
                        HStack {
                            Image(systemName: tab.icon)
                            Text(tab.displayName)
                        }
                        .font(.subheadline)
                        .fontWeight(selectedTab == tab ? .semibold : .regular)
                        
                        if selectedTab == tab {
                            Rectangle()
                                .fill(Color.accentColor)
                                .frame(height: 2)
                        } else {
                            Rectangle()
                                .fill(Color.clear)
                                .frame(height: 2)
                        }
                    }
                    .padding(.horizontal)
                    .padding(.vertical, 8)
                    .contentShape(Rectangle())
                }
                .buttonStyle(.plain)
            }
            
            Spacer()
        }
        .background(Color(NSColor.controlBackgroundColor).opacity(0.5))
    }
    
    // MARK: - Content Area
    
    @ViewBuilder
    private var contentArea: some View {
        switch selectedTab {
        case .discover:
            discoveredWorldsList
        case .favorites:
            favoriteWorldsList
        case .recent:
            recentWorldsList
        case .local:
            localWorldsView
        }
    }
    
    // MARK: - Discovered Worlds
    
    private var discoveredWorldsList: some View {
        ScrollView {
            LazyVStack(spacing: 8) {
                if worldDiscoveryService.isDiscovering {
                    ProgressView("Discovering worlds...")
                        .frame(maxWidth: .infinity)
                        .padding()
                } else {
                    ForEach(filteredDiscoveredWorlds, id: \.id) { world in
                        WorldCard(world: world, onConnect: { connectToWorld(world) })
                            .environmentObject(worldDiscoveryService)
                    }
                    
                    if filteredDiscoveredWorlds.isEmpty && !searchText.isEmpty {
                        Text("No worlds found matching '\(searchText)'")
                            .foregroundColor(.secondary)
                            .padding()
                    }
                }
            }
            .padding()
        }
    }
    
    // MARK: - Favorite Worlds
    
    private var favoriteWorldsList: some View {
        ScrollView {
            LazyVStack(spacing: 8) {
                ForEach(worldDiscoveryService.favoriteWorlds, id: \.id) { world in
                    WorldCard(world: world, onConnect: { connectToWorld(world) })
                        .environmentObject(worldDiscoveryService)
                }
                
                if worldDiscoveryService.favoriteWorlds.isEmpty {
                    VStack(spacing: 16) {
                        Image(systemName: "heart")
                            .font(.system(size: 48))
                            .foregroundColor(.secondary)
                        
                        Text("No Favorite Worlds")
                            .font(.title2)
                            .fontWeight(.semibold)
                        
                        Text("Add worlds to favorites by clicking the heart icon")
                            .foregroundColor(.secondary)
                            .multilineTextAlignment(.center)
                    }
                    .frame(maxWidth: .infinity, maxHeight: .infinity)
                    .padding()
                }
            }
            .padding()
        }
    }
    
    // MARK: - Recent Worlds
    
    private var recentWorldsList: some View {
        ScrollView {
            LazyVStack(spacing: 8) {
                ForEach(worldDiscoveryService.recentWorlds, id: \.id) { world in
                    WorldCard(world: world, onConnect: { connectToWorld(world) })
                        .environmentObject(worldDiscoveryService)
                }
                
                if worldDiscoveryService.recentWorlds.isEmpty {
                    VStack(spacing: 16) {
                        Image(systemName: "clock")
                            .font(.system(size: 48))
                            .foregroundColor(.secondary)
                        
                        Text("No Recent Worlds")
                            .font(.title2)
                            .fontWeight(.semibold)
                        
                        Text("Recently connected worlds will appear here")
                            .foregroundColor(.secondary)
                    }
                    .frame(maxWidth: .infinity, maxHeight: .infinity)
                    .padding()
                }
            }
            .padding()
        }
    }
    
    // MARK: - Local Worlds
    
    private var localWorldsView: some View {
        VStack(spacing: 24) {
            // Quick start local world
            VStack(spacing: 16) {
                Image(systemName: "house.circle")
                    .font(.system(size: 64))
                    .foregroundColor(.blue)
                
                Text("Local Sandbox World")
                    .font(.title2)
                    .fontWeight(.bold)
                
                Text("Create and explore in your own private virtual world")
                    .foregroundColor(.secondary)
                    .multilineTextAlignment(.center)
                
                Button("Create Local World") {
                    createLocalWorld()
                }
                .buttonStyle(.borderedProminent)
                .controlSize(.large)
            }
            .frame(maxWidth: 400)
            
            Divider()
            
            // Local world options
            VStack(alignment: .leading, spacing: 12) {
                Text("Local World Features")
                    .font(.headline)
                
                FeatureRow(icon: "cube.box", text: "Built-in physics simulation")
                FeatureRow(icon: "brain", text: "AI-powered NPCs and environments")
                FeatureRow(icon: "wand.and.rays", text: "Procedural content generation")
                FeatureRow(icon: "slider.horizontal.3", text: "Full avatar customization")
                FeatureRow(icon: "hammer", text: "Building and scripting tools")
            }
            .frame(maxWidth: 400, alignment: .leading)
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .padding()
    }
    
    // MARK: - Bottom Actions
    
    private var bottomActions: some View {
        HStack {
            if isConnecting {
                ProgressView()
                    .scaleEffect(0.8)
                Text("Connecting...")
                    .foregroundColor(.secondary)
            }
            
            Spacer()
            
            Button("Cancel") {
                presentationMode.wrappedValue.dismiss()
            }
            .buttonStyle(.bordered)
        }
        .padding()
        .background(Color(NSColor.controlBackgroundColor).opacity(0.5))
    }
    
    // MARK: - Helper Properties
    
    private var filteredDiscoveredWorlds: [WorldConfiguration] {
        var worlds = worldDiscoveryService.discoveredWorlds
        
        // Filter by protocol
        if let protocol = selectedProtocol {
            worlds = worlds.filter { $0.protocol == protocol }
        }
        
        // Filter by search text
        if !searchText.isEmpty {
            worlds = worldDiscoveryService.searchWorlds(searchText)
        }
        
        return worlds
    }
    
    // MARK: - Actions
    
    private func connectToWorld(_ world: WorldConfiguration) {
        guard !isConnecting else { return }
        
        logger.info("Attempting to connect to world: \(world.name)")
        
        isConnecting = true
        connectionError = nil
        
        Task {
            do {
                try await stormEngine.connectToWorld(world)
                worldDiscoveryService.addToRecent(world)
                
                await MainActor.run {
                    isConnecting = false
                    presentationMode.wrappedValue.dismiss()
                }
                
                logger.info("Successfully connected to world: \(world.name)")
            } catch {
                await MainActor.run {
                    isConnecting = false
                    connectionError = error.localizedDescription
                }
                
                logger.error("Failed to connect to world: \(error)")
            }
        }
    }
    
    private func createLocalWorld() {
        let localWorld = WorldConfiguration.local(name: "My Sandbox")
        connectToWorld(localWorld)
    }
}

// MARK: - Supporting Types

enum WorldTab: String, CaseIterable {
    case discover = "Discover"
    case favorites = "Favorites"
    case recent = "Recent"
    case local = "Local"
    
    var displayName: String {
        return rawValue
    }
    
    var icon: String {
        switch self {
        case .discover: return "globe"
        case .favorites: return "heart"
        case .recent: return "clock"
        case .local: return "house"
        }
    }
}

// MARK: - World Card

struct WorldCard: View {
    let world: WorldConfiguration
    let onConnect: () -> Void
    @EnvironmentObject var worldDiscoveryService: WorldDiscoveryService
    
    var body: some View {
        HStack(spacing: 12) {
            // Protocol icon
            VStack {
                Image(systemName: world.protocol.icon)
                    .font(.title2)
                    .foregroundColor(protocolColor)
                    .frame(width: 40, height: 40)
                    .background(protocolColor.opacity(0.1))
                    .cornerRadius(8)
                
                Spacer()
            }
            
            // World info
            VStack(alignment: .leading, spacing: 4) {
                HStack {
                    Text(world.name)
                        .font(.headline)
                        .fontWeight(.semibold)
                    
                    Spacer()
                    
                    // Favorite button
                    Button {
                        if worldDiscoveryService.isFavorite(world) {
                            worldDiscoveryService.removeFromFavorites(world)
                        } else {
                            worldDiscoveryService.addToFavorites(world)
                        }
                    } label: {
                        Image(systemName: worldDiscoveryService.isFavorite(world) ? "heart.fill" : "heart")
                            .foregroundColor(worldDiscoveryService.isFavorite(world) ? .red : .secondary)
                    }
                    .buttonStyle(.plain)
                }
                
                Text(world.protocol.displayName)
                    .font(.caption)
                    .padding(.horizontal, 6)
                    .padding(.vertical, 2)
                    .background(protocolColor.opacity(0.1))
                    .foregroundColor(protocolColor)
                    .cornerRadius(4)
                
                if !world.metadata.description.isEmpty {
                    Text(world.metadata.description)
                        .font(.subheadline)
                        .foregroundColor(.secondary)
                        .lineLimit(2)
                }
                
                // Features
                HStack {
                    ForEach(Array(world.metadata.features.prefix(4)), id: \.self) { feature in
                        Image(systemName: feature.icon)
                            .font(.
