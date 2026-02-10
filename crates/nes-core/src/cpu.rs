//! CPU module - 2A03 (6502 variant) implementation
//!
//! The NES uses a modified 6502 CPU without decimal mode.

use std::fmt;

/// 2A03 CPU registers
#[derive(Debug, Clone, Copy)]
pub struct CpuRegisters {
    pub a: u8,    // Accumulator
    pub x: u8,    // X index register
    pub y: u8,    // Y index register
    pub p: u8,    // Processor status
    pub sp: u8,   // Stack pointer
    pub pc: u16,  // Program counter
}

impl Default for CpuRegisters {
    fn default() -> Self {
        Self {
            a: 0,
            x: 0,
            y: 0,
            p: 0x24,  // B flag set, interrupt disable set, unknown flag set
            sp: 0xFD, // Stack starts at $01FD
            pc: 0,    // Will be set by reset vector
        }
    }
}

/// CPU status flags
#[derive(Debug, Clone, Copy)]
pub struct StatusFlags(u8);

impl StatusFlags {
    pub const CARRY: u8 = 0b00000001;
    pub const ZERO: u8 = 0b00000010;
    pub const INTERRUPT: u8 = 0b00000100;
    pub const DECIMAL: u8 = 0b00001000;
    pub const BREAK: u8 = 0b00010000;
    pub const UNUSED: u8 = 0b00100000;
    pub const OVERFLOW: u8 = 0b01000000;
    pub const NEGATIVE: u8 = 0b10000000;

    pub fn new(flags: u8) -> Self {
        Self(flags)
    }

    pub fn carry(&self) -> bool {
        (self.0 & Self::CARRY) != 0
    }

    pub fn zero(&self) -> bool {
        (self.0 & Self::ZERO) != 0
    }

    pub fn interrupt(&self) -> bool {
        (self.0 & Self::INTERRUPT) != 0
    }

    pub fn decimal(&self) -> bool {
        (self.0 & Self::DECIMAL) != 0
    }

    pub fn overflow(&self) -> bool {
        (self.0 & Self::OVERFLOW) != 0
    }

    pub fn negative(&self) -> bool {
        (self.0 & Self::NEGATIVE) != 0
    }

    pub fn set_carry(&mut self, val: bool) {
        self.0 = if val { self.0 | Self::CARRY } else { self.0 & !Self::CARRY };
    }

    pub fn set_zero(&mut self, val: bool) {
        self.0 = if val { self.0 | Self::ZERO } else { self.0 & !Self::ZERO };
    }

    pub fn set_interrupt(&mut self, val: bool) {
        self.0 = if val { self.0 | Self::INTERRUPT } else { self.0 & !Self::INTERRUPT };
    }

    pub fn set_overflow(&mut self, val: bool) {
        self.0 = if val { self.0 | Self::OVERFLOW } else { self.0 & !Self::OVERFLOW };
    }

    pub fn set_negative(&mut self, val: bool) {
        self.0 = if val { self.0 | Self::NEGATIVE } else { self.0 & !Self::NEGATIVE };
    }
}

impl fmt::Display for StatusFlags {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "C:{} Z:{} I:{} D:{} B:{} U:{} V:{} N:{}",
            self.carry() as u8,
            self.zero() as u8,
            self.interrupt() as u8,
            self.decimal() as u8,
            false,  // B is set internally
            1,      // U is always 1
            self.overflow() as u8,
            self.negative() as u8
        )
    }
}

/// CPU instruction opcode
#[derive(Debug, Clone, Copy)]
pub enum Opcode {
    // ADC - Add with Carry
    ADCImmediate, ADCZeroPage, ADCZeroPageX, ADCAbsolute, ADCAbsoluteX, ADCAbsoluteY, ADCIndirectX, ADCIndirectY,

    // AND - Logical AND
    ANDImmediate, ANDZeroPage, ANDZeroPageX, ANDAbsolute, ANDAbsoluteX, ANDAbsoluteY, ANDIndirectX, ANDIndirectY,

    // ASL - Arithmetic Shift Left
    ASLAccumulator, ASLZeroPage, ASLZeroPageX, ASLAbsolute, ASLAbsoluteX,

    // BCC - Branch if Carry Clear
    BCCRelative,

    // BCS - Branch if Carry Set
    BCSRelative,

    // BEQ - Branch if Equal
    BEQRelative,

    // BIT - Test Bits
    BITZeroPage, BITAbsolute,

    // BMI - Branch if Minus
    BMIRelative,

