//! NES System Integration
//!
//! This module integrates all NES components (CPU, PPU, APU, Bus) into a working system.

use crate::bus::{Bus, SimpleCartridge};
use crate::cpu::Bus as CpuBus;
use crate::cartridge::{Cartridge, CartridgeError};
use crate::cpu::{Cpu, CpuError};
use crate::ppu::Ppu;
use crate::apu::Apu;

/// NES System - integrates all components
#[derive(Debug, Clone)]
pub struct NesSystem {
    cpu: Cpu,
    ppu: Ppu,
    apu: Apu,
    bus: Bus,
    /// Frame counter
    frame_count: u64,
    /// Track if PPU has been initialized
    ppu_initialized: bool,
}

impl NesSystem {
    /// Create a new NES system with no cartridge
    pub fn new() -> Self {
        Self {
            cpu: Cpu::new(),
            ppu: Ppu::new(),
            apu: Apu::new(),
            bus: Bus::new(),
            frame_count: 0,
            ppu_initialized: false,
        }
    }

    /// Initialize PPU with CHR ROM and set up PPU reference in bus
    pub fn initialize_ppu(&mut self) {
        if self.ppu_initialized {
            return;
        }

        // Get CHR ROM from cartridge and set it on PPU
        if let Some(chr_rom) = self.bus.chr_rom() {
            self.ppu.set_chr_rom(chr_rom.to_vec());
        }

        self.ppu_initialized = true;
    }

    /// Load a simple cartridge into the system
    pub fn load_simple_cartridge(&mut self, cartridge: SimpleCartridge) {
        self.bus.set_cartridge(cartridge);
    }

    /// Load an iNES ROM file into the system
    pub fn load_rom(&mut self, rom_data: &[u8]) -> Result<(), CartridgeError> {
        let cartridge = Cartridge::from_rom(rom_data)?;
        self.bus.set_cartridge(SimpleCartridge::new(
            cartridge.prg_rom().to_vec(),
            cartridge.chr_rom().to_vec(),
        ));
        Ok(())
    }

    /// Reset the NES system
    pub fn reset(&mut self) {
        self.cpu.reset();
        self.ppu.reset();
        self.apu.reset();
        self.frame_count = 0;
    }

    /// Step the system by one instruction (CPU)
    /// This also steps PPU appropriately (3 PPU cycles per CPU cycle)
    pub fn step(&mut self) -> Result<bool, CpuError> {
        // Sync PPU registers from bus to PPU before CPU reads
        self.sync_ppu_registers();

        // Get opcode and decode it before stepping
        let opcode_byte = self.bus.read(self.cpu.registers().pc);
        let opcode = match self.cpu.decode_opcode(opcode_byte) {
            Ok(op) => op,
            Err(e) => return Err(e),
        };
        let instruction_cycles = self.cpu.instruction_cycles(opcode).max(1);

        // Step CPU
        let running = self.cpu.step(&mut self.bus)?;
        if !running {
            return Ok(false);
        }

        // Step PPU (3 cycles for each CPU cycle)
        for _ in 0..(instruction_cycles as usize * 3) {
            self.ppu.step();
        }

        // Step APU
        self.apu.step(instruction_cycles as u8);

        Ok(true)
    }

    /// Sync PPU internal state from bus registers
    pub fn sync_ppu_registers(&mut self) {
        // Read values from bus's ppu_registers and sync to PPU
        // The bus stores writes to PPU registers ($2000-$2007)
        self.ppu.write(0x2000, self.bus.get_ppu_register(0)); // PPUCTRL
        self.ppu.write(0x2001, self.bus.get_ppu_register(1)); // PPUMASK
        self.ppu.write(0x2003, self.bus.get_ppu_register(3)); // OAMADDR
        self.ppu.write(0x2005, self.bus.get_ppu_register(5)); // PPUSCROLL
        self.ppu.write(0x2006, self.bus.get_ppu_register(6)); // PPUADDR
    }

    /// Run for N frames
    pub fn run_frames(&mut self, frames: u64) -> Result<(), Box<dyn std::error::Error>> {
        // NTSC: ~29780 cycles per frame
        let cycles_per_frame = 29780;

        for _ in 0..frames {
            // Run for one frame
            for _ in 0..cycles_per_frame {
                self.step()?;
            }
            self.frame_count += 1;
        }
        Ok(())
    }

    /// Run until VBLANK is set (one frame)
    pub fn run_until_vblank(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut cycles = 0u64;
        let max_cycles = 30000; // Safety limit

        while cycles < max_cycles {
            self.step()?;
            cycles += 1;

            if self.ppu.status().vblank() {
                return Ok(());
            }
        }

        Err("Timeout waiting for VBLANK".into())
    }

    /// Get CPU reference
    pub fn cpu(&self) -> &Cpu {
        &self.cpu
    }

    /// Get mutable CPU reference
    pub fn cpu_mut(&mut self) -> &mut Cpu {
        &mut self.cpu
    }

    /// Get PPU reference
    pub fn ppu(&self) -> &Ppu {
        &self.ppu
    }

    /// Get mutable PPU reference
    pub fn ppu_mut(&mut self) -> &mut Ppu {
        &mut self.ppu
    }

    /// Get CHR ROM data
    pub fn chr_rom(&self) -> Option<&[u8]> {
        self.bus.chr_rom()
    }

    /// Get APU reference
    pub fn apu(&self) -> &Apu {
        &self.apu
    }

    /// Get mutable APU reference
    pub fn apu_mut(&mut self) -> &mut Apu {
        &mut self.apu
    }

    /// Get frame count
    pub fn frame_count(&self) -> u64 {
        self.frame_count
    }

    /// Read a byte from memory via the bus
    pub fn read_memory(&mut self, address: u16) -> u8 {
        self.bus.read(address)
    }

    /// Write a byte to memory via the bus
    pub fn write_memory(&mut self, address: u16, value: u8) {
        self.bus.write(address, value);
    }

    /// Get a reference to the bus's cartridge
    pub fn bus_cartridge(&self) -> Option<&SimpleCartridge> {
        self.bus.cartridge()
    }
}

impl Default for NesSystem {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bus::SimpleCartridge;

    #[test]
    fn test_system_reset() {
        let mut system = NesSystem::new();
        system.reset();
        // After reset, CPU should be ready
        assert_eq!(system.cpu().registers().pc, 0xFFFC);
    }

    #[test]
    fn test_system_with_cartridge() {
        let prg_rom = vec![0xFF; 16384]; // 16KB
        let chr_rom = vec![0x00; 8192];  // 8KB
        let cartridge = SimpleCartridge::new(prg_rom, chr_rom);

        let mut system = NesSystem::new();
        system.load_simple_cartridge(cartridge);
        system.reset();

        assert!(system.cpu().registers().pc == 0xFFFC);
    }
}