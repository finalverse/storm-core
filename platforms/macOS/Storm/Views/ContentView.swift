// File: ContentView.swift
// Description: Main content view that orchestrates the entire Storm application
// Provides navigation, world management, and coordinates all major UI components

import SwiftUI
import RealityKit
import Combine
import os

struct ContentView: View {
    @StateObject private var stormEngine = StormEngine()
    @StateObject private var worldDiscoveryService = WorldDiscoveryService()
    @StateObject private var connectionManager = WorldConnectionManager()
    
    @State private var selectedTab: MainTab = .world
    @State private var showingWorldSelector = false
    @State private var showingSettings = false
    @State private var showingAvatarCreator = false
    @State private var isInitialized = false
    @State private var initializationError: String?
    
    private let logger = Logger(subsystem: "com.storm.client", category: "ContentView")
    
    var body: some View {
        Group {
            if isInitialized {
                mainInterface
            } else if let error = initializationError {
                errorView(error)
            } else {
                initializationView
            }
        }
        .task {
            await initializeStorm()
        }
    }
    
    // MARK: - Main Interface
    
    private var mainInterface: some View {
        NavigationSplitView {
            // Sidebar
            VStack(spacing: 0) {
                // Connection status header
                connectionStatusHeader
                
                Divider()
                
                // Navigation tabs
                navigationSidebar
                
                Divider()
                
                // World connections
                connectedWorldsList
                
                Spacer()
                
                // Bottom controls
                bottomControlsPanel
            }
            .frame(minWidth: 250, idealWidth: 300)
            .background(Color(NSColor.controlBackgroundColor))
        } detail: {
            // Main content area
            mainContentArea
        }
        .environmentObject(stormEngine)
        .environmentObject(worldDiscoveryService)
        .environmentObject(connectionManager)
        .navigationTitle("Storm")
        .toolbar {
            ToolbarItemGroup(placement: .primaryAction) {
                toolbarButtons
            }
        }
        .sheet(isPresented: $showingWorldSelector) {
            WorldSelectorView()
                .environmentObject(worldDiscoveryService)
                .environmentObject(stormEngine)
        }
        .sheet(isPresented: $showingAvatarCreator) {
            AvatarCreatorView()
                .environmentObject(stormEngine)
        }
        .sheet(isPresented: $showingSettings) {
            SettingsView()
                .environmentObject(stormEngine)
        }
    }
    
    // MARK: - Connection Status Header
    
    private var connectionStatusHeader: some View {
        VStack(alignment: .leading, spacing: 8) {
            HStack {
                Circle()
                    .fill(stormEngine.networkStatus.color)
                    .frame(width: 8, height: 8)
                
                Text(stormEngine.networkStatus.displayText)
                    .font(.caption)
                    .fontWeight(.medium)
                
                Spacer()
                
                if stormEngine.isInitialized {
                    Image(systemName: "checkmark.circle.fill")
                        .foregroundColor(.green)
                        .font(.caption)
                }
            }
            
            if connectionManager.hasActiveConnections {
                Text("\(connectionManager.activeConnections.count) world(s) connected")
                    .font(.caption2)
                    .foregroundColor(.secondary)
            }
        }
        .padding()
    }
    
    // MARK: - Navigation Sidebar
    
    private var navigationSidebar: some View {
        VStack(spacing: 0) {
            ForEach(MainTab.allCases, id: \.self) { tab in
                Button {
                    selectedTab = tab
                } label: {
                    HStack {
                        Image(systemName: tab.icon)
                            .frame(width: 20)
                        Text(tab.displayName)
                        Spacer()
                        
                        if tab == .world && connectionManager.hasActiveConnections {
                            Text("\(connectionManager.activeConnections.count)")
                                .font(.caption)
                                .padding(.horizontal, 6)
                                .padding(.vertical, 2)
                                .background(Color.accentColor)
                                .foregroundColor(.white)
                                .clipShape(Capsule())
                        }
                    }
                    .padding(.horizontal)
                    .padding(.vertical, 8)
                    .background(selectedTab == tab ? Color.accentColor.opacity(0.2) : Color.clear)
                    .contentShape(Rectangle())
                }
                .buttonStyle(.plain)
            }
        }
    }
    
