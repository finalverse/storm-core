#!/bin/bash
# File: scripts/run_tests.sh

echo "🧪 Running StormCore Tests"

echo "Testing core components..."
cargo test -p storm-math
cargo test -p storm-ecs
cargo test -p storm-ai
cargo test -p storm-core

echo "Testing integration..."
cargo test -p opensim-client

echo "Running basic integration test..."
cargo test --test basic_integration

echo "✅ Tests completed!"