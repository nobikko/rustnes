//! 6502 CPU Emulator
//!
//! Implements the Ricoh 2A03 CPU used in the NES.

/// CPU status flags
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StatusFlags {
    pub carry: bool,
    pub zero: bool,
    pub interrupt: bool,
    pub decimal: bool,
    pub overflow: bool,
    pub sign: bool,
}

impl StatusFlags {
    pub fn new() -> Self {
        Self {
            carry: false,
            zero: true,
            interrupt: true,
            decimal: false,
            overflow: false,
            sign: false,
        }
    }

    pub fn from_u8(value: u8) -> Self {
        Self {
            carry: (value & 0x01) != 0,
            zero: (value & 0x02) != 0,
            interrupt: (value & 0x04) != 0,
            decimal: (value & 0x08) != 0,
            overflow: (value & 0x40) != 0,
            sign: (value & 0x80) != 0,
        }
    }

    pub fn to_u8(&self) -> u8 {
        let mut value = 0u8;
        if self.carry {
            value |= 0x01;
        }
        if self.zero {
            value |= 0x02;
        }
        if self.interrupt {
            value |= 0x04;
        }
        if self.decimal {
            value |= 0x08;
        }
        value |= 0x20;
        if self.overflow {
            value |= 0x40;
        }
        if self.sign {
            value |= 0x80;
        }
        value
    }
}

impl Default for StatusFlags {
    fn default() -> Self {
        Self::new()
    }
}

/// CPU Registers
#[derive(Debug, Clone, Copy)]
pub struct Registers {
    pub a: u8,
    pub x: u8,
    pub y: u8,
    pub sp: u8,
    pub pc: u16,
}

impl Registers {
    pub fn new() -> Self {
        Self {
            a: 0,
            x: 0,
            y: 0,
            sp: 0xFD,
            pc: 0,
        }
    }
}

impl Default for Registers {
    fn default() -> Self {
        Self::new()
    }
}

/// CPU Interrupt Request Types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum IrqRequest {
    None,
    Normal,
    Nmi,
    Reset,
}

/// CPU Addressing Modes
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AddressingMode {
    Implied,
    Accumulator,
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
    IndirectAbsolute,
}

/// CPU Opcodes
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Opcode {
    LDA, LDX, LDY,
    STA, STX, STY,
    TAX, TAY, TSX, TXA, TXS, TYA,
    ADC, SBC,
    CMP, CPX, CPY,
    AND, EOR, ORA,
    ASL, LSR, ROL, ROR, BIT,
    BCC, BCS, BEQ, BNE, BPL, BMI, BVC, BVS,
    JSR, RTS, RTI,
    BRK, NOP,
    SEC, CLC, SED, CLD, SEI, CLI, CLV,
    PHA, PHP, PLA, PLP,
    ANC, ALR, ARR, AXS,
    DCP, ISC, RLA, RRA, SLO, SRE,
    LAX, SAX, SHA, SHS, SHX, SHY, LAS,
    KIL,
    // Additional opcodes
    JMP, DEY, INY, DEX, INX, DEC, INC,
    ANE, TAS, LXA,
    AslA, LsrA, RolA, RorA,
}

/// Instruction information
#[derive(Debug, Clone, Copy)]
pub struct InstructionInfo {
    pub opcode: Opcode,
    pub mode: AddressingMode,
    pub cycles: u8,
}

/// The 6502 CPU emulator
#[derive(Debug)]
pub struct CPU {
    pub registers: Registers,
    pub flags: StatusFlags,
    pub memory: [u8; 0x10000],
    pub data_bus: u8,

    pub cycles: u64,
    pub irq_delay: u8,

    pub irq_request: IrqRequest,
    pub nmi_pending: bool,
    pub nmi_prev_low: bool,

    pub cycles_to_halt: u64,
    pub ppu_catchup_dots: u64,
    pub apu_catchup_cycles: u64,
}

impl CPU {
    pub fn new() -> Self {
        let mut cpu = Self {
            registers: Registers::new(),
            flags: StatusFlags::new(),
            memory: [0u8; 0x10000],
            data_bus: 0,
            cycles: 0,
            irq_delay: 0,
            irq_request: IrqRequest::None,
            nmi_pending: false,
            nmi_prev_low: true,
            cycles_to_halt: 0,
            ppu_catchup_dots: 0,
            apu_catchup_cycles: 0,
        };
        cpu.reset();
        cpu
    }

    pub fn reset(&mut self) {
        self.flags = StatusFlags::new();
        self.registers.sp = 0xFD;

        let lo = self.memory[0xFFFC] as u16;
        let hi = self.memory[0xFFFD] as u16;
        self.registers.pc = lo | (hi << 8);

        self.irq_request = IrqRequest::Reset;
        self.cycles = 0;
        self.irq_delay = 0;
    }

    pub fn request_irq(&mut self, irq_type: IrqRequest) {
        match irq_type {
            IrqRequest::Nmi => {
                self.nmi_pending = true;
            }
            IrqRequest::Normal => {
                if !self.flags.interrupt {
                    self.irq_request = irq_type;
                }
            }
            IrqRequest::Reset => {
                self.irq_request = irq_type;
            }
            IrqRequest::None => {}  // Ignore None
        }
    }

