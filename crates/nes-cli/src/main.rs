//! NES CLI - Command line interface for NES emulator

use clap::Parser;
use nes_core::cartridge::Cartridge;
use nes_core::system::NesSystem;
use std::fs;
use std::path::PathBuf;

/// NES Emulator CLI
#[derive(Parser, Debug)]
#[command(name = "nes-cli")]
#[command(about = "A NES emulator CLI", long_about = None)]
struct Args {
    /// Path to the iNES ROM file
    #[arg(short, long)]
    rom: PathBuf,

    /// Number of frames to run
    #[arg(short, long, default_value = "60")]
    frames: u64,

    /// Dump CPU state after execution
    #[arg(short = 'c', long)]
    dump_cpu: bool,

    /// Dump PPU state after execution
    #[arg(short = 'p', long)]
    dump_ppu: bool,
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

    println!("\nRunning {} frames...", args.frames);

    // Run for specified frames
    if let Err(e) = system.run_frames(args.frames) {
        eprintln!("Error running system: {}", e);
        std::process::exit(1);
    }

    println!("Completed {} frames.", system.frame_count());

    // Dump state if requested
    if args.dump_cpu {
        dump_cpu_state(&system);
    }

    if args.dump_ppu {
        dump_ppu_state(&system);
    }
}

fn dump_cpu_state(system: &NesSystem) {
    let cpu = system.cpu();
    let regs = cpu.registers();
    let status = cpu.status();

    println!("\nCPU State:");
    println!("  A:    ${:02X}", regs.a);
    println!("  X:    ${:02X}", regs.x);
    println!("  Y:    ${:02X}", regs.y);
    println!("  PC:   ${:04X}", regs.pc);
    println!("  SP:   ${:02X}", regs.sp);
    println!("  P:    {} ({})", status, status);
    println!("  Cycles: {}", cpu.total_cycles());
}

fn dump_ppu_state(system: &NesSystem) {
    let ppu = system.ppu();

    println!("\nPPU State:");
    println!("  Scanline: {}", ppu.scanline());
    println!("  Dot: {}", ppu.dot());
    println!("  VBLANK: {}", ppu.status().vblank());
}