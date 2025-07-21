# Storm macOS Application - Project Setup Guide

## Project Structure

```
platforms/macOS/
├── Storm.xcodeproj/
├── Storm/
│   ├── StormApp.swift                    # Main app entry point
│   ├── Core/
│   │   ├── StormEngine.swift            # Core engine interface
│   │   ├── StormFFIBridge.swift         # Rust FFI bridge
│   │   ├── WorldConfiguration.swift     # World connection types
│   │   └── NetworkingClient.swift       # Network utilities
│   ├── Views/
│   │   ├── WorldRenderView.swift        # RealityKit 3D scene
│   │   ├── AvatarCustomizationView.swift # MetaHuman-style editor
│   │   ├── WorldConnectionPanel.swift   # Connection controls
│   │   ├── AvatarControlPanel.swift     # Avatar management
│   │   └── SettingsView.swift           # App settings
│   ├── RealityKit/
│   │   ├── StormRenderSystem.swift      # Custom render system
│   │   └── Components.swift             # ECS components
│   ├── Resources/
│   │   ├── Assets.xcassets
│   │   └── Info.plist
│   └── Storm-Bridging-Header.h          # C bridge header
├── Frameworks/
│   └── libstorm_ffi.dylib              # Rust library
└── README.md
```

## Prerequisites

1. **Xcode 15.0+** with macOS 14.0+ deployment target
2. **Rust toolchain** with cargo
3. **Storm-core** Rust library built for macOS

## Building Storm Core for macOS

```bash
# Navigate to storm-core directory
cd ../../../storm-core

# Build for macOS (Apple Silicon)
cargo build --release --target aarch64-apple-darwin

# Build for Intel Macs
cargo build --release --target x86_64-apple-darwin

# Create universal binary (optional)
lipo -create \
  target/aarch64-apple-darwin/release/libstorm_ffi.dylib \
  target/x86_64-apple-darwin/release/libstorm_ffi.dylib \
  -output platforms/macOS/Frameworks/libstorm_ffi.dylib

# Copy FFI headers
cbindgen --config cbindgen.toml --crate storm-ffi-swift --output platforms/macOS/Storm/Storm-Bridging-Header.h
```

## Xcode Project Configuration

### 1. Project Settings

- **Product Name**: Storm
- **Bundle Identifier**: com.finalverse.storm
- **Deployment Target**: macOS 14.0
- **Architectures**: arm64, x86_64
- **Language**: Swift 5.9+

### 2. Build Settings

#### Swift Compiler Settings
```
SWIFT_OBJC_BRIDGING_HEADER = Storm/Storm-Bridging-Header.h
SWIFT_VERSION = 5.9
SWIFT_OPTIMIZATION_LEVEL = -O (Release), -Onone (Debug)
```

#### Linking Settings
```
LIBRARY_SEARCH_PATHS = $(SRCROOT)/Frameworks
OTHER_LDFLAGS = -lstorm_ffi
RUNPATH_SEARCH_PATHS = @executable_path/../Frameworks
```

#### Code Signing
```
CODE_SIGN_STYLE = Automatic
DEVELOPMENT_TEAM = [Your Team ID]
CODE_SIGN_IDENTITY = Apple Development
```

### 3. Framework Integration

#### Required Apple Frameworks
- **SwiftUI.framework** - Modern UI framework
- **RealityKit.framework** - 3D rendering and AR
- **Metal.framework** - GPU acceleration
- **MetalKit.framework** - Metal utilities
- **simd.framework** - Vector mathematics
- **Combine.framework** - Reactive programming
- **Network.framework** - Networking
- **CryptoKit.framework** - Cryptography

#### Custom Frameworks
- **libstorm_ffi.dylib** - Storm Rust core library

### 4. Build Phases

#### Copy Files Phase
```
Destination: Frameworks
Files:
- libstorm_ffi.dylib (Copy if newer)
```