    pub fn load(&mut self, address: u16) -> u8 {
        let value = self.memory[address as usize];
        self.data_bus = value;
        value
    }

    pub fn load16(&mut self, address: u16) -> u16 {
        let lo = self.load(address) as u16;
        let hi = self.load(address + 1) as u16;
        let result = lo | (hi << 8);
        result
    }

    pub fn write(&mut self, address: u16, value: u8) {
        self.memory[address as usize] = value;
        self.data_bus = value;
    }

    pub fn push(&mut self, value: u8) {
        self.write(0x0100 | (self.registers.sp as u16), value);
        self.registers.sp = self.registers.sp.wrapping_sub(1);
    }

    pub fn pull(&mut self) -> u8 {
        self.registers.sp = self.registers.sp.wrapping_add(1);
        self.load(0x0100 | (self.registers.sp as u16))
    }

    fn effective_address(&mut self, mode: AddressingMode) -> u16 {
        match mode {
            AddressingMode::Immediate => {
                self.registers.pc
            }
            AddressingMode::ZeroPage => {
                self.memory[self.registers.pc as usize] as u16
            }
            AddressingMode::ZeroPageX => {
                let base = self.memory[self.registers.pc as usize];
                (base.wrapping_add(self.registers.x)) as u16
            }
            AddressingMode::ZeroPageY => {
                let base = self.memory[self.registers.pc as usize];
                (base.wrapping_add(self.registers.y)) as u16
            }
            AddressingMode::Absolute => {
                let addr = self.load16(self.registers.pc);
                addr
            }
            AddressingMode::AbsoluteX => {
                let base = self.load16(self.registers.pc);
                base.wrapping_add(self.registers.x as u16)
            }
            AddressingMode::AbsoluteY => {
                let base = self.load16(self.registers.pc);
                base.wrapping_add(self.registers.y as u16)
            }
            AddressingMode::IndirectX => {
                let zero = self.memory[self.registers.pc as usize] as u8;
                let effective = zero.wrapping_add(self.registers.x);
                let lo = self.memory[effective as usize] as u16;
                let hi = self.memory[(effective.wrapping_add(1) as usize)] as u16;
                lo | (hi << 8)
            }
            AddressingMode::IndirectY => {
                let zero = self.memory[self.registers.pc as usize] as u8;
                let base = {
                    let lo = self.memory[zero as usize] as u16;
                    let hi = self.memory[(zero + 1) as usize] as u16;
                    lo | (hi << 8)
                };
                base.wrapping_add(self.registers.y as u16)
            }
            AddressingMode::Relative => {
                let offset = self.memory[self.registers.pc as usize] as i8;
                self.registers.pc.wrapping_add(offset as u16)
            }
            _ => self.registers.pc,
        }
    }

    // Data Transfer Instructions
    fn op_lda(&mut self, value: u8) {
        self.registers.a = value;
        self.set_flags_zn(self.registers.a);
    }

    fn op_ldx(&mut self, value: u8) {
        self.registers.x = value;
        self.set_flags_zn(self.registers.x);
    }

    fn op_ldy(&mut self, value: u8) {
        self.registers.y = value;
        self.set_flags_zn(self.registers.y);
    }

    fn op_sta(&mut self, address: u16) {
        self.write(address, self.registers.a);
    }

    fn op_stx(&mut self, address: u16) {
        self.write(address, self.registers.x);
    }

    fn op_sty(&mut self, address: u16) {
        self.write(address, self.registers.y);
    }

    fn op_tax(&mut self) {
        self.registers.x = self.registers.a;
        self.set_flags_zn(self.registers.x);
    }

    fn op_tay(&mut self) {
        self.registers.y = self.registers.a;
        self.set_flags_zn(self.registers.y);
    }

    fn op_tsx(&mut self) {
        self.registers.x = self.registers.sp;
        self.set_flags_zn(self.registers.x);
    }

    fn op_txa(&mut self) {
        self.registers.a = self.registers.x;
        self.set_flags_zn(self.registers.a);
    }

    fn op_txs(&mut self) {
        self.registers.sp = self.registers.x;
    }

    fn op_tya(&mut self) {
        self.registers.a = self.registers.y;
        self.set_flags_zn(self.registers.a);
    }

    // Arithmetic Instructions
    fn op_adc(&mut self, value: u8) {
        let old_a = self.registers.a;
        let carry = if self.flags.carry { 1 } else { 0 };
        let result = old_a as u16 + value as u16 + carry as u16;

        self.flags.carry = result > 0xFF;
        let overflow = ((old_a ^ value) & 0x80) == 0 && ((old_a ^ result as u8) & 0x80) != 0;
        self.flags.overflow = overflow;

        self.registers.a = result as u8;
        self.set_flags_zn(self.registers.a);
    }

