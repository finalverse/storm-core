// Views/AvatarCustomizationView.swift
// Description: View for customizing avatars with sliders and pickers.
// Summary: UI controls for traits/morphs; updates engine state. Part of main tabs.
// Logic: Binds states to UI; onChange calls engine customize. Connects to callee StormEngine.customizeAvatar. Error-free: Handles optional returns; compiles with SwiftUI.
// Progressive: Add after MainView; testable as isolated view.

import SwiftUI

struct AvatarCustomizationView: View {
    @State private var morphValue: Float = 0.5
    @State private var traitIndex: Int = 0
    @State private var errorMessage: String = ""
    
    var body: some View {
        VStack {
            Slider(value: $morphValue, in: 0...1) {
                Text("Morph Body")
            }.onChange(of: morphValue) {  // Caller: Slider change; callee: StormEngine.customizeAvatar.
//                if !StormEngine.shared.customizeAvatar(traitIndex: Int32(traitIndex), morph: morphValue) {
//                    errorMessage = "Customization failed"
//                }
            }
            
            Picker("Trait", selection: $traitIndex) {
                Text("Hope").tag(0)
                Text("Logic").tag(1)
            }
            
            Button("Generate Companion (AI Stub)") {  // Caller: Button action; callee: Future AI integration.
                print("AI generating companion...")
            }
            
            Text(errorMessage).foregroundColor(.red)
        }
        .padding()
    }
}