#### Run Script Phase (Optional - Auto-build Rust)
```bash
#!/bin/bash
cd "${SRCROOT}/../../../storm-core"
cargo build --release --target aarch64-apple-darwin
cp target/aarch64-apple-darwin/release/libstorm_ffi.dylib "${BUILT_PRODUCTS_DIR}/${PRODUCT_NAME}.app/Contents/Frameworks/"
```

## Bridging Header Configuration

### Storm-Bridging-Header.h
```c
#ifndef Storm_Bridging_Header_h
#define Storm_Bridging_Header_h

#include <stdint.h>
#include <stdbool.h>

// Core engine functions
extern void* storm_core_init(void);
extern int32_t storm_core_shutdown(void* handle);
extern const char* storm_core_get_version(void);

// ECS functions
extern int32_t init_ecs_world(void);
extern uint64_t ffi_create_avatar(float x, float y, float z);
extern int32_t ffi_customize_avatar(uint64_t entity_id, int32_t trait_index, float morph);
extern uint64_t ffi_create_npc(int32_t echo_index, float x, float y, float z);
extern int32_t ffi_set_entity_position(uint64_t entity_id, float x, float y, float z, int32_t use_physics);

// AI functions
extern uint8_t* ffi_process_ai_enhancement(uint64_t entity_id, int32_t enhancement_type, const uint8_t* parameters, int32_t param_size, int32_t* result_size);
extern void ffi_free_ai_result(uint8_t* result);

// World connection functions
extern uint64_t ffi_connect_to_world(const void* config);
extern int32_t ffi_disconnect_from_world(uint64_t connection_id);

// Asset functions
extern uint64_t ffi_load_asset(const char* asset_path, int32_t asset_type);
extern uint8_t* ffi_get_asset_data(uint64_t asset_id, int32_t* data_size);
extern void ffi_free_asset_data(uint8_t* data);

// Performance functions
extern uint8_t* ffi_get_performance_metrics(int32_t* metrics_size);
extern void ffi_free_performance_metrics(uint8_t* metrics);

// Error handling
extern const char* ffi_get_last_error(void);
extern void ffi_clear_last_error(void);
extern void ffi_set_debug_logging(int32_t enabled);

#endif /* Storm_Bridging_Header_h */
```

## App Configuration

### Info.plist
```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleDevelopmentRegion</key>
    <string>en</string>
    <key>CFBundleExecutable</key>
    <string>$(EXECUTABLE_NAME)</string>
    <key>CFBundleIdentifier</key>
    <string>$(PRODUCT_BUNDLE_IDENTIFIER)</string>
    <key>CFBundleInfoDictionaryVersion</key>
    <string>6.0</string>
    <key>CFBundleName</key>
    <string>$(PRODUCT_NAME)</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleShortVersionString</key>
    <string>1.0</string>
    <key>CFBundleVersion</key>
    <string>1</string>
    <key>LSMinimumSystemVersion</key>
    <string>14.0</string>
    <key>NSHumanReadableCopyright</key>
    <string>Copyright © 2025 Finalverse. All rights reserved.</string>
    <key>NSSupportsAutomaticTermination</key>
    <true/>
    <key>NSSupportsSuddenTermination</key>
    <true/>
    <key>NSCameraUsageDescription</key>
    <string>Storm uses the camera for AR features and avatar customization.</string>
    <key>NSMicrophoneUsageDescription</key>
    <string>Storm uses the microphone for voice chat and AI interaction.</string>
    <key>NSNetworkVolumesUsageDescription</key>
    <string>Storm connects to virtual worlds and content servers.</string>
</dict>
</plist>
```

## Build Script for Automation

