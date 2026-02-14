# NES Emulator Testing Guide

**Source:** JSNES (https://github.com/bfirsh/jsnes)

---

## Table of Contents

1. [Testing Overview](#testing-overview)
2. [Test ROMs](#test-roms)
3. [Test Procedure](#test-procedure)
4. [Debugging Tools](#debugging-tools)
5. [Common Issues](#common-issues)

---

## Testing Overview

### Three-Phase Testing Approach

```
Phase 1: CPU Testing
├── nestest.nes       - CPU instruction validation
├── lj65.nes          - 6502 instruction tests
└── cpu.spec.js       - Unit tests (Mocha/Chai)

Phase 2: Hardware Testing
├── AccuracyCoin.nes  - Comprehensive accuracy tests
└── mappers.spec.js   - Mapper hardware emulation

Phase 3: Integration Testing
├── Real ROMs         - Visual/audio verification
└── nes.spec.js       - NES initialization tests
```

---

## Test ROMs

### 1. nestest.nes

**Description:** Ultimate CPU test ROM by Kevin Horton

**File:** `roms/nestest/nestest.nes` (24KB)

**Purpose:**
- Validates all 6502 CPU instructions
- Tests all addressing modes
- Tests flag operations (N, V, Z, C)
- Tests stack operations
- Tests branch instructions
- Tests invalid opcodes

**How to Use:**
1. Load `nestest.nes` into emulator
2. Run normally - it auto-tests
3. Results displayed on screen:
   - "Passed" or "Failed" for each test group
   - Last failure code shown at $02-$03

**Pass Criteria:**
- Nintendulator: Passes
- Real NES: Passes
- All CPU instructions must pass

**Failure Codes:**
```
$01-$0C: Branch test failures
  $01: BCS
  $02: BCC
  $03: BEQ
  $04: BNE
  $05: BVS
  $06: BVC
  $07: BPL
  $08: BMI

$10-$17: Indirect,X failures
$18-$38: Immediate instruction failures
$40-$47: Zeropage failures
$48-$4F: Absolute failures
$50-$57: Indexed,Y failures
$58-$5F: Indexed,X failures
$60-$67: Indirect failures
$68-$6F: Implied failures
$70-$75: Stack failures
$76-$9B: Zeropage instruction failures
$9C-$BF: Absolute instruction failures

$001-$015: Invalid opcode failures
  $001: LAX
  $002: SAX
  $003: SBC
  $004: DCP
  $005: ISB
  $006: SLO
  $007: RLA
  $008: SRE
  $009: RRA
```

---

### 2. AccuracyCoin.nes

**Description:** Comprehensive accuracy test suite

**File:** `roms/AccuracyCoin/AccuracyCoin.nes` (40KB)

**Test Categories:**

#### CPU Behavior Tests
- ROM immutability (ROM should not be writable)
- RAM mirroring (0x0000-0x1FFF)
- PC wraparound ($FFFF -> $0000)
- Decimal flag behavior
- B flag behavior
- Dummy read/write cycles
- Open bus behavior

#### Unofficial Instructions Tests
- SLO (Shift Left then OR)
- RLA (Rotate Left then AND)
- SRE (Shift Right then EOR)
- RRA (Rotate Right then Add)
- SAX (Store A and X)
- LAX (Load A and X)
- DCP (Decrement then Compare)
- ISC (Increment then Subtract)
- SH* variants (SHA, SHS, SHX, SHY)

#### PPU Tests
- CHR ROM protection
- Palette quirk tests
- VBlank timing
- NMI control
- Sprite evaluation timing
- Sprite overflow flag
- Sprite 0 hit flag

#### APU Tests
- Length counter behavior
- Frame counter sequencing
- DMC channel operation
- Envelope decay
- Sweep unit

#### DMA Tests
- Bus conflicts
- Explicit aborts
- Implicit aborts
- DMA timing

---

### 3. lj65.nes

**Description:** 6502 instruction test ROM

**File:** `roms/lj65/lj65.nes` (24KB)

**Purpose:**
- Tests 6502 instruction set
- Validates flag settings
- Tests addressing modes

---

### 4. croom.nes

**Description:** Simple frame buffer test

**File:** `roms/croom/croom.nes` (24KB)

**Purpose:**
- Tests PPU rendering
- Verifies frame buffer writes
- Simple visual output test

---

### 5. Test Suite Files

#### cpu.spec.js
- 173 CPU instruction tests
- Uses Mocha/Chai framework
- Tests all instructions across addressing modes
- Validates cycle counts

#### nestest.spec.js
- Integration tests for nestest ROM
- Verifies test output matches expectations

#### accuracycoin.spec.js
- 134 accuracy tests
- Tests CPU, PPU, APU, and DMA

#### mappers.spec.js
- Mapper hardware emulation tests
- Tests basic mapper functionality

#### gamegenie.spec.js
- Game Genie code decoding/encoding tests

---

## Test Procedure

### Phase 1: CPU Validation

1. **Run nestest.nes**
   ```
   Expected: All tests pass, "Passed" displayed
   If fails: Check CPU implementation
   ```

2. **Run lj65.nes**
   ```
   Expected: All tests pass
   If fails: Check instruction implementations
   ```

3. **Run unit tests**
   ```
   npm test  # If using JavaScript
   ```

### Phase 2: Hardware Validation

4. **Run AccuracyCoin.nes**
   ```
   Expected: All tests pass
   If fails: Check PPU/APU/DMA implementation
   ```

5. **Test mappers**
   - Load ROMs using different mappers
   - Verify banking works correctly
   - Check IRQ handling if applicable

### Phase 3: Integration Validation

6. **Test real ROMs**
   - Super Mario Bros. (Mapper 0)
   - The Legend of Zelda (Mapper 1)
   - Castlevania (Mapper 4)

7. **Verify features**
   - Graphics display correctly
   - Audio plays
   - Input responds
   - Save states work

---

## Debugging Tools

### CPU Debugging

```javascript
// Add these to CPU class for debugging

logInstruction() {
  let pc = this.REG_PC;
  let opcode = this.mem[pc];
  let addrMode = this.addrModes[opcode];
  let cycles = this.cycleCounts[opcode];

  console.log(`PC:$${pc.toString(16).toUpperCase().padStart(4, '0')} ` +
              `OP:$${opcode.toString(16).toUpperCase().padStart(2, '0')} ` +
              `A:$${this.REG_ACC.toString(16).toUpperCase().padStart(2, '0')} ` +
              `X:$${this.REG_X.toString(16).toUpperCase().padStart(2, '0')} ` +
              `Y:$${this.REG_Y.toString(16).toUpperCase().padStart(2, '0')} ` +
              `SP:$${this.REG_SP.toString(16).toUpperCase().padStart(2, '0')} ` +
              `P:${this.formatFlags()} ${this.instructionNames[opcode]} ${addrMode}`);
}

formatFlags() {
  let flags = '';
  flags += this.F_SIGN ? 'N' : 'n';
  flags += this.F_OVERFLOW ? 'V' : 'v';
  flags += '1';  // Unused
  flags += this.F_BRK ? 'B' : 'b';
  flags += this.F_DECIMAL ? 'D' : 'd';
  flags += this.F_INTERRUPT ? 'I' : 'i';
  flags += this.F_ZERO ? 'Z' : 'z';
  flags += this.F_CARRY ? 'C' : 'c';
  return flags;
}
```

### PPU Debugging

```javascript
// Add debug output to PPU

logScanline(scanline, curX) {
  console.log(`Scanline: ${scanline} X: ${curX} ` +
              `VBlank: ${this.f_nmiOnVblank} ` +
              `BgVis: ${this.f_bgVisibility} SpVis: ${this.f_spVisibility}`);
}

dumpVRAM() {
  console.log('VRAM Dump:');
  for (let i = 0; i < 0x4000; i += 16) {
    let hex = '';
    for (let j = 0; j < 16; j++) {
      hex += this.vramMem[i + j].toString(16).toUpperCase().padStart(2, '0') + ' ';
    }
    console.log(`$${i.toString(16).toUpperCase().padStart(4, '0')}: ${hex}`);
  }
}

dumpOAM() {
  console.log('OAM Dump:');
  for (let i = 0; i < 256; i += 4) {
    let y = this.spriteMem[i];
    let tile = this.spriteMem[i + 1];
    let attr = this.spriteMem[i + 2];
    let x = this.spriteMem[i + 3];
    console.log(`Sprite ${i/4}: Y=${y} Tile=${tile} Attr=${attr.toString(2)} X=${x}`);
  }
}
```

### APU Debugging

```javascript
// Add debug output to PAPU

logAPU() {
  console.log('APU Status:');
  console.log(`  Square1: ${this.square1.getOutput()}`);
  console.log(`  Square2: ${this.square2.getOutput()}`);
  console.log(`  Triangle: ${this.triangle.getOutput()}`);
  console.log(`  Noise: ${this.noise.getOutput()}`);
  console.log(`  DMC: ${this.dmc.getOutput()}`);
}

logFrameCounter() {
  console.log(`Frame Counter: Step ${this.frameStep} ` +
              `Cycle ${this.frameCycleCounter}`);
}
```

---

## Common Issues

### Issue 1: CPU Instructions Fail

**Symptoms:** nestest.nes fails on specific instructions

**Causes:**
- Incorrect addressing mode implementation
- Wrong cycle counts
- Flag calculation errors
- Memory access issues

**Debug Steps:**
1. Check instruction implementation matches 6502 docs
2. Verify cycle counts for addressing modes
3. Check flag setting logic
4. Verify memory access (open bus, page crossing)

---

### Issue 2: PPU Rendering Wrong

**Symptoms:** Graphics display incorrectly or not at all

**Causes:**
- VRAM access issues
- Scrolling register problems
- Nametable mirroring errors
- Sprite OAM issues

**Debug Steps:**
1. Verify $2005/$2006 two-byte writes
2. Check nametable mirroring logic
3. Verify scroll register behavior
4. Check OAM DMA timing (513 cycles)

---

### Issue 3: Audio Not Working

**Symptoms:** No audio or distorted audio

**Causes:**
- Channel register access wrong
- Frame counter timing off
- DAC table issues
- Sample rate calculation wrong

**Debug Steps:**
1. Check $4015 channel enable register
2. Verify frame counter step timing
3. Verify DAC table formulas
4. Check sample timer calculation

---

### Issue 4: Mapper Not Working

**Symptoms:** ROM loads but crashes or runs incorrectly

**Causes:**
- Register address wrong
- Banking calculation incorrect
- Interrupt handling missing
- State not persisted

**Debug Steps:**
1. Verify register addresses in mapper
2. Check bank switching logic
3. Verify CHR ROM loading
4. Check save state serialization

---

### Issue 5: Controller Not Working

**Symptoms:** Input not responding

**Causes:**
- Strobe logic wrong
- Shift register position off
- Button state encoding incorrect

**Debug Steps:**
1. Verify strobe write behavior
2. Check shift register advance
3. Verify button state values (0x40/0x41)
4. Check $4016/$4017 register access

---

## Testing Checklist

- [ ] nestest.nes passes
- [ ] lj65.nes passes
- [ ] AccuracyCoin.nes passes
- [ ] Super Mario Bros. runs (Mapper 0)
- [ ] The Legend of Zelda runs (Mapper 1)
- [ ] Castlevania runs (Mapper 4)
- [ ] Graphics display correctly
- [ ] Audio plays
- [ ] Input responds
- [ ] Save states work
- [ ] PAL timing correct (if implemented)

---

**Document Version:** 1.0
**Last Updated:** 2026-02-14
**Based on:** JSNES 1.2.1