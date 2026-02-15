//! NES ROM loading and parsing

use std::fs::File;
use std::io::{self, Read, Write};

/// NES ROM header magic number
pub const NES_MAGIC: [u8; 4] = [0x4E, 0x45, 0x53, 0x1A];  // "NES\x1A"

/// Mirroring modes
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Mirroring {
    Horizontal = 0,
    Vertical = 1,
    FourScreen = 2,
    SingleScreenA = 3,
    SingleScreenB = 4,
}

impl Mirroring {
    pub fn from_value(value: u8) -> Self {
        match value {
            0 => Mirroring::Horizontal,
            1 => Mirroring::Vertical,
            2 => Mirroring::FourScreen,
            3 => Mirroring::SingleScreenA,
            4 => Mirroring::SingleScreenB,
            _ => Mirroring::Horizontal,
        }
    }
}

/// Trainer presence
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Trainer {
    None = 0,
    Present = 1,
}

/// Mapper types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Mapper {
    NoMapper,               // NROM
    MMC1,                   // MMC1
    UNROM,                  // UNROM
    CNROM,                  // CNROM
    MMC3,                   // MMC3
    MMC5,                   // MMC5
    AxROM,                  // AxROM
    ColorDreams,            // Color Dreams
    BNROM,                  // BNROM
    MMC3Variant,            // MMC3 variant
    GxROM,                  // GxROM
    UN1ROM,                 // UN1ROM
    NINA06,                 // NINA-06
    MMC3Variant2,           // MMC3 variant
    UNROMVariant,           // UNROM variant
    UNROMVariant2,          // UNROM variant
    Suborisk,               // Suborisk
    FK23C,                  // FK23C
    Other(u8),              // Unknown mapper
}

impl Mapper {
    pub fn from_value(value: u8) -> Self {
        match value {
            0 => Mapper::NoMapper,
            1 => Mapper::MMC1,
            2 => Mapper::UNROM,
            3 => Mapper::CNROM,
            4 => Mapper::MMC3,
            5 => Mapper::MMC5,
            7 => Mapper::AxROM,
            11 => Mapper::ColorDreams,
            34 => Mapper::BNROM,
            38 => Mapper::MMC3Variant,
            66 => Mapper::GxROM,
            94 => Mapper::UN1ROM,
            140 => Mapper::NINA06,
            180 => Mapper::MMC3Variant2,
            240 => Mapper::UNROMVariant,
            241 => Mapper::UNROMVariant2,
            242 => Mapper::Suborisk,
            243 => Mapper::FK23C,
            x => Mapper::Other(x),
        }
    }

    pub fn to_value(&self) -> u8 {
        match self {
            Mapper::NoMapper => 0,
            Mapper::MMC1 => 1,
            Mapper::UNROM => 2,
            Mapper::CNROM => 3,
            Mapper::MMC3 => 4,
            Mapper::MMC5 => 5,
            Mapper::AxROM => 7,
            Mapper::ColorDreams => 11,
            Mapper::BNROM => 34,
            Mapper::MMC3Variant => 38,
            Mapper::GxROM => 66,
            Mapper::UN1ROM => 94,
            Mapper::NINA06 => 140,
            Mapper::MMC3Variant2 => 180,
            Mapper::UNROMVariant => 240,
            Mapper::UNROMVariant2 => 241,
            Mapper::Suborisk => 242,
            Mapper::FK23C => 243,
            Mapper::Other(x) => *x,
        }
    }
}

/// NES ROM header
#[derive(Debug, Clone)]
pub struct RomHeader {
    pub prg_rom_size: usize,      // in 16KB units
    pub chr_rom_size: usize,      // in 8KB units
    pub mapper: Mapper,
    pub mirroring: Mirroring,
    pub has_battery_ram: bool,
    pub has_trainer: bool,
    pub four_screen: bool,
}

impl RomHeader {
    pub fn parse(data: &[u8]) -> Result<Self, io::Error> {
        if data.len() < 16 {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "ROM too small"));
        }

        // Check magic number
        if &data[0..4] != NES_MAGIC.as_slice() {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid NES header"));
        }

        let prg_rom_size = data[4] as usize;
        let chr_rom_size = data[5] as usize;
        let flags6 = data[6];
        let flags7 = data[7];

        // Parse flags
        let mirroring = Mirroring::from_value(if (flags6 & 0x01) != 0 { 1 } else { 0 });
        let has_battery_ram = (flags6 & 0x02) != 0;
        let has_trainer = (flags6 & 0x04) != 0;
        let four_screen = (flags6 & 0x08) != 0;

        // Mapper is split across flags 6 and 7
        let mapper_high = (flags6 >> 4) & 0x0F;
        let mapper_low = (flags7 >> 4) & 0x0F;
        let mapper_value = (mapper_high | mapper_low) as u8;
        let mapper = Mapper::from_value(mapper_value);

        Ok(Self {
            prg_rom_size,
            chr_rom_size,
            mapper,
            mirroring,
            has_battery_ram,
            has_trainer,
            four_screen,
        })
    }
}