    // MARK: - Connected Worlds List
    
    private var connectedWorldsList: some View {
        VStack(alignment: .leading, spacing: 0) {
            if connectionManager.hasActiveConnections {
                Text("Connected Worlds")
                    .font(.headline)
                    .padding(.horizontal)
                    .padding(.top)
                
                ScrollView {
                    LazyVStack(spacing: 4) {
                        ForEach(Array(connectionManager.activeConnections.values), id: \.connectionId) { connection in
                            ConnectedWorldRow(connection: connection)
                                .environmentObject(stormEngine)
                                .environmentObject(connectionManager)
                        }
                    }
                    .padding(.horizontal)
                }
            }
        }
    }
    
    // MARK: - Bottom Controls
    
    private var bottomControlsPanel: some View {
        VStack(spacing: 8) {
            Button("Connect to World") {
                showingWorldSelector = true
            }
            .buttonStyle(.borderedProminent)
            .controlSize(.large)
            
            HStack {
                Button("New Avatar") {
                    showingAvatarCreator = true
                }
                .buttonStyle(.bordered)
                .controlSize(.small)
                
                Button("Settings") {
                    showingSettings = true
                }
                .buttonStyle(.bordered)
                .controlSize(.small)
            }
        }
        .padding()
    }
    
    // MARK: - Main Content Area
    
    @ViewBuilder
    private var mainContentArea: some View {
        switch selectedTab {
        case .world:
            WorldRenderView()
                .environmentObject(stormEngine)
        case .avatar:
            AvatarCustomizationView()
                .environmentObject(stormEngine)
        case .inventory:
            InventoryView()
                .environmentObject(stormEngine)
        case .social:
            SocialView()
                .environmentObject(stormEngine)
        case .marketplace:
            MarketplaceView()
                .environmentObject(stormEngine)
        }
    }
    
    // MARK: - Toolbar Buttons
    
    private var toolbarButtons: some View {
        Group {
            Button {
                showingWorldSelector = true
            } label: {
                Image(systemName: "plus.circle")
            }
            .help("Connect to World")
            
            Button {
                Task {
                    await refreshConnections()
                }
            } label: {
                Image(systemName: "arrow.clockwise")
            }
            .help("Refresh Connections")
            
            Menu {
                Button("Performance Monitor") {
                    // TODO: Show performance monitor
                }
                Button("Debug Console") {
                    // TODO: Show debug console
                }
                Divider()
                Button("About Storm") {
                    // TODO: Show about dialog
                }
            } label: {
                Image(systemName: "ellipsis.circle")
            }
            .help("More Options")
        }
    }
    
    // MARK: - Initialization View
    
