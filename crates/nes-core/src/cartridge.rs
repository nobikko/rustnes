//! Cartridge and mapper support
//!
//! This module handles cartridge ROM loading and mapper logic.
//! Mappers are used to expand the addressable memory beyond the NES limitations.

use crate::bus::Mapper;

/// iNES header size
pub const HEADER_SIZE: usize = 16;

/// iNES header structure
#[derive(Debug, Clone)]
pub struct InesHeader {
    /// Magic number: "NES\x1A"
    pub magic: [u8; 4],
    /// PRG ROM size in 16KB units
    pub prg_rom_size: u8,
    /// CHR ROM size in 8KB units
    pub chr_rom_size: u8,
    /// Flags 6
    pub flags_6: u8,
    /// Flags 7
    pub flags_7: u8,
    /// PRG RAM size in 8KB units
    pub prg_ram_size: u8,
    /// Flags 9
    pub flags_9: u8,
    /// Flags 10
    pub flags_10: u8,
    /// Padding
    pub padding: [u8; 5],
}

impl InesHeader {
    /// Parse an iNES header from bytes
    pub fn parse(bytes: &[u8]) -> Result<Self, CartridgeError> {
        if bytes.len() < HEADER_SIZE {
            return Err(CartridgeError::InvalidHeader("Too short"));
        }

        let magic = [bytes[0], bytes[1], bytes[2], bytes[3]];
        if magic != [b'N', b'E', b'S', 0x1A] {
            return Err(CartridgeError::InvalidHeader("Invalid magic"));
        }

        Ok(Self {
            magic,
            prg_rom_size: bytes[4],
            chr_rom_size: bytes[5],
            flags_6: bytes[6],
            flags_7: bytes[7],
            prg_ram_size: bytes[8],
            flags_9: bytes[9],
            flags_10: bytes[10],
            padding: [bytes[11], bytes[12], bytes[13], bytes[14], bytes[15]],
        })
    }

    /// Get the mapper number from flags
    pub fn mapper_number(&self) -> u8 {
        ((self.flags_6 >> 4) as u8) | (self.flags_7 as u8 & 0xF0)
    }

    /// Check if trainer is present
    pub fn has_trainer(&self) -> bool {
        (self.flags_6 & 0x04) != 0
    }

    /// Check if mirror mode is horizontal
    pub fn is_horizontal_mirror(&self) -> bool {
        (self.flags_6 & 0x01) != 0
    }

    /// Check if SRAM is present
    pub fn has_sram(&self) -> bool {
        (self.flags_6 & 0x02) != 0
    }
}

/// Cartridge structure
#[derive(Debug, Clone)]
pub struct Cartridge {
    /// iNES header
    header: InesHeader,
    /// PRG ROM data
    prg_rom: Vec<u8>,
    /// CHR ROM data
    chr_rom: Vec<u8>,
    /// PRG RAM data (if present)
    prg_ram: Option<Vec<u8>>,
    /// Mapper type
    mapper: Mapper,
    /// Trainer data (if present)
    trainer: Option<Vec<u8>>,
}

impl Cartridge {
    /// Create a new cartridge from iNES ROM data
    pub fn from_rom(rom_data: &[u8]) -> Result<Self, CartridgeError> {
        if rom_data.len() < HEADER_SIZE {
            return Err(CartridgeError::InvalidHeader("ROM too small"));
        }

        let header = InesHeader::parse(&rom_data[..HEADER_SIZE])?;

        let mut offset = HEADER_SIZE;

        // Skip trainer if present
        let trainer = if header.has_trainer() {
            let trainer_data = rom_data[offset..offset + 512].to_vec();
            offset += 512;
            Some(trainer_data)
        } else {
            None
        };

        // PRG ROM
        let prg_rom_size = header.prg_rom_size as usize * 16 * 1024;
        let prg_rom = rom_data[offset..offset + prg_rom_size].to_vec();
        offset += prg_rom_size;

        // CHR ROM
        let chr_rom_size = header.chr_rom_size as usize * 8 * 1024;
        let chr_rom = rom_data[offset..offset + chr_rom_size].to_vec();

        // Determine mapper
        let mapper = match header.mapper_number() {
            0 => Mapper::NROM,
            1 => Mapper::UXROM,
            2 => Mapper::CNROM,
            _ => Mapper::NROM, // Default to NROM for unknown mappers
        };

        Ok(Self {
            header,
            prg_rom,
            chr_rom,
            prg_ram: Some(vec![0xFF; 8192]), // Default 8KB PRG RAM
            mapper,
            trainer,
        })
    }

    /// Get the iNES header
    pub fn header(&self) -> &InesHeader {
        &self.header
    }

