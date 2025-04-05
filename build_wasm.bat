@echo off
echo Building WASM package...
call wasm-pack build --target web --out-dir web/pkg

echo Build complete! The app is ready in the 'web' directory.
echo To test locally, you can use a local HTTP server:
echo cd web ^&^& python -m http.server
echo Then navigate to http://localhost:8000 in your browser. 