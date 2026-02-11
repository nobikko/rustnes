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

    /// Get PPU framebuffer (256x240 RGB pixels)
    /// Returns raw RGB data (76800 bytes: 256 * 240 * 3)
    #[wasm_bindgen(getter)]
    pub fn framebuffer_rgb(&self) -> Vec<u8> {
        let ppu = self.system.ppu();
        let mut framebuffer = Vec::with_capacity(256 * 240 * 3);

        // For now, render a simple test pattern based on scanline
        // In a full implementation, this would render the actual PPU output
        let scanline = ppu.scanline() as usize;
        for y in 0..240 {
            for x in 0..256 {
                // Simple pattern: gradient based on position and scanline
                let r = ((x / 16) as u8 * 16) % 255;
                let g = ((y / 16) as u8 * 16) % 255;
                let b = ((scanline / 4) as u8 * 4) % 255;
                framebuffer.push(r);
                framebuffer.push(g);
                framebuffer.push(b);
            }
        }
        framebuffer
    }

    /// Get current PPU scanline
    pub fn scanline(&self) -> u32 {
        self.system.ppu().scanline() as u32
    }

    /// Get current PPU dot
    pub fn dot(&self) -> u32 {
        self.system.ppu().dot() as u32
    }

    /// Check if VBLANK is active
    pub fn vblank(&self) -> bool {
        self.system.ppu().status().vblank()
    }

    /// Get CPU cycles
    pub fn cpu_cycles(&self) -> u32 {
        self.system.cpu().total_cycles() as u32
    }
}

#[wasm_bindgen]
pub fn version() -> String {
    "0.1.0".to_string()
}