    // BNE - Branch if Not Equal
    BNERelative,

    // BPL - Branch if Positive
    BPLRelative,

    // BRK - Break
    BRKImplied,

    // BVC - Branch if Overflow Clear
    BVCRelative,

    // BVS - Branch if Overflow Set
    BVSRelative,

    // CLC - Clear Carry
    CLCImplied,

    // CLD - Clear Decimal
    CLDImplied,

    // CLI - Clear Interrupt
    CLIImplied,

    // CLV - Clear Overflow
    CLVImplied,

    // CMP - Compare
    CMPImmediate, CMPZeroPage, CMPZeroPageX, CmpAbsolute, CmpAbsoluteX, CmpAbsoluteY, CMPIndirectX, CMPIndirectY,

    // CPX - Compare X Register
    CPXImmediate, CPXZeroPage, CPXAbsolute,

    // CPY - Compare Y Register
    CPYImmediate, CPYZeroPage, CPYAbsolute,

    // DEC - Decrement
    DECZeroPage, DECZeroPageX, DECAbsolute, DECAbsoluteX,

    // DEX - Decrement X
    DEXImplied,

    // DEY - Decrement Y
    DEYImplied,

    // EOR - Exclusive OR
    EORImmediate, EORZeroPage, EORZeroPageX, EORAbsolute, EORAbsoluteX, EORAbsoluteY, EORIndirectX, EORIndirectY,

    // INC - Increment
    INCZeroPage, INCZeroPageX, INCAbsolute, INCAbsoluteX,

    // INX - Increment X
    INXImplied,

    // INY - Increment Y
    INYImplied,

    // JMP - Jump
    JMPAbsolute, JMPIndirect,

    // JSR - Jump to Subroutine
    JSRAbsolute,

    // LDA - Load Accumulator
    LDAImmediate, LDAZeroPage, LDAZeroPageX, LDAAbsolute, LDAAbsoluteX, LDAAbsoluteY, LDAIndirectX, LDAIndirectY,

    // LDX - Load X Register
    LDXImmediate, LDXZeroPage, LDXZeroPageY, LDXAbsolute, LDXAbsoluteY,

    // LDY - Load Y Register
    LDYImmediate, LDYZeroPage, LDYZeroPageX, LDYAbsolute, LDYAbsoluteX,

    // LSR - Logical Shift Right
    LSRAccumulator, LSRZeroPage, LSRZeroPageX, LSRAbsolute, LSRAbsoluteX,

    // NOP - No Operation
    NOPImplied,

    // ORA - Logical OR
    ORAImmediate, ORAZeroPage, ORAZeroPageX, ORAAbsolute, ORAAbsoluteX, ORAAbsoluteY, ORAIndirectX, ORAIndirectY,

    // PHA - Push Accumulator
    PHAImplied,

    // PHP - Push Processor Status
    PHPImplied,

    // PLA - Pull Accumulator
    PLAImplied,

    // PLP - Pull Processor Status
    PLPImplied,

    // ROL - Rotate Left
    ROLAccumulator, ROLZeroPage, ROLZeroPageX, ROLAbsolute, ROLAbsoluteX,

    // ROR - Rotate Right
    RORAccumulator, RORZeroPage, RORZeroPageX, RORAbsolute, RORAbsoluteX,

    // RTI - Return from Interrupt
    RTIImplied,

    // RTS - Return from Subroutine
    RTSImplied,

    // SBC - Subtract with Carry
    SBCImmediate, SBCZeroPage, SBCZeroPageX, SBCAbsolute, SBCAbsoluteX, SBCAbsoluteY, SBCIndirectX, SBCIndirectY,

    // SEC - Set Carry
    SECImplied,

    // SEI - Set Interrupt
    SEIImplied,

    // STX - Store X Register
    STXZeroPage, STXZeroPageY, STXAbsolute,

    // STY - Store Y Register
    STYZeroPage, STYZeroPageX, STYAbsolute,

    // STA - Store Accumulator
    STAZeroPage, STAZeroPageX, STAAbsolute, STAAbsoluteX, STAAbsoluteY, STAIndirectX, STAIndirectY,

    // TAX - Transfer A to X
    TAXImplied,

    // TAY - Transfer A to Y
    TAYImplied,

    // TSX - Transfer S to X
    TSXImplied,

    // TXA - Transfer X to A
    TXAImplied,

    // TXS - Transfer X to S
    TXSImplied,

    // TYA - Transfer Y to A
    TYAImplied,
}

