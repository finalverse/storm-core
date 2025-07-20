// platforms/macos/Storm/Views/MainView.swift
// Description: Top-level view composing the app's main interface.
// Summary: Tab-based or stacked view hosting AvatarCustomizationView and WorldView. Caller for subviews.
// Logic: Uses SwiftUI TabView; onAppear initializes engine. Connects to callees in Core (StormEngine.initialize) and subviews. Error-free: Standard layout, compiles with SwiftUI.
// Progressive: Add after Core files; displays tabs with placeholders.

import SwiftUI

struct MainView: View {
    @State private var errorMessage: String = ""
    
    var body: some View {
        TabView {
            AvatarCustomizationView()
                .tabItem { Text("Customize") }  // Caller: Renders AvatarCustomizationView.
            
            WorldView()
                .tabItem { Text("World") }  // Caller: Renders WorldView.
        }
        .onAppear {
//            // Caller: View lifecycle; callee: StormEngine.initialize.
//            if !StormEngine.shared.initialize() {
//                errorMessage = "Engine init failed"
//            }
//            StormEngine.shared.createAvatarEntity()
//            StormEngine.shared.createNPCEntity()
        }
        .overlay(Text(errorMessage).foregroundColor(.red))
    }
}
