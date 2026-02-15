# NES Emulator Documentation

**Source:** JSNES (https://github.com/bfirsh/jsnes)
**Version:** 1.2.1
**Author:** Ben Firshman
**License:** Apache-2.0

---

## Table of Contents

1. [Overview](#overview)
2. [Architecture](#architecture)
3. [Component Specifications](#component-specifications)
4. [CPU (6502) Implementation](#cpu-6502-implementation)
5. [PPU (Picture Processing Unit)](#ppu-picture-processing-unit)
6. [APU (Audio Processing Unit)](#apu-audio-processing-unit)
7. [Memory Map](#memory-map)
8. [ROM Format](#rom-format)
9. [Mapper Implementations](#mapper-implementations)
10. [Controller Implementation](#controller-implementation)
11. [Timing and Synchronization](#timing-and-synchronization)
12. [Implementation Checklist](#implementation-checklist)

---

## Overview

This document provides ultra-detailed specifications for building a NES (Nintendo Entertainment System) emulator. The specifications are derived from the JSNES JavaScript implementation, which is a fully functional NES emulator that runs in both browser and Node.js environments.

### NES System Overview

The NES is an 8-bit home video game console released by Nintendo in 1985. Key specifications:

- **CPU:** Ricoh 2A03 (6502-based) running at ~1.79 MHz (NTSC) / ~1.66 MHz (PAL)
- **PPU:** Ricoh 2C02 picture processing unit
- **Audio:** 5-channel audio processor (2 square, 1 triangle, 1 noise, 1 DMC)
- **RAM:** 2KB internal RAM
- **VRAM:** 2KB internal RAM (32KB addressable with external)
- **Game Cartridge:** Contains PRG-ROM (program) and optional CHR-ROM (graphics)

### Emulator Goals

To create a NES emulator, you need to accurately simulate:
1. **6502 CPU** with all 256 opcodes and 13 addressing modes
2. **PPU** with proper timing, rendering pipeline, and sprite handling
3. **APU** with 5 audio channels and frame counter
4. **Memory mapping** including ROM loading and mapper support
5. **Input handling** for controllers and light gun
6. **Audio output** with proper sample rate conversion

---

## Architecture

The JSNES emulator is organized into the following core components:

```
jsnes/
├── src/
│   ├── index.js          # Main exports (NES, Controller, GameGenie)
│   ├── nes.js            # Main NES class (orchestrator)
│   ├── cpu.js            # 6502 CPU emulator
│   ├── rom.js            # ROM loading and parsing
│   ├── controller.js     # Input controller
│   ├── gamegenie.js      # Cheat code support
│   ├── tile.js           # 8x8 tile rendering
│   ├── utils.js          # Utility functions
│   ├── ppu/
│   │   ├── index.js      # Main PPU class
│   │   ├── nametable.js  # Nametable data structure
│   │   └── palette-table.js # Color palette management
│   ├── papu/
│   │   ├── index.js      # Main APU class
│   │   ├── channel-square.js   # Square wave channels
│   │   ├── channel-triangle.js # Triangle wave channel
│   │   ├── channel-noise.js    # Noise channel
│   │   └── channel-dm.js       # DMC channel
│   └── mappers/
│       ├── 0.js          # NoMapper (Mapper 0)
│       ├── 1.js          # MMC1
│       ├── 2.js          # UNROM
│       ├── 3.js          # CNROM
│       ├── 4.js          # MMC3
│       └── ... (16 total mappers)
└── test/                 # Test suite
```

### Component Dependencies

```
NES (main orchestrator)
├── CPU (6502 emulator)
├── PPU (Picture Processing Unit)
│   ├── Nametable (background tile map)
│   └── PaletteTable (color palette)
├── PAPU (Audio Processing Unit)
│   ├── SquareWave1
│   ├── SquareWave2
│   ├── TriangleWave
│   ├── Noise
│   └── DMC (Delta Modulation Channel)
├── ROM ( cartridge data)
└── Mapper (memory mapping, varies by ROM)
```

---

## Component Specifications

### NES Class (`src/nes.js`)

The main emulator class that orchestrates all components.

#### Constructor Options

```javascript
const nes = new NES({
  onFrame: function(frameBuffer) {},   // Called with 256x240 frame buffer
  onAudioSample: function(left, right) {}, // Called for each audio sample
  onStatusUpdate: function(status) {},   // Status updates
  onBatteryRamWrite: function(address, value) {}, // Battery RAM save callback
  preferredFrameRate: 60,      // Target frame rate (default: 60)
  emulateSound: true,          // Enable sound emulation
  sampleRate: 48000,           // Audio sample rate
});
```

#### Key Methods

| Method | Description |
|--------|-------------|
| `loadROM(data)` | Load a ROM from raw data (Uint8Array or string) |
| `frame()` | Run one complete frame (60fps) |
| `reset()` | Reset the entire system |
| `buttonDown(controller, button)` | Simulate button press (1-2, BUTTON_A etc.) |
| `buttonUp(controller, button)` | Simulate button release |
| `zapperMove(x, y)` | Set light gun position (0-255, 0-239) |
| `zapperFireDown()` | Simulate light gun trigger press |
| `zapperFireUp()` | Simulate light gun trigger release |
| `getFPS()` | Get frames per second (resetting counter) |
| `toJSON()` | Serialize emulator state to JSON |
| `fromJSON(state)` | Restore emulator state from JSON |

#### Frame Loop Implementation

The NES class implements cycle-accurate CPU/PPU synchronization with a 3:1 CPU:PPU ratio:

```javascript
// Key variables for synchronization:
- cpu.cyclesToHalt: How many CPU cycles to run before PPU catchup
- cpu.ppuCatchupDots: PPU dots to catch up
- cpu.apuCatchupCycles: APU cycles to catch up
- ppu.curX: Current X position in frame (0-340)
```

**Timing relationship:**
- CPU runs at 1.79 MHz (NTSC)
- PPU runs at 5.37 MHz (NTSC) = 3x CPU speed
- 3 CPU cycles = 1 PPU dot
- 341 dots per scanline (NTSC)
- 262 scanlines per frame (NTSC)

---

## CPU (6502) Implementation

### CPU Registers

The 6502 CPU has the following registers:

| Register | Size | Description |
|----------|------|-------------|
| `A` | 8-bit | Accumulator |
| `X` | 8-bit | Index register X |
| `Y` | 8-bit | Index register Y |
| `SP` | 8-bit | Stack pointer (0x00-0xFF, maps to $0100-$01FF) |
| `PC` | 16-bit | Program counter |
| `Status` | 8-bit | Processor status flags |

### Status Flags

| Bit | Flag | Description |
|-----|------|-------------|
| 7 | N (Negative) | Set if result is negative (bit 7 = 1) |
| 6 | V (Overflow) | Set if signed arithmetic overflow occurs |
| 5 | Unused | Always reads as 1 |
| 4 | B (Break) | Set on BRK instruction |
| 3 | D (Decimal) | Set for decimal mode BCD operations |
| 2 | I (Interrupt) | Set to disable IRQ interrupts |
| 1 | Z (Zero) | Set if result is zero |
| 0 | C (Carry) | Set if arithmetic carry/borrow occurs |

### Memory Map

The 6502 has a 64KB address space:

| Range | Description |
|-------|-------------|
| $0000-$07FF | CPU RAM (2KB, mirrored to $0800-$0FFF) |
| $0800-$0FFF | CPU RAM mirror |
| $1000-$1FFF | CPU RAM mirror |
| $2000-$2007 | PPU registers (mirrored every 8 bytes) |
| $2008-$3FFF | PPU registers mirror ($2000-$2007 repeated) |
| $4000-$4013 | APU registers |
| $4014 | OAM DMA register |
| $4015 | APU channel enable |
| $4016 | Controller 1 strobe |
| $4017 | Controller 2 strobe / Frame counter |
| $4018-$5FFF | Unusable (expansion port) |
| $6000-$7FFF | Battery-backed RAM (if present) |
| $8000-$FFFF | PRG-ROM (16KB or 32KB) |

### Addressing Modes

The 6502 supports 13 addressing modes:

| Mode | Code | Operand Bytes | Description |
|------|------|---------------|-------------|
| Implied | 2 | 0 | No operand (e.g., NOP, CLC) |
| Accumulator | 4 | 0 | Uses A register (e.g., ASL A) |
| Immediate | 5 | 1 | Value is in next byte (e.g., LDA #$FF) |
| Zero Page | 0 | 1 | Address in page 0 ($00-$FF) |
| Zero Page,X | 6 | 1 | Zero page + X (e.g., LDA $10,X) |
| Zero Page,Y | 7 | 1 | Zero page + Y (e.g., LDA $10,Y) |
| Absolute | 3 | 2 | Full 16-bit address |
| Absolute,X | 8 | 2 | Absolute + X (page crossed = +1 cycle) |
| Absolute,Y | 9 | 2 | Absolute + Y (page crossed = +1 cycle) |
| Preindexed Indirect | 10 | 1 | (operand + X) points to address |
| Postindexed Indirect | 11 | 1 | operand points to address, then + Y |
| Relative | 1 | 1 | Branch offset (signed 8-bit) |
| Indirect Absolute | 12 | 2 | (address) contains target address |

### Instruction Set

The 6502 has 256 possible opcodes, of which 151 are used by the NES. JSNES implements all 256 opcodes including unofficial/undocumented instructions.

#### Data Transfer Instructions

| Opcode | Mnemonic | Description | Cycles |
|--------|----------|-------------|--------|
| $A9 | LDA #imm | Load accumulator immediate | 2 |
| $A2 | LDX #imm | Load X register immediate | 2 |
| $A0 | LDY #imm | Load Y register immediate | 2 |
| $85 | STA zpg | Store accumulator zero page | 3 |
| $86 | STX zpg | Store X register zero page | 3 |
| $84 | STY zpg | Store Y register zero page | 3 |
| $A5 | LDA zpg | Load accumulator zero page | 3 |
| $B5 | LDA zpg,X | Load accumulator zero page, X | 4 |
| $B6 | LDX zpg,Y | Load X register zero page, Y | 4 |
| $BC | LDY abs,X | Load Y register absolute, X | 4+ |
| $AC | LDY abs | Load Y register absolute | 4 |
| $AD | LDA abs | Load accumulator absolute | 4 |
| $BD | LDA abs,X | Load accumulator absolute, X | 4+ |
| $B9 | LDA abs,Y | Load accumulator absolute, Y | 4+ |
| $A6 | LDA zpg,X | Load X register zero page, Y | 4 |
| $BE | LDX abs,Y | Load X register absolute, Y | 4+ |
| $A4 | LDY zpg | Load Y register zero page | 3 |
| $B4 | LDY zpg,X | Load Y register zero page, X | 4 |
| $95 | STA zpg,X | Store accumulator zero page, X | 4 |
| $96 | STX zpg,Y | Store X register zero page, Y | 4 |
| $94 | STY zpg,X | Store Y register zero page, X | 4 |
| $8D | STA abs | Store accumulator absolute | 4 |
| $9D | STA abs,X | Store accumulator absolute, X | 5 |
| $99 | STA abs,Y | Store accumulator absolute, Y | 5 |
| $AA | TAX | Transfer A to X | 2 |
| $A8 | TAY | Transfer A to Y | 2 |
| $BA | TSX | Transfer S to X | 2 |
| $8A | TXA | Transfer X to A | 2 |
| $9A | TXS | Transfer X to S | 2 |
| $98 | TYA | Transfer Y to A | 2 |

#### Arithmetic Instructions

| Opcode | Mnemonic | Description | Cycles |
|--------|----------|-------------|--------|
| $69 | ADC #imm | Add with carry | 2 |
| $65 | ADC zpg | Add with carry zero page | 3 |
| $75 | ADC zpg,X | Add with carry zero page, X | 4 |
| $6D | ADC abs | Add with carry absolute | 4 |
| $7D | ADC abs,X | Add with carry absolute, X | 4+ |
| $79 | ADC abs,Y | Add with carry absolute, Y | 4+ |
| $61 | ADC (zpg,X) | Add with carry preindexed indirect | 6 |
| $71 | ADC (zpg),Y | Add with carry postindexed indirect | 5+ |
| $E9 | SBC #imm | Subtract with carry | 2 |
| $E5 | SBC zpg | Subtract with carry zero page | 3 |
| $F5 | SBC zpg,X | Subtract with carry zero page, X | 4 |
| $ED | SBC abs | Subtract with carry absolute | 4 |
| $FD | SBC abs,X | Subtract with carry absolute, X | 4+ |
| $F9 | SBC abs,Y | Subtract with carry absolute, Y | 4+ |
| $E1 | SBC (zpg,X) | Subtract with carry preindexed indirect | 6 |
| $F1 | SBC (zpg),Y | Subtract with carry postindexed indirect | 5+ |
| $C9 | CMP #imm | Compare accumulator | 2 |
| $C5 | CMP zpg | Compare accumulator zero page | 3 |
| $D5 | CMP zpg,X | Compare accumulator zero page, X | 4 |
| $CD | CMP abs | Compare accumulator absolute | 4 |
| $DD | CMP abs,X | Compare accumulator absolute, X | 4+ |
| $D9 | CMP abs,Y | Compare accumulator absolute, Y | 4+ |
| $C1 | CMP (zpg,X) | Compare accumulator preindexed indirect | 6 |
| $D1 | CMP (zpg),Y | Compare accumulator postindexed indirect | 5+ |
| $E0 | CPX #imm | Compare X register immediate | 2 |
| $E4 | CPX zpg | Compare X register zero page | 3 |
| $EC | CPX abs | Compare X register absolute | 4 |
| $C0 | CPY #imm | Compare Y register immediate | 2 |
| $C4 | CPY zpg | Compare Y register zero page | 3 |
| $CC | CPY abs | Compare Y register absolute | 4 |

#### Logic Instructions

| Opcode | Mnemonic | Description | Cycles |
|--------|----------|-------------|--------|
| $29 | AND #imm | AND accumulator | 2 |
| $25 | AND zpg | AND accumulator zero page | 3 |
| $35 | AND zpg,X | AND accumulator zero page, X | 4 |
| $2D | AND abs | AND accumulator absolute | 4 |
| $3D | AND abs,X | AND accumulator absolute, X | 4+ |
| $39 | AND abs,Y | AND accumulator absolute, Y | 4+ |
| $21 | AND (zpg,X) | AND accumulator preindexed indirect | 6 |
| $31 | AND (zpg),Y | AND accumulator postindexed indirect | 5+ |
| $49 | EOR #imm | EOR accumulator | 2 |
| $45 | EOR zpg | EOR accumulator zero page | 3 |
| $55 | EOR zpg,X | EOR accumulator zero page, X | 4 |
| $4D | EOR abs | EOR accumulator absolute | 4 |
| $5D | EOR abs,X | EOR accumulator absolute, X | 4+ |
| $59 | EOR abs,Y | EOR accumulator absolute, Y | 4+ |
| $41 | EOR (zpg,X) | EOR accumulator preindexed indirect | 6 |
| $51 | EOR (zpg),Y | EOR accumulator postindexed indirect | 5+ |
| $09 | ORA #imm | OR accumulator | 2 |
| $05 | ORA zpg | OR accumulator zero page | 3 |
| $15 | ORA zpg,X | OR accumulator zero page, X | 4 |
| $0D | ORA abs | OR accumulator absolute | 4 |
| $1D | ORA abs,X | OR accumulator absolute, X | 4+ |
| $19 | ORA abs,Y | OR accumulator absolute, Y | 4+ |
| $01 | ORA (zpg,X) | OR accumulator preindexed indirect | 6 |
| $11 | ORA (zpg),Y | OR accumulator postindexed indirect | 5+ |

#### Bit Manipulation Instructions

| Opcode | Mnemonic | Description | Cycles |
|--------|----------|-------------|--------|
| $0A | ASL A | Arithmetic shift left accumulator | 2 |
| $06 | ASL zpg | Arithmetic shift left zero page | 5 |
| $16 | ASL zpg,X | Arithmetic shift left zero page, X | 6 |
| $0E | ASL abs | Arithmetic shift left absolute | 6 |
| $1E | ASL abs,X | Arithmetic shift left absolute, X | 7 |
| $4A | LSR A | Logical shift right accumulator | 2 |
| $46 | LSR zpg | Logical shift right zero page | 5 |
| $56 | LSR zpg,X | Logical shift right zero page, X | 6 |
| $4E | LSR abs | Logical shift right absolute | 6 |
| $5E | LSR abs,X | Logical shift right absolute, X | 7 |
| $2A | ROL A | Rotate left accumulator | 2 |
| $26 | ROL zpg | Rotate left zero page | 5 |
| $36 | ROL zpg,X | Rotate left zero page, X | 6 |
| $2E | ROL abs | Rotate left absolute | 6 |
| $3E | ROL abs,X | Rotate left absolute, X | 7 |
| $6A | ROR A | Rotate right accumulator | 2 |
| $66 | ROR zpg | Rotate right zero page | 5 |
| $76 | ROR zpg,X | Rotate right zero page, X | 6 |
| $6E | ROR abs | Rotate right absolute | 6 |
| $7E | ROR abs,X | Rotate right absolute, X | 7 |
| $24 | BIT zpg | Test bits zero page | 3 |
| $2C | BIT abs | Test bits absolute | 4 |

#### Branch Instructions

| Opcode | Mnemonic | Description | Cycles |
|--------|----------|-------------|--------|
| $90 | BCC rel | Branch if carry clear | 2/3+ |
| $B0 | BCS rel | Branch if carry set | 2/3+ |
| $F0 | BEQ rel | Branch if equal (zero set) | 2/3+ |
| $D0 | BNE rel | Branch if not equal (zero clear) | 2/3+ |
| $10 | BPL rel | Branch if positive | 2/3+ |
| $30 | BMI rel | Branch if negative | 2/3+ |
| $50 | BVC rel | Branch if overflow clear | 2/3+ |
| $70 | BVS rel | Branch if overflow set | 2/3+ |

#### Subroutine Instructions

| Opcode | Mnemonic | Description | Cycles |
|--------|----------|-------------|--------|
| $20 | JSR abs | Jump to subroutine | 6 |
| $60 | RTS | Return from subroutine | 6 |
| $40 | RTI | Return from interrupt | 6 |

#### Interrupt Instructions

| Opcode | Mnemonic | Description | Cycles |
|--------|----------|-------------|--------|
| $00 | BRK | Software interrupt | 7 |
| $40 | RTI | Return from interrupt | 6 |

#### Status Flag Instructions

| Opcode | Mnemonic | Description | Cycles |
|--------|----------|-------------|--------|
| $38 | SEC | Set carry | 2 |
| $18 | CLC | Clear carry | 2 |
| $F8 | SED | Set decimal | 2 |
| $D8 | CLD | Clear decimal | 2 |
| $78 | SEI | Set interrupt | 2 |
| $58 | CLI | Clear interrupt | 2 |
| $B8 | CLV | Clear overflow | 2 |

#### Stack Instructions

| Opcode | Mnemonic | Description | Cycles |
|--------|----------|-------------|--------|
| $48 | PHA | Push accumulator | 3 |
| $08 | PHP | Push processor status | 3 |
| $68 | PLA | Pull accumulator | 4 |
| $28 | PLP | Pull processor status | 4 |

#### No Operation

| Opcode | Mnemonic | Description | Cycles |
|--------|----------|-------------|--------|
| $EA | NOP | No operation | 2 |

### Unofficial/Undocumented Instructions

JSNES also implements many unofficial 6502 opcodes (often called "illegal" instructions):

| Opcode | Mnemonic | Description |
|--------|----------|-------------|
| $0B | ANC #imm | AND with accumulator, set carry |
| $2B | ALR #imm | AND then LSR |
| $8B | ANC #imm | AND with accumulator, set carry |
| $6B | ARR #imm | AND then ROR |
| $CB | AXS #imm | AND X, subtract from X |
| $1B | ORA (zpg),Y | KIL |
| $3B | ORA (zpg),Y | KIL |
| $5B | ORA (zpg),Y | KIL |
| $7B | ORA (zpg),Y | KIL |
| $9B | ORA (zpg),Y | KIL |
| $BB | ORA (zpg),Y | KIL |
| $DB | ORA (zpg),Y | KIL |
| $FB | ORA (zpg),Y | KIL |
| $AB | LXA #imm | Load A and X |
| $BB | ORA (abs),Y | KIL |
| $CF | RLA zpg | Rotate left then AND |
| $DF | RLA abs | Rotate left then AND |
| $BF | RLA abs,Y | Rotate left then AND |
| $AF | LAX zpg | Load A and X |
| $BF | LAX abs,Y | Load A and X |
| $2F | RLA zpg | Rotate left then AND |
| $3F | RLA abs | Rotate left then AND |
| $3B | ORA (zpg),Y | KIL |
| $EB | SBC #imm |Alternate SBC |
| $C7 | DCP zpg | Decrement then compare |
| $D7 | DCP zpg,X | Decrement then compare |
| $CF | DCP abs | Decrement then compare |
| $DB | DCP abs,X | Decrement then compare |
| $FB | DCP abs,Y | Decrement then compare |
| $E7 | ISC zpg | Increment then subtract |
| $F7 | ISC zpg,X | Increment then subtract |
| $EF | ISC abs | Increment then subtract |
| $FF | ISC abs,X | Increment then subtract |
| $FF | ISC abs,Y | Increment then subtract |
| $67 | RRA zpg | Rotate right then add |
| $77 | RRA zpg,X | Rotate right then add |
| $6F | RRA abs | Rotate right then add |
| $7F | RRA abs,X | Rotate right then add |
| $7B | RRA abs,Y | Rotate right then add |
| $07 | SLO zpg | Shift left then OR |
| $17 | SLO zpg,X | Shift left then OR |
| $0F | SLO abs | Shift left then OR |
| $1F | SLO abs,X | Shift left then OR |
| $1B | SLO abs,Y | Shift left then OR |
| $47 | SRE zpg | Shift right then EOR |
| $57 | SRE zpg,X | Shift right then EOR |
| $4F | SRE abs | Shift right then EOR |
| $5F | SRE abs,X | Shift right then EOR |
| $5B | SRE abs,Y | Shift right then EOR |

### CPU State Serialization

The CPU implements `toJSON()` and `fromJSON()` for save states:

```javascript
// JSON properties for serialization
JSON_PROPERTIES = [
  'REG_ACC',    // Accumulator
  'REG_X',      // X register
  'REG_Y',      // Y register
  'REG_SP',     // Stack pointer
  'REG_PC',     // Program counter
  'F_CARRY',    // Carry flag
  'F_ZERO',     // Zero flag
  'F_INTERRUPT',// Interrupt flag
  'F_DECIMAL',  // Decimal flag
  'F_OVERFLOW', // Overflow flag
  'F_SIGN',     // Sign flag
  'dataBus',    // Data bus latch
  'mem',        // 64KB memory array
];
```

---

## PPU (Picture Processing Unit)

### PPU Overview

The PPU (Picture Processing Unit) is responsible for rendering graphics. Key specifications:

- **Clock speed:** 5.37 MHz (NTSC) / 5.0 MHz (PAL) = 3x CPU speed
- **VRAM:** 2KB internal + nametables
- **Sprite RAM:** 256 bytes (OAM)
- **Palette RAM:** 32 bytes
- **Output:** 256x240 pixels (NTSC)

### PPU Registers

| Address | Name | Access | Description |
|---------|------|--------|-------------|
| $2000 | PPUCTRL | Write | Control register 1 |
| $2001 | PPUMASK | Write | Control register 2 |
| $2002 | PPUSTATUS | Read | Status register |
| $2003 | OAMADDR | Write | OAM address |
| $2004 | OAMDATA | Read/Write | OAM data |
| $2005 | PPUSCROLL | Write | Scroll register (2-byte write) |
| $2006 | PPUADDR | Write | VRAM address (2-byte write) |
| $2007 | PPUDATA | Read/Write | VRAM data |
| $4014 | OAMDMA | Write | OAM DMA transfer |

### PPUCTRL ($2000) - Control Register 1

```
Bit 7: NMI on VBlank enable (1=enabled)
Bit 6: Unused (read as 0)
Bit 5: Sprite size (0=8x8, 1=8x16)
Bit 4: Background pattern table (0=$0000, 1=$1000)
Bit 3: Sprite pattern table (0=$0000, 1=$1000)
Bit 2: Address increment (0=+1, 1=+32)
Bit 1-0: Nametable select (00=$2000, 01=$2400, 10=$2800, 11=$2C00)
```

### PPUMASK ($2001) - Control Register 2

```
Bit 7: Unused (read as 0)
Bit 6: Unused (read as 0)
Bit 5-7: Color emphasis (red, green, blue - 3 bits)
Bit 4: Sprite visibility (1=visible)
Bit 3: Background visibility (1=visible)
Bit 2: Sprite clipping (0=clip left 8 pixels on TV)
Bit 1: Background clipping (0=clip left 8 pixels on TV)
Bit 0: Display type (0=color, 1=monochrome)
```

### PPUSTATUS ($2002) - Status Register

```
Bit 7: VBlank flag (1=VBlank in progress)
Bit 6: Sprite 0 hit flag
Bit 5: Sprite overflow flag
Bit 4-0: Open bus latch (lower 5 bits)
```

### VRAM Map

```
$0000-$0FFF: Pattern table 0 (4KB) - Tile data for $0000-$0FFF
$1000-$1FFF: Pattern table 1 (4KB) - Tile data for $1000-$1FFF
$2000-$23FF: Nametable 0 (1KB)
$2400-$27FF: Nametable 1 (1KB)
$2800-$2BFF: Nametable 2 (1KB)
$2C00-$2FFF: Nametable 3 (1KB)
$3000-$33FF: Nametable 0 mirror
$3400-$37FF: Nametable 1 mirror
$3800-$3BFF: Nametable 2 mirror
$3C00-$3FFF: Nametable 3 mirror
$3F00-$3F1F: Palette memory (32 bytes)
$3F20-$3FFF: Palette mirror
```

### Palette Colors ($3F00-$3F1F)

```
$3F00-$3F0F: Background palette (16 colors)
$3F10-$3F1F: Sprite palette (16 colors)
```

Each entry is 6 bits (0-63), stored as RGB555 format internally.

### NTSC Palette Colors (RGB555 format)

The NES has 64 possible colors (54 visible, 12 black). The JSNES implementation uses these RGB values:

```javascript
// Palette indices 0-15: Background palette 0
// Palette indices 16-31: Background palette 1
// Palette indices 32-47: Background palette 2
// Palette indices 48-63: Background palette 3

// Example color values (RGB):
0x525252, 0xB40000, 0xA00000, 0xB1003D, 0x740069, 0x00005B, 0x00005F, 0x001840,
0x002F10, 0x084A08, 0x006700, 0x124200, 0x6D2800, 0x000000, 0x000000, 0x000000,
0xC4D5E7, 0xFF4000, 0xDC0E22, 0xFF476B, 0xD7009F, 0x680AD7, 0x0019BC, 0x0054B1,
0x006A5B, 0x008C03, 0x00AB00, 0x2C8800, 0xA47200, 0x000000, 0x000000, 0x000000,
0xF8F8F8, 0xFFAB3C, 0xFF7981, 0xFF5BC5, 0xFF48F2, 0xDF49FF, 0x476DFF, 0x00B4F7,
0x00E0FF, 0x00E375, 0x03F42B, 0x78B82E, 0xE5E218, 0x787878, 0x000000, 0x000000,
0xFFFFFF, 0xFFF2BE, 0xF8B8B8, 0xF8B8D8, 0xFFB6FF, 0xFFC3FF, 0xC7D1FF, 0x9ADAFF,
0x88EDF8, 0x83FFDD, 0xB8F8B8, 0xF5F8AC, 0xFFFFB0, 0xF8D8F8, 0x000000, 0x000000
```

### Color Emphasis

Bits 5-7 of $2001 control color emphasis (shadow/highlight effects):

```
Bit 7: Blue emphasis (dim blue)
Bit 6: Green emphasis (dim green)
Bit 5: Red emphasis (dim red)
```

When set, that color component is reduced to 75% intensity.

### Rendering Pipeline

#### Frame Structure (NTSC)

```
Scanline 0-19:  Pre-render scanlines (hidden, for timing)
Scanline 20:    VBlank start (clears flags, resets counters)
Scanline 21-260: Active rendering (240 visible scanlines)
Scanline 261:   VBlank (NMI triggered, CPU can access VRAM)
```

#### Timing Details

- Each scanline has 341 dots (NTSC)
- Each CPU instruction takes 3 dots
- Frame time: ~16.6ms (60fps)

#### Rendering Steps

1. **Pre-render scanline (scanline 0-19):** Hidden, timing preparation
2. **Scanline 20:** VBlank begins, flags cleared, counters reset
3. **Active scanlines (21-260):** Render background and sprites
4. **Scanline 261:** VBlank active, NMI triggered if enabled

### Background Rendering

Background rendering uses the following components:

1. **Nametables:** 32x30 tile grid storing tile indices
2. **Attribute Tables:** 32x30 tile grid storing palette selection (4x4 tile blocks)
3. **Pattern Tables:** 4KB storage for 8x8 pixel tile graphics
4. **Scroll Registers:** FV (fine Y), V (vertical), H (horizontal), VT (vertical tile), HT (horizontal tile)

#### Tile Format

Each 8x8 pixel tile requires 16 bytes:
- Bytes 0-7: Low bits for each pixel
- Bytes 8-15: High bits for each pixel

Pixel value = low_bit + (high_bit << 1), giving values 0-3:
- 0: Transparent (palette 0)
- 1: Palette 1
- 2: Palette 2
- 3: Palette 3

### Sprite OAM (Object Attribute Memory)

OAM is 256 bytes storing 64 sprites (4 bytes each):

```
Sprite Entry Format:
Byte 0 (Y): Sprite Y position (0-255, -1 for off-top)
Byte 1 (Tile): Tile index into pattern table
Byte 2 (Attributes):
  Bit 7: Vertical flip
  Bit 6: Horizontal flip
  Bit 5: Priority (0=under BG, 1=over BG)
  Bit 4-0: Unused
Byte 3 (X): Sprite X position (0-255)
```

#### OAM Operations

- **$2003:** Set OAM address (0-255)
- **$2004:** Read/write OAM data at current address (auto-increments)
- **$4014:** OAM DMA transfer (256 bytes from CPU memory)

### Sprite 0 Hit Detection

Sprite 0 hit is a hardware feature for collision detection:

**Conditions for Sprite 0 Hit:**
1. Sprite 0 (first sprite in OAM) is visible
2. Sprite 0's Y position brackets current scanline
3. Sprite 0's X position brackets current X position
4. A non-transparent pixel in Sprite 0 overlaps a non-transparent background pixel

**Usage in Games:**
- Raster effects: Change colors/palettes mid-frame
- Score updates: Update display while game runs
- Timing synchronization

### Nametable Mirroring

The NES supports 5 mirroring modes:

| Mode | Constant | Description |
|------|----------|-------------|
| 0 | Horizontal | Nametables 0,1 = NT0; Nametables 2,3 = NT1 (side-by-side) |
| 1 | Vertical | Nametables 0,2 = NT0; Nametables 1,3 = NT1 (stacked) |
| 2 | Four-Screen | Each nametable separate (requires cartridge support) |
| 3 | Single-Screen | All nametables map to NT0 |
| 4 | Single-Screen 2 | All nametables map to NT1 |

### PPU State Serialization

```javascript
// JSON properties for PPU serialization
JSON_PROPERTIES = [
  'openBusLatch',        // Open bus value
  'openBusDecayFrames',  // Frames until bus decays
  'firstWrite',          // First write flag for two-byte registers
  'vramAddress',         // Current VRAM address
  'vramBufferedValue',   // Buffered VRAM read value
  'curX',                // Current X position in frame
  'scanline',            // Current scanline number
  'f_nmiOnVblank',       // NMI on VBlank enable
  'f_spriteSize',        // Sprite size (0=8x8, 1=8x16)
  'f_bgPatternTable',    // Background pattern table base
  'f_spPatternTable',    // Sprite pattern table base
  'f_addressIncrement',  // Address increment (0=+1, 1=+32)
  'f_nametableSelect',   // Nametable base address
  'f_spVisibility',      // Sprite visibility
  'f_bgVisibility',      // Background visibility
  'f_spClipping',        // Sprite clipping
  'f_bgClipping',        // Background clipping
  'f_displayType',       // Display type (0=color, 1=mono)
  'f_emphasis',          // Color emphasis bits
  'requestEndFrame',     // Request to end frame
  'nmiCounter',          // NMI counter
  'spriteOverflow',      // Sprite overflow flag
  'spr0Hit',             // Sprite 0 hit flag
  'spr0HitX',            // X position of sprite 0 hit
  'spr0HitY',            // Y position of sprite 0 hit
  'vramMem',             // 32KB VRAM
  'spriteMem',           // 256-byte OAM
  'ptTile',              // 256 tiles for pattern table
  'nameTable',           // 4 nametables
  'paletteTable',        // 64-color palette
  // Sprite state arrays (64 sprites each)
  'sprX', 'sprY', 'sprTile', 'sprCol',
  'vertFlip', 'horiFlip', 'bgPriority',
];
```

---

## APU (Audio Processing Unit)

### APU Overview

The APU (Audio Processing Unit) generates audio through 5 channels:

| Channel | Type | Description |
|---------|------|-------------|
| 1 | Square Wave | Pulse width modulation, sweep support |
| 2 | Square Wave | Pulse width modulation, sweep support |
| 3 | Triangle Wave | Linear counter, fixed length |
| 4 | Noise | 15-bit shift register, two modes |
| 5 | DMC | Delta Modulation Channel (sample playback) |

### APU Registers

| Address | Name | Description |
|---------|------|-------------|
| $4000 | Square 1 Control | Envelope decay, duty mode, length enable |
| $4001 | Square 1 Sweep | Sweep period, add/sub, shift amount |
| $4002 | Square 1 Timer Low | Timer low byte |
| $4003 | Square 1 Timer High | Timer high (3 bits), length load (5 bits) |
| $4004 | Square 2 Control | Same as $4000 |
| $4005 | Square 2 Sweep | Same as $4001 |
| $4006 | Square 2 Timer Low | Same as $4002 |
| $4007 | Square 2 Timer High | Same as $4003 |
| $4008 | Triangle Control | Linear counter control, length load |
| $4009 | Triangle Unused | Unused, read as 0 |
| $400A | Triangle Timer Low | Timer low byte |
| $400B | Triangle Timer High | Timer high (3 bits), length load (5 bits) |
| $400C | Noise Control | Envelope decay, loop, length enable |
| $400D | Noise Unused | Unused, read as 0 |
| $400E | Noise Wavelength | Wavelength (4 bits), random mode (1 bit) |
| $400F | Noise Length | Length load (5 bits) |
| $4010 | DMC Control | Play mode (2 bits), frequency (4 bits) |
| $4011 | DMC DAC | Delta counter (6 bits), DAC LSB (1 bit) |
| $4012 | DMC Start Address | Start address (6 bits + $C000) |
| $4013 | DMC Length | Length (8 bits + 1) |
| $4015 | Channel Enable | Channel enable (5 bits), DMC length reset |
| $4017 | Frame Counter | Frame IRQ enable, count sequence |

### Square Wave Channel ($4000-$4007)

#### Register $4000/$4004 - Control

```
Bit 7-4: Envelope decay rate (0-15, 15 = hold volume)
Bit 3: Unused
Bit 2: Loop envelope decay (1=loop, 0=stop at 0)
Bit 1: Unused
Bit 7-6: Duty mode (0-3, selects pattern)
```

**Duty Patterns:**
- 0: 1/8 (1 pulse, 7 quiet)
- 1: 2/8 (2 pulses, 6 quiet)
- 2: 4/8 (4 pulses, 4 quiet)
- 3: 8/8 (7 pulses, 1 quiet - negative duty)

#### Register $4001/$4005 - Sweep

```
Bit 7: Sweep active (1=enabled)
Bit 6-4: Sweep period (0-7)
Bit 3: Add/sub mode (0=add, 1=subtract)
Bit 2-0: Sweep shift amount (0-7)
```

#### Timer Calculation

Output frequency (Hz) = 1789772.5 / (16 * (timer + 1))

### Triangle Wave Channel ($4008-$400B)

Triangle channel uses a linear counter that counts down independently.

#### Register $4008 - Linear Control

```
Bit 7: Linear counter control (1=halt, 0=load from register)
Bit 6-0: Linear counter load value (0-127)
```

### Noise Channel ($400C-$400F)

#### Register $400E - Noise Wavelength

```
Bit 7: Random mode (0=7-stage, 1=15-stage)
Bit 3-0: Wavelength (frequency table index)
```

**Wavelength Table:**
```
[4, 8, 16, 32, 64, 96, 128, 160, 202, 254, 380, 508, 762, 1016, 2034, 4068]
```

### DMC Channel ($4010-$4015)

#### Register $4010 - DMC Control

```
Bit 7-6: Play mode (00=normal, 01=loop, 10=IRQ on completion)
Bit 3-0: Frequency index (0-15)
```

**DMC Frequency Table:**
```
[537472, 478016, 424704, 399872, 357824, 318720, 298944, 279168,
 260608, 239008, 212352, 199936, 178912, 159360, 149472, 134848]
```

#### DMC Operation

- Reads 8-bit samples from cartridge ROM
- Starts at address: $C000 + (value << 6)
- Length: (value + 1) * 16 bytes
- Plays at 1 bit per 8 cycles (LSB first)
- Delta counter (6-bit) adjusts by ±1 per bit

### Frame Counter

The frame counter generates events at specific intervals for envelope/length updates.

#### 4-Step Sequence (countSequence = 0)

Steps occur at CPU cycle counts:
- Step 0: 7457 cycles
- Step 1: 14913 cycles
- Step 2: 22371 cycles
- Step 3: 29829 cycles (triggers frame IRQ)

#### 5-Step Sequence (countSequence = 1)

Steps occur at CPU cycle counts:
- Step 0: 7457 cycles
- Step 1: 14913 cycles
- Step 2: 22371 cycles
- Step 3: 29829 cycles
- Step 4: 37281 cycles (triggers frame IRQ)

### Quarter Frame Operations

These operations occur at each frame step:

- Square channels: Clock envelope decay
- Noise channel: Clock envelope decay
- Triangle: Clock linear counter

### Half Frame Operations

These occur every other frame step:

- All channels except DMC: Clock length counter
- Square channels: Clock sweep unit

### DAC (Digital-to-Analog Conversion)

The APU uses nonlinear DAC tables for volume calculation:

#### Square Table

32 * 16 entries based on formula:
```
value = 95.52 / (8128.0 / (i / 16.0) + 100.0) * 0.98411 * 50000.0
```

#### TND Table (Triangle/Noise/DMC)

204 * 16 entries based on formula:
```
value = 163.67 / (24329.0 / (i / 16.0) + 100.0) * 0.98411 * 50000.0
```

### Volume Calculation

Output volume uses a non-linear scale from 0-15:

| Value | Relative Volume |
|-------|-----------------|
| 0 | 0% |
| 1 | 1.2% |
| 2 | 2.4% |
| 3 | 3.6% |
| 4 | 4.8% |
| 5 | 6.0% |
| 6 | 7.1% |
| 7 | 8.3% |
| 8 | 9.5% |
| 9 | 10.6% |
| 10 | 11.8% |
| 11 | 12.9% |
| 12 | 14.1% |
| 13 | 15.2% |
| 14 | 16.4% |
| 15 | 17.5% |

### Audio Sample Generation

```javascript
// Sample timer calculation
sampleTimerMax = Math.floor((1024.0 * CPU_FREQ_NTSC * preferredFrameRate) / (sampleRate * 60.0));

// Stereo panning positions
stereoPosL1 = 64;  // Square 1 left
stereoPosR1 = 0;   // Square 1 right
stereoPosL2 = 0;   // Square 2 left
stereoPosR2 = 64;  // Square 2 right
stereoPosLT = 64;  // Triangle left
stereoPosLN = 32;  // Noise left
stereoPosLD = 32;  // DMC left
```

---

## Memory Map

### 6502 Address Space

| Range | Description |
|-------|-------------|
| $0000-$07FF | CPU RAM (2KB) |
| $0800-$0FFF | CPU RAM mirror |
| $1000-$1FFF | CPU RAM mirror |
| $2000-$2007 | PPU registers (8 bytes) |
| $2008-$3FFF | PPU registers mirror |
| $4000-$4013 | APU registers |
| $4014 | OAM DMA |
| $4015 | APU channel enable |
| $4016 | Controller 1 |
| $4017 | Controller 2 / Frame counter |
| $4018-$5FFF | Unusable |
| $6000-$7FFF | Battery RAM (if present) |
| $8000-$FFFF | PRG-ROM |

### PPU Register Map

| Address | Name | Access | Description |
|---------|------|--------|-------------|
| $2000 | PPUCTRL | Write | Control register 1 |
| $2001 | PPUMASK | Write | Control register 2 |
| $2002 | PPUSTATUS | Read | Status register |
| $2003 | OAMADDR | Write | OAM address |
| $2004 | OAMDATA | Read/Write | OAM data |
| $2005 | PPUSCROLL | Write | Scroll register |
| $2006 | PPUADDR | Write | VRAM address |
| $2007 | PPUDATA | Read/Write | VRAM data |

### APU Register Map

| Address | Name | Access | Description |
|---------|------|--------|-------------|
| $4000 | Square 1 Control | Write | Envelope, duty |
| $4001 | Square 1 Sweep | Write | Sweep settings |
| $4002 | Square 1 Timer Low | Write | Timer low |
| $4003 | Square 1 Timer High | Write | Timer high |
| $4004 | Square 2 Control | Write | Envelope, duty |
| $4005 | Square 2 Sweep | Write | Sweep settings |
| $4006 | Square 2 Timer Low | Write | Timer low |
| $4007 | Square 2 Timer High | Write | Timer high |
| $4008 | Triangle Control | Write | Linear counter |
| $400A | Triangle Timer Low | Write | Timer low |
| $400B | Triangle Timer High | Write | Timer high |
| $400C | Noise Control | Write | Envelope |
| $400E | Noise Wavelength | Write | Frequency |
| $400F | Noise Length | Write | Length load |
| $4010 | DMC Control | Write | Play mode, freq |
| $4011 | DMC DAC | Write | Delta counter |
| $4012 | DMC Start Address | Write | Start address |
| $4013 | DMC Length | Write | Length |
| $4015 | Channel Enable | Read/Write | Enable channels |
| $4017 | Frame Counter | Read/Write | IRQ, sequence |

---

## ROM Format

The NES ROM format (.nes files) is a simple container:

### File Structure

```
Offset 0-3:   "NES\x1a" magic header
Offset 4:     PRG-ROM size in 16KB units
Offset 5:     CHR-ROM size in 8KB units (2 per byte)
Offset 6:     Flags 6 (mirroring, battery, trainer)
Offset 7:     Flags 7 (mapper, VS UniSys, PlayChoice-10)
Offset 8-15:  Reserved (zero-filled)
Offset 16+:   PRG-ROM data
Offset 16 + (PRG * 16384): CHR-ROM data
```

### Flag 6 ($0006)

```
Bit 0: Mirroring (0=horizontal, 1=vertical)
Bit 1: Battery RAM (1=has battery backup)
Bit 2: Trainer (1=has 512-byte trainer)
Bit 3: Four-screen (1=four-screen mirroring)
Bit 7-4: Mapper high nibble
```

### Flag 7 ($0007)

```
Bit 7-4: Mapper low nibble
```

### Mapper Type Calculation

```
mapperType = (flags6 >> 4) | (flags7 & 0xF0)
```

### Mirroring Types

| Value | Name | Description |
|-------|------|-------------|
| 0 | Horizontal | Nametables 0,1 = NT0; 2,3 = NT1 |
| 1 | Vertical | Nametables 0,2 = NT0; 1,3 = NT1 |
| 2 | Four-Screen | Each nametable separate |
| 3 | Single-Screen | All map to NT0 |

### ROM Loading Process

1. Validate "NES\x1a" magic header
2. Parse header (PRG/CHR count, mapper type, mirroring)
3. Load PRG-ROM banks into memory
4. Load CHR-ROM banks into PPU VRAM
5. Create mapper instance based on mapper type
6. Load mapper-specific ROM banks
7. Reset CPU and trigger reset interrupt

---

## Mapper Implementations

The NES uses cartridges with memory management units (MMUs) called "mappers" to extend beyond the NES's native 32KB PRG and 8KB CHR limitations.

### Supported Mappers (16 total)

| Mapper | Name | PRG Banking | VROM Banking | Features |
|--------|------|-------------|--------------|----------|
| 0 | NoMapper | None | None | Simple mapping |
| 1 | MMC1 | 16/32KB | 4/8KB | Configurable, single port |
| 2 | UNROM | 8KB | None | Simple PRG banking |
| 3 | CNROM | None | 8KB | Simple VROM banking |
| 4 | MMC3 | 8KB | 1KB | Complex, 6KB banks, IRQ |
| 5 | MMC5 | 8KB | 1KB | Extended, ExRAM, multi-mode |
| 7 | AxROM | 32KB | None | Simple PRG + mirroring |
| 11 | Color Dreams | 16KB | 8KB | Combined banking |
| 34 | BNROM | 32KB | None | Simple PRG banking |
| 38 | MMC3 variant | 32KB | 8KB | Simplified MMC3 |
| 66 | GxROM | 32KB | 8KB | Simple banking |
| 94 | UN1ROM | 16KB | None | Simple PRG banking |
| 140 | NINA-06 | 32KB | 8KB | Combined banking |
| 180 | MMC3 variant | 16KB | None | Simplified |
| 240 | UNROM variant | 32KB | 8KB | Combined banking |
| 241 | UNROM variant | 32KB | None | Simple PRG banking |

### Base Mapper Class (Mapper 0)

Mapper 0 (NoMapper) is the base class for all mappers:

```javascript
class Mapper0 {
  constructor(nes) {
    this.nes = nes;
  }

  // Reset controller state
  reset();

  // Memory access
  write(address, value);    // Write to memory
  read(address);            // Read from memory
  load(address);            // Load from cartridge
  writelow(address, value); // Low-level write
  regLoad(address);         // Load from PPU/APU
  regWrite(address, value); // Write to PPU/APU
  joy1Read();               // Read controller 1
  joy2Read();               // Read controller 2

  // ROM loading
  loadROM();                // Load ROM into system
  loadPRGROM();             // Load PRG banks
  loadCHRROM();             // Load CHR banks
  loadBatteryRam();         // Load battery RAM

  // Banking helpers
  loadRomBank(bank, address);           // 16KB PRG bank
  load32kRomBank(bank, address);        // 32KB PRG bank
  loadVromBank(bank, address);          // 4KB VROM bank
  load8kVromBank(bank4kStart, address); // 8KB VROM bank
  load1kVromBank(bank1k, address);      // 1KB VROM bank
  load2kVromBank(bank2k, address);      // 2KB VROM bank
  load8kRomBank(bank8k, address);       // 8KB PRG bank

  // IRQ handling
  clockIrqCounter();   // Clock IRQ counter
  latchAccess(addr);   // Latch access

  // State serialization
  toJSON();
  fromJSON(state);
}
```

### Mapper 0 - NoMapper

The simplest mapper with no banking capability.

**PRG Mapping:**
- 16KB ROM: Same bank at $8000 and $C000
- 32KB ROM: Bank 0 at $8000, Bank 1 at $C000

**CHR Mapping:**
- 4KB ROM: Same bank at $0000 and $1000
- 8KB ROM: Bank 0 at $0000, Bank 1 at $1000

**Usage:** Donkey Kong, Punch-Out

### Mapper 1 - MMC1

Programmable mapper with a 5-bit serial shift register.

**Key Features:**
- Single write port ($8000-$FFFF)
- Reset bit (bit 7) resets shift register
- Configurable PRG/CHR banking
- Mirroring control

**Registers:**
- $8000-$9FFF: Register 0 (control)
- $A000-$BFFF: Register 1 (CHR bank 0)
- $C000-$DFFF: Register 2 (CHR bank 1)
- $E000-$FFFF: Register 3 (PRG bank)

**PRG Banking:**
- Bit 3 (prgSwitchingSize): 0=32KB, 1=16KB
- Bit 2 (prgSwitchingArea): 0=16KB at $C000, 1=16KB at $8000

**CHR Banking:**
- Bit 4 (vromSwitchingSize): 0=8KB, 1=4KB
- Register 1 controls first bank
- Register 2 controls second bank (4KB mode only)

**Usage:** Super Mario Bros., The Legend of Zelda

### Mapper 2 - UNROM

Simple 8KB PRG banking mapper.

**PRG Banking:**
- Lower 8KB ($8000) is switchable
- Upper 8KB ($C000) is fixed to last bank

**Usage:** Final Fantasy, BurgerTime

### Mapper 3 - CNROM

Simple 8KB VROM banking mapper.

**VROM Banking:**
- Write to $8000-$FFFF selects 8KB VROM bank
- PRG ROM is fixed

**Usage:** Solomon's Key, Arkanoid

### Mapper 4 - MMC3

Complex mapper with 6 command-based bank switching.

**Key Features:**
- Command register ($8000)
- Data register ($8001)
- IRQ counter
- 8KB PRG banking (2 banks)
- 1KB VROM banking (6 banks)

**Commands:**
- 0: Two 1KB VROM banks at $0000+$0400
- 1: Two 1KB VROM banks at $0800+$0C00
- 2: One 1KB VROM at $1000
- 3: One 1KB VROM at $1400
- 4: One 1KB VROM at $1800
- 5: One 1KB VROM at $1C00
- 6: PRG page 1 selection
- 7: PRG page 2 selection

**Mirroring:**
- $A000: Horizontal (bit 0 = 1) or Vertical (bit 0 = 0)

**IRQ Registers:**
- $C000: IRQ counter
- $C001: IRQ latch
- $E000: Disable IRQ
- $E001: Enable IRQ

**Usage:** Super Mario Bros. 2, Castlevania

### Mapper 5 - MMC5

Advanced mapper with extended features.

**Key Features:**
- 8KB PRG banking (4 banks)
- 1KB VROM banking (8 independent or 4 duplicated)
- 8KB ExRAM at $6000
- Split screen mode
- Hardware multiplier
- IRQ controller

**PRG Banking ($5114-$5117):**
- Each register controls one 8KB CPU bank

**CHR Banking ($5120-$512B):**
- Mode 0: 8 independent 1KB banks
- Mode 1: 4 duplicated 1KB banks

**Nametable Banking ($5105):**
- Controls 4 individual 1KB nametables

**Split Screen ($5200-$5202):**
- Enables scanline-by-scanline scrolling changes

**Usage:** Castlevania 3, Just Breed

### Mapper 7 - AxROM

Simple mapper with 32KB PRG banking and mirroring control.

**PRG Banking:**
- Lower 3 bits select 32KB bank

**Mirroring:**
- Bit 4 controls mirroring mode

**Usage:** Battletoads

### Mapper 11 - Color Dreams

Combined PRG/VROM banking.

**PRG Banking:**
- Lower nibble selects 16KB PRG bank pair

**VROM Banking:**
- Upper nibble selects 8KB VROM bank pair

**Usage:** Crystal Mines, Metal Fighter

### Mapper 66 - GxROM

Simple combined banking.

**PRG Banking:**
- Upper nibble (bits 4-5) selects 32KB bank

**VROM Banking:**
- Lower nibble (bits 0-1) selects 8KB VROM bank

**Usage:** Dragon Ninja

### Mapper 34 - BNROM

Simple 32KB PRG banking.

**PRG Banking:**
- Write to $8000-$FFFF selects 32KB bank

**Usage:** Mission Impossible 2

### Mapper 94 - UN1ROM

16KB PRG banking with upper bits.

**PRG Banking:**
- Upper 6 bits (bits 2-7) select 16KB bank

**Usage:** Battletoads 2

### Mapper 140 - NINA-06

Combined PRG/VROM banking at $6000-$7FFF.

**PRG Banking:**
- Upper nibble selects 32KB bank

**VROM Banking:**
- Lower nibble selects 8KB VROM bank

### Mapper 180 - MMC3 Variant

16KB PRG banking variant.

**PRG Banking:**
- Write to $8000-$FFFF selects 16KB bank at $C000
- Fixed at $8000

### Mapper 240 - UNROM Variant

Combined banking at $4020-$5FFF.

**PRG Banking:**
- Upper nibble selects 32KB bank

**VROM Banking:**
- Lower nibble selects 8KB VROM bank

### Mapper 241 - UNROM Variant

Simple 32KB PRG banking.

**PRG Banking:**
- Write to $8000-$FFFF selects 32KB bank

---

## Controller Implementation

### Controller Types

The NES supports several input devices:

| Controller | Type | Description |
|------------|------|-------------|
| Standard | 8-button | A, B, Select, Start, D-Pad |
| Zapper | Light Gun | Position detection, trigger |
| Power Pad | Floor Mat | 12 buttons (side) or 9 (front) |
| Ardundrum | Drum Kit | 5 drum buttons + start |

### Standard Controller

The standard controller has 8 buttons connected via a shift register.

#### Button Constants

```javascript
Controller.BUTTON_A = 0;
Controller.BUTTON_B = 1;
Controller.BUTTON_SELECT = 2;
Controller.BUTTON_START = 3;
Controller.BUTTON_UP = 4;
Controller.BUTTON_DOWN = 5;
Controller.BUTTON_LEFT = 6;
Controller.BUTTON_RIGHT = 7;
```

#### Controller State

Each button state is stored as a byte:
- `0x40`: Button up (default)
- `0x41`: Button down

#### Controller Registers

- **$4016:** Controller 1 strobe
- **$4017:** Controller 2 strobe

#### Controller Reading

1. Write strobe = 1, then strobe = 0 to reset shift register
2. Read controller port, bit 0 contains button state
3. Shift register advances on each read
4. 8 reads needed for full button state

### Zapper Light Gun

The Zapper detects screen brightness at a specific position.

#### Zapper State

- **$4017 bit 3:** Zapper trigger (0=fired, 1=not fired)
- **$4017 bit 4:** Light sensor (0=white pixel, 1=non-white)

#### Position Tracking

- X position: 0-255 (horizontal pixel)
- Y position: 0-239 (vertical scanline)

#### Usage

- `zapperMove(x, y)`: Set light gun position
- `zapperFireDown()`: Simulate trigger press
- `zapperFireUp()`: Simulate trigger release

### Controller Implementation

```javascript
class Controller {
  static BUTTON_A = 0;
  static BUTTON_B = 1;
  static BUTTON_SELECT = 2;
  static BUTTON_START = 3;
  static BUTTON_UP = 4;
  static BUTTON_DOWN = 5;
  static BUTTON_LEFT = 6;
  static BUTTON_RIGHT = 7;

  constructor() {
    this.state = new Array(8);
    for (let i = 0; i < this.state.length; i++) {
      this.state[i] = 0x40;  // Default: button up
    }
    this.strobeState = 0;
  }

  buttonDown(button) {
    this.state[button] = 0x41;
  }

  buttonUp(button) {
    this.state[button] = 0x40;
  }

  read() {
    // Called when CPU reads controller port
    // Returns current button state based on strobe
  }
}
```

---

## Timing and Synchronization

### CPU/PPU Relationship

The CPU and PPU run in lockstep with a 3:1 ratio:

```
NTSC:
- CPU clock: 1.7897725 MHz
- PPU clock: 5.3693175 MHz (3x CPU)
- 3 CPU cycles = 1 PPU dot

PAL:
- CPU clock: 1.662607 MHz
- PPU clock: 4.987821 MHz (3x CPU)
```

### Frame Timing (NTSC)

```
Scanline 0-19:  Pre-render (hidden)
Scanline 20:    VBlank start
Scanline 21-260: Active rendering (240 lines)
Scanline 261:   VBlank (NMI triggered)

Total: 262 scanlines × 341 dots = 89,142 dots per frame
Frame rate: ~60.09 Hz
Frame time: ~16.64 ms
```

### APU Timing

The APU runs at the same speed as the CPU but generates audio at a different rate.

**Sample Generation:**

```javascript
// Calculate sample period based on desired rate
sampleTimerMax = Math.floor((1024.0 * CPU_FREQ_NTSC * preferredFrameRate) / (sampleRate * 60.0));

// Generate sample when timer expires
if (sampleTimer >= sampleTimerMax) {
  const left = calculateStereo(0);   // Left channel
  const right = calculateStereo(1);  // Right channel
  onAudioSample(left, right);
  sampleTimer = 0;
}
```

### Frame Counter Sequencing

The frame counter uses different step values for 4-step and 5-step modes:

**4-Step Mode:**
- Step 0: 7457 cycles
- Step 1: 14913 cycles
- Step 2: 22371 cycles
- Step 3: 29829 cycles (frame IRQ)

**5-Step Mode:**
- Step 0: 7457 cycles
- Step 1: 14913 cycles
- Step 2: 22371 cycles
- Step 3: 29829 cycles
- Step 4: 37281 cycles (frame IRQ)

### Cycle-Accurate Implementation

```javascript
// Main emulator loop
function frame() {
  ppu.startFrame();
  let cycles = 0;

  for (;;) {
    if (cpu.cyclesToHalt === 0) {
      // Run CPU
      cycles = cpu.emulate();

      // Update APU
      papu.clockFrameCounter(cycles, cpu.apuCatchupCycles);
      cpu.apuCatchupCycles = 0;

      // Calculate PPU catchup
      cycles = cycles * 3 - cpu.ppuCatchupDots;
      cpu.ppuCatchupDots = 0;

      // Check if frame ended
      if (cpu.ppuFrameEnded) {
        ppu.curX += cycles;
        cpu.ppuFrameEnded = false;
        break;
      }
    } else {
      // PPU catchup phase
      if (cpu.cyclesToHalt > 8) {
        cycles = 24;
        papu.clockFrameCounter(8);
        cpu.cyclesToHalt -= 8;
      } else {
        cycles = cpu.cyclesToHalt * 3;
        papu.clockFrameCounter(cpu.cyclesToHalt);
        cpu.cyclesToHalt = 0;
      }
    }

    // PPU dot-by-dot processing
    for (; cycles > 0; cycles--) {
      ppu.curX++;

      if (ppu.curX === 341) {
        ppu.curX = 0;
        ppu.endScanline();
      }
    }
  }
}
```

---

## Implementation Checklist

### Phase 1: Core CPU

- [ ] Implement 6502 registers (A, X, Y, SP, PC)
- [ ] Implement status flags (N, V, B, D, I, Z, C)
- [ ] Implement all 13 addressing modes
- [ ] Implement all 256 opcodes (standard + unofficial)
- [ ] Implement memory map (64KB)
- [ ] Implement interrupts (NMI, Reset, IRQ)
- [ ] Implement cycle counting

### Phase 2: Memory System

- [ ] Implement 2KB CPU RAM
- [ ] Implement PPU register access ($2000-$3FFF)
- [ ] Implement APU register access ($4000-$4017)
- [ ] Implement controller registers ($4016-$4017)
- [ ] Implement cartridge ROM loading
- [ ] Implement battery-backed RAM

### Phase 3: PPU

- [ ] Implement 2KB VRAM
- [ ] Implement nametables (4x 1KB)
- [ ] Implement attribute tables
- [ ] Implement pattern tables
- [ ] Implement sprite OAM (256 bytes)
- [ ] Implement palette RAM (32 bytes)
- [ ] Implement PPU registers ($2000-$2007)
- [ ] Implement rendering pipeline
- [ ] Implement VBlank/NMI generation
- [ ] Implement sprite 0 hit detection
- [ ] Implement scroll registers

### Phase 4: APU

- [ ] Implement square wave channel 1
- [ ] Implement square wave channel 2
- [ ] Implement triangle wave channel
- [ ] Implement noise channel
- [ ] Implement DMC channel
- [ ] Implement frame counter
- [ ] Implement envelope decay
- [ ] Implement sweep units
- [ ] Implement length counters
- [ ] Implement DAC tables
- [ ] Implement audio sample generation

### Phase 5: Controllers

- [ ] Implement standard controller
- [ ] Implement zapper light gun
- [ ] Implement strobe logic
- [ ] Implement button state tracking

### Phase 6: Mappers

- [ ] Implement Mapper 0 (NoMapper)
- [ ] Implement Mapper 1 (MMC1)
- [ ] Implement Mapper 2 (UNROM)
- [ ] Implement Mapper 3 (CNROM)
- [ ] Implement Mapper 4 (MMC3)
- [ ] Implement additional mappers as needed

### Phase 7: System Integration

- [ ] Implement NES class (main orchestrator)
- [ ] Implement ROM loading
- [ ] Implement frame loop
- [ ] Implement state serialization
- [ ] Implement FPS calculation

### Phase 8: Testing

- [ ] Test with nestest.nes
- [ ] Test with accuracy tests
- [ ] Test with real ROMs
- [ ] Verify timing accuracy
- [ ] Verify audio output
- [ ] Verify graphics rendering

---

## Testing Resources

### Test ROMs

1. **nestest.nes:** Official Nintendo test ROM for CPU instruction verification
2. **accuracycoin.nes:** 134 accuracy tests for CPU, PPU, and APU
3. **ppu_tests.nes:** PPU-specific tests
4. **apu_tests.nes:** APU-specific tests

### Reference Documentation

- [NES Dev Wiki](https://www.nesdev.org/wiki/)
- [6502 Processor Documentation](https://www.nesdev.org/wiki/6502)
- [PPU Programming](https://www.nesdev.org/wiki/PPU)
- [APU Programming](https://www.nesdev.org/wiki/APU)

---

## License

This documentation is derived from the JSNES project (https://github.com/bfirsh/jsnes) which is licensed under Apache-2.0.

---

**Document Version:** 1.0
**Generated:** Based on JSNES 1.2.1
**Source Repository:** https://github.com/bfirsh/jsnes