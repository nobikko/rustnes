//! NES WASM - WASM wrapper for NES emulator

use nes_core::system::NesSystem;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::JsError;
use js_sys::Uint8Array;
use js_sys::ArrayBuffer;
use std::slice;

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
    /// Returns true on success, false on failure
    pub fn load_rom(&mut self, rom_data: &[u8]) -> bool {
        self.system.load_rom(rom_data).is_ok()
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
    pub fn framebuffer_rgb(&self) -> Uint8Array {
        let ppu = self.system.ppu();
        let scanline = ppu.scanline() as usize;

        // Create a fixed-size buffer on the heap
        let buffer_size = 256 * 240 * 3;
        let mut buffer = Vec::with_capacity(buffer_size);
        buffer.resize(buffer_size, 0);

        // Render a simple test pattern based on scanline
        for y in 0..240 {
            for x in 0..256 {
                let idx = (y * 256 + x) * 3;
                // Simple pattern: gradient based on position and scanline
                buffer[idx] = ((x / 16) as u8 * 16) % 255;
                buffer[idx + 1] = ((y / 16) as u8 * 16) % 255;
                buffer[idx + 2] = ((scanline / 4) as u8 * 4) % 255;
            }
        }

        // Get the pointer and length
        let len = buffer.len();
        let ptr = buffer.as_ptr() as usize;

        // Create a new ArrayBuffer
        let array_buffer = ArrayBuffer::new(buffer_size as u32);

        // Create Uint8Array from the ArrayBuffer
        let arr = Uint8Array::new(&array_buffer);

        // Copy data using copy_from
        unsafe {
            let src_slice = slice::from_raw_parts(ptr as *const u8, len);
            arr.copy_from(src_slice);
        }

        // Forget the buffer to prevent double free
        std::mem::forget(buffer);

        arr
    }

    /// Get PPU framebuffer length
    pub fn framebuffer_len(&self) -> usize {
        256 * 240 * 3
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