/// Addressing mode
#[derive(Debug, Clone, Copy)]
pub enum AddressingMode {
    Implied,
    Immediate,
    ZeroPage,
    ZeroPageX,
    ZeroPageY,
    Absolute,
    AbsoluteX,
    AbsoluteY,
    IndirectX,
    IndirectY,
    Relative,
    Accumulator,
}

/// CPU instruction info
#[derive(Debug, Clone, Copy)]
pub struct InstructionInfo {
    pub opcode: Opcode,
    pub mode: AddressingMode,
    pub cycles: u8,
    pub page_cycle: bool, // Extra cycle if page crossed
}

/// CPU emulator state
#[derive(Debug, Clone)]
pub struct Cpu {
    registers: CpuRegisters,
    status: StatusFlags,
    /// Remaining cycles for current instruction
    remaining_cycles: u8,
    /// Total cycles executed
    total_cycles: u64,
}

impl Cpu {
    /// Create a new CPU instance
    pub fn new() -> Self {
        Self {
            registers: CpuRegisters::default(),
            status: StatusFlags::new(0x24),
            remaining_cycles: 0,
            total_cycles: 0,
        }
    }

    /// Reset the CPU to its initial state
    pub fn reset(&mut self) {
        self.registers = CpuRegisters::default();
        self.status = StatusFlags::new(0x24);
        self.remaining_cycles = 0;
        self.total_cycles = 0;
        // Set PC from reset vector
        self.registers.pc = 0xFFFC;
    }

    /// Get CPU registers
    pub fn registers(&self) -> &CpuRegisters {
        &self.registers
    }

    /// Get CPU status flags
    pub fn status(&self) -> &StatusFlags {
        &self.status
    }

    /// Get total cycles executed
    pub fn total_cycles(&self) -> u64 {
        self.total_cycles
    }

    /// Read a byte from memory (abstract - to be implemented by bus)
    pub fn read_memory(&self, _address: u16) -> u8 {
        0
    }

    /// Write a byte to memory (abstract - to be implemented by bus)
    pub fn write_memory(&mut self, _address: u16, _value: u8) {}

    /// Step one instruction
    pub fn step(&mut self, _bus: &mut impl Bus) -> Result<bool, CpuError> {
        // Fetch opcode
        let opcode_byte = self.read_memory(self.registers.pc);
        let opcode = self.decode_opcode(opcode_byte)?;

        // TODO: Implement instruction execution
        Ok(true)
    }