    private var initializationView: some View {
        VStack(spacing: 24) {
            Image(systemName: "tornado")
                .font(.system(size: 64))
                .foregroundColor(.blue)
                .symbolEffect(.pulse)
            
            Text("Storm")
                .font(.largeTitle)
                .fontWeight(.bold)
            
            Text("AI-Driven Virtual World Client")
                .font(.headline)
                .foregroundColor(.secondary)
            
            ProgressView("Initializing Storm Engine...")
                .progressViewStyle(CircularProgressViewStyle())
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .background(Color(NSColor.windowBackgroundColor))
    }
    
    // MARK: - Error View
    
    private func errorView(_ error: String) -> some View {
        VStack(spacing: 16) {
            Image(systemName: "exclamationmark.triangle")
                .font(.system(size: 48))
                .foregroundColor(.red)
            
            Text("Initialization Failed")
                .font(.title2)
                .fontWeight(.semibold)
            
            Text(error)
                .font(.body)
                .foregroundColor(.secondary)
                .multilineTextAlignment(.center)
                .padding(.horizontal)
            
            Button("Retry") {
                Task {
                    initializationError = nil
                    await initializeStorm()
                }
            }
            .buttonStyle(.borderedProminent)
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .background(Color(NSColor.windowBackgroundColor))
    }
    
    // MARK: - Initialization Logic
    
    private func initializeStorm() async {
        logger.info("Starting Storm initialization")
        
        do {
            try await stormEngine.initialize()
            await worldDiscoveryService.discoverWorlds()
            
            await MainActor.run {
                isInitialized = true
                logger.info("Storm initialization completed successfully")
            }
        } catch {
            await MainActor.run {
                initializationError = error.localizedDescription
                logger.error("Storm initialization failed: \(error)")
            }
        }
    }
    
    private func refreshConnections() async {
        logger.info("Refreshing world connections")
        // Refresh logic here
    }
}

// MARK: - Supporting Types

enum MainTab: String, CaseIterable {
    case world = "World"
    case avatar = "Avatar"
    case inventory = "Inventory"
    case social = "Social"
    case marketplace = "Marketplace"
    
    var displayName: String {
        return rawValue
    }
    
    var icon: String {
        switch self {
        case .world: return "globe"
        case .avatar: return "person.crop.circle"
        case .inventory: return "shippingbox"
        case .social: return "person.2"
        case .marketplace: return "storefront"
        }
    }
}

// MARK: - Network Status Extension

extension NetworkStatus {
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

// MARK: - Connected World Row

struct ConnectedWorldRow: View {
    let connection: ConnectionState
    @EnvironmentObject var stormEngine: StormEngine
    @EnvironmentObject var connectionManager: WorldConnectionManager
    
    var body: some View {
        HStack {
            Circle()
                .fill(connection.status.color)
                .frame(width: 8, height: 8)
            
            VStack(alignment: .leading, spacing: 2) {
                Text(connection.world.name)
                    .font(.caption)
                    .fontWeight(.medium)
                
                Text(connection.status.rawValue)
                    .font(.caption2)
                    .foregroundColor(.secondary)
            }
            
            Spacer()
            
            Menu {
                Button("View Details") {
                    // TODO: Show connection details
                }
                Button("Teleport Here") {
                    // TODO: Teleport to this world
                }
                Divider()
                Button("Disconnect", role: .destructive) {
                    Task {
                        try? await stormEngine.disconnectFromWorld(connection.world.id)
                    }
                }
            } label: {
                Image(systemName: "ellipsis")
                    .font(.caption)
            }
            .menuStyle(.borderlessButton)
            .frame(width: 20, height: 20)
        }
        .padding(.vertical, 4)
        .padding(.horizontal, 8)
        .background(Color.clear)
        .cornerRadius(4)
    }
}

// MARK: - Placeholder Views

struct InventoryView: View {
    @EnvironmentObject var stormEngine: StormEngine
    
    var body: some View {
        VStack {
            Text("Inventory")
                .font(.largeTitle)
                .fontWeight(.bold)
            
            Text("Asset and item management coming soon")
                .foregroundColor(.secondary)
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
    }
}

struct SocialView: View {
    @EnvironmentObject var stormEngine: StormEngine
    
    var body: some View {
        VStack {
            Text("Social")
                .font(.largeTitle)
                .fontWeight(.bold)
            
            Text("Friends, groups, and messaging coming soon")
                .foregroundColor(.secondary)
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
    }
}

struct MarketplaceView: View {
    @EnvironmentObject var stormEngine: StormEngine
    
    var body: some View {
        VStack {
            Text("Marketplace")
                .font(.largeTitle)
                .fontWeight(.bold)
            
            Text("Virtual asset marketplace coming soon")
                .foregroundColor(.secondary)
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
    }
}

struct SettingsView: View {
    @EnvironmentObject var stormEngine: StormEngine
    @Environment(\.presentationMode) var presentationMode
    
    var body: some View {
        NavigationView {
            Form {
                Section("Engine") {
                    Text("Version: \(StormFFIBridge.getCoreVersion())")
                    Text("Status: \(stormEngine.isInitialized ? "Initialized" : "Not Ready")")
                }
                
                Section("Performance") {
                    Text("Performance settings coming soon")
                }
                
                Section("Network") {
                    Text("Network settings coming soon")
                }
            }
            .navigationTitle("Settings")
            .toolbar {
                ToolbarItem(placement: .confirmationAction) {
                    Button("Done") {
                        presentationMode.wrappedValue.dismiss()
                    }
                }
            }
        }
        .frame(width: 500, height: 400)
    }
}

#Preview {
    ContentView()
}
