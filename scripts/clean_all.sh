#!/bin/bash
# Clean build artifacts

echo "🧹 Cleaning StormCore Build Artifacts"
echo "===================================="

cargo clean
rm -rf target/
rm -rf */target/
rm -rf */*/target/

echo "✅ Clean complete!"