### build_storm.sh
```bash
#!/bin/bash

set -e

echo "🌪️  Building Storm macOS Application"

# Configuration
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
STORM_CORE_PATH="${PROJECT_ROOT}/../../../storm-core"
MACOS_PATH="${PROJECT_ROOT}/platforms/macOS"
FRAMEWORKS_PATH="${MACOS_PATH}/Frameworks"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

print_step() {
    echo -e "${BLUE}📋 $1${NC}"
}

print_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

# Step 1: Build Storm Core
print_step "Building Storm Core Rust library..."

if [ ! -d "$STORM_CORE_PATH" ]; then
    print_error "Storm core not found at $STORM_CORE_PATH"
    exit 1
fi

cd "$STORM_CORE_PATH"

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    print_error "Cargo (Rust) not found. Please install Rust toolchain."
    exit 1
fi

# Build for macOS targets
print_step "Building for Apple Silicon (arm64)..."
cargo build --release --target aarch64-apple-darwin --features "ffi"

print_step "Building for Intel (x86_64)..."
cargo build --release --target x86_64-apple-darwin --features "ffi"

print_success "Rust libraries built successfully"

# Step 2: Create Universal Binary
print_step "Creating universal binary..."

mkdir -p "$FRAMEWORKS_PATH"

if [ -f "target/aarch64-apple-darwin/release/libstorm_ffi.dylib" ] && [ -f "target/x86_64-apple-darwin/release/libstorm_ffi.dylib" ]; then
    lipo -create \
        "target/aarch64-apple-darwin/release/libstorm_ffi.dylib" \
        "target/x86_64-apple-darwin/release/libstorm_ffi.dylib" \
        -output "$FRAMEWORKS_PATH/libstorm_ffi.dylib"
    print_success "Universal binary created"
else
    print_warning "One or both architecture builds missing, copying available build"
    if [ -f "target/aarch64-apple-darwin/release/libstorm_ffi.dylib" ]; then
        cp "target/aarch64-apple-darwin/release/libstorm_ffi.dylib" "$FRAMEWORKS_PATH/libstorm_ffi.dylib"
    elif [ -f "target/x86_64-apple-darwin/release/libstorm_ffi.dylib" ]; then
        cp "target/x86_64-apple-darwin/release/libstorm_ffi.dylib" "$FRAMEWORKS_PATH/libstorm_ffi.dylib"
    else
        print_error "No valid builds found"
        exit 1
    fi
fi

# Step 3: Generate FFI Headers
print_step "Generating FFI headers..."

if command -v cbindgen &> /dev/null; then
    cbindgen --config cbindgen.toml --crate storm-ffi-swift --output "$MACOS_PATH/Storm/Storm-Bridging-Header.h"
    print_success "FFI headers generated"
else
    print_warning "cbindgen not found. Using existing headers."
fi

# Step 4: Build Xcode Project
print_step "Building Xcode project..."

cd "$MACOS_PATH"

if [ -f "Storm.xcodeproj/project.pbxproj" ]; then
    xcodebuild -project Storm.xcodeproj -scheme Storm -configuration Release build
    print_success "Xcode project built successfully"
else
    print_warning "Xcode project not found. Please create it manually."
fi

print_success "Storm macOS build complete! 🎉"

# Step 5: Optional - Code Signing
if [ "$1" = "--sign" ]; then
    print_step "Code signing application..."
    codesign --force --deep --sign "Apple Development" "build/Release/Storm.app"
    print_success "Application signed"
fi

# Step 6: Optional - Create DMG
if [ "$1" = "--dmg" ]; then
    print_step "Creating DMG installer..."
    hdiutil create -size 100m -fs HFS+ -volname "Storm" Storm.dmg
    hdiutil attach Storm.dmg
    cp -R "build/Release/Storm.app" "/Volumes/Storm/"
    hdiutil detach "/Volumes/Storm"
    print_success "DMG created: Storm.dmg"
fi

echo -e "${GREEN}🌪️  Storm is ready to revolutionize virtual worlds!${NC}"
```

## Development Workflow

