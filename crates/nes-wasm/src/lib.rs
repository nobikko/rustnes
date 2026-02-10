//! NES WASM - WASM wrapper for NES emulator

use nes_core::system::NesSystem;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::JsError;

/// NES Emulator wrapper for WASM
#[wasm_bindgen]
pub struct NesEmulator {
    system: NesSystem,
}

#[wasm_bindgen]
impl NesEmulator {
    /// Create a new NES emulator
    #[wasm_bindgen(constructor)]
    pub fn new() -> NesEmulator {
        Self {
            system: NesSystem::new(),
        }
    }

    /// Load a ROM from bytes
    pub fn load_rom(&mut self, rom_data: &[u8]) -> Result<(), JsError> {
        self.system.load_rom(rom_data).map_err(|e| JsError::new(&e.to_string()))?;
        Ok(())
    }

    /// Reset the emulator
    pub fn reset(&mut self) {
        self.system.reset();
    }

    /// Step the emulator once
    pub fn step(&mut self) -> bool {
        match self.system.step() {
            Ok(running) => running,
            Err(_) => false,
        }
    }

    /// Run for N frames
    pub fn run_frames(&mut self, frames: u32) {
        let _ = self.system.run_frames(frames as u64);
    }

    /// Get the current frame count
    pub fn frame_count(&self) -> u32 {
        self.system.frame_count() as u32
    }

    /// Get PPU framebuffer (simulated - for testing)
    /// Returns a simple pattern for now
    #[wasm_bindgen(getter)]
    pub fn framebuffer(&self) -> Vec<u8> {
        // Return a simple test pattern
        vec![0x00; 256 * 240] // 256x240 grayscale
    }
}

#[wasm_bindgen]
pub fn version() -> String {
    "0.1.0".to_string()
}