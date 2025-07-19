# File: examples/wasm-client/wasm-pack-build.sh
#!/bin/bash
# Build script for StormCore WASM client

set -e

echo "🌩️ Building StormCore WASM Client..."

# Build the WASM package
wasm-pack build --target web --out-dir pkg --dev

# Copy HTML and assets
cp index.html pkg/
cp -r assets/ pkg/ 2>/dev/null || echo "No assets directory found"

# Create a simple HTTP server script
cat > pkg/serve.py << 'EOF'
#!/usr/bin/env python3
import http.server
import socketserver
import os
import webbrowser
from pathlib import Path

PORT = 8000
DIRECTORY = Path(__file__).parent

class MyHTTPRequestHandler(http.server.SimpleHTTPRequestHandler):
    def __init__(self, *args, **kwargs):
        super().__init__(*args, directory=DIRECTORY, **kwargs)

    def end_headers(self):
        self.send_header('Cross-Origin-Embedder-Policy', 'require-corp')
        self.send_header('Cross-Origin-Opener-Policy', 'same-origin')
        super().end_headers()

os.chdir(DIRECTORY)
with socketserver.TCPServer(("", PORT), MyHTTPRequestHandler) as httpd:
    print(f"🚀 Serving StormCore WASM Demo at http://localhost:{PORT}")
    print("Press Ctrl+C to stop")

    # Try to open browser
    try:
        webbrowser.open(f'http://localhost:{PORT}')
    except:
        pass

    httpd.serve_forever()
EOF

chmod +x pkg/serve.py

echo "✅ Build complete!"
echo ""
echo "To run the demo:"
echo "  cd pkg && python3 serve.py"
echo ""
echo "Or use any HTTP server that supports WASM:"
echo "  cd pkg && python3 -m http.server 8000"
echo "  cd pkg && npx serve ."
echo ""