### 1. Initial Setup
```bash
# Clone the repository
git clone https://github.com/finalverse/storm-core.git
cd storm-core/platforms/macOS

# Make build script executable
chmod +x build_storm.sh

# Build everything
./build_storm.sh
```

### 2. Daily Development
```bash
# Quick Rust rebuild
cd ../../../storm-core
cargo build --release --target aarch64-apple-darwin

# Copy to Xcode project
cp target/aarch64-apple-darwin/release/libstorm_ffi.dylib platforms/macOS/Frameworks/

# Build in Xcode or via command line
cd platforms/macOS
xcodebuild -project Storm.xcodeproj -scheme Storm build
```

### 3. Testing and Debugging

#### Rust Core Testing
```bash
cd storm-core
cargo test --workspace
cargo test --features "ffi"
```

#### Swift Unit Testing
```bash
cd platforms/macOS
xcodebuild test -project Storm.xcodeproj -scheme Storm
```

#### Integration Testing
```bash
# Test FFI bridge
cd platforms/macOS
xcodebuild test -project Storm.xcodeproj -scheme Storm -only-testing:StormTests/FFIBridgeTests
```

## Performance Optimization

### 1. Rust Optimizations
```toml
# In storm-core/Cargo.toml
[profile.release]
lto = true
codegen-units = 1
panic = "abort"
strip = true
```

### 2. Swift Optimizations
```swift
// Use @inlinable for performance-critical FFI calls
@inlinable
func createAvatar(at position: simd_float3) -> UInt64 {
    return ffi_create_avatar(position.x, position.y, position.z)
}
```

### 3. RealityKit Optimizations
- Use Level-of-Detail (LOD) for distant objects
- Implement frustum culling
- Batch similar entities
- Use instanced rendering for repeated objects

## Troubleshooting

### Common Issues

1. **Library Not Found**
   ```
   Error: dyld: Library not loaded: libstorm_ffi.dylib
   Solution: Check library search paths and runpath settings
   ```

2. **FFI Symbol Errors**
   ```
   Error: Undefined symbol: _ffi_create_avatar
   Solution: Verify bridging header and Rust exports
   ```

3. **RealityKit Crashes**
   ```
   Error: RealityKit crash on entity creation
   Solution: Check entity component compatibility and threading
   ```

4. **Performance Issues**
   ```
   Issue: Low FPS in RealityKit
   Solution: Profile with Instruments, optimize entity counts
   ```

### Debug Configuration

```swift
#if DEBUG
StormFFIBridge.setDebugLogging(true)
#endif
```

## Deployment

### 1. App Store Distribution
- Enable App Sandbox
- Configure entitlements for network access
- Include privacy descriptions in Info.plist
- Test on clean macOS installation

### 2. Direct Distribution
```bash
# Code sign for direct distribution
codesign --force --deep --sign "Developer ID Application: Your Name" Storm.app

# Notarize for macOS Gatekeeper
xcrun notarytool submit Storm.app --apple-id your@email.com --team-id TEAMID --password app-password
```

### 3. Enterprise Distribution
- Use enterprise provisioning profile
- Configure MDM deployment settings
- Include deployment scripts

## Advanced Features

### 1. AR/VR Integration
```swift
// Enable ARKit in RealityView
RealityView { content in
    setupARScene(content: content)
} update: { content in
    updateARScene(content: content)
}
.modifier(ARViewModifier())
```

### 2. Metal Performance Shaders
```swift
// Custom compute shaders for AI acceleration
let library = device.makeDefaultLibrary()
let function = library?.makeFunction(name: "ai_enhancement_compute")
```

### 3. Core ML Integration
```swift
// Local AI model inference
import CoreML
let model = try? StormAIModel(configuration: MLModelConfiguration())
```

This completes the comprehensive Storm macOS application setup with SwiftUI + RealityKit frontend connecting to the Storm Rust core via high-performance FFI. The application provides MetaHuman-style avatar customization, native physics and audio engines, and seamless OpenSim server connectivity.