    fn op_sbc(&mut self, value: u8) {
        let old_a = self.registers.a;
        let carry = if self.flags.carry { 0 } else { 1 };
        // Use wrapping_sub to avoid overflow panic when old_a < value + carry
        let result = (old_a as u16).wrapping_sub(value as u16).wrapping_sub(carry as u16);

        self.flags.carry = result <= 0xFF;
        let overflow = ((old_a ^ value) & 0x80) != 0 && ((old_a ^ result as u8) & 0x80) != 0;
        self.flags.overflow = overflow;

        self.registers.a = result as u8;
        self.set_flags_zn(self.registers.a);
    }

    fn op_cmp(&mut self, value: u8) {
        // In 6502, carry is set if A >= value (no underflow), clear otherwise
        self.flags.carry = self.registers.a >= value;
        // Use wrapping_sub to avoid panic in debug mode when A < value
        let result = (self.registers.a as u16).wrapping_sub(value as u16);
        self.set_flags_zn(result as u8);
    }

    fn op_cpx(&mut self, value: u8) {
        // In 6502, carry is set if X >= value (no underflow), clear otherwise
        self.flags.carry = self.registers.x >= value;
        // Use wrapping_sub to avoid panic in debug mode when X < value
        let result = (self.registers.x as u16).wrapping_sub(value as u16);
        self.set_flags_zn(result as u8);
    }

    fn op_cpy(&mut self, value: u8) {
        // In 6502, carry is set if Y >= value (no underflow), clear otherwise
        self.flags.carry = self.registers.y >= value;
        // Use wrapping_sub to avoid panic in debug mode when Y < value
        let result = (self.registers.y as u16).wrapping_sub(value as u16);
        self.set_flags_zn(result as u8);
    }

    // Logic Instructions
    fn op_and(&mut self, value: u8) {
        self.registers.a &= value;
        self.set_flags_zn(self.registers.a);
    }

    fn op_eor(&mut self, value: u8) {
        self.registers.a ^= value;
        self.set_flags_zn(self.registers.a);
    }

    fn op_ora(&mut self, value: u8) {
        self.registers.a |= value;
        self.set_flags_zn(self.registers.a);
    }

    // Bit Manipulation Instructions
    fn op_asl(&mut self, value: u8) -> u8 {
        self.flags.carry = (value & 0x80) != 0;
        let result = value << 1;
        self.set_flags_zn(result);
        result
    }

    fn op_asl_a(&mut self) {
        self.flags.carry = (self.registers.a & 0x80) != 0;
        self.registers.a <<= 1;
        self.set_flags_zn(self.registers.a);
    }

    fn op_lsr(&mut self, value: u8) -> u8 {
        self.flags.carry = (value & 0x01) != 0;
        let result = value >> 1;
        self.set_flags_zn(result);
        result
    }

    fn op_lsr_a(&mut self) {
        self.flags.carry = (self.registers.a & 0x01) != 0;
        self.registers.a >>= 1;
        self.set_flags_zn(self.registers.a);
    }

    fn op_rol(&mut self, value: u8) -> u8 {
        let new_carry = (value & 0x80) != 0;
        let result = (value << 1) | if self.flags.carry { 1 } else { 0 };
        self.flags.carry = new_carry;
        self.set_flags_zn(result);
        result
    }

    fn op_rol_a(&mut self) {
        let new_carry = (self.registers.a & 0x80) != 0;
        self.registers.a = (self.registers.a << 1) | if self.flags.carry { 1 } else { 0 };
        self.flags.carry = new_carry;
        self.set_flags_zn(self.registers.a);
    }

    fn op_ror(&mut self, value: u8) -> u8 {
        let new_carry = (value & 0x01) != 0;
        let result = (value >> 1) | if self.flags.carry { 0x80 } else { 0 };
        self.flags.carry = new_carry;
        self.set_flags_zn(result);
        result
    }

    fn op_ror_a(&mut self) {
        let new_carry = (self.registers.a & 0x01) != 0;
        self.registers.a = (self.registers.a >> 1) | if self.flags.carry { 0x80 } else { 0 };
        self.flags.carry = new_carry;
        self.set_flags_zn(self.registers.a);
    }

    fn op_bit(&mut self, value: u8) {
        let result = self.registers.a & value;
        self.flags.zero = result == 0;
        self.flags.sign = (value & 0x80) != 0;
        self.flags.overflow = (value & 0x40) != 0;
    }

    // Branch Instructions
    fn op_bcc(&mut self, addr: u16) {
        if !self.flags.carry {
            self.registers.pc = addr;
        }
    }

    fn op_bcs(&mut self, addr: u16) {
        if self.flags.carry {
            self.registers.pc = addr;
        }
    }

    fn op_beq(&mut self, addr: u16) {
        if self.flags.zero {
            self.registers.pc = addr;
        }
    }

    fn op_bne(&mut self, addr: u16) {
        if !self.flags.zero {
            self.registers.pc = addr;
        }
    }

    fn op_bpl(&mut self, addr: u16) {
        if !self.flags.sign {
            self.registers.pc = addr;
        }
    }

    fn op_bmi(&mut self, addr: u16) {
        if self.flags.sign {
            self.registers.pc = addr;
        }
    }

    fn op_bvc(&mut self, addr: u16) {
        if !self.flags.overflow {
            self.registers.pc = addr;
        }
    }

