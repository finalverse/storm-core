#!/bin/bash
# File: scripts/quick_build.sh

echo "🚀 StormCore Quick Build"

echo "📦 Building core workspace..."

# Build in dependency order to catch issues early
echo "Building core math and ECS..."
cargo build -p storm-math || exit 1
cargo build -p storm-ecs || exit 1

echo "Building networking and AI..."
cargo build -p storm-networking || exit 1
cargo build -p storm-ai || exit 1

echo "Building rendering and audio..."
cargo build -p storm-rendering || exit 1
cargo build -p storm-audio || exit 1

echo "Building protocol adapters..."
cargo build -p storm-protocol-adapters || exit 1

echo "Building main storm-core..."
cargo build -p storm-core || exit 1

echo "Building examples..."
cargo build -p opensim-client || exit 1
cargo build -p virtual-world-showcase || exit 1

# Try to build WASM client
cargo build -p storm-wasm-client || echo "⚠️ WASM client build failed, continuing..."

echo "Building WASM (if wasm-pack available)..."
if command -v wasm-pack &> /dev/null; then
    cd crates/storm-wasm
    wasm-pack build --target web --dev || echo "⚠️ WASM pack failed"
    cd ../..
else
    echo "⚠️ wasm-pack not found, skipping WASM build"
fi

echo "✅ Quick build completed!"
echo ""
echo "🔍 To check what built successfully:"
echo "  cargo build --workspace --message-format=short"
echo ""
echo "🧪 To run tests:"
echo "  ./scripts/run_tests.sh"