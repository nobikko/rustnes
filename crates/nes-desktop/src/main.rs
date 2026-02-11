//! NES Desktop - Desktop NES emulator with minifb rendering
//!
//! This is a desktop version of the NES emulator that uses:
//! - minifb for simple window creation and rendering

use clap::Parser;
use nes_core::cartridge::Cartridge;
use nes_core::system::NesSystem;
use std::fs;
use std::path::PathBuf;
use minifb::{Window, WindowOptions, Key};

/// NES Emulator Desktop App
#[derive(Parser, Debug)]
#[command(name = "nes-desktop")]
#[command(about = "A NES emulator desktop app", long_about = None)]
struct Args {
    /// Path to the iNES ROM file
    #[arg(short, long)]
    rom: PathBuf,

    /// Screen scale factor (1-4)
    #[arg(short, long, default_value = "2")]
    scale: usize,
}

fn main() {
    let args = Args::parse();

    // Load ROM file
    let rom_data = match fs::read(&args.rom) {
        Ok(data) => data,
        Err(e) => {
            eprintln!("Failed to read ROM file: {}", e);
            std::process::exit(1);
        }
    };

    // Load iNES cartridge
    let cartridge = match Cartridge::from_rom(&rom_data) {
        Ok(cart) => cart,
        Err(e) => {
            eprintln!("Failed to load cartridge: {}", e);
            std::process::exit(1);
        }
    };

    println!("Loaded cartridge:");
    println!("  PRG ROM: {} bytes", cartridge.prg_rom().len());
    println!("  CHR ROM: {} bytes", cartridge.chr_rom().len());
    println!("  Mapper: {:?}", cartridge.mapper());

    // Create and initialize system
    let mut system = NesSystem::new();
    if let Err(e) = system.load_rom(&rom_data) {
        eprintln!("Failed to load ROM: {}", e);
        std::process::exit(1);
    }
    system.reset();

    // NES resolution is 256x240
    let nes_width = 256;
    let nes_height = 240;

    // Create window with specified scale
    let scale = args.scale.min(4).max(1);
    let window_width = nes_width * scale;
    let window_height = nes_height * scale;

    let mut window = Window::new(
        "NES Emulator",
        window_width,
        window_height,
        WindowOptions {
            resize: false,
            ..WindowOptions::default()
        },
    ).expect("Failed to create window");

    // Framebuffer storage (RGB bytes for PPU)
    let mut framebuffer = vec![0u8; nes_width * nes_height * 3];

    // Convert RGB to RGBA for minifb (0xAABBGGRR format - little endian BGRA)
    let mut rgba_buffer = vec![0u32; nes_width * nes_height];

    println!("\nStarting NES emulation...");
    println!("Press ESC or close the window to exit.");

    while window.is_open() && !window.is_key_down(Key::Escape) {
        // Run one frame of emulation
        let _ = system.run_frames(1);

        // Get PPU scanline for rendering effect
        let ppu = system.ppu();
        let scanline = ppu.scanline() as usize;

        // Render framebuffer (test pattern for now)
        render_framebuffer(&mut framebuffer, nes_width, nes_height, scanline);

        // Convert RGB to RGBA (minifb uses 0xAABBGGRR format)
        for i in 0..nes_width * nes_height {
            let r = framebuffer[i * 3];
            let g = framebuffer[i * 3 + 1];
            let b = framebuffer[i * 3 + 2];
            // Convert to minifb format: 0xAABBGGRR (little endian)
            rgba_buffer[i] = ((255u32) << 24) | ((b as u32) << 16) | ((g as u32) << 8) | (r as u32);
        }

        // Update window with framebuffer
        window
            .update_with_buffer(&rgba_buffer, nes_width, nes_height)
            .expect("Failed to update window");
    }

    println!("Emulator closed.");
}

fn render_framebuffer(framebuffer: &mut [u8], width: usize, height: usize, scanline: usize) {
    for y in 0..height {
        for x in 0..width {
            let idx = (y * width + x) * 3;
            // Simple gradient pattern based on position and scanline
            framebuffer[idx] = ((x / 16) as u8 * 16) % 255;
            framebuffer[idx + 1] = ((y / 16) as u8 * 16) % 255;
            framebuffer[idx + 2] = ((scanline / 4) as u8 * 4) % 255;
        }
    }
}