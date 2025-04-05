#!/bin/bash
set -e

# Build WASM package
echo "Building WASM package..."
wasm-pack build --target web --out-dir web/pkg

# Copy the index.html if it's not already in the web directory
if [ ! -f "web/index.html" ]; then
  echo "Copying index.html to web directory..."
  cp index.html web/
fi

echo "Build complete! The app is ready in the 'web' directory."
echo "To test locally, you can use a local HTTP server:"
echo "cd web && python -m http.server"
echo "Then navigate to http://localhost:8000 in your browser." 