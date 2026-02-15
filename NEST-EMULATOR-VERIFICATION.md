# NES Emulator Verification Guide

**Source:** JSNES (https://github.com/bfirsh/jsnes)
**Test ROMs:** Located in `test_roms/` directory

---

## Table of Contents

1. [Test ROMs Overview](#test-roms-overview)
2. [nestest.nes - CPU Verification](#nestestnes---cpu-verification)
3. [AccuracyCoin.nes - Comprehensive Hardware Tests](#accuracycoinnes---comprehensive-hardware-tests)
4. [Expected Test Results](#expected-test-results)
5. [PPU Rendering Debugging Method](#ppu-rendering-debugging-method)
6. [Automated Test Framework](#automated-test-framework)

---

## Test ROMs Overview

### Available Test ROMs

| File | Size | Description |
|------|------|-------------|
| `test_roms/nestest/nestest.nes` | 24KB | CPU instruction validation |
| `test_roms/AccuracyCoin/AccuracyCoin.nes` | 40KB | 134 comprehensive accuracy tests |
| `test_roms/croom/croom.nes` | 24KB | Frame buffer test (PPU) |
| `test_roms/lj65/lj65.nes` | 24KB | 6502 instruction tests |

### Document Files

| File | Description |
|------|-------------|
| `test_roms/nestest/nestest.txt` | CPU test documentation with all error codes |
| `test_roms/AccuracyCoin/README.md` | 777-line test documentation |

---

## nestest.nes - CPU Verification

### Test Overview

**Creator:** Kevin Horton (09/06/04)
**Purpose:** Validate all 6502 CPU instructions and addressing modes
**Pass Criteria:** All tests pass, "OK" displayed on screen

### How to Run

1. Load `nestest.nes` into emulator
2. Run normally - tests auto-execute
3. Press **Select** to test invalid opcodes (may crash emulators)

### Automation Mode

To run tests programmatically:
1. Set PC to `$C000`
2. Emulate until tests complete
3. Read results from memory:
   - `mem[0x10]` = byte 02h (official opcodes)
   - `mem[0x11]` = byte 03h (unofficial opcodes)

**Expected:** Both bytes = `$00` (all tests pass)

### Error Code Reference

#### Byte 02h - Official Opcodes

| Code | Test Group | Description |
|------|------------|-------------|
| `$00` | All tests passed |  |
| `$01-$10` | Branch tests | BCS, BCC, BEQ, BNE, BVS, BVC, BPL, BMI |
| `$11-$17` | Flag tests | PHP, PLP, PHA, PLA |
| `$18-$3D` | Immediate tests | ORA, AND, EOR, ADC, CMP, CPY, CPX, LDX, LDY |
| `$3E-$45` | Implied tests | INX, DEX, INY, DEY, TAX, TXA, TAY, TYA, TXS, TSX |
| `$46-$49` | Stack tests | JSR, RTS, RTI |
| `$4A-$4D` | Accumulator tests | LSR, ASL, ROR, ROL |
| `$58-$70` | (indirect,X) tests | Indirect addressing with X |
| `$71-$75` | SBC tests | Subtraction with carry |
| `$76-$AF` | Zeropage tests | Zero page addressing |
| `$B0-$E9` | Absolute tests | Absolute addressing |
| `$EA-$FE` | (indirect),Y tests | Indirect with Y |

#### Byte 03h - Invalid Opcodes & More

| Code | Description |
|------|-------------|
| `$00-$06` | SBC failures |
| `$07` | JMP (indirect) wraparound issue |
| `$08-$31` | Zeropage,X failures |
| `$36-$50` | Absolute,Y failures |
| `$51-$7B` | Absolute,X failures |
| `$7C-$87` | LAX failures |
| `$88-$8F` | SAX failures |
| `$90-$94` | SBC (unofficial) failures |
| `$95-$A9` | DCP failures |
| `$AA-$BE` | ISC failures |
| `$BF-$D3` | SLO failures |
| `$D4-$E8` | RLA failures |
| `$E9-$FD` | SRE failures |

---

## AccuracyCoin.nes - Comprehensive Hardware Tests

### Test Overview

**Creator:** NES Dev Community
**Total Tests:** 134 tests
**Categories:**
- CPU Behavior (ROM immutability, RAM mirroring, PC wraparound, decimal flag, B flag, dummy reads, open bus)
- Unofficial Instructions (SLO, RLA, SRE, RRA, SAX, LAX, DCP, ISC, SH*, ANC, ASR, ARR, ANE, LXA, AXS)
- CPU Interrupts (IRQ, NMI timing)
- APU Tests (length counters, frame counter, DMC channel)
- PPU Tests (CHR ROM protection, palette quirks, VBlank timing, NMI control, sprite evaluation)
- DMA Tests (bus conflicts, aborts)
- Controller Tests (strobe, clocking)

### How to Run

1. Load `AccuracyCoin.nes` into emulator
2. Press **Start** to run all tests automatically
3. Results table displays PASS/FAIL for each test
4. Press **Select** to view debug menu with RAM values

### Automation Mode

```javascript
// Simulate Start button to trigger "run all tests" mode
nes.frame();
nes.buttonDown(1, Controller.BUTTON_START);
for (let i = 0; i < 5; i++) {
  nes.frame();
}
nes.buttonUp(1, Controller.BUTTON_START);

// Run frames until tests complete
let maxFrames = 30000;
for (let f = 0; f < maxFrames; f++) {
  nes.frame();
  if (nes.cpu.mem[0x35] === 0) {  // RunningAllTests flag cleared
    break;
  }
}

// Collect results from all test addresses
let results = {};
ALL_TESTS.forEach(function(test) {
  results[test.addr] = nes.cpu.mem[test.addr];
});
```

### Result Encoding

Tests store results at their specific memory addresses:

| Value | Meaning |
|-------|---------|
| `$00` | Not run |
| `(n << 2) | 1` = `$01, $05, $09...` | **PASS** |
| `(n << 2) | 2` = `$06, $0A, $0E...` | **FAIL** |
| `$FF` | Skipped |

**Helper function to check result:**
```javascript
function isPass(value) {
    return (value & 3) === 1;
}

function formatResult(value) {
    if (value === 0x00) return "NOT RUN";
    if (isPass(value)) return "PASS";
    if (value === 0xff) return "SKIPPED";
    return "FAIL (error 0x" + value.toString(16).toUpperCase().padStart(2, "0") + ")";
}
```

### Test Pages and Addresses

#### Page 1: CPU Behavior
| Address | Test | Expected |
|---------|------|----------|
| $0400 | ROM is not Writable | PASS |
| $0401 | RAM Mirroring | PASS |
| $0402 | PC Wraparound | PASS |
| $0403 | The Decimal Flag | PASS |
| $0404 | The B Flag | PASS |
| $0405-$0414 | Dummy read/write cycles | PASS |
| $0415-$0421 | Open Bus | PASS |
| $0422-$0450 | All NOP Instructions | PASS |

#### Page 16: PPU Behavior
| Address | Test | Expected |
|---------|------|----------|
| $0485 | CHR ROM is not Writable | PASS |
| $0486 | Rendering Flag Behavior | **May fail** (requires dot-accurate PPU) |
| $0487 | BG Serial In | **May fail** |
| $0488 | PPU Register Mirroring | PASS |
| $0489 | PPU Register Open Bus | PASS |
| $048A | $2007 Read w/ Rendering | **May fail** |

#### Page 17: PPU VBlank Timing
| Address | Test | Expected |
|---------|------|----------|
| $0450 | VBlank beginning | **May fail** |
| $0451 | VBlank end | **May fail** |
| $0452 | NMI Control | **May fail** |
| $0453 | NMI Timing | **May fail** |
| $0454 | NMI Suppression | **May fail** |
| $0455 | NMI at VBlank end | **May fail** |
| $0456 | NMI disabled at VBlank | **May fail** |

#### Page 18: Sprite Evaluation
| Address | Test | Expected |
|---------|------|----------|
| $0457 | Sprite 0 Hit behavior | **May fail** |
| $0458 | Arbitrary Sprite zero | **May fail** |
| $0459 | Sprite overflow behavior | **May fail** |
| $0489 | Suddenly Resize Sprite | **May fail** |
| $045A | Misaligned OAM behavior | **May fail** |
| $045B | Address $2004 behavior | **May fail** |
| $047B | OAM Corruption | **May fail** |
| $0480 | INC $4014 | **May fail** |

### Known Failures (in test file)

The `accuracycoin.spec.js` file documents these tests as known failures requiring dot-accurate PPU:

```
0x0486: "Rendering flag behavior not accurate"
0x048a: "$2007 read during rendering not accurate"

0x0450-0x0456: "VBlank/NMI timing not accurate"

0x0457-0x0480: "Sprite evaluation not accurate"
  - 0x0457: Sprite 0 hit
  - 0x0489: Suddenly resize sprite
  - 0x0458: Arbitrary sprite zero
  - 0x045a: Misaligned OAM
  - 0x045b: Address $2004
  - 0x047b: OAM corruption
  - 0x0480: INC $4014

0x0481-0x0484: "PPU misc not accurate"
```

---

## Expected Test Results

### Pass/Fail Criteria Summary

#### nestest.nes
```
Expected: mem[$10] = $00, mem[$11] = $00
Interpretation: All official and unofficial opcode tests passed

If mem[$10] != $00: Byte 02h contains last failure code
If mem[$11] != $00: Byte 03h contains last failure code
```

#### AccuracyCoin.nes
```
Expected: All test addresses contain (n << 2) | 1 values
Interpretation: All 134 tests passed

To check: (value & 3) === 1 means PASS
To get failure code: value >> 2 for non-pass values
```

### Sample Test Results

#### Passing nestest.nes
```
Before test: mem[$10] = 0xFF, mem[$11] = 0xFF
After test:  mem[$10] = 0x00, mem[$11] = 0x00

Result: ALL TESTS PASSED
```

#### Failing nestest.nes (branch test)
```
Before test: mem[$10] = 0xFF, mem[$11] = 0xFF
After test:  mem[$10] = 0x03, mem[$11] = 0x00

Interpretation: BCC test failed (error $03 = BCC branched when it shouldn't)
```

#### Failing nestest.nes (invalid opcode)
```
Before test: mem[$10] = 0x00, mem[$11] = 0xFF
After test:  mem[$10] = 0x00, mem[$11] = 0x80

Interpretation: LAX absolute failed (error $80 = bit 7-4 = 8, low 4 = 0)
                = LAX error code 8 (A register incorrect)
```

#### Passing AccuracyCoin.nes (partial)
```
Test $0400 (ROM is not Writable): 0x05 (pass)
Test $0401 (RAM Mirroring): 0x05 (pass)
Test $0402 (PC Wraparound): 0x05 (pass)
Test $0485 (CHR ROM is not Writable): 0x05 (pass)
```

#### Failing AccuracyCoin.nes (partial)
```
Test $0450 (VBlank beginning): 0x06 (fail, sub-test 1)
Test $0457 (Sprite 0 hit): 0x06 (fail, sub-test 1)
Test $0458 (Arbitrary Sprite zero): 0x06 (fail, sub-test 1)
```

---

## PPU Rendering Debugging Method

### Goal: Automated Verification Without Human Operator

### Method 1: Pixel Comparison Test (croom.nes)

The `croom.nes` test ROM renders specific white pixels at known positions.

**Test Logic:**
```javascript
// Check the first index of a white pixel (0xFFFFFF) on the first 6 frames
let expectedIndexes = [-1, -1, -1, 2056, 4104, 4104];

for (let i = 0; i < 6; i++) {
  nes.frame();
  let actualIndex = onFrame.lastCall.args[0].indexOf(0xFFFFFF);
  assert.equal(actualIndex, expectedIndexes[i]);
}
```

**Expected Pixel Positions:**
- Frame 0: No white pixels (`-1`)
- Frame 1: No white pixels (`-1`)
- Frame 2: No white pixels (`-1`)
- Frame 3: White pixel at index `2056` (X=2056%256=232, Y=2056/256=8)
- Frame 4: White pixel at index `4104` (X=4104%256=24, Y=4104/256=16)
- Frame 5: White pixel at index `4104` (X=24, Y=16)

**Automated Verification:**
```javascript
function verifyFrameBuffer(nes, frameNumber, expectedWhiteIndex) {
  let onFrame = sinon.spy();
  nes.loadROM(romData);

  for (let i = 0; i <= frameNumber; i++) {
    nes.frame();
  }

  let actualWhiteIndex = onFrame.lastCall.args[0].indexOf(0xFFFFFF);
  if (actualWhiteIndex !== expectedWhiteIndex) {
    return {
      passed: false,
      expected: expectedWhiteIndex,
      actual: actualWhiteIndex,
      coords: {
        expected: { x: expectedWhiteIndex % 256, y: Math.floor(expectedWhiteIndex / 256) },
        actual: { x: actualWhiteIndex % 256, y: Math.floor(actualWhiteIndex / 256) }
      }
    };
  }
  return { passed: true };
}
```

### Method 2: Sprite 0 Hit Detection

Sprite 0 hit is a hardware feature that can be tested without human verification.

**Test Logic:**
```javascript
// Set up sprite 0 at specific position
nes.ppu.sprX[0] = 128;   // X position
nes.ppu.sprY[0] = 100;   // Y position
nes.ppu.sprTile[0] = 0;  // Tile index
nes.ppu.sprCol[0] = 0;
nes.ppu.vertFlip[0] = 0;
nes.ppu.horiFlip[0] = 0;
nes.ppu.bgPriority[0] = 0;

// Render frame
nes.frame();

// Check if sprite 0 hit flag was set
if (nes.ppu.spr0HitX !== -1 && nes.ppu.spr0HitY !== -1) {
  // Sprite 0 hit occurred - verify coordinates
  console.log("Sprite 0 hit at:", nes.ppu.spr0HitX, nes.ppu.spr0HitY);
}
```

**Automated Verification:**
```javascript
function verifySprite0Hit(nes, expectedX, expectedY) {
  nes.frame();
  nes.ppu.triggerRendering();  // Ensure rendering is complete

  let actualX = nes.ppu.spr0HitX;
  let actualY = nes.ppu.spr0HitY;

  if (actualX === expectedX && actualY === expectedY) {
    return { passed: true };
  }
  return {
    passed: false,
    expected: { x: expectedX, y: expectedY },
    actual: { x: actualX, y: actualY }
  };
}
```

### Method 3: PPU Status Register Verification

Monitor PPU status register changes during VBlank.

**Test Logic:**
```javascript
// VBlank should start at scanline 261
nes.frame();  // This triggers VBlank

// Check PPU status
let status = nes.ppu.readStatusRegister();
let vblankSet = (status & 0x80) !== 0;  // Bit 7 = VBlank flag
let sprite0Hit = (status & 0x40) !== 0;  // Bit 6 = Sprite 0 hit
```

### Method 4: VRAM Content Verification

Read back VRAM to verify writes:

```javascript
function verifyVRAMWrite(nes, address, expectedValue) {
  // Write to VRAM
  nes.ppu.writeVRAMAddress(address >> 8);
  nes.ppu.writeVRAMAddress(address & 0xFF);
  nes.ppu.vramWrite(expectedValue);

  // Read back
  nes.ppu.writeVRAMAddress(address >> 8);
  nes.ppu.writeVRAMAddress(address & 0xFF);
  let actualValue = nes.ppu.vramLoad();

  return actualValue === expectedValue;
}
```

### Method 5: Complete PPU Debug Output

For comprehensive debugging, add this to your PPU implementation:

```javascript
PPU.prototype.debugRender = function() {
  console.log("=== PPU Debug ===");
  console.log("Scanline:", this.scanline);
  console.log("CurX:", this.curX);
  console.log("VBlank:", this.f_nmiOnVblank);
  console.log("NMI Counter:", this.nmiCounter);
  console.log("Sprites Visible:", this.f_spVisibility);
  console.log("Background Visible:", this.f_bgVisibility);
  console.log("Sprite 0 Hit:", this.spr0HitX !== -1);
  if (this.spr0HitX !== -1) {
    console.log("  Hit at:", this.spr0HitX, this.spr0HitY);
  }

  // Print first 16 bytes of VRAM
  console.log("VRAM[0-15]:", Array.from(this.vramMem.slice(0, 16))
    .map(x => x.toString(16).toUpperCase().padStart(2, '0'))
    .join(' '));
};
```

### Method 6: Frame Comparison (Headless Testing)

Generate and compare full frame buffers:

```javascript
function compareFrames(actualFrame, expectedFrame) {
  let differences = 0;
  let maxDiffs = 10;  // Only report first 10

  for (let i = 0; i < actualFrame.length; i++) {
    if (actualFrame[i] !== expectedFrame[i]) {
      differences++;
      if (differences <= maxDiffs) {
        let x = i % 256;
        let y = Math.floor(i / 256);
        console.log(`Pixel ${i} at (${x},${y}): expected ${expectedFrame[i].toString(16)}, got ${actualFrame[i].toString(16)}`);
      }
    }
  }

  return { passed: differences === 0, diffCount: differences };
}

function generateExpectedFrame ROM name) {
  // Known frame from reference emulator or hardware
  // Store as binary file or array
}
```

---

## Automated Test Framework

### Test Runner (Node.js)

```javascript
const fs = require('fs');
const NES = require('./nes').NES;
const Controller = require('./controller').Controller;

function runNestest(romPath) {
  return new Promise((resolve) => {
    const romData = fs.readFileSync(romPath);

    const nes = new NES({
      onFrame: function() {},  // Discard frames
      onAudioSample: function() {},
    });

    nes.loadROM(romData.toString('binary'));

    // Enter automation mode - set PC to $C000
    nes.cpu.REG_PC = 0xc000 - 1;

    // Run until test completes (max 100000 instructions)
    let count = 0;
    while (count < 100000) {
      nes.cpu.emulate();
      count++;

      // Check for test completion (mem[0x35] cleared)
      if (nes.cpu.mem[0x35] === 0) break;
    }

    resolve({
      mem10: nes.cpu.mem[0x10],  // Official opcodes
      mem11: nes.cpu.mem[0x11],  // Unofficial opcodes
      instructions: count
    });
  });
}

function runAccuracyCoin(romPath) {
  return new Promise((resolve) => {
    const romData = fs.readFileSync(romPath);

    const nes = new NES({
      onFrame: function() {},
      onAudioSample: function() {},
    });

    nes.loadROM(romData.toString('binary'));

    // Simulate Start button
    nes.frame();
    nes.buttonDown(1, Controller.BUTTON_START);
    for (let i = 0; i < 5; i++) nes.frame();
    nes.buttonUp(1, Controller.BUTTON_START);

    // Run until tests complete
    let maxFrames = 30000;
    for (let f = 0; f < maxFrames; f++) {
      nes.frame();
      if (nes.cpu.mem[0x35] === 0) break;
    }

    // Collect results
    const results = {};
    // List of all test addresses would go here

    resolve({ results });
  });
}

// Run tests
async function runAllTests() {
  const nestestResult = await runNestest('test_roms/nestest/nestest.nes');
  console.log('nestest.nes:', nestestResult);

  const accuracyResult = await runAccuracyCoin('test_roms/AccuracyCoin/AccuracyCoin.nes');
  console.log('AccuracyCoin.nes:', accuracyResult);
}

runAllTests();
```

---

## Testing Checklist

### Phase 1: CPU Only (nestest.nes)
- [ ] Test ROM loads without crash
- [ ] PC is set to `$C000` correctly
- [ ] Tests complete within 100000 instructions
- [ ] `mem[$10] = $00` (official opcodes pass)
- [ ] `mem[$11] = $00` (unofficial opcodes pass)

### Phase 2: Basic PPU (croom.nes)
- [ ] ROM loads without crash
- [ ] Frame buffer is 256x240 pixels
- [ ] White pixels appear at expected positions
- [ ] Frame 3 has white pixel at index 2056
- [ ] Frame 4 has white pixel at index 4104

### Phase 3: Comprehensive (AccuracyCoin.nes)
- [ ] Start button triggers test run
- [ ] All tests complete within 30000 frames
- [ ] All test results are PASS (or known acceptable failures)

---

## Summary

| Test ROM | Size | Pass Criteria | Automation |
|----------|------|---------------|------------|
| nestest.nes | 24KB | `mem[$10] = $00`, `mem[$11] = $00` | PC=$C000, check mem[0x10,0x11] |
| croom.nes | 24KB | White pixel at expected indices | Pixel comparison |
| AccuracyCoin.nes | 40KB | All tests PASS | Start button, check mem |

**Key Points:**
- **nestest.nes** is the gold standard for CPU verification
- **croom.nes** enables headless PPU rendering tests
- **AccuracyCoin.nes** provides comprehensive 134-test hardware verification
- All tests can run fully automated with no human operator

---

**Document Version:** 1.0
**Last Updated:** 2026-02-14
**Based on:** JSNES 1.2.1
**Test ROMs Location:** `test_roms/` directory