/// NES ROM data
#[derive(Debug, Clone)]
pub struct Rom {
    pub header: RomHeader,
    pub prg_rom: Vec<u8>,
    pub chr_rom: Vec<u8>,
    pub trainer: Option<Vec<u8>>,
    pub battery_ram: Option<Vec<u8>>,
}

impl Rom {
    /// Load ROM from file
    pub fn load_from_file(path: &str) -> Result<Self, io::Error> {
        let mut file = File::open(path)?;
        let mut data = Vec::new();
        file.read_to_end(&mut data)?;

        Self::load_from_data(&data)
    }

    /// Load ROM from data bytes
    pub fn load_from_data(data: &[u8]) -> Result<Self, io::Error> {
        let header = RomHeader::parse(data)?;

        let mut offset = 16;

        // Skip trainer if present
        let trainer = if header.has_trainer {
            let trainer_data = data[offset..offset + 512].to_vec();
            offset += 512;
            Some(trainer_data)
        } else {
            None
        };

        // Load PRG-ROM
        let prg_size = header.prg_rom_size * 16384;
        let prg_rom = data[offset..offset + prg_size].to_vec();
        offset += prg_size;

        // Load CHR-ROM
        let chr_size = header.chr_rom_size * 8192;
        let chr_rom = data[offset..offset + chr_size].to_vec();

        Ok(Self {
            header,
            prg_rom,
            chr_rom,
            trainer,
            battery_ram: None,
        })
    }

    /// Save battery RAM to file
    pub fn save_battery_ram(&self, path: &str) -> Result<(), io::Error> {
        if let Some(bat_ram) = &self.battery_ram {
            let mut file = File::create(path)?;
            file.write_all(bat_ram)?;
        }
        Ok(())
    }

    /// Load battery RAM from file
    pub fn load_battery_ram(&mut self, path: &str) -> Result<(), io::Error> {
        let mut file = File::open(path)?;
        let mut data = Vec::new();
        file.read_to_end(&mut data)?;
        self.battery_ram = Some(data);
        Ok(())
    }
}

/// Mapper factory
pub fn create_mapper(mapper: Mapper) -> Box<dyn MapperInterface> {
    match mapper {
        Mapper::NoMapper => Box::new(NoMapper::new()),
        Mapper::MMC1 => Box::new(MMC1::new()),
        Mapper::UNROM => Box::new(UNROM::new()),
        Mapper::CNROM => Box::new(CNROM::new()),
        Mapper::MMC3 => Box::new(MMC3::new()),
        _ => Box::new(NoMapper::new()),
    }
}

/// Mapper trait
pub trait MapperInterface {
    fn reset(&mut self);
    fn read_low(&mut self, address: u16) -> u8;
    fn write_low(&mut self, address: u16, value: u8);
    fn load_rom(&mut self, rom: &Rom);
    fn read_prg(&mut self, address: u16) -> u8;
    fn write_prg(&mut self, address: u16, value: u8);
    fn read_chr(&mut self, address: u16) -> u8;
    fn write_chr(&mut self, address: u16, value: u8);
}

/// NoMapper - simplest mapper
#[derive(Debug)]
pub struct NoMapper {
    prg_banks: Vec<u8>,
    chr_banks: Vec<u8>,
}

impl NoMapper {
    pub fn new() -> Self {
        Self {
            prg_banks: Vec::new(),
            chr_banks: Vec::new(),
        }
    }
}

impl Default for NoMapper {
    fn default() -> Self {
        Self::new()
    }
}

impl MapperInterface for NoMapper {
    fn reset(&mut self) {}

    fn read_low(&mut self, _address: u16) -> u8 {
        0
    }

    fn write_low(&mut self, _address: u16, _value: u8) {}

    fn load_rom(&mut self, rom: &Rom) {
        self.prg_banks = rom.prg_rom.clone();
        self.chr_banks = rom.chr_rom.clone();
    }

    fn read_prg(&mut self, address: u16) -> u8 {
        let addr = address as usize;

        // Map PRG-ROM to $8000-$FFFF
        if addr >= 0x8000 {
            let offset = addr - 0x8000;
            if offset < self.prg_banks.len() {
                return self.prg_banks[offset];
            }
            // Mirror the last bank for addresses beyond PRG-ROM size
            let bank_size = self.prg_banks.len();
            let mirrored_offset = offset % bank_size;
            if mirrored_offset < self.prg_banks.len() {
                return self.prg_banks[mirrored_offset];
            }
        }

        0
    }