    fn op_bvs(&mut self, addr: u16) {
        if self.flags.overflow {
            self.registers.pc = addr;
        }
    }

    // Subroutine Instructions
    fn op_jsr(&mut self, address: u16) {
        let ret = self.registers.pc + 2;
        self.push((ret >> 8) as u8);
        self.push(ret as u8);
        self.registers.pc = address;
    }

    fn op_rts(&mut self) {
        let lo = self.pull() as u16;
        let hi = self.pull() as u16;
        let ret = (hi << 8) | lo;
        self.registers.pc = ret + 1;
    }

    fn op_rti(&mut self) {
        let flags = self.pull();
        self.flags = StatusFlags::from_u8(flags);
        let lo = self.pull() as u16;
        let hi = self.pull() as u16;
        self.registers.pc = (hi << 8) | lo;
    }

    // Interrupt Instructions
    fn op_brk(&mut self) {
        let pc = self.registers.pc + 2;
        self.push((pc >> 8) as u8);
        self.push(pc as u8);

        let mut flags = self.flags.to_u8();
        flags |= 0x10;
        self.push(flags);

        self.flags.interrupt = true;
        let lo = self.load(0xFFFE) as u16;
        let hi = self.load(0xFFFF) as u16;
        self.registers.pc = lo | (hi << 8);
    }

    fn op_nop(&mut self) {}

    // Status Flag Instructions
    fn op_sec(&mut self) { self.flags.carry = true; }
    fn op_clc(&mut self) { self.flags.carry = false; }
    fn op_sed(&mut self) { self.flags.decimal = true; }
    fn op_cld(&mut self) { self.flags.decimal = false; }
    fn op_sei(&mut self) { self.flags.interrupt = true; }
    fn op_cli(&mut self) { self.flags.interrupt = false; }
    fn op_clv(&mut self) { self.flags.overflow = false; }

    // Stack Instructions
    fn op_pha(&mut self) { self.push(self.registers.a); }
    fn op_php(&mut self) {
        let mut flags = self.flags.to_u8();
        flags |= 0x10;
        self.push(flags);
    }
    fn op_pla(&mut self) -> u8 {
        let value = self.pull();
        self.registers.a = value;
        self.set_flags_zn(value);
        value
    }
    fn op_plp(&mut self) {
        let flags = self.pull();
        self.flags = StatusFlags::from_u8(flags);
        self.flags.interrupt = true;
    }

    // Helper functions
    fn set_flags_zn(&mut self, value: u8) {
        self.flags.zero = value == 0;
        self.flags.sign = (value & 0x80) != 0;
    }

