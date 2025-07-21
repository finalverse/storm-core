// File: platforms/macOS/Storm/StormApp.swift
// Description: Main SwiftUI application entry point for Storm macOS client
// Core initialization, window management, and app lifecycle handling

import SwiftUI
import RealityKit
import Foundation

@main
struct StormApp: App {
    @StateObject private var stormEngine = StormEngine()
    @State private var showingWorldSelector = false
    @State private var showingAvatarCustomization = false
    @State private var errorMessage: String = ""
    @State private var isInitialized = false
    
    var body: some Scene {
        WindowGroup {
            Group {
                if isInitialized {
                    MainContentView()
                        .environmentObject(stormEngine)
                        .alert("Storm Error", isPresented: .constant(!errorMessage.isEmpty)) {
                            Button("OK") {
                                errorMessage = ""
                            }
                        } message: {
                            Text(errorMessage)
                        }
                } else {
                    InitializationView()
                        .environmentObject(stormEngine)
                }
            }
            .onAppear {
                initializeStorm()
            }
        }
        .windowResizability(.contentSize)
        .commands {
            CommandGroup(after: .newItem) {
                Button("Connect to World...") {
                    showingWorldSelector = true
                }
                .keyboardShortcut("w", modifiers: .command)
                
                Button("Customize Avatar...") {
                    showingAvatarCustomization = true
                }
                .keyboardShortcut("a", modifiers: .command)
                
                Divider()
                
                Button("Show World Info") {
                    // TODO: Implement world info display
                }
                .keyboardShortcut("i", modifiers: .command)
            }
        }
    }
    
    /// Initialize Storm engine and core systems
    private func initializeStorm() {
        Task {
            do {
                try await stormEngine.initialize()
                await MainActor.run {
                    isInitialized = true
                }
            } catch {
                await MainActor.run {
                    errorMessage = "Failed to initialize Storm: \(error.localizedDescription)"
                }
            }
        }
    }
}

/// Main content view with world and avatar management
struct MainContentView: View {
    @EnvironmentObject var stormEngine: StormEngine
    @State private var showingWorldSelector = false
    @State private var showingAvatarCustomization = false
    @State private var selectedTab = 0
    
    var body: some View {
        HSplitView {
            // Left sidebar with world controls
            VStack(alignment: .leading, spacing: 16) {
                WorldConnectionPanel()
                    .environmentObject(stormEngine)
                
                AvatarControlPanel()
                    .environmentObject(stormEngine)
                
                NetworkStatusPanel()
                    .environmentObject(stormEngine)
                
                Spacer()
            }
            .frame(minWidth: 250, maxWidth: 300)
            .padding()
            .background(Color(NSColor.controlBackgroundColor))
            
            // Main 3D view area
            VStack {
                TabView(selection: $selectedTab) {
                    WorldRenderView()
                        .environmentObject(stormEngine)
                        .tabItem {
                            Image(systemName: "globe")
                            Text("World")
                        }
                        .tag(0)
                    
                    AvatarCustomizationView()
                        .environmentObject(stormEngine)
                        .tabItem {
                            Image(systemName: "person.3.sequence")
                            Text("Avatar")
                        }
                        .tag(1)
                    
                    SettingsView()
                        .environmentObject(stormEngine)
                        .tabItem {
                            Image(systemName: "gear")
                            Text("Settings")
                        }
                        .tag(2)
                }
                .frame(minWidth: 600, minHeight: 400)
            }
        }
        .navigationTitle("Storm - Virtual World Client")
    }
}

/// Initialization view shown during startup
struct InitializationView: View {
    @EnvironmentObject var stormEngine: StormEngine
    @State private var initProgress: Double = 0.0
    @State private var currentStep = "Initializing..."
    
    var body: some View {
        VStack(spacing: 24) {
            Image(systemName: "cloud.bolt.rain")
                .font(.system(size: 64))
                .foregroundColor(.blue)
                .symbolEffect(.pulse)
            
            Text("Storm")
                .font(.largeTitle)
                .fontWeight(.bold)
            
            Text("AI-Driven Virtual World Client")
                .font(.headline)
                .foregroundColor(.secondary)
            
            VStack(spacing: 8) {
                ProgressView(value: initProgress)
                    .progressViewStyle(LinearProgressViewStyle())
                    .frame(width: 300)
                
                Text(currentStep)
                    .font(.caption)
                    .foregroundColor(.secondary)
            }
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .background(Color(NSColor.windowBackgroundColor))
        .onReceive(stormEngine.initializationProgress) { progress in
            withAnimation(.easeInOut(duration: 0.3)) {
                self.initProgress = progress.progress
                self.currentStep = progress.step
            }
        }
    }
}

/// Represents initialization progress
struct InitializationProgress {
    let progress: Double
    let step: String
}

#Preview {
    MainContentView()
        .environmentObject(StormEngine())
}