    /// Get PRG ROM data
    pub fn prg_rom(&self) -> &[u8] {
        &self.prg_rom
    }

    /// Get CHR ROM data
    pub fn chr_rom(&self) -> &[u8] {
        &self.chr_rom
    }

    /// Read from PRG ROM with mapper addressing
    pub fn read_prd_rom(&self, address: u16) -> u8 {
        match self.mapper {
            Mapper::NROM => {
                // NROM: Simple mapping, no bank switching
                // PRG ROM is mirrored - same content appears in both $8000-$BFFF and $C000-$FFFF
                let offset = (address - 0x8000) as usize;
                let prg_size = self.prg_rom.len();
                let mirrored_offset = offset % prg_size;
                self.prg_rom[mirrored_offset]
            }
            Mapper::UXROM => {
                // UXROM: Lower 16KB fixed, upper 16KB switchable
                let prgrom_len = self.prg_rom.len();
                if address < 0xC000 {
                    // Fixed 16KB at $8000-$BFFF
                    let offset = (address - 0x8000) as usize;
                    if offset < prgrom_len {
                        self.prg_rom[offset]
                    } else {
                        // Mirror if beyond PRG ROM size
                        self.prg_rom[offset % prgrom_len]
                    }
                } else {
                    // Switchable 16KB at $C000-$FFFF
                    let bank_size = 16 * 1024;
                    let num_banks = prgrom_len / bank_size;
                    let bank = (address - 0xC000) as usize / bank_size;
                    let offset = (address - 0xC000) as usize % bank_size;
                    if bank < num_banks {
                        self.prg_rom[bank * bank_size + offset]
                    } else {
                        // Mirror if beyond bank range
                        self.prg_rom[offset]
                    }
                }
            }
            Mapper::CNROM => {
                // CNROM: Similar to UXROM for PRG
                let offset = (address - 0x8000) as usize;
                let prg_size = self.prg_rom.len();
                let mirrored_offset = offset % prg_size;
                self.prg_rom[mirrored_offset]
            }
        }
    }

    /// Read from PRG RAM
    pub fn read_prm_ram(&self, _address: u16) -> u8 {
        if let Some(ref prg_ram) = self.prg_ram {
            if self.header.has_sram() {
                // Simple direct mapping for now
                prg_ram[0] // TODO: proper addressing
            } else {
                0xFF
            }
        } else {
            0xFF
        }
    }

    /// Write to PRG RAM
    pub fn write_prm_ram(&mut self, _address: u16, value: u8) {
        if let Some(ref mut prg_ram) = self.prg_ram {
            if self.header.has_sram() {
                // Simple direct mapping for now
                prg_ram[0] = value; // TODO: proper addressing
            }
        }
    }

    /// Get mapper type
    pub fn mapper(&self) -> Mapper {
        self.mapper
    }
}

/// Cartridge error types
#[derive(Debug, Clone, Copy)]
pub enum CartridgeError {
    InvalidHeader(&'static str),
    InvalidData(&'static str),
}

impl std::fmt::Display for CartridgeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CartridgeError::InvalidHeader(msg) => write!(f, "Invalid iNES header: {}", msg),
            CartridgeError::InvalidData(msg) => write!(f, "Invalid cartridge data: {}", msg),
        }
    }
}

impl std::error::Error for CartridgeError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_header_parsing() {
        // Create a minimal valid iNES header
        let mut header_data = [0u8; HEADER_SIZE];
        header_data[0..4].copy_from_slice(b"NES\x1A");
        header_data[4] = 1;  // 1x 16KB PRG ROM
        header_data[5] = 1;  // 1x 8KB CHR ROM

        let header = InesHeader::parse(&header_data).unwrap();
        assert_eq!(header.prg_rom_size, 1);
        assert_eq!(header.chr_rom_size, 1);
    }

    #[test]
    fn test_cartridge_from_rom() {
        // Create a minimal iNES ROM
        let mut rom = Vec::new();
        rom.extend_from_slice(b"NES\x1A");
        rom.push(1); // PRG ROM: 16KB
        rom.push(1); // CHR ROM: 8KB
        rom.push(0); // Flags 6
        rom.push(0); // Flags 7
        rom.push(0); // PRG RAM size
        rom.push(0); // Flags 9
        rom.push(0); // Flags 10
        rom.extend_from_slice(&[0u8; 5]); // Padding
        rom.extend_from_slice(&[0xFFu8; 16384]); // PRG ROM
        rom.extend_from_slice(&[0x00u8; 8192]); // CHR ROM

        let cart = Cartridge::from_rom(&rom);
        assert!(cart.is_ok());
    }
}