    pub fn get_instruction(&self, opcode: u8) -> InstructionInfo {
        const MODES: [AddressingMode; 256] = [AddressingMode::Implied, AddressingMode::IndirectX, AddressingMode::Implied, AddressingMode::IndirectX, AddressingMode::ZeroPage, AddressingMode::ZeroPage, AddressingMode::ZeroPage, AddressingMode::ZeroPage, AddressingMode::Implied, AddressingMode::Immediate, AddressingMode::Accumulator, AddressingMode::Immediate, AddressingMode::ZeroPage, AddressingMode::Absolute, AddressingMode::Absolute, AddressingMode::Absolute, AddressingMode::Relative, AddressingMode::IndirectY, AddressingMode::Implied, AddressingMode::IndirectY, AddressingMode::ZeroPageX, AddressingMode::ZeroPageX, AddressingMode::ZeroPageX, AddressingMode::ZeroPageX, AddressingMode::Implied, AddressingMode::AbsoluteY, AddressingMode::Implied, AddressingMode::AbsoluteY, AddressingMode::AbsoluteX, AddressingMode::AbsoluteX, AddressingMode::AbsoluteX, AddressingMode::AbsoluteX, AddressingMode::Absolute, AddressingMode::IndirectX, AddressingMode::Implied, AddressingMode::IndirectX, AddressingMode::ZeroPage, AddressingMode::ZeroPage, AddressingMode::ZeroPage, AddressingMode::ZeroPage, AddressingMode::Implied, AddressingMode::Immediate, AddressingMode::Accumulator, AddressingMode::Immediate, AddressingMode::Absolute, AddressingMode::Absolute, AddressingMode::Absolute, AddressingMode::Absolute, AddressingMode::Relative, AddressingMode::IndirectY, AddressingMode::Implied, AddressingMode::IndirectY, AddressingMode::ZeroPageX, AddressingMode::ZeroPageX, AddressingMode::ZeroPageX, AddressingMode::ZeroPageX, AddressingMode::Implied, AddressingMode::AbsoluteY, AddressingMode::Implied, AddressingMode::AbsoluteY, AddressingMode::AbsoluteX, AddressingMode::AbsoluteX, AddressingMode::AbsoluteX, AddressingMode::AbsoluteX, AddressingMode::Implied, AddressingMode::IndirectX, AddressingMode::Implied, AddressingMode::IndirectX, AddressingMode::ZeroPage, AddressingMode::ZeroPage, AddressingMode::ZeroPage, AddressingMode::ZeroPage, AddressingMode::Implied, AddressingMode::Immediate, AddressingMode::Accumulator, AddressingMode::Immediate, AddressingMode::Absolute, AddressingMode::Absolute, AddressingMode::Absolute, AddressingMode::Absolute, AddressingMode::Relative, AddressingMode::IndirectY, AddressingMode::Implied, AddressingMode::IndirectY, AddressingMode::ZeroPageX, AddressingMode::ZeroPageX, AddressingMode::ZeroPageX, AddressingMode::ZeroPageX, AddressingMode::Implied, AddressingMode::AbsoluteY, AddressingMode::Implied, AddressingMode::AbsoluteY, AddressingMode::AbsoluteX, AddressingMode::AbsoluteX, AddressingMode::AbsoluteX, AddressingMode::AbsoluteX, AddressingMode::Implied, AddressingMode::IndirectX, AddressingMode::Implied, AddressingMode::IndirectX, AddressingMode::ZeroPage, AddressingMode::ZeroPage, AddressingMode::ZeroPage, AddressingMode::ZeroPage, AddressingMode::Implied, AddressingMode::Immediate, AddressingMode::Accumulator, AddressingMode::Immediate, AddressingMode::IndirectAbsolute, AddressingMode::Absolute, AddressingMode::Absolute, AddressingMode::Absolute, AddressingMode::Relative, AddressingMode::IndirectY, AddressingMode::Implied, AddressingMode::IndirectY, AddressingMode::ZeroPageX, AddressingMode::ZeroPageX, AddressingMode::ZeroPageX, AddressingMode::ZeroPageX, AddressingMode::Implied, AddressingMode::AbsoluteY, AddressingMode::Implied, AddressingMode::AbsoluteY, AddressingMode::AbsoluteX, AddressingMode::AbsoluteX, AddressingMode::AbsoluteX, AddressingMode::AbsoluteX, AddressingMode::Immediate, AddressingMode::ZeroPageX, AddressingMode::Immediate, AddressingMode::ZeroPageX, AddressingMode::ZeroPage, AddressingMode::ZeroPage, AddressingMode::ZeroPage, AddressingMode::ZeroPage, AddressingMode::Implied, AddressingMode::Immediate, AddressingMode::Implied, AddressingMode::Immediate, AddressingMode::Absolute, AddressingMode::Absolute, AddressingMode::Absolute, AddressingMode::Absolute, AddressingMode::Relative, AddressingMode::IndirectY, AddressingMode::Implied, AddressingMode::IndirectY, AddressingMode::ZeroPageX, AddressingMode::ZeroPageX, AddressingMode::ZeroPageY, AddressingMode::ZeroPageY, AddressingMode::Implied, AddressingMode::AbsoluteY, AddressingMode::Implied, AddressingMode::AbsoluteY, AddressingMode::AbsoluteX, AddressingMode::AbsoluteX, AddressingMode::AbsoluteY, AddressingMode::AbsoluteY, AddressingMode::Immediate, AddressingMode::Immediate, AddressingMode::Immediate, AddressingMode::Immediate, AddressingMode::ZeroPage, AddressingMode::ZeroPage, AddressingMode::ZeroPage, AddressingMode::ZeroPage, AddressingMode::Implied, AddressingMode::Immediate, AddressingMode::Implied, AddressingMode::Immediate, AddressingMode::Absolute, AddressingMode::Absolute, AddressingMode::Absolute, AddressingMode::Absolute, AddressingMode::Relative, AddressingMode::IndirectY, AddressingMode::Implied, AddressingMode::IndirectY, AddressingMode::ZeroPageX, AddressingMode::ZeroPageX, AddressingMode::ZeroPageY, AddressingMode::ZeroPageY, AddressingMode::Implied, AddressingMode::AbsoluteY, AddressingMode::Implied, AddressingMode::AbsoluteY, AddressingMode::AbsoluteX, AddressingMode::AbsoluteX, AddressingMode::AbsoluteY, AddressingMode::AbsoluteY, AddressingMode::Immediate, AddressingMode::ZeroPageX, AddressingMode::Immediate, AddressingMode::ZeroPageX, AddressingMode::ZeroPage, AddressingMode::ZeroPage, AddressingMode::ZeroPage, AddressingMode::ZeroPage, AddressingMode::Implied, AddressingMode::Immediate, AddressingMode::Implied, AddressingMode::Immediate, AddressingMode::Absolute, AddressingMode::Absolute, AddressingMode::Absolute, AddressingMode::Absolute, AddressingMode::Relative, AddressingMode::IndirectY, AddressingMode::Implied, AddressingMode::IndirectY, AddressingMode::ZeroPageX, AddressingMode::ZeroPageX, AddressingMode::ZeroPageX, AddressingMode::ZeroPageX, AddressingMode::Implied, AddressingMode::AbsoluteY, AddressingMode::Implied, AddressingMode::AbsoluteY, AddressingMode::AbsoluteX, AddressingMode::AbsoluteX, AddressingMode::AbsoluteX, AddressingMode::AbsoluteX, AddressingMode::Immediate, AddressingMode::ZeroPageX, AddressingMode::Immediate, AddressingMode::ZeroPageX, AddressingMode::ZeroPage, AddressingMode::ZeroPage, AddressingMode::ZeroPage, AddressingMode::ZeroPage, AddressingMode::Implied, AddressingMode::Immediate, AddressingMode::Implied, AddressingMode::Immediate, AddressingMode::Absolute, AddressingMode::Absolute, AddressingMode::Absolute, AddressingMode::Absolute, AddressingMode::Relative, AddressingMode::IndirectY, AddressingMode::Implied, AddressingMode::IndirectY, AddressingMode::ZeroPageX, AddressingMode::ZeroPageX, AddressingMode::ZeroPageX, AddressingMode::ZeroPageX, AddressingMode::Implied, AddressingMode::AbsoluteY, AddressingMode::Implied, AddressingMode::AbsoluteY, AddressingMode::AbsoluteX, AddressingMode::AbsoluteX, AddressingMode::AbsoluteX, AddressingMode::AbsoluteX, ];

        const OPCODES: [Opcode; 256] = [
            Opcode::BRK, Opcode::ORA, Opcode::KIL, Opcode::SLO, Opcode::NOP, Opcode::ORA, Opcode::ASL, Opcode::SLO,
            Opcode::PHP, Opcode::ORA, Opcode::ASL, Opcode::ANC, Opcode::NOP, Opcode::ORA, Opcode::ASL, Opcode::SLO,
            Opcode::BPL, Opcode::ORA, Opcode::KIL, Opcode::SLO, Opcode::NOP, Opcode::ORA, Opcode::ASL, Opcode::SLO,
            Opcode::CLC, Opcode::ORA, Opcode::NOP, Opcode::SLO, Opcode::NOP, Opcode::ORA, Opcode::ASL, Opcode::SLO,
            Opcode::JSR, Opcode::AND, Opcode::KIL, Opcode::RLA, Opcode::BIT, Opcode::AND, Opcode::ROL, Opcode::RLA,
            Opcode::PLP, Opcode::AND, Opcode::ROL, Opcode::ANC, Opcode::BIT, Opcode::AND, Opcode::ROL, Opcode::RLA,
            Opcode::BMI, Opcode::AND, Opcode::KIL, Opcode::RLA, Opcode::NOP, Opcode::AND, Opcode::ROL, Opcode::RLA,
            Opcode::SEC, Opcode::AND, Opcode::NOP, Opcode::RLA, Opcode::NOP, Opcode::AND, Opcode::ROL, Opcode::RLA,
            Opcode::RTI, Opcode::EOR, Opcode::KIL, Opcode::SRE, Opcode::NOP, Opcode::EOR, Opcode::LSR, Opcode::SRE,
            Opcode::PHA, Opcode::EOR, Opcode::LSR, Opcode::ALR, Opcode::JMP, Opcode::EOR, Opcode::LSR, Opcode::SRE,
            Opcode::BVC, Opcode::EOR, Opcode::KIL, Opcode::SRE, Opcode::NOP, Opcode::EOR, Opcode::LSR, Opcode::SRE,
            Opcode::CLI, Opcode::EOR, Opcode::NOP, Opcode::SRE, Opcode::NOP, Opcode::EOR, Opcode::LSR, Opcode::SRE,
            Opcode::RTS, Opcode::ADC, Opcode::KIL, Opcode::RRA, Opcode::NOP, Opcode::ADC, Opcode::ROR, Opcode::RRA,
            Opcode::PLA, Opcode::ADC, Opcode::ROR, Opcode::ARR, Opcode::JMP, Opcode::ADC, Opcode::ROR, Opcode::RRA,
            Opcode::BVS, Opcode::ADC, Opcode::KIL, Opcode::RRA, Opcode::NOP, Opcode::ADC, Opcode::ROR, Opcode::RRA,
            Opcode::SEI, Opcode::ADC, Opcode::NOP, Opcode::RRA, Opcode::NOP, Opcode::ADC, Opcode::ROR, Opcode::RRA,
            Opcode::NOP, Opcode::STA, Opcode::NOP, Opcode::SAX, Opcode::STY, Opcode::STA, Opcode::STX, Opcode::SAX,
            Opcode::DEY, Opcode::NOP, Opcode::TXA, Opcode::ANE, Opcode::STY, Opcode::STA, Opcode::STX, Opcode::SAX,
            Opcode::BCC, Opcode::STA, Opcode::KIL, Opcode::SHA, Opcode::STY, Opcode::STA, Opcode::STX, Opcode::SAX,
            Opcode::TYA, Opcode::STA, Opcode::TXS, Opcode::TAS, Opcode::SHY, Opcode::STA, Opcode::SHX, Opcode::SHA,
            Opcode::LDY, Opcode::LDA, Opcode::LDX, Opcode::LAX, Opcode::LDY, Opcode::LDA, Opcode::LDX, Opcode::LAX,
            Opcode::TAY, Opcode::LDA, Opcode::TAX, Opcode::LXA, Opcode::LDY, Opcode::LDA, Opcode::LDX, Opcode::LAX,
            Opcode::BCS, Opcode::LDA, Opcode::KIL, Opcode::LAX, Opcode::LDY, Opcode::LDA, Opcode::LDX, Opcode::LAX,
            Opcode::CLV, Opcode::LDA, Opcode::TSX, Opcode::LAS, Opcode::LDY, Opcode::LDA, Opcode::LDX, Opcode::LAX,
            Opcode::CPY, Opcode::CMP, Opcode::NOP, Opcode::DCP, Opcode::CPY, Opcode::CMP, Opcode::DEC, Opcode::DCP,
            Opcode::INY, Opcode::CMP, Opcode::DEX, Opcode::AXS, Opcode::CPY, Opcode::CMP, Opcode::DEC, Opcode::DCP,
            Opcode::BNE, Opcode::CMP, Opcode::KIL, Opcode::DCP, Opcode::NOP, Opcode::CMP, Opcode::DEC, Opcode::DCP,
            Opcode::CLD, Opcode::CMP, Opcode::NOP, Opcode::DCP, Opcode::NOP, Opcode::CMP, Opcode::DEC, Opcode::DCP,
            Opcode::CPX, Opcode::SBC, Opcode::NOP, Opcode::ISC, Opcode::CPX, Opcode::SBC, Opcode::INC, Opcode::ISC,
            Opcode::INX, Opcode::SBC, Opcode::NOP, Opcode::SBC, Opcode::CPX, Opcode::SBC, Opcode::INC, Opcode::ISC,
            Opcode::BEQ, Opcode::SBC, Opcode::KIL, Opcode::ISC, Opcode::NOP, Opcode::SBC, Opcode::INC, Opcode::ISC,
            Opcode::SED, Opcode::SBC, Opcode::NOP, Opcode::ISC, Opcode::NOP, Opcode::SBC, Opcode::INC, Opcode::ISC,
        ];

        const CYCLES: [u8; 256] = [
            7, 6, 2, 8, 3, 3, 5, 5, 3, 2, 2, 2, 4, 4, 6, 6,
            2, 5, 2, 8, 4, 4, 6, 6, 2, 4, 2, 7, 5, 5, 7, 7,
            6, 6, 2, 8, 3, 3, 5, 5, 4, 2, 2, 2, 4, 4, 6, 6,
            2, 5, 2, 8, 4, 4, 6, 6, 2, 4, 2, 7, 5, 5, 7, 7,
            6, 6, 2, 8, 3, 3, 5, 5, 3, 2, 2, 2, 3, 4, 6, 6,
            2, 5, 2, 8, 4, 4, 6, 6, 2, 4, 2, 7, 5, 5, 7, 7,
            6, 6, 2, 8, 3, 3, 5, 5, 4, 2, 2, 2, 5, 4, 6, 6,
            2, 5, 2, 8, 4, 4, 6, 6, 2, 4, 2, 7, 5, 5, 7, 7,
            2, 6, 2, 6, 3, 3, 3, 3, 2, 2, 2, 2, 4, 4, 4, 4,
            2, 6, 2, 6, 4, 4, 4, 4, 2, 5, 2, 5, 5, 5, 5, 5,
            2, 2, 2, 2, 3, 3, 3, 3, 2, 2, 2, 2, 4, 4, 4, 4,
            2, 5, 2, 5, 4, 4, 4, 4, 2, 4, 2, 4, 4, 4, 4, 4,
            2, 6, 2, 6, 3, 3, 3, 3, 2, 2, 2, 2, 4, 4, 4, 4,
            2, 5, 2, 8, 4, 4, 6, 6, 2, 4, 2, 7, 5, 5, 7, 7,
            2, 6, 2, 6, 3, 3, 3, 3, 2, 2, 2, 2, 4, 4, 4, 4,
            2, 5, 2, 8, 4, 4, 6, 6, 2, 4, 2, 7, 5, 5, 7, 7,
        ];

        InstructionInfo {
            opcode: OPCODES[opcode as usize],
            mode: MODES[opcode as usize],
            cycles: CYCLES[opcode as usize],
        }
    }