    /// Decode an opcode to its instruction info
    fn decode_opcode(&self, opcode: u8) -> Result<Opcode, CpuError> {
        // 6502 opcode table
        match opcode {
            0x69 => Ok(Opcode::ADCImmediate),
            0x65 => Ok(Opcode::ADCZeroPage),
            0x75 => Ok(Opcode::ADCZeroPageX),
            0x6D => Ok(Opcode::ADCAbsolute),
            0x7D => Ok(Opcode::ADCAbsoluteX),
            0x79 => Ok(Opcode::ADCAbsoluteY),
            0x61 => Ok(Opcode::ADCIndirectX),
            0x71 => Ok(Opcode::ADCIndirectY),
            0x29 => Ok(Opcode::ANDImmediate),
            0x25 => Ok(Opcode::ANDZeroPage),
            0x35 => Ok(Opcode::ANDZeroPageX),
            0x2D => Ok(Opcode::ANDAbsolute),
            0x3D => Ok(Opcode::ANDAbsoluteX),
            0x39 => Ok(Opcode::ANDAbsoluteY),
            0x21 => Ok(Opcode::ANDIndirectX),
            0x31 => Ok(Opcode::ANDIndirectY),
            0x0A => Ok(Opcode::ASLAccumulator),
            0x06 => Ok(Opcode::ASLZeroPage),
            0x16 => Ok(Opcode::ASLZeroPageX),
            0x0E => Ok(Opcode::ASLAbsolute),
            0x1E => Ok(Opcode::ASLAbsoluteX),
            0x90 => Ok(Opcode::BCCRelative),
            0xB0 => Ok(Opcode::BCSRelative),
            0xF0 => Ok(Opcode::BEQRelative),
            0x24 => Ok(Opcode::BITZeroPage),
            0x2C => Ok(Opcode::BITAbsolute),
            0x30 => Ok(Opcode::BMIRelative),
            0xD0 => Ok(Opcode::BNERelative),
            0x10 => Ok(Opcode::BPLRelative),
            0x00 => Ok(Opcode::BRKImplied),
            0x50 => Ok(Opcode::BVCRelative),
            0x70 => Ok(Opcode::BVSRelative),
            0x18 => Ok(Opcode::CLCImplied),
            0xD8 => Ok(Opcode::CLDImplied),
            0x58 => Ok(Opcode::CLIImplied),
            0xB8 => Ok(Opcode::CLVImplied),
            0xC9 => Ok(Opcode::CMPImmediate),
            0xC5 => Ok(Opcode::CMPZeroPage),
            0xD5 => Ok(Opcode::CMPZeroPageX),
            0xCD => Ok(Opcode::CmpAbsolute),
            0xDD => Ok(Opcode::CmpAbsoluteX),
            0xD9 => Ok(Opcode::CmpAbsoluteY),
            0xC1 => Ok(Opcode::CMPIndirectX),
            0xD1 => Ok(Opcode::CMPIndirectY),
            0xE0 => Ok(Opcode::CPXImmediate),
            0xE4 => Ok(Opcode::CPXZeroPage),
            0xEC => Ok(Opcode::CPXAbsolute),
            0xC0 => Ok(Opcode::CPYImmediate),
            0xC4 => Ok(Opcode::CPYZeroPage),
            0xCC => Ok(Opcode::CPYAbsolute),
            0xC6 => Ok(Opcode::DECZeroPage),
            0xD6 => Ok(Opcode::DECZeroPageX),
            0xCE => Ok(Opcode::DECAbsolute),
            0xDE => Ok(Opcode::DECAbsoluteX),
            0xCA => Ok(Opcode::DEXImplied),
            0x88 => Ok(Opcode::DEYImplied),
            0x49 => Ok(Opcode::EORImmediate),
            0x45 => Ok(Opcode::EORZeroPage),
            0x55 => Ok(Opcode::EORZeroPageX),
            0x4D => Ok(Opcode::EORAbsolute),
            0x5D => Ok(Opcode::EORAbsoluteX),
            0x59 => Ok(Opcode::EORAbsoluteY),
            0x41 => Ok(Opcode::EORIndirectX),
            0x51 => Ok(Opcode::EORIndirectY),
            0xE6 => Ok(Opcode::INCZeroPage),
            0xF6 => Ok(Opcode::INCZeroPageX),
            0xEE => Ok(Opcode::INCAbsolute),
            0xFE => Ok(Opcode::INCAbsoluteX),
            0xE8 => Ok(Opcode::INXImplied),
            0xC8 => Ok(Opcode::INYImplied),
            0x4C => Ok(Opcode::JMPAbsolute),
            0x6C => Ok(Opcode::JMPIndirect),
            0x20 => Ok(Opcode::JSRAbsolute),
            0xA9 => Ok(Opcode::LDAImmediate),
            0xA5 => Ok(Opcode::LDAZeroPage),
            0xB5 => Ok(Opcode::LDAZeroPageX),
            0xAD => Ok(Opcode::LDAAbsolute),
            0xBD => Ok(Opcode::LDAAbsoluteX),
            0xB9 => Ok(Opcode::LDAAbsoluteY),
            0xA1 => Ok(Opcode::LDAIndirectX),
            0xB1 => Ok(Opcode::LDAIndirectY),
            0xA2 => Ok(Opcode::LDXImmediate),
            0xA6 => Ok(Opcode::LDXZeroPage),
            0xB6 => Ok(Opcode::LDXZeroPageY),
            0xAE => Ok(Opcode::LDXAbsolute),
            0xBE => Ok(Opcode::LDXAbsoluteY),
            0xA0 => Ok(Opcode::LDYImmediate),
            0xA4 => Ok(Opcode::LDYZeroPage),
            0xB4 => Ok(Opcode::LDYZeroPageX),
            0xAC => Ok(Opcode::LDYAbsolute),
            0xBC => Ok(Opcode::LDYAbsoluteX),
            0x4A => Ok(Opcode::LSRAccumulator),
            0x46 => Ok(Opcode::LSRZeroPage),
            0x56 => Ok(Opcode::LSRZeroPageX),
            0x4E => Ok(Opcode::LSRAbsolute),
            0x5E => Ok(Opcode::LSRAbsoluteX),
            0xEA => Ok(Opcode::NOPImplied),
            0x09 => Ok(Opcode::ORAImmediate),
            0x05 => Ok(Opcode::ORAZeroPage),
            0x15 => Ok(Opcode::ORAZeroPageX),
            0x0D => Ok(Opcode::ORAAbsolute),
            0x1D => Ok(Opcode::ORAAbsoluteX),
            0x19 => Ok(Opcode::ORAAbsoluteY),
            0x01 => Ok(Opcode::ORAIndirectX),
            0x11 => Ok(Opcode::ORAIndirectY),
            0x48 => Ok(Opcode::PHAImplied),
            0x08 => Ok(Opcode::PHPImplied),
            0x68 => Ok(Opcode::PLAImplied),
            0x28 => Ok(Opcode::PLPImplied),
            0x2A => Ok(Opcode::ROLAccumulator),
            0x26 => Ok(Opcode::ROLZeroPage),
            0x36 => Ok(Opcode::ROLZeroPageX),
            0x2E => Ok(Opcode::ROLAbsolute),
            0x3E => Ok(Opcode::ROLAbsoluteX),
            0x6A => Ok(Opcode::RORAccumulator),
            0x66 => Ok(Opcode::RORZeroPage),
            0x76 => Ok(Opcode::RORZeroPageX),
            0x6E => Ok(Opcode::RORAbsolute),
            0x7E => Ok(Opcode::RORAbsoluteX),
            0x40 => Ok(Opcode::RTIImplied),
            0x60 => Ok(Opcode::RTSImplied),
            0xE9 => Ok(Opcode::SBCImmediate),
            0xE5 => Ok(Opcode::SBCZeroPage),
            0xF5 => Ok(Opcode::SBCZeroPageX),
            0xED => Ok(Opcode::SBCAbsolute),
            0xFD => Ok(Opcode::SBCAbsoluteX),
            0xF9 => Ok(Opcode::SBCAbsoluteY),
            0xE1 => Ok(Opcode::SBCIndirectX),
            0xF1 => Ok(Opcode::SBCIndirectY),
            0x38 => Ok(Opcode::SECImplied),
            0x78 => Ok(Opcode::SEIImplied),
            0x86 => Ok(Opcode::STXZeroPage),
            0x96 => Ok(Opcode::STXZeroPageY),
            0x8E => Ok(Opcode::STXAbsolute),
            0x84 => Ok(Opcode::STYZeroPage),
            0x94 => Ok(Opcode::STYZeroPageX),
            0x8C => Ok(Opcode::STYAbsolute),
            0x85 => Ok(Opcode::STAZeroPage),
            0x95 => Ok(Opcode::STAZeroPageX),
            0x8D => Ok(Opcode::STAAbsolute),
            0x9D => Ok(Opcode::STAAbsoluteX),
            0x99 => Ok(Opcode::STAAbsoluteY),
            0x81 => Ok(Opcode::STAIndirectX),
            0x91 => Ok(Opcode::STAIndirectY),
            0xAA => Ok(Opcode::TAXImplied),
            0xA8 => Ok(Opcode::TAYImplied),
            0xBA => Ok(Opcode::TSXImplied),
            0x8A => Ok(Opcode::TXAImplied),
            0x9A => Ok(Opcode::TXSImplied),
            0x98 => Ok(Opcode::TYAImplied),
            _ => Err(CpuError::InvalidOpcode(opcode)),
        }
    }
}