    fn write_prg(&mut self, _address: u16, _value: u8) {}

    fn read_chr(&mut self, address: u16) -> u8 {
        let addr = address as usize;
        if addr < self.chr_banks.len() {
            return self.chr_banks[addr];
        }
        0
    }

    fn write_chr(&mut self, _address: u16, _value: u8) {}
}

/// MMC1 Mapper
#[derive(Debug)]
pub struct MMC1 {
    shift_register: u8,
    control: u8,
    chr_bank0: u8,
    chr_bank1: u8,
    prg_bank: u8,
}

impl MMC1 {
    pub fn new() -> Self {
        Self {
            shift_register: 0x80,  // Reset state
            control: 0,
            chr_bank0: 0,
            chr_bank1: 0,
            prg_bank: 0,
        }
    }
}

impl Default for MMC1 {
    fn default() -> Self {
        Self::new()
    }
}

impl MapperInterface for MMC1 {
    fn reset(&mut self) {
        self.shift_register = 0x80;
    }

    fn read_low(&mut self, _address: u16) -> u8 {
        0
    }

    fn write_low(&mut self, address: u16, value: u8) {
        // MMC1 uses $8000-$FFFF for all register writes
        if address < 0x8000 {
            return;
        }

        // Write to shift register
        let write_enable = (value & 0x80) == 0;
        self.shift_register = (self.shift_register >> 1) | (value & 0x80);

        if write_enable && (self.shift_register & 0x10) != 0 {
            // Register write complete
            let reg = (address >> 13) & 0x03;
            self.shift_register = 0x80;

            match reg {
                0 => self.control = self.shift_register,
                1 => self.chr_bank0 = self.shift_register,
                2 => self.chr_bank1 = self.shift_register,
                3 => self.prg_bank = self.shift_register,
                _ => {}
            }
        }
    }

    fn load_rom(&mut self, _rom: &Rom) {}

    fn read_prg(&mut self, _address: u16) -> u8 {
        0
    }

    fn write_prg(&mut self, _address: u16, _value: u8) {}

    fn read_chr(&mut self, _address: u16) -> u8 {
        0
    }

    fn write_chr(&mut self, _address: u16, _value: u8) {}
}

/// UNROM Mapper
#[derive(Debug)]
pub struct UNROM {
    prg_banks: Vec<u8>,
    chr_banks: Vec<u8>,
    current_prg_bank: usize,
}

impl UNROM {
    pub fn new() -> Self {
        Self {
            prg_banks: Vec::new(),
            chr_banks: Vec::new(),
            current_prg_bank: 0,
        }
    }
}

impl Default for UNROM {
    fn default() -> Self {
        Self::new()
    }
}

impl MapperInterface for UNROM {
    fn reset(&mut self) {}

    fn read_low(&mut self, _address: u16) -> u8 {
        0
    }

    fn write_low(&mut self, address: u16, value: u8) {
        if address >= 0x8000 && address < 0xA000 {
            self.current_prg_bank = (value as usize) & 0x7F;
        }
    }

    fn load_rom(&mut self, rom: &Rom) {
        self.prg_banks = rom.prg_rom.clone();
        self.chr_banks = rom.chr_rom.clone();
    }

    fn read_prg(&mut self, address: u16) -> u8 {
        let addr = address as usize;
        let bank_size = 0x8000;

        if addr < bank_size {
            // Fixed bank at $8000
            let offset = addr;
            if offset < self.prg_banks.len() {
                return self.prg_banks[offset];
            }
        } else if addr >= 0x8000 {
            // Switchable bank at $C000
            let offset = (addr - 0x8000) + (self.current_prg_bank * bank_size);
            if offset < self.prg_banks.len() {
                return self.prg_banks[offset];
            }
        }

        0
    }

    fn write_prg(&mut self, _address: u16, _value: u8) {}

    fn read_chr(&mut self, address: u16) -> u8 {
        let addr = address as usize;
        if addr < self.chr_banks.len() {
            return self.chr_banks[addr];
        }
        0
    }

    fn write_chr(&mut self, _address: u16, _value: u8) {}
}

/// CNROM Mapper
#[derive(Debug)]
pub struct CNROM {
    prg_banks: Vec<u8>,
    chr_banks: Vec<u8>,
    current_chr_bank: usize,
}

impl CNROM {
    pub fn new() -> Self {
        Self {
            prg_banks: Vec::new(),
            chr_banks: Vec::new(),
            current_chr_bank: 0,
        }
    }
}

impl Default for CNROM {
    fn default() -> Self {
        Self::new()
    }
}

impl MapperInterface for CNROM {
    fn reset(&mut self) {}