    pub fn emulate(&mut self) -> u8 {
        let pc = self.registers.pc;
        let opcode = self.load(pc);
        self.registers.pc = self.registers.pc.wrapping_add(1);

        let info = self.get_instruction(opcode);
        let cycles = info.cycles;
        self.cycles += cycles as u64;

        // Calculate effective address (reads operands from current PC)
        // Then advance PC past the operands
        let addr = self.effective_address(info.mode);

        let operand_bytes = match info.mode {
            AddressingMode::Immediate => 1,
            AddressingMode::ZeroPage => 1,
            AddressingMode::ZeroPageX => 1,
            AddressingMode::ZeroPageY => 1,
            AddressingMode::Relative => 1,
            AddressingMode::Absolute => 2,
            AddressingMode::AbsoluteX => 2,
            AddressingMode::AbsoluteY => 2,
            AddressingMode::IndirectX => 1,
            AddressingMode::IndirectY => 1,
            AddressingMode::Accumulator => 0,
            AddressingMode::Implied => 0,
            AddressingMode::IndirectAbsolute => 0,
        };
        self.registers.pc = self.registers.pc + operand_bytes as u16;

        let mut extra_cycles = 0;
        if info.mode == AddressingMode::AbsoluteX || info.mode == AddressingMode::AbsoluteY || info.mode == AddressingMode::IndirectY {
            let base = match info.mode {
                AddressingMode::AbsoluteX | AddressingMode::AbsoluteY => {
                    let lo = self.memory[pc as usize] as u16;
                    let hi = self.memory[(pc as usize + 1) as usize] as u16;
                    lo | (hi << 8)
                }
                _ => {
                    let zero = self.memory[pc as usize] as u8;
                    let lo = self.memory[zero as usize] as u16;
                    let hi = self.memory[(zero + 1) as usize] as u16;
                    lo | (hi << 8)
                }
            };
            if (addr & 0xFF00) != (base & 0xFF00) {
                extra_cycles = 1;
            }
        }

        match info.opcode {
            Opcode::LDA => {
                let value = self.load(addr);
                self.op_lda(value);
            }
            Opcode::LDX => {
                let value = self.load(addr);
                self.op_ldx(value);
            }
            Opcode::LDY => {
                let value = self.load(addr);
                self.op_ldy(value);
            }
            Opcode::STA => {
                self.op_sta(addr);
            }
            Opcode::STX => {
                self.op_stx(addr);
            }
            Opcode::STY => {
                self.op_sty(addr);
            }
            Opcode::TAX => self.op_tax(),
            Opcode::TAY => self.op_tay(),
            Opcode::TSX => self.op_tsx(),
            Opcode::TXA => self.op_txa(),
            Opcode::TXS => self.op_txs(),
            Opcode::TYA => self.op_tya(),
            Opcode::ADC => {
                // addr already calculated at top of emulate function
                let value = self.load(addr);
                self.op_adc(value);
            }
            Opcode::SBC => {
                // addr already calculated at top of emulate function
                let value = self.load(addr);
                self.op_sbc(value);
            }
            Opcode::CMP => {
                // addr already calculated at top of emulate function
                let value = self.load(addr);
                self.op_cmp(value);
            }
            Opcode::CPX => {
                // addr already calculated at top of emulate function
                let value = self.load(addr);
                self.op_cpx(value);
            }
            Opcode::CPY => {
                // addr already calculated at top of emulate function
                let value = self.load(addr);
                self.op_cpy(value);
            }
            Opcode::AND => {
                // addr already calculated at top of emulate function
                let value = self.load(addr);
                self.op_and(value);
            }
            Opcode::EOR => {
                // addr already calculated at top of emulate function
                let value = self.load(addr);
                self.op_eor(value);
            }
            Opcode::ORA => {
                // addr already calculated at top of emulate function
                let value = self.load(addr);
                self.op_ora(value);
            }
            Opcode::ASL => {
                // addr already calculated at top of emulate function
                let value = self.load(addr);
                let result = self.op_asl(value);
                self.write(addr, result);
            }
            Opcode::AslA => self.op_asl_a(),
            Opcode::LSR => {
                // addr already calculated at top of emulate function
                let value = self.load(addr);
                let result = self.op_lsr(value);
                self.write(addr, result);
            }
            Opcode::LsrA => self.op_lsr_a(),
            Opcode::ROL => {
                // addr already calculated at top of emulate function
                let value = self.load(addr);
                let result = self.op_rol(value);
                self.write(addr, result);
            }
            Opcode::RolA => self.op_rol_a(),
            Opcode::ROR => {
                // addr already calculated at top of emulate function
                let value = self.load(addr);
                let result = self.op_ror(value);
                self.write(addr, result);
            }
            Opcode::RorA => self.op_ror_a(),
            Opcode::BIT => {
                // addr already calculated at top of emulate function
                let value = self.load(addr);
                self.op_bit(value);
            }
            Opcode::BCC => self.op_bcc(addr),
            Opcode::BCS => self.op_bcs(addr),
            Opcode::BEQ => self.op_beq(addr),
            Opcode::BNE => self.op_bne(addr),
            Opcode::BPL => self.op_bpl(addr),
            Opcode::BMI => self.op_bmi(addr),
            Opcode::BVC => self.op_bvc(addr),
            Opcode::BVS => self.op_bvs(addr),
            Opcode::JSR => {
                // addr already calculated at top of emulate function
                self.op_jsr(addr);
            }
            Opcode::RTS => self.op_rts(),
            Opcode::RTI => self.op_rti(),
            Opcode::BRK => self.op_brk(),
            Opcode::NOP => self.op_nop(),
            Opcode::SEC => self.op_sec(),
            Opcode::CLC => self.op_clc(),
            Opcode::SED => self.op_sed(),
            Opcode::CLD => self.op_cld(),
            Opcode::SEI => self.op_sei(),
            Opcode::CLI => self.op_cli(),
            Opcode::CLV => self.op_clv(),
            Opcode::PHA => self.op_pha(),
            Opcode::PHP => self.op_php(),
            Opcode::PLA => {
                self.op_pla();
            }
            Opcode::PLP => self.op_plp(),
            Opcode::JMP => {
                self.registers.pc = addr;
            }
            _ => {}
        }

        cycles + extra_cycles
    }
}

impl Default for CPU {
    fn default() -> Self {
        Self::new()
    }
}