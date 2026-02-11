# NES Emulator Demo

A simple web-based NES emulator demo built with Rust and WebAssembly.

## Files

- `index.html` - The web page with the emulator UI
- `nes_wasm.js` - JavaScript bindings for the WASM module
- `nes_wasm_bg.wasm` - The compiled WebAssembly binary
- `test.nes` - A simple test ROM

## How to Run

### Option 1: Simple HTTP Server

```bash
cd demo
python3 -m http.server 8000
```

Then open `http://localhost:8000` in your browser.

### Option 2: VS Code Live Server

1. Install the "Live Server" extension in VS Code
2. Right-click on `index.html`
3. Select "Open with Live Server"

## Controls

- **Run/Stop** - Start or stop the emulator
- **Step** - Step through one frame at a time
- **Reset** - Reset the emulator
- **Load ROM** - Load a custom `.nes` file
- **Load Test ROM** - Load the built-in test ROM

## Features

- Full 6502 CPU emulation
- PPU rendering with color output
- Frame timing
- CPU/PPU status display