impl Default for Cpu {
    fn default() -> Self {
        Self::new()
    }
}

/// CPU error types
#[derive(Debug, Clone, Copy)]
pub enum CpuError {
    InvalidOpcode(u8),
    StackOverflow,
    StackUnderflow,
}

impl fmt::Display for CpuError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CpuError::InvalidOpcode(op) => write!(f, "Invalid opcode: 0x{:02X}", op),
            CpuError::StackOverflow => write!(f, "Stack overflow"),
            CpuError::StackUnderflow => write!(f, "Stack underflow"),
        }
    }
}

/// Bus trait for memory and I/O access
pub trait Bus {
    /// Read a byte from the given address
    fn read(&mut self, address: u16) -> u8;
    /// Write a byte to the given address
    fn write(&mut self, address: u16, value: u8);
}

impl std::error::Error for CpuError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cpu_reset() {
        let mut cpu = Cpu::new();
        cpu.reset();
        // After reset, PC should point to reset vector
        assert_eq!(cpu.registers.pc, 0xFFFC);
    }

    #[test]
    fn test_status_flags() {
        let mut flags = StatusFlags::new(0xFF);
        assert!(flags.carry());
        assert!(flags.zero());
        assert!(flags.interrupt());
        assert!(flags.overflow());
        assert!(flags.negative());

        flags.set_carry(false);
        assert!(!flags.carry());
    }
}