    fn read_low(&mut self, _address: u16) -> u8 {
        0
    }

    fn write_low(&mut self, address: u16, value: u8) {
        if address >= 0x8000 {
            self.current_chr_bank = (value as usize) & 0x03;
        }
    }

    fn load_rom(&mut self, rom: &Rom) {
        self.prg_banks = rom.prg_rom.clone();
        self.chr_banks = rom.chr_rom.clone();
    }

    fn read_prg(&mut self, address: u16) -> u8 {
        let addr = address as usize;

        if addr >= 0x8000 {
            let offset = addr - 0x8000;
            if offset < self.prg_banks.len() {
                return self.prg_banks[offset];
            }
        }

        0
    }

    fn write_prg(&mut self, _address: u16, _value: u8) {}

    fn read_chr(&mut self, address: u16) -> u8 {
        let addr = address as usize;
        let bank_size = 0x2000;

        let offset = (addr % bank_size) + (self.current_chr_bank * bank_size);
        if offset < self.chr_banks.len() {
            return self.chr_banks[offset];
        }
        0
    }

    fn write_chr(&mut self, _address: u16, _value: u8) {}
}

/// MMC3 Mapper
#[derive(Debug)]
pub struct MMC3 {
    prg_banks: Vec<u8>,
    chr_banks: Vec<u8>,
    command: u8,
    prg_banks_select: [u8; 2],
    chr_banks_select: [u8; 6],
}

impl MMC3 {
    pub fn new() -> Self {
        Self {
            prg_banks: Vec::new(),
            chr_banks: Vec::new(),
            command: 0,
            prg_banks_select: [0, 1],
            chr_banks_select: [0, 1, 2, 3, 4, 5],
        }
    }
}

impl Default for MMC3 {
    fn default() -> Self {
        Self::new()
    }
}

impl MapperInterface for MMC3 {
    fn reset(&mut self) {
        self.command = 0;
    }

    fn read_low(&mut self, _address: u16) -> u8 {
        0
    }

    fn write_low(&mut self, address: u16, value: u8) {
        match address {
            0x8000 => {
                self.command = value;
            }
            0x8001 => {
                // Write to selected register based on command
                match self.command & 0x07 {
                    0 => {
                        // Two 1KB VROM banks at $0000+$0400
                        self.chr_banks_select[0] = value & 0xFE;
                        self.chr_banks_select[1] = (value & 0xFE) | 1;
                    }
                    1 => {
                        // Two 1KB VROM banks at $0800+$0C00
                        self.chr_banks_select[2] = value & 0xFE;
                        self.chr_banks_select[3] = (value & 0xFE) | 1;
                    }
                    2 => {
                        // One 1KB VROM at $1000
                        self.chr_banks_select[4] = value;
                    }
                    3 => {
                        // One 1KB VROM at $1400
                        self.chr_banks_select[5] = value;
                    }
                    4 => {
                        // One 1KB VROM at $1800
                        self.chr_banks_select[4] = value;
                    }
                    5 => {
                        // One 1KB VROM at $1C00
                        self.chr_banks_select[5] = value;
                    }
                    6 => {
                        // PRG page 1 selection
                        self.prg_banks_select[1] = value & 0x07;
                    }
                    7 => {
                        // PRG page 2 selection
                        self.prg_banks_select[0] = value & 0x07;
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    fn load_rom(&mut self, rom: &Rom) {
        self.prg_banks = rom.prg_rom.clone();
        self.chr_banks = rom.chr_rom.clone();
    }

    fn read_prg(&mut self, address: u16) -> u8 {
        let addr = address as usize;

        if addr < 0x8000 {
            return 0;
        }

        let bank_size = 0x2000;
        let offset = if addr < 0xC000 {
            // Page 2 at $8000
            (addr - 0x8000) + (self.prg_banks_select[0] as usize * bank_size)
        } else {
            // Page 1 at $C000
            (addr - 0xC000) + (self.prg_banks_select[1] as usize * bank_size)
        };

        if offset < self.prg_banks.len() {
            return self.prg_banks[offset];
        }
        0
    }

    fn write_prg(&mut self, _address: u16, _value: u8) {}

    fn read_chr(&mut self, address: u16) -> u8 {
        let addr = address as usize;
        let bank_size = 0x0400;

        let offset = if addr < 0x1000 {
            // Pattern table 0
            (addr % bank_size) + (self.chr_banks_select[0] as usize * bank_size)
        } else {
            // Pattern table 1
            (addr % bank_size) + (self.chr_banks_select[4] as usize * bank_size)
        };

        if offset < self.chr_banks.len() {
            return self.chr_banks[offset];
        }
        0
    }

    fn write_chr(&mut self, _address: u16, _value: u8) {}
}