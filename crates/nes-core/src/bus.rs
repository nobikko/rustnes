//! Memory bus and mapping
//!
//! The NES memory map:
//! $0000-$07FF - 2KB Internal RAM
//! $0800-$1FFF - RAM mirroring (repeats every $0800 bytes)
//! $2000-$2007 - PPU registers (mirrored every $08 bytes)
//! $2008-$3FFF - PPU registers (mirrored every $08 bytes)
//! $4000-$4017 - APU and I/O registers
//! $4020-$5FFF - Cartridge expansion (NA, PRG ROM at $8000-$FFFF)
//! $6000-$7FFF - Cartridge PRG RAM (if present)
//! $8000-$FFFF - Cartridge PRG ROM

use crate::cpu::Bus as CpuBus;

/// RAM size in bytes
pub const RAM_SIZE: usize = 2048; // 2KB

/// PPU register count
pub const PPU_REGISTER_COUNT: usize = 8;

/// APU/IO register count
pub const APU_REGISTER_COUNT: usize = 24;

/// Memory bus structure
#[derive(Debug, Clone)]
pub struct Bus {
    /// 2KB internal RAM (with mirroring)
    ram: [u8; RAM_SIZE],
    /// PPU registers
    ppu_registers: [u8; PPU_REGISTER_COUNT],
    /// APU/IO registers
    apu_registers: [u8; APU_REGISTER_COUNT],
    /// Cartridge (PRG and CHR memory)
    cartridge: Option<SimpleCartridge>,
}

impl Bus {
    /// Create a new bus with no cartridge
    pub fn new() -> Self {
        Self {
            ram: [0; RAM_SIZE],
            ppu_registers: [0; PPU_REGISTER_COUNT],
            apu_registers: [0; APU_REGISTER_COUNT],
            cartridge: None,
        }
    }

    /// Set the cartridge for this bus
    pub fn set_cartridge(&mut self, cartridge: SimpleCartridge) {
        self.cartridge = Some(cartridge);
    }

    /// Get a reference to the cartridge, if present
    pub fn cartridge(&self) -> Option<&SimpleCartridge> {
        self.cartridge.as_ref()
    }

    /// Get a reference to the CHR ROM, if present
    pub fn chr_rom(&self) -> Option<&[u8]> {
        self.cartridge.as_ref().map(|c| c.chr_rom.as_slice())
    }
}

impl CpuBus for Bus {
    /// Read a byte from the given address
    fn read(&mut self, address: u16) -> u8 {
        match address {
            // $0000-$07FF - Internal RAM
            0x0000..=0x07FF => {
                // Reflect address to first 2KB
                self.ram[(address & 0x07FF) as usize]
            }
            // $0800-$1FFF - RAM mirroring
            0x0800..=0x1FFF => {
                self.ram[(address & 0x07FF) as usize]
            }
            // $2000-$2007 - PPU registers
            0x2000..=0x2007 => {
                self.ppu_registers[(address & 0x0007) as usize]
            }
            // $2008-$3FFF - PPU register mirroring (every 8 bytes)
            0x2008..=0x3FFF => {
                self.ppu_registers[((address - 0x2008) & 0x0007) as usize]
            }
            // $4000-$4017 - APU and I/O registers
            0x4000..=0x4017 => {
                self.apu_registers[(address - 0x4000) as usize]
            }
            // $4020-$5FFF - Cartridge expansion (NA)
            0x4020..=0x5FFF => {
                // ExpansionROM access - return 0xFF for now
                0xFF
            }
            // $6000-$7FFF - Cartridge PRG RAM (if present)
            0x6000..=0x7FFF => {
                if let Some(ref cart) = self.cartridge {
                    cart.read_prm_ram(address)
                } else {
                    0xFF
                }
            }
            // $8000-$FFFF - Cartridge PRG ROM
            0x8000..=0xFFFF => {
                if let Some(ref cart) = self.cartridge {
                    cart.read_prd_rom(address)
                } else {
                    0xFF
                }
            }
            _ => 0xFF,
        }
    }

