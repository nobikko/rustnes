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

    pub fn set_decimal(&mut self, val: bool) {
        self.0 = if val { self.0 | Self::DECIMAL } else { self.0 & !Self::DECIMAL };
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
    pub fn step(&mut self, bus: &mut impl Bus) -> Result<bool, CpuError> {
        // Fetch opcode
        let opcode_byte = bus.read(self.registers.pc);
        let opcode = self.decode_opcode(opcode_byte)?;

        // Calculate address based on addressing mode
        let (address, extra_cycles) = self.get_address(bus, opcode)?;

        // Execute instruction
        self.execute(bus, opcode, address)?;

        // Update PC by adding 1 + address mode bytes
        let addr_bytes = match self.addressing_mode(opcode) {
            AddressingMode::Implied | AddressingMode::Accumulator => 0,
            _ => 1,
        };
        self.registers.pc = self.registers.pc.wrapping_add(1 + addr_bytes as u16);

        // Update total cycles
        self.total_cycles += self.instruction_cycles(opcode) as u64 + extra_cycles as u64;

        Ok(true)
    }

    /// Get address for addressing mode
    fn get_address(&self, bus: &mut impl Bus, opcode: Opcode) -> Result<(u16, u8), CpuError> {
        let addr = match self.addressing_mode(opcode) {
            AddressingMode::Immediate => {
                let low = bus.read(self.registers.pc.wrapping_add(1)) as u16;
                (low, 0)
            }
            AddressingMode::ZeroPage => {
                let addr = bus.read(self.registers.pc.wrapping_add(1)) as u16;
                (addr, 0)
            }
            AddressingMode::ZeroPageX => {
                let zero_page = bus.read(self.registers.pc.wrapping_add(1)) as u16;
                let addr = (zero_page + self.registers.x as u16) & 0xFF;
                (addr, 0)
            }
            AddressingMode::ZeroPageY => {
                let zero_page = bus.read(self.registers.pc.wrapping_add(1)) as u16;
                let addr = (zero_page + self.registers.y as u16) & 0xFF;
                (addr, 0)
            }
            AddressingMode::Absolute => {
                let low = bus.read(self.registers.pc.wrapping_add(1)) as u16;
                let high = bus.read(self.registers.pc.wrapping_add(2)) as u16;
                (low | (high << 8), 0)
            }
            AddressingMode::AbsoluteX => {
                let low = bus.read(self.registers.pc.wrapping_add(1)) as u16;
                let high = bus.read(self.registers.pc.wrapping_add(2)) as u16;
                let base = low | (high << 8);
                let addr = base.wrapping_add(self.registers.x as u16);
                let extra = if (base ^ addr) & 0xFF00 != 0 { 1 } else { 0 };
                (addr, extra)
            }
            AddressingMode::AbsoluteY => {
                let low = bus.read(self.registers.pc.wrapping_add(1)) as u16;
                let high = bus.read(self.registers.pc.wrapping_add(2)) as u16;
                let base = low | (high << 8);
                let addr = base.wrapping_add(self.registers.y as u16);
                let extra = if (base ^ addr) & 0xFF00 != 0 { 1 } else { 0 };
                (addr, extra)
            }
            AddressingMode::IndirectX => {
                let zero_page = (bus.read(self.registers.pc.wrapping_add(1)) + self.registers.x) as u16;
                let low = bus.read(zero_page & 0xFF) as u16;
                let high = bus.read((zero_page + 1) & 0xFF) as u16;
                (low | (high << 8), 0)
            }
            AddressingMode::IndirectY => {
                let zero_page = bus.read(self.registers.pc.wrapping_add(1)) as u16;
                let low = bus.read(zero_page) as u16;
                let high = bus.read((zero_page + 1) as u8 as u16) as u16;
                let base = low | (high << 8);
                let addr = base.wrapping_add(self.registers.y as u16);
                let extra = if (base ^ addr) & 0xFF00 != 0 { 1 } else { 0 };
                (addr, extra)
            }
            AddressingMode::Relative => {
                let offset = bus.read(self.registers.pc.wrapping_add(1)) as i8;
                let addr = self.registers.pc.wrapping_add(2) as i16 + offset as i16;
                // Branch cycle penalty if taken
                let extra = if ((self.registers.pc as i16 ^ addr) & 0xFF00u16 as i16) != 0 { 1 } else { 0 };
                (addr as u16, extra)
            }
            AddressingMode::Accumulator => {
                (0, 0)
            }
            AddressingMode::Implied => {
                // Implied addressing mode has no operand
                (0, 0)
            }
        };
        Ok(addr)
    }

    /// Execute an instruction
    fn execute(&mut self, bus: &mut impl Bus, opcode: Opcode, address: u16) -> Result<(), CpuError> {
        match opcode {
            // ADC - Add with Carry
            Opcode::ADCImmediate => self.adc(bus.read(address)),
            Opcode::ADCZeroPage => self.adc(bus.read(address)),
            Opcode::ADCZeroPageX => self.adc(bus.read(address)),
            Opcode::ADCAbsolute => self.adc(bus.read(address)),
            Opcode::ADCAbsoluteX => self.adc(bus.read(address)),
            Opcode::ADCAbsoluteY => self.adc(bus.read(address)),
            Opcode::ADCIndirectX => self.adc(bus.read(address)),
            Opcode::ADCIndirectY => self.adc(bus.read(address)),

            // AND - Logical AND
            Opcode::ANDImmediate => self.and(bus.read(address)),
            Opcode::ANDZeroPage => self.and(bus.read(address)),
            Opcode::ANDZeroPageX => self.and(bus.read(address)),
            Opcode::ANDAbsolute => self.and(bus.read(address)),
            Opcode::ANDAbsoluteX => self.and(bus.read(address)),
            Opcode::ANDAbsoluteY => self.and(bus.read(address)),
            Opcode::ANDIndirectX => self.and(bus.read(address)),
            Opcode::ANDIndirectY => self.and(bus.read(address)),

            // ASL - Arithmetic Shift Left
            Opcode::ASLAccumulator => self.asl_accumulator(),
            Opcode::ASLZeroPage => self.asl_zero_page(bus, address),
            Opcode::ASLZeroPageX => self.asl_zero_page(bus, address),
            Opcode::ASLAbsolute => self.asl_absolute(bus, address),
            Opcode::ASLAbsoluteX => self.asl_absolute(bus, address),

            // Branch instructions
            Opcode::BCCRelative => self.bcc(address),
            Opcode::BCSRelative => self.bcs(address),
            Opcode::BEQRelative => self.beq(address),
            Opcode::BMIRelative => self.bmi(address),
            Opcode::BNERelative => self.bne(address),
            Opcode::BPLRelative => self.bpl(address),
            Opcode::BVCRelative => self.bvc(address),
            Opcode::BVSRelative => self.bvs(address),

            // BIT - Test Bits
            Opcode::BITZeroPage => self.bit(bus.read(address)),
            Opcode::BITAbsolute => self.bit(bus.read(address)),

            // Clear flags
            Opcode::CLCImplied => { self.status.set_carry(false); Ok(()) }
            Opcode::CLDImplied => { self.status.set_decimal(false); Ok(()) }
            Opcode::CLIImplied => { self.status.set_interrupt(false); Ok(()) }
            Opcode::CLVImplied => { self.status.set_overflow(false); Ok(()) }

            // CMP - Compare
            Opcode::CMPImmediate => self.cmp(bus.read(address)),
            Opcode::CMPZeroPage => self.cmp(bus.read(address)),
            Opcode::CMPZeroPageX => self.cmp(bus.read(address)),
            Opcode::CmpAbsolute => self.cmp(bus.read(address)),
            Opcode::CmpAbsoluteX => self.cmp(bus.read(address)),
            Opcode::CmpAbsoluteY => self.cmp(bus.read(address)),
            Opcode::CMPIndirectX => self.cmp(bus.read(address)),
            Opcode::CMPIndirectY => self.cmp(bus.read(address)),

            // CPX - Compare X Register
            Opcode::CPXImmediate => self.cpx(bus.read(address)),
            Opcode::CPXZeroPage => self.cpx(bus.read(address)),
            Opcode::CPXAbsolute => self.cpx(bus.read(address)),

            // CPY - Compare Y Register
            Opcode::CPYImmediate => self.cpy(bus.read(address)),
            Opcode::CPYZeroPage => self.cpy(bus.read(address)),
            Opcode::CPYAbsolute => self.cpy(bus.read(address)),

            // DEC - Decrement
            Opcode::DECZeroPage => self.dec_zero_page(bus, address),
            Opcode::DECZeroPageX => self.dec_zero_page(bus, address),
            Opcode::DECAbsolute => self.dec_absolute(bus, address),
            Opcode::DECAbsoluteX => self.dec_absolute(bus, address),

            // DEX - Decrement X
            Opcode::DEXImplied => { self.registers.x = self.registers.x.wrapping_sub(1); self.set_flags_zn(self.registers.x); Ok(()) }

            // DEY - Decrement Y
            Opcode::DEYImplied => { self.registers.y = self.registers.y.wrapping_sub(1); self.set_flags_zn(self.registers.y); Ok(()) }

            // EOR - Exclusive OR
            Opcode::EORImmediate => self.eor(bus.read(address)),
            Opcode::EORZeroPage => self.eor(bus.read(address)),
            Opcode::EORZeroPageX => self.eor(bus.read(address)),
            Opcode::EORAbsolute => self.eor(bus.read(address)),
            Opcode::EORAbsoluteX => self.eor(bus.read(address)),
            Opcode::EORAbsoluteY => self.eor(bus.read(address)),
            Opcode::EORIndirectX => self.eor(bus.read(address)),
            Opcode::EORIndirectY => self.eor(bus.read(address)),

            // INC - Increment
            Opcode::INCZeroPage => self.inc_zero_page(bus, address),
            Opcode::INCZeroPageX => self.inc_zero_page(bus, address),
            Opcode::INCAbsolute => self.inc_absolute(bus, address),
            Opcode::INCAbsoluteX => self.inc_absolute(bus, address),

            // INX - Increment X
            Opcode::INXImplied => { self.registers.x = self.registers.x.wrapping_add(1); self.set_flags_zn(self.registers.x); Ok(()) }

            // INY - Increment Y
            Opcode::INYImplied => { self.registers.y = self.registers.y.wrapping_add(1); self.set_flags_zn(self.registers.y); Ok(()) }

            // JMP - Jump
            Opcode::JMPAbsolute => { self.registers.pc = address; Ok(()) }
            Opcode::JMPIndirect => {
                // Read address from (addr) - handles page wrap correctly
                let low_addr = address;
                let high_addr = (address & 0xFF00) | ((address + 1) & 0x00FF);
                let low = bus.read(low_addr) as u16;
                let high = bus.read(high_addr) as u16;
                self.registers.pc = low | (high << 8);
                Ok(())
            }

            // JSR - Jump to Subroutine
            Opcode::JSRAbsolute => {
                let ret_addr = self.registers.pc.wrapping_add(2);
                self.push(bus, (ret_addr >> 8) as u8)?;
                self.push(bus, ret_addr as u8)?;
                self.registers.pc = address;
                Ok(())
            }

            // LDA - Load Accumulator
            Opcode::LDAImmediate => { self.registers.a = bus.read(address); self.set_flags_zn(self.registers.a); Ok(()) }
            Opcode::LDAZeroPage => { self.registers.a = bus.read(address); self.set_flags_zn(self.registers.a); Ok(()) }
            Opcode::LDAZeroPageX => { self.registers.a = bus.read(address); self.set_flags_zn(self.registers.a); Ok(()) }
            Opcode::LDAAbsolute => { self.registers.a = bus.read(address); self.set_flags_zn(self.registers.a); Ok(()) }
            Opcode::LDAAbsoluteX => { self.registers.a = bus.read(address); self.set_flags_zn(self.registers.a); Ok(()) }
            Opcode::LDAAbsoluteY => { self.registers.a = bus.read(address); self.set_flags_zn(self.registers.a); Ok(()) }
            Opcode::LDAIndirectX => { self.registers.a = bus.read(address); self.set_flags_zn(self.registers.a); Ok(()) }
            Opcode::LDAIndirectY => { self.registers.a = bus.read(address); self.set_flags_zn(self.registers.a); Ok(()) }

            // LDX - Load X Register
            Opcode::LDXImmediate => { self.registers.x = bus.read(address); self.set_flags_zn(self.registers.x); Ok(()) }
            Opcode::LDXZeroPage => { self.registers.x = bus.read(address); self.set_flags_zn(self.registers.x); Ok(()) }
            Opcode::LDXZeroPageY => { self.registers.x = bus.read(address); self.set_flags_zn(self.registers.x); Ok(()) }
            Opcode::LDXAbsolute => { self.registers.x = bus.read(address); self.set_flags_zn(self.registers.x); Ok(()) }
            Opcode::LDXAbsoluteY => { self.registers.x = bus.read(address); self.set_flags_zn(self.registers.x); Ok(()) }

            // LDY - Load Y Register
            Opcode::LDYImmediate => { self.registers.y = bus.read(address); self.set_flags_zn(self.registers.y); Ok(()) }
            Opcode::LDYZeroPage => { self.registers.y = bus.read(address); self.set_flags_zn(self.registers.y); Ok(()) }
            Opcode::LDYZeroPageX => { self.registers.y = bus.read(address); self.set_flags_zn(self.registers.y); Ok(()) }
            Opcode::LDYAbsolute => { self.registers.y = bus.read(address); self.set_flags_zn(self.registers.y); Ok(()) }
            Opcode::LDYAbsoluteX => { self.registers.y = bus.read(address); self.set_flags_zn(self.registers.y); Ok(()) }

            // LSR - Logical Shift Right
            Opcode::LSRAccumulator => self.lsr_accumulator(),
            Opcode::LSRZeroPage => self.lsr_zero_page(bus, address),
            Opcode::LSRZeroPageX => self.lsr_zero_page(bus, address),
            Opcode::LSRAbsolute => self.lsr_absolute(bus, address),
            Opcode::LSRAbsoluteX => self.lsr_absolute(bus, address),

            // NOP - No Operation
            Opcode::NOPImplied => Ok(()),

            // ORA - Logical OR
            Opcode::ORAImmediate => self.ora(bus.read(address)),
            Opcode::ORAZeroPage => self.ora(bus.read(address)),
            Opcode::ORAZeroPageX => self.ora(bus.read(address)),
            Opcode::ORAAbsolute => self.ora(bus.read(address)),
            Opcode::ORAAbsoluteX => self.ora(bus.read(address)),
            Opcode::ORAAbsoluteY => self.ora(bus.read(address)),
            Opcode::ORAIndirectX => self.ora(bus.read(address)),
            Opcode::ORAIndirectY => self.ora(bus.read(address)),

            // PHA - Push Accumulator
            Opcode::PHAImplied => self.push(bus, self.registers.a),

            // PHP - Push Processor Status
            Opcode::PHPImplied => {
                let mut p = self.status.0;
                p |= StatusFlags::BREAK | StatusFlags::UNUSED;
                self.push(bus, p)
            }

            // PLA - Pull Accumulator
            Opcode::PLAImplied => {
                self.registers.a = self.pull(bus)?;
                self.set_flags_zn(self.registers.a);
                Ok(())
            }

            // PLP - Pull Processor Status
            Opcode::PLPImplied => {
                let p = self.pull(bus)?;
                // Clear break and unused flags
                self.status = StatusFlags::new(p & 0xEF);
                Ok(())
            }

            // ROL - Rotate Left
            Opcode::ROLAccumulator => self.rol_accumulator(),
            Opcode::ROLZeroPage => self.rol_zero_page(bus, address),
            Opcode::ROLZeroPageX => self.rol_zero_page(bus, address),
            Opcode::ROLAbsolute => self.rol_absolute(bus, address),
            Opcode::ROLAbsoluteX => self.rol_absolute(bus, address),

            // ROR - Rotate Right
            Opcode::RORAccumulator => self.ror_accumulator(),
            Opcode::RORZeroPage => self.ror_zero_page(bus, address),
            Opcode::RORZeroPageX => self.ror_zero_page(bus, address),
            Opcode::RORAbsolute => self.ror_absolute(bus, address),
            Opcode::RORAbsoluteX => self.ror_absolute(bus, address),

            // RTI - Return from Interrupt
            Opcode::RTIImplied => {
                self.pull(bus)?; // Skip processor status
                let low = self.pull(bus)?;
                let high = self.pull(bus)?;
                self.registers.pc = (high as u16) << 8 | (low as u16);
                Ok(())
            }

            // RTS - Return from Subroutine
            Opcode::RTSImplied => {
                let low = self.pull(bus)?;
                let high = self.pull(bus)?;
                self.registers.pc = ((high as u16) << 8 | (low as u16)).wrapping_add(1);
                Ok(())
            }

            // SBC - Subtract with Carry
            Opcode::SBCImmediate => self.sbc(bus.read(address)),
            Opcode::SBCZeroPage => self.sbc(bus.read(address)),
            Opcode::SBCZeroPageX => self.sbc(bus.read(address)),
            Opcode::SBCAbsolute => self.sbc(bus.read(address)),
            Opcode::SBCAbsoluteX => self.sbc(bus.read(address)),
            Opcode::SBCAbsoluteY => self.sbc(bus.read(address)),
            Opcode::SBCIndirectX => self.sbc(bus.read(address)),
            Opcode::SBCIndirectY => self.sbc(bus.read(address)),

            // SEC - Set Carry
            Opcode::SECImplied => { self.status.set_carry(true); Ok(()) }

            // SEI - Set Interrupt
            Opcode::SEIImplied => { self.status.set_interrupt(true); Ok(()) }

            // STX - Store X Register
            Opcode::STXZeroPage => { bus.write(address, self.registers.x); Ok(()) }
            Opcode::STXZeroPageY => { bus.write(address, self.registers.x); Ok(()) }
            Opcode::STXAbsolute => { bus.write(address, self.registers.x); Ok(()) }

            // STY - Store Y Register
            Opcode::STYZeroPage => { bus.write(address, self.registers.y); Ok(()) }
            Opcode::STYZeroPageX => { bus.write(address, self.registers.y); Ok(()) }
            Opcode::STYAbsolute => { bus.write(address, self.registers.y); Ok(()) }

            // STA - Store Accumulator
            Opcode::STAZeroPage => { bus.write(address, self.registers.a); Ok(()) }
            Opcode::STAZeroPageX => { bus.write(address, self.registers.a); Ok(()) }
            Opcode::STAAbsolute => { bus.write(address, self.registers.a); Ok(()) }
            Opcode::STAAbsoluteX => { bus.write(address, self.registers.a); Ok(()) }
            Opcode::STAAbsoluteY => { bus.write(address, self.registers.a); Ok(()) }
            Opcode::STAIndirectX => { bus.write(address, self.registers.a); Ok(()) }
            Opcode::STAIndirectY => { bus.write(address, self.registers.a); Ok(()) }

            // TAX - Transfer A to X
            Opcode::TAXImplied => { self.registers.x = self.registers.a; self.set_flags_zn(self.registers.x); Ok(()) }

            // TAY - Transfer A to Y
            Opcode::TAYImplied => { self.registers.y = self.registers.a; self.set_flags_zn(self.registers.y); Ok(()) }

            // TSX - Transfer S to X
            Opcode::TSXImplied => { self.registers.x = self.registers.sp; self.set_flags_zn(self.registers.x); Ok(()) }

            // TXA - Transfer X to A
            Opcode::TXAImplied => { self.registers.a = self.registers.x; self.set_flags_zn(self.registers.a); Ok(()) }

            // TXS - Transfer X to S
            Opcode::TXSImplied => { self.registers.sp = self.registers.x; Ok(()) }

            // TYA - Transfer Y to A
            Opcode::TYAImplied => { self.registers.a = self.registers.y; self.set_flags_zn(self.registers.a); Ok(()) }

            // BRK - Break
            Opcode::BRKImplied => {
                // Push PC + 2, then push P
                let pc_high = (self.registers.pc.wrapping_add(2) >> 8) as u8;
                let pc_low = (self.registers.pc.wrapping_add(2)) as u8;
                self.push(bus, pc_high)?;
                self.push(bus, pc_low)?;
                let mut p = self.status.0;
                p |= StatusFlags::BREAK;
                self.push(bus, p)?;
                // Set interrupt flag and get vector
                self.status.set_interrupt(true);
                let low = bus.read(0xFFFE) as u16;
                let high = bus.read(0xFFFF) as u16;
                self.registers.pc = low | (high << 8);
                Ok(())
            }
        }
    }

    // Helper functions for ALU operations
    fn adc(&mut self, value: u8) -> Result<(), CpuError> {
        let carry = if self.status.carry() { 1 } else { 0 };
        let sum = self.registers.a as u16 + value as u16 + carry as u16;

        // Overflow detection (signed addition)
        let a_negative = (self.registers.a & 0x80) != 0;
        let v_negative = (value & 0x80) != 0;
        let result_negative = (sum & 0x100) != 0;
        let overflow = (a_negative == v_negative) && (result_negative != a_negative);

        self.status.set_overflow(overflow);
        self.status.set_carry(sum > 0xFF);
        self.registers.a = (sum & 0xFF) as u8;
        self.set_flags_zn(self.registers.a);
        Ok(())
    }

    fn and(&mut self, value: u8) -> Result<(), CpuError> {
        self.registers.a &= value;
        self.set_flags_zn(self.registers.a);
        Ok(())
    }

    fn asl_accumulator(&mut self) -> Result<(), CpuError> {
        let val = self.registers.a;
        self.status.set_carry((val & 0x80) != 0);
        self.registers.a = val << 1;
        self.set_flags_zn(self.registers.a);
        Ok(())
    }

    fn asl_zero_page(&mut self, bus: &mut impl Bus, address: u16) -> Result<(), CpuError> {
        let val = bus.read(address);
        self.status.set_carry((val & 0x80) != 0);
        let result = val << 1;
        bus.write(address, result);
        self.set_flags_zn(result);
        Ok(())
    }

    fn asl_absolute(&mut self, bus: &mut impl Bus, address: u16) -> Result<(), CpuError> {
        let val = bus.read(address);
        self.status.set_carry((val & 0x80) != 0);
        let result = val << 1;
        bus.write(address, result);
        self.set_flags_zn(result);
        Ok(())
    }

    fn bcc(&mut self, address: u16) -> Result<(), CpuError> {
        if !self.status.carry() {
            self.registers.pc = address;
        }
        Ok(())
    }

    fn bcs(&mut self, address: u16) -> Result<(), CpuError> {
        if self.status.carry() {
            self.registers.pc = address;
        }
        Ok(())
    }

    fn beq(&mut self, address: u16) -> Result<(), CpuError> {
        if self.status.zero() {
            self.registers.pc = address;
        }
        Ok(())
    }

    fn bmi(&mut self, address: u16) -> Result<(), CpuError> {
        if self.status.negative() {
            self.registers.pc = address;
        }
        Ok(())
    }

    fn bne(&mut self, address: u16) -> Result<(), CpuError> {
        if !self.status.zero() {
            self.registers.pc = address;
        }
        Ok(())
    }

    fn bpl(&mut self, address: u16) -> Result<(), CpuError> {
        if !self.status.negative() {
            self.registers.pc = address;
        }
        Ok(())
    }

    fn bvc(&mut self, address: u16) -> Result<(), CpuError> {
        if !self.status.overflow() {
            self.registers.pc = address;
        }
        Ok(())
    }

    fn bvs(&mut self, address: u16) -> Result<(), CpuError> {
        if self.status.overflow() {
            self.registers.pc = address;
        }
        Ok(())
    }

    fn bit(&mut self, value: u8) -> Result<(), CpuError> {
        let and_result = self.registers.a & value;
        self.status.set_zero(and_result == 0);
        self.status.set_negative((value & 0x80) != 0);
        self.status.set_overflow((value & 0x40) != 0);
        Ok(())
    }

    fn cmp(&mut self, value: u8) -> Result<(), CpuError> {
        let result = self.registers.a.wrapping_sub(value);
        self.status.set_carry(self.registers.a >= value);
        self.set_flags_zn(result as u8);
        Ok(())
    }

    fn cpx(&mut self, value: u8) -> Result<(), CpuError> {
        let result = self.registers.x.wrapping_sub(value);
        self.status.set_carry(self.registers.x >= value);
        self.set_flags_zn(result as u8);
        Ok(())
    }

    fn cpy(&mut self, value: u8) -> Result<(), CpuError> {
        let result = self.registers.y.wrapping_sub(value);
        self.status.set_carry(self.registers.y >= value);
        self.set_flags_zn(result as u8);
        Ok(())
    }

    fn dec_zero_page(&mut self, bus: &mut impl Bus, address: u16) -> Result<(), CpuError> {
        let val = bus.read(address).wrapping_sub(1);
        bus.write(address, val);
        self.set_flags_zn(val);
        Ok(())
    }

    fn dec_absolute(&mut self, bus: &mut impl Bus, address: u16) -> Result<(), CpuError> {
        let val = bus.read(address).wrapping_sub(1);
        bus.write(address, val);
        self.set_flags_zn(val);
        Ok(())
    }

    fn eor(&mut self, value: u8) -> Result<(), CpuError> {
        self.registers.a ^= value;
        self.set_flags_zn(self.registers.a);
        Ok(())
    }

    fn inc_zero_page(&mut self, bus: &mut impl Bus, address: u16) -> Result<(), CpuError> {
        let val = bus.read(address).wrapping_add(1);
        bus.write(address, val);
        self.set_flags_zn(val);
        Ok(())
    }

    fn inc_absolute(&mut self, bus: &mut impl Bus, address: u16) -> Result<(), CpuError> {
        let val = bus.read(address).wrapping_add(1);
        bus.write(address, val);
        self.set_flags_zn(val);
        Ok(())
    }

    fn lsr_accumulator(&mut self) -> Result<(), CpuError> {
        let val = self.registers.a;
        self.status.set_carry(val & 0x01 != 0);
        self.registers.a = val >> 1;
        self.set_flags_zn(self.registers.a);
        Ok(())
    }

    fn lsr_zero_page(&mut self, bus: &mut impl Bus, address: u16) -> Result<(), CpuError> {
        let val = bus.read(address);
        self.status.set_carry(val & 0x01 != 0);
        let result = val >> 1;
        bus.write(address, result);
        self.set_flags_zn(result);
        Ok(())
    }

    fn lsr_absolute(&mut self, bus: &mut impl Bus, address: u16) -> Result<(), CpuError> {
        let val = bus.read(address);
        self.status.set_carry(val & 0x01 != 0);
        let result = val >> 1;
        bus.write(address, result);
        self.set_flags_zn(result);
        Ok(())
    }

    fn ora(&mut self, value: u8) -> Result<(), CpuError> {
        self.registers.a |= value;
        self.set_flags_zn(self.registers.a);
        Ok(())
    }

    fn push(&mut self, bus: &mut impl Bus, value: u8) -> Result<(), CpuError> {
        let addr = 0x0100 | (self.registers.sp as u16);
        self.registers.sp = self.registers.sp.wrapping_sub(1);
        bus.write(addr, value);
        Ok(())
    }

    fn pull(&mut self, bus: &mut impl Bus) -> Result<u8, CpuError> {
        self.registers.sp = self.registers.sp.wrapping_add(1);
        let addr = 0x0100 | (self.registers.sp as u16);
        Ok(bus.read(addr))
    }

    fn rol_accumulator(&mut self) -> Result<(), CpuError> {
        let val = self.registers.a;
        let old_carry = self.status.carry() as u8;
        self.status.set_carry((val & 0x80) != 0);
        self.registers.a = (val << 1) | old_carry;
        self.set_flags_zn(self.registers.a);
        Ok(())
    }

    fn rol_zero_page(&mut self, bus: &mut impl Bus, address: u16) -> Result<(), CpuError> {
        let val = bus.read(address);
        let old_carry = self.status.carry() as u8;
        self.status.set_carry((val & 0x80) != 0);
        let result = (val << 1) | old_carry;
        bus.write(address, result);
        self.set_flags_zn(result);
        Ok(())
    }

    fn rol_absolute(&mut self, bus: &mut impl Bus, address: u16) -> Result<(), CpuError> {
        let val = bus.read(address);
        let old_carry = self.status.carry() as u8;
        self.status.set_carry((val & 0x80) != 0);
        let result = (val << 1) | old_carry;
        bus.write(address, result);
        self.set_flags_zn(result);
        Ok(())
    }

    fn ror_accumulator(&mut self) -> Result<(), CpuError> {
        let val = self.registers.a;
        let old_carry = self.status.carry() as u8;
        self.status.set_carry(val & 0x01 != 0);
        self.registers.a = (val >> 1) | (old_carry << 7);
        self.set_flags_zn(self.registers.a);
        Ok(())
    }

    fn ror_zero_page(&mut self, bus: &mut impl Bus, address: u16) -> Result<(), CpuError> {
        let val = bus.read(address);
        let old_carry = self.status.carry() as u8;
        self.status.set_carry(val & 0x01 != 0);
        let result = (val >> 1) | (old_carry << 7);
        bus.write(address, result);
        self.set_flags_zn(result);
        Ok(())
    }

    fn ror_absolute(&mut self, bus: &mut impl Bus, address: u16) -> Result<(), CpuError> {
        let val = bus.read(address);
        let old_carry = self.status.carry() as u8;
        self.status.set_carry(val & 0x01 != 0);
        let result = (val >> 1) | (old_carry << 7);
        bus.write(address, result);
        self.set_flags_zn(result);
        Ok(())
    }

    fn sbc(&mut self, value: u8) -> Result<(), CpuError> {
        let carry = if self.status.carry() { 0 } else { 1 };
        let result = self.registers.a as i16 - value as i16 - carry as i16;

        // Overflow detection
        let a_negative = (self.registers.a & 0x80) != 0;
        let v_negative = (value & 0x80) != 0;
        let result_negative = (result as u8 & 0x80) != 0;
        let overflow = (a_negative != v_negative) && (result_negative != a_negative);

        self.status.set_overflow(overflow);
        self.status.set_carry(result >= 0);
        self.registers.a = (result as u8) & 0xFF;
        self.set_flags_zn(self.registers.a);
        Ok(())
    }

    fn set_flags_zn(&mut self, value: u8) {
        self.status.set_zero(value == 0);
        self.status.set_negative((value & 0x80) != 0);
    }

    /// Get the addressing mode for an opcode
    fn addressing_mode(&self, opcode: Opcode) -> AddressingMode {
        match opcode {
            Opcode::ADCImmediate | Opcode::ANDImmediate | Opcode::CMPImmediate
            | Opcode::CPXImmediate | Opcode::CPYImmediate | Opcode::EORImmediate
            | Opcode::LDXImmediate | Opcode::LDYImmediate | Opcode::LDAImmediate
            | Opcode::ORAImmediate | Opcode::SBCImmediate => AddressingMode::Immediate,

            Opcode::ADCZeroPage | Opcode::ANDZeroPage | Opcode::CMPZeroPage
            | Opcode::EORZeroPage | Opcode::LDXZeroPage | Opcode::LDYZeroPage
            | Opcode::LDAZeroPage | Opcode::ORAZeroPage | Opcode::SBCZeroPage
            | Opcode::BITZeroPage | Opcode::DECZeroPage | Opcode::INCZeroPage
            | Opcode::LSRZeroPage | Opcode::ROLZeroPage | Opcode::RORZeroPage
            | Opcode::STAZeroPage | Opcode::STXZeroPage | Opcode::STYZeroPage
            | Opcode::ASLZeroPage | Opcode::CPXZeroPage | Opcode::CPYZeroPage => AddressingMode::ZeroPage,

            Opcode::ADCZeroPageX | Opcode::ANDZeroPageX | Opcode::CMPZeroPageX
            | Opcode::EORZeroPageX | Opcode::LDAZeroPageX | Opcode::ORAZeroPageX
            | Opcode::SBCZeroPageX | Opcode::DECZeroPageX | Opcode::INCZeroPageX
            | Opcode::LSRZeroPageX | Opcode::ROLZeroPageX | Opcode::RORZeroPageX
            | Opcode::STAZeroPageX | Opcode::ASLZeroPageX => AddressingMode::ZeroPageX,

            Opcode::LDXZeroPageY | Opcode::LDYZeroPageX | Opcode::STXZeroPageY | Opcode::STYZeroPageX => AddressingMode::ZeroPageY,

            Opcode::ADCAbsolute | Opcode::ANDAbsolute | Opcode::CmpAbsolute
            | Opcode::EORAbsolute | Opcode::LDAAbsolute | Opcode::LDXAbsolute
            | Opcode::LDYAbsolute | Opcode::ORAAbsolute | Opcode::SBCAbsolute
            | Opcode::BITAbsolute | Opcode::DECAbsolute | Opcode::INCAbsolute
            | Opcode::LSRAbsolute | Opcode::ROLAbsolute | Opcode::RORAbsolute
            | Opcode::STAAbsolute | Opcode::STXAbsolute | Opcode::STYAbsolute
            | Opcode::ASLAbsolute | Opcode::CPXAbsolute | Opcode::CPYAbsolute
            | Opcode::STAAbsoluteX | Opcode::STAAbsoluteY => AddressingMode::Absolute,

            Opcode::ADCAbsoluteX | Opcode::ANDAbsoluteX | Opcode::CmpAbsoluteX
            | Opcode::EORAbsoluteX | Opcode::LDAAbsoluteX | Opcode::ORAAbsoluteX
            | Opcode::SBCAbsoluteX | Opcode::DECAbsoluteX | Opcode::INCAbsoluteX
            | Opcode::LDXAbsoluteY | Opcode::LDYAbsoluteX | Opcode::ASLAbsoluteX
            | Opcode::LSRAbsoluteX | Opcode::ROLAbsoluteX | Opcode::RORAbsoluteX => AddressingMode::AbsoluteX,

            Opcode::ADCAbsoluteY | Opcode::ANDAbsoluteY | Opcode::CmpAbsoluteY
            | Opcode::EORAbsoluteY | Opcode::LDAAbsoluteY | Opcode::ORAAbsoluteY
            | Opcode::SBCAbsoluteY => AddressingMode::AbsoluteY,

            Opcode::ADCIndirectX | Opcode::ANDIndirectX | Opcode::CMPIndirectX
            | Opcode::EORIndirectX | Opcode::LDAIndirectX | Opcode::ORAIndirectX
            | Opcode::SBCIndirectX => AddressingMode::IndirectX,

            Opcode::ADCIndirectY | Opcode::ANDIndirectY | Opcode::CMPIndirectY
            | Opcode::EORIndirectY | Opcode::LDAIndirectY | Opcode::ORAIndirectY
            | Opcode::SBCIndirectY | Opcode::STAIndirectX | Opcode::STAIndirectY => AddressingMode::IndirectY,

            Opcode::BCCRelative | Opcode::BCSRelative | Opcode::BEQRelative
            | Opcode::BMIRelative | Opcode::BNERelative | Opcode::BPLRelative
            | Opcode::BVCRelative | Opcode::BVSRelative => AddressingMode::Relative,

            Opcode::ASLAccumulator | Opcode::LSRAccumulator | Opcode::ROLAccumulator
            | Opcode::RORAccumulator => AddressingMode::Accumulator,

            Opcode::BRKImplied | Opcode::CLCImplied | Opcode::CLDImplied
            | Opcode::CLIImplied | Opcode::CLVImplied | Opcode::DEXImplied
            | Opcode::DEYImplied | Opcode::INXImplied | Opcode::INYImplied
            | Opcode::JMPAbsolute | Opcode::JMPIndirect | Opcode::JSRAbsolute
            | Opcode::NOPImplied | Opcode::PHAImplied | Opcode::PHPImplied
            | Opcode::PLAImplied | Opcode::PLPImplied | Opcode::RTIImplied
            | Opcode::RTSImplied | Opcode::SECImplied | Opcode::SEIImplied
            | Opcode::TAXImplied | Opcode::TAYImplied | Opcode::TSXImplied
            | Opcode::TXAImplied | Opcode::TXSImplied | Opcode::TYAImplied => AddressingMode::Implied,
        }
    }

    /// Get the base cycle count for an opcode
    fn instruction_cycles(&self, opcode: Opcode) -> u8 {
        match opcode {
            // ADC, AND, CMP, EOR, LDA, ORA, SBC
            Opcode::ADCImmediate | Opcode::ANDImmediate | Opcode::CMPImmediate
            | Opcode::EORImmediate | Opcode::LDXImmediate | Opcode::LDYImmediate
            | Opcode::LDAImmediate | Opcode::ORAImmediate | Opcode::SBCImmediate => 2,

            Opcode::ADCZeroPage | Opcode::ANDZeroPage | Opcode::CMPZeroPage
            | Opcode::EORZeroPage | Opcode::LDAZeroPage | Opcode::ORAZeroPage
            | Opcode::SBCZeroPage | Opcode::BITZeroPage | Opcode::DECZeroPage
            | Opcode::INCZeroPage | Opcode::LSRZeroPage | Opcode::ROLZeroPage
            | Opcode::RORZeroPage | Opcode::STAZeroPage | Opcode::STXZeroPage
            | Opcode::STYZeroPage => 3,

            Opcode::ADCZeroPageX | Opcode::ANDZeroPageX | Opcode::CMPZeroPageX
            | Opcode::EORZeroPageX | Opcode::LDAZeroPageX | Opcode::ORAZeroPageX
            | Opcode::SBCZeroPageX | Opcode::DECZeroPageX | Opcode::INCZeroPageX
            | Opcode::LSRZeroPageX | Opcode::ROLZeroPageX | Opcode::RORZeroPageX
            | Opcode::STAZeroPageX => 4,

            Opcode::LDXZeroPageY | Opcode::LDYZeroPageX => 4,

            Opcode::ADCAbsolute | Opcode::ANDAbsolute | Opcode::CmpAbsolute
            | Opcode::EORAbsolute | Opcode::LDAAbsolute | Opcode::LDXAbsolute
            | Opcode::LDYAbsolute | Opcode::ORAAbsolute | Opcode::SBCAbsolute
            | Opcode::BITAbsolute | Opcode::DECAbsolute | Opcode::INCAbsolute
            | Opcode::LSRAbsolute | Opcode::ROLAbsolute | Opcode::RORAbsolute
            | Opcode::STAAbsolute | Opcode::STXAbsolute | Opcode::STYAbsolute => 4,

            Opcode::ADCAbsoluteX | Opcode::ANDAbsoluteX | Opcode::CmpAbsoluteX
            | Opcode::EORAbsoluteX | Opcode::LDAAbsoluteX | Opcode::ORAAbsoluteX
            | Opcode::SBCAbsoluteX | Opcode::DECAbsoluteX | Opcode::INCAbsoluteX => 4,

            Opcode::ADCAbsoluteY | Opcode::ANDAbsoluteY | Opcode::CmpAbsoluteY
            | Opcode::EORAbsoluteY | Opcode::LDAAbsoluteY | Opcode::ORAAbsoluteY
            | Opcode::SBCAbsoluteY | Opcode::LDXAbsoluteY | Opcode::LDYAbsoluteX => 4,

            Opcode::ADCIndirectX | Opcode::ANDIndirectX | Opcode::CMPIndirectX
            | Opcode::EORIndirectX | Opcode::LDAIndirectX | Opcode::ORAIndirectX
            | Opcode::SBCIndirectX => 6,

            Opcode::ADCIndirectY | Opcode::ANDIndirectY | Opcode::CMPIndirectY
            | Opcode::EORIndirectY | Opcode::LDAIndirectY | Opcode::ORAIndirectY
            | Opcode::SBCIndirectY => 5,

            Opcode::STAAbsoluteX => 5,
            Opcode::STAIndirectX => 6,
            Opcode::STAIndirectY => 6,

            // Branch instructions (2 + 1 if taken, +1 if page crossed)
            Opcode::BCCRelative | Opcode::BCSRelative | Opcode::BEQRelative
            | Opcode::BMIRelative | Opcode::BNERelative | Opcode::BPLRelative
            | Opcode::BVCRelative | Opcode::BVSRelative => 2,

            // Single-cycle instructions
            Opcode::ASLAccumulator | Opcode::CLCImplied | Opcode::CLDImplied
            | Opcode::CLIImplied | Opcode::CLVImplied | Opcode::DEXImplied
            | Opcode::DEYImplied | Opcode::INXImplied | Opcode::INYImplied
            | Opcode::NOPImplied | Opcode::SECImplied | Opcode::SEIImplied
            | Opcode::TXSImplied | Opcode::TYAImplied => 2,

            
            Opcode::CPXAbsolute | Opcode::CPYAbsolute => 4,

            // JMP Absolute = 3 cycles, JMP Indirect = 5 cycles
            Opcode::JMPAbsolute => 3,
            Opcode::JMPIndirect => 5,

            // JSR = 6 cycles, RTS = 6 cycles
            Opcode::JSRAbsolute => 6,
            Opcode::RTSImplied => 6,

            // BRK = 7 cycles, RTI = 6 cycles
            Opcode::BRKImplied => 7,
            Opcode::RTIImplied => 6,

            // PHA = 3 cycles, PHP = 3 cycles
            Opcode::PHAImplied => 3,
            Opcode::PHPImplied => 3,

            // PLA = 4 cycles, PLP = 4 cycles
            Opcode::PLAImplied => 4,
            Opcode::PLPImplied => 4,

            // Transfer instructions
            Opcode::TAXImplied | Opcode::TAYImplied | Opcode::TSXImplied
            | Opcode::TXAImplied => 2,

            // STX ZeroPageY
            Opcode::STXZeroPageY => 4,

            // Unknown - default to 2
            _ => 2,
        }
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