    /// Write a byte to the given address
    fn write(&mut self, address: u16, value: u8) {
        match address {
            // $0000-$07FF - Internal RAM
            0x0000..=0x07FF => {
                self.ram[(address & 0x07FF) as usize] = value;
            }
            // $0800-$1FFF - RAM mirroring
            0x0800..=0x1FFF => {
                self.ram[(address & 0x07FF) as usize] = value;
            }
            // $2000-$2007 - PPU registers
            0x2000..=0x2007 => {
                self.ppu_registers[(address & 0x0007) as usize] = value;
            }
            // $2008-$3FFF - PPU register mirroring
            0x2008..=0x3FFF => {
                self.ppu_registers[((address - 0x2008) & 0x0007) as usize] = value;
            }
            // $4000-$4017 - APU and I/O registers
            0x4000..=0x4017 => {
                self.apu_registers[(address - 0x4000) as usize] = value;
            }
            // $4020-$5FFF - Cartridge expansion (NA)
            0x4020..=0x5FFF => {
                // ExpansionROM write - ignore for now
            }
            // $6000-$7FFF - Cartridge PRG RAM (if present)
            0x6000..=0x7FFF => {
                if let Some(ref mut cart) = self.cartridge {
                    cart.write_prm_ram(address, value);
                }
            }
            // $8000-$FFFF - Cartridge PRG ROM (write-protected, ignore)
            0x8000..=0xFFFF => {
                // PRG ROM is write-protected
            }
            _ => {}
        }
    }
}

/// SimpleCartridge - basic cartridge without iNES parsing
#[derive(Debug, Clone)]
pub struct SimpleCartridge {
    /// PRG ROM data
    prg_rom: Vec<u8>,
    /// PRG RAM data (if present)
    prg_ram: Option<Vec<u8>>,
    /// CHR ROM data
    chr_rom: Vec<u8>,
}

impl SimpleCartridge {
    /// Create a new simple cartridge from PRG and CHR ROM data
    pub fn new(prg_rom: Vec<u8>, chr_rom: Vec<u8>) -> Self {
        Self {
            prg_rom,
            prg_ram: Some(vec![0xFF; 8192]), // Default 8KB PRG RAM
            chr_rom,
        }
    }

    /// Read from PRG ROM
    pub fn read_prd_rom(&self, address: u16) -> u8 {
        // Remove the $8000 offset
        let offset = (address - 0x8000) as usize;
        if offset < self.prg_rom.len() {
            self.prg_rom[offset]
        } else {
            // For 16KB PRG ROM, $8000-$BFFF and $C000-$FFFF both map to same data (mirroring)
            // Wrap the offset to the first 16KB
            let prg_size = self.prg_rom.len();
            let mirrored_offset = offset % prg_size;
            self.prg_rom[mirrored_offset]
        }
    }

    /// Read from PRG RAM
    pub fn read_prm_ram(&self, _address: u16) -> u8 {
        if let Some(ref prg_ram) = self.prg_ram {
            // Simple direct mapping for now
            prg_ram[0] // TODO: proper addressing
        } else {
            0xFF
        }
    }

    /// Write to PRG RAM
    pub fn write_prm_ram(&mut self, _address: u16, value: u8) {
        if let Some(ref mut prg_ram) = self.prg_ram {
            // Simple direct mapping for now
            prg_ram[0] = value; // TODO: proper addressing
        }
    }

    /// Get PRG ROM size in bytes
    pub fn prg_rom_size(&self) -> usize {
        self.prg_rom.len()
    }

    /// Get CHR ROM size in bytes
    pub fn chr_rom_size(&self) -> usize {
        self.chr_rom.len()
    }
}

/// Mapper types
#[derive(Debug, Clone, Copy)]
pub enum Mapper {
    /// NROM - Simple mapper, no bank switching
    NROM,
    /// UxROM - Simple mapper with PRG bank switching
    UXROM,
    /// CNROM - Simple mapper with CHR bank switching
    CNROM,
}

impl Default for Mapper {
    fn default() -> Self {
        Mapper::NROM
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bus_read_write() {
        let mut bus = Bus::new();

        // Write to RAM
        bus.write(0x0000, 0x42);
        assert_eq!(bus.read(0x0000), 0x42);

        // Test RAM mirroring
        bus.write(0x0001, 0x43);
        assert_eq!(bus.read(0x0801), 0x43);
    }

    #[test]
    fn test_cartridge_creation() {
        let prg_rom = vec![0xFF; 16384]; // 16KB
        let chr_rom = vec![0x00; 8192];  // 8KB
        let cart = SimpleCartridge::new(prg_rom, chr_rom);

        assert_eq!(cart.prg_rom_size(), 16384);
        assert_eq!(cart.chr_rom_size(), 8192);
    }
}