# NES Emulator Implementation Details

**Source:** JSNES (https://github.com/bfirsh/jsnes)
**Additional Documentation for Implementation**

---

## Table of Contents

1. [CPU Implementation Reference](#cpu-implementation-reference)
2. [PPU Rendering Implementation](#ppu-rendering-implementation)
3. [APU Channel Implementations](#apu-channel-implementations)
4. [Test ROMs and Verification](#test-roms-and-verification)
5. [Example Implementation](#example-implementation)

---

## CPU Implementation Reference

### Complete 6502 CPU Class Structure

```javascript
class CPU {
  // Registers
  REG_ACC = 0;     // Accumulator
  REG_X = 0;       // Index X
  REG_Y = 0;       // Index Y
  REG_SP = 0xff;   // Stack pointer ($0100-$01FF)
  REG_PC = 0;      // Program counter

  // Status flags
  F_CARRY = 0;
  F_ZERO = 1;
  F_INTERRUPT = 1;  // Disabled by default
  F_DECIMAL = 0;
  F_OVERFLOW = 0;
  F_SIGN = 0;

  // Memory
  mem = new Uint8Array(0x10000);  // 64KB
  dataBus = 0;  // Open bus latch

  // Cycle tracking
  cycles = 0;
  apuCatchupCycles = 0;
  ppuCatchupDots = 0;

  // Interrupts
  IRQ_NORMAL = 1;
  IRQ_NMI = 2;
  IRQ_RESET = 3;

  // Methods
  reset();
  emulate();  // Run one instruction, return cycle count
  requestIrq(type);
  load(address);
  write(address, value);
  load16bit(address);
  push(value);
  pull();
  toJSON();
  fromJSON(state);
}
```

### Interrupt Vectors

```
$FFFA-$FFFB: NMI vector (read on NMI)
$FFFC-$FFFD: Reset vector (read on reset)
$FFFE-$FFFF: IRQ/BRK vector (read on IRQ or BRK)
```

### CPU Reset Sequence

```javascript
reset() {
  // Reset flags
  this.F_CARRY = 0;
  this.F_ZERO = 1;
  this.F_INTERRUPT = 1;
  this.F_DECIMAL = 0;
  this.F_OVERFLOW = 0;
  this.F_SIGN = 0;
  this.REG_SP = 0xfd;  // Stack starts at $01FD

  // Read reset vector
  let lo = this.mem[0xfffc] | (this.mem[0xfffd] << 8);
  this.REG_PC = lo;

  // Request reset interrupt
  this.requestIrq(this.IRQ_RESET);
}
```

### Key Implementation Notes

1. **Open Bus Behavior**: Reads from unmapped addresses return the last written value
2. **Page Crossing Penalty**: Absolute indexed addressing costs +1 cycle when crossing page boundary
3. **Dummy Reads**: RMW instructions (ASL, LSR, ROL, ROR, INC, DEC) require a dummy read before write
4. **BRK Instruction**: Forces B flag, sets I flag, jumps to $FFFE

---

## PPU Rendering Implementation

### Complete PPU Class Structure

```javascript
class PPU {
  // Memory
  vramMem = new Uint8Array(0x8000);    // 32KB VRAM
  spriteMem = new Uint8Array(0x100);   // 256-byte OAM
  buffer = new Uint32Array(256 * 240); // Frame buffer (RGB565 or RGB888)
  bgbuffer = new Uint32Array(256 * 240); // Background buffer
  pixrendered = new Uint32Array(256 * 240); // Priority tracking (65 = background)

  // Pattern tables (512 tiles, each with 64 bytes)
  ptTile = new Array(512);

  // Nametables (4)
  nameTable = [new NameTable(32, 30), ...];

  // Palette table
  paletteTable = new PaletteTable();
  imgPalette = new Uint32Array(64);  // Current palette with emphasis

  // Sprite data (64 sprites)
  sprX = new Uint8Array(64);
  sprY = new Uint8Array(64);
  sprTile = new Uint8Array(64);
  sprCol = new Uint8Array(64);
  vertFlip = new Uint8Array(64);
  horiFlip = new Uint8Array(64);
  bgPriority = new Uint8Array(64);

  // Rendering state
  curX = 0;        // Current PPU dot (0-340)
  scanline = -1;   // Current scanline (-1 to 261)
  lastRenderedScanline = -1;

  // Scrolling counters
  cntFV = 0;  // Fine vertical scroll
  cntV = 0;   // Vertical nametable select
  cntH = 0;   // Horizontal nametable select
  cntVT = 0;  // Vertical tile index
  cntHT = 0;  // Horizontal tile index
  regFH = 0;  // Fine horizontal scroll

  // Register mirrors
  regFV = 0, regV = 0, regH = 0, regVT = 0, regHT = 0, regS = 0;

  // Control flags
  f_nmiOnVblank = 0;
  f_spriteSize = 0;      // 0=8x8, 1=8x16
  f_bgPatternTable = 0;  // 0=$0000, 1=$1000
  f_spPatternTable = 0;
  f_addressIncrement = 0;  // 0=+1, 1=+32
  f_nametableSelect = 0;   // 0=$2000, 1=$2400, etc.
  f_spVisibility = 0;
  f_bgVisibility = 0;
  f_spClipping = 0;
  f_bgClipping = 0;
  f_dispType = 0;  // 0=color, 1=mono
  f_emphasis = 0;

  // Status flags
  STATUS_VBLANK = 0x80;
  STATUS_SPRITE0HIT = 0x40;
  STATUS_SPRITEOVERFLOW = 0x20;

  requestEndFrame = false;
  nmiCounter = 0;

  // Methods
  startFrame();
  endScanline();
  startVBlank();
  triggerRendering();
  renderFramePartially();
  renderBgScanline();
  renderSpritesPartially();
  checkSprite0();
  readStatusRegister();
  writeSRAMAddress();
  sramWrite();
  scrollWrite();
  writeVRAMAddress();
  vramWrite();
  sramDMA();
  setMirroring();
  updateControlReg1();
  updateControlReg2();
  isPixelWhite();
  toJSON();
  fromJSON();
}
```

### PPU Timing Constants

```javascript
const NTSC_DOTS_PER_SCANLINE = 341;
const NTSC_SCANLINES_PER_FRAME = 262;
const NTSC_PIXELS_PER_SCANLINE = 256;

// NTSC Clock speeds
const CPU_FREQ_NTSC = 1789772.5;  // Hz
const PPU_FREQ_NTSC = 5369317.5;  // Hz (3x CPU)
```

### Frame Timing Sequence

```
Scanline -1:   Pre-render (initializes scanline 0)
Scanline 0-19: Pre-render scanlines (hidden, timing setup)
Scanline 20:   VBlank start (clears flags, resets scroll)
Scanline 21:   First visible scanline
Scanline 260:  Last visible scanline
Scanline 261:  VBlank (NMI triggered if enabled)
```

### VRAM Addressing

The 15-bit VRAM address is stored in two parts:
- **cntFV, cntV, cntH, cntVT, cntHT**: Current address counters
- **regFV, regV, regH, regVT, regHT**: Register mirrors

Address calculation:
```
vramAddress = (cntV << 11) | (cntH << 10) | (cntVT << 5) | cntHT
```

### OAM DMA ($4014)

```
1. Write page address to $4014
2. CPU stalls for 513 cycles
3. Transfer 256 bytes from CPU page to OAM
4. Transfer time: 512 cycles + 1 cycle overhead
```

---

## APU Channel Implementations

### Complete PAPU Class Structure

```javascript
class PAPU {
  // Channels
  square1 = new ChannelSquare(this, true);
  square2 = new ChannelSquare(this, false);
  triangle = new ChannelTriangle(this);
  noise = new ChannelNoise(this);
  dmc = new ChannelDM(this);

  // Frame counter
  frameCycleCounter = 0;
  frameStep = 0;
  countSequence = 0;  // 0=4-step, 1=5-step
  frameIrqActive = false;

  // Timing constants
  FRAME_STEPS_4 = [7457, 14913, 22371, 29829];
  FRAME_STEPS_5 = [7457, 14913, 22371, 29829, 37281];

  // Audio output
  sampleTimer = 0;
  sampleTimerMax = 0;
  accSample = 0;  // Accumulator for averaging
  accCount = 0;

  // DAC tables (precomputed)
  square_table = new Uint32Array(32 * 16);
  tnd_table = new Uint32Array(204 * 16);
  dcValue = 0;
  dacRange = 0;

  // Output
  onAudioSample = null;
  sampleRate = 48000;
  masterVolume = 1.0;

  // Stereo panning
  stereoPosL1 = 64, stereoPosR1 = 0;
  stereoPosL2 = 0, stereoPosR2 = 64;
  stereoPosLT = 64, stereoPosLN = 32, stereoPosLD = 32;
  stereoPosRR1 = 0, stereoPosRL1 = 64;
  stereoPosRR2 = 64, stereoPosRL2 = 0;
  stereoPosRT = 64, stereoPosRN = 32, stereoPosRD = 32;

  // Methods
  clockFrameCounter(nCycles, catchup);
  advanceFrameCounter(nCycles);
  fireFrameStep(step);
  clockQuarterFrame();
  clockHalfFrame();
  updateChannelEnable(value);
  sample();
  setSampleRate();
  readReg(address);
  writeReg(address, value);
  exWrite(address, value);
  toJSON();
  fromJSON();
}
```

### Frame Counter Sequencing

**4-Step Mode (countSequence = 0):**
- Step 0 (7457 cycles): Quarter frame - envelope decay, linear counter
- Step 1 (14913 cycles): Half frame - length counter, sweep
- Step 2 (22371 cycles): Quarter frame - envelope decay, linear counter
- Step 3 (29829 cycles): Half frame + frame IRQ flag

**5-Step Mode (countSequence = 1):**
- Steps 0-3 same as 4-step
- Step 4 (37281 cycles): Quarter + half frame operations

### DAC Volume Calculation

**Square Table Formula:**
```javascript
for (i = 0; i < 32 * 16; i++) {
  let volume = i / 16.0;
  value = 95.52 / (8128.0 / volume + 100.0) * 0.98411 * 50000.0;
  square_table[i] = value;
}
```

**TND Table Formula:**
```javascript
for (i = 0; i < 204 * 16; i++) {
  let volume = i / 16.0;
  value = 163.67 / (24329.0 / volume + 100.0) * 0.98411 * 50000.0;
  tnd_table[i] = value;
}
```

### Audio Mixing

```javascript
// Square channels
sq_index = (smpSquare1 * stereoPosL1 + smpSquare2 * stereoPosL2) >> 8;

// TND channels (3x multiplier for triangle)
tnd_index = (3 * smpTriangle * stereoPosLT +
             (smpNoise << 1) * stereoPosLN +
             smpDmc * stereoPosLD) >> 8;

// Combine and subtract DC offset
sampleValueL = square_table[sq_index] + tnd_table[tnd_index] - dcValue;
```

---

## Test ROMs and Verification

### Test ROMs Available

| ROM | Size | Purpose |
|-----|------|---------|
| nestest.nes | 24KB | CPU instruction validation (Kevin Horton) |
| AccuracyCoin.nes | 40KB | Comprehensive accuracy tests (134+ tests) |
| lj65.nes | 24KB | 6502 instruction tests |
| croom.nes | 24KB | Frame buffer test |

### nestest.nes Testing

**Purpose**: Validates CPU instruction implementation

**How it works:**
1. Writes test results to $02-$03
2. Tests all addressing modes
3. Tests flag operations
4. Tests branch instructions
5. Tests invalid opcodes (when Select held)

**Pass indicators:**
- Nintendulator: Passes
- Real NES: Passes
- Nesten: Fails
- Nesticle: Fails

### AccuracyCoin.nes Testing

**Test Categories:**
- CPU behavior (RAM mirroring, PC wraparound, decimal flag)
- Unofficial instructions (SLO, RLA, SRE, RRA, SAX, LAX, DCP, ISC, SH*)
- PPU tests (CHR ROM protection, palette quirks, VBlank timing)
- APU tests (length counters, frame counter, DMC)
- DMA tests (bus conflicts, aborts)

### Test Verification Workflow

```
1. Run nestest.nes
   - If fails: Check CPU implementation (6502 opcodes, flags, addressing)

2. Run AccuracyCoin.nes
   - If fails: Check PPU/APU/Controller implementation

3. Run real ROMs
   - Visual verification of graphics
   - Audio verification
   - Input verification
```

---

## Example Implementation

### Minimal NES Emulator (Pseudo-code)

```javascript
// 1. Initialize components
let cpu = new CPU();
let ppu = new PPU();
let papu = new PAPU();
let mmap = null;  // Will be set after ROM load

// 2. Load ROM
function loadROM(romData) {
  // Validate header
  if (romData.substring(0, 4) !== "NES\x1a") {
    throw new Error("Invalid NES ROM");
  }

  // Parse header
  let prgCount = romData.charCodeAt(4);
  let chrCount = romData.charCodeAt(5) * 2;
  let mapperType = ((romData.charCodeAt(6) >> 4) | (romData.charCodeAt(7) & 0xF0)) & 0xFF;

  // Load PRG-ROM
  let rom = new Array(prgCount);
  for (let i = 0; i < prgCount; i++) {
    rom[i] = new Uint8Array(16384);
    for (let j = 0; j < 16384; j++) {
      rom[i][j] = romData.charCodeAt(16 + i * 16384 + j);
    }
  }

  // Load CHR-ROM
  let vrom = new Array(chrCount);
  for (let i = 0; i < chrCount; i++) {
    vrom[i] = new Uint8Array(4096);
    for (let j = 0; j < 4096; j++) {
      vrom[i][j] = romData.charCodeAt(16 + prgCount * 16384 + i * 4096 + j);
    }
  }

  // Create mapper
  mmap = createMapper(mapperType);
  mmap.loadROM();

  // Load CHR into PPU
  ppu.loadCHR(vrom);
}

// 3. Main frame loop
function runFrame() {
  ppu.startFrame();

  for (;;) {
    if (cpu.cyclesToHalt === 0) {
      // Run CPU instruction
      let cycles = cpu.emulate();

      // Update APU
      papu.clockFrameCounter(cycles, cpu.apuCatchupCycles);
      cpu.apuCatchupCycles = 0;

      // Calculate PPU catchup (3 CPU cycles = 1 PPU dot)
      cycles = cycles * 3 - cpu.ppuCatchupDots;
      cpu.ppuCatchupDots = 0;

      if (cpu.ppuFrameEnded) {
        ppu.curX += cycles;
        cpu.ppuFrameEnded = false;
        break;  // Frame complete
      }
    } else {
      // PPU catchup
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
      // Sprite 0 hit detection
      if (ppu.curX === ppu.spr0HitX && ppu.f_spVisibility === 1) {
        ppu.setStatusFlag(ppu.STATUS_SPRITE0HIT, true);
      }

      // VBlank check
      if (ppu.requestEndFrame) {
        ppu.nmiCounter--;
        if (ppu.nmiCounter === 0) {
          ppu.startVBlank();
          break;
        }
      }

      ppu.curX++;
      if (ppu.curX === 341) {
        ppu.curX = 0;
        ppu.endScanline();
      }
    }
  }

  // Render frame to screen
  renderFrame(ppu.buffer);

  // Get audio samples
  let left, right;
  papu.sample(left, right);
  playAudio(left, right);
}

// 4. Keyboard input mapping
function handleKey(key, pressed) {
  switch(key) {
    case "ArrowUp": pressed ? buttonDown(1, BUTTON_UP) : buttonUp(1, BUTTON_UP); break;
    case "ArrowDown": pressed ? buttonDown(1, BUTTON_DOWN) : buttonUp(1, BUTTON_DOWN); break;
    case "ArrowLeft": pressed ? buttonDown(1, BUTTON_LEFT) : buttonUp(1, BUTTON_LEFT); break;
    case "ArrowRight": pressed ? buttonDown(1, BUTTON_RIGHT) : buttonUp(1, BUTTON_RIGHT); break;
    case "a": case "A": pressed ? buttonDown(1, BUTTON_A) : buttonUp(1, BUTTON_A); break;
    case "s": case "S": pressed ? buttonDown(1, BUTTON_B) : buttonUp(1, BUTTON_B); break;
    case "Tab": pressed ? buttonDown(1, BUTTON_SELECT) : buttonUp(1, BUTTON_SELECT); break;
    case "Enter": pressed ? buttonDown(1, BUTTON_START) : buttonUp(1, BUTTON_START); break;
  }
}

// 5. Controller strobe logic
function controllerRead(controllerNum) {
  let controller = controllers[controllerNum];

  if (joypadLastWrite & 1) {
    // Strobe mode - return button state
    return controller.state[0];
  }

  // Shift register mode
  let ret = controller.state[controller.strobeState];
  controller.strobeState++;
  if (controller.strobeState >= 8) {
    controller.strobeState = 0;
  }
  return ret;
}
```

### Memory Access Pattern

```
CPU Memory Map:
$0000-$07FF: CPU RAM (2KB, first bank)
$0800-$0FFF: CPU RAM mirror
$1000-$1FFF: CPU RAM mirror

$2000-$2007: PPU registers (8 bytes)
$2008-$3FFF: PPU registers mirror

$4000-$4013: APU registers
$4014: OAM DMA
$4015: APU channel enable
$4016: Controller 1
$4017: Controller 2 / Frame counter

$4018-$5FFF: Unused (expansion port)

$6000-$7FFF: Battery RAM (if present)

$8000-$FFFF: PRG-ROM (via mapper)
```

### PPU Register Access

```
$2000 (PPUCTRL) - Write only
  - NMI enable
  - Sprite size
  - Pattern table addresses
  - Address increment
  - Nametable select

$2001 (PPUMASK) - Write only
  - Color emphasis
  - Sprite/BG visibility
  - Clipping
  - Display type

$2002 (PPUSTATUS) - Read only
  - VBlank flag
  - Sprite 0 hit
  - Sprite overflow
  - Open bus latch

$2003 (OAMADDR) - Write only
  - OAM address pointer

$2004 (OAMDATA) - Read/Write
  - OAM data I/O

$2005 (PPUSCROLL) - Write only (2-byte sequence)
  - First write: Horizontal scroll
  - Second write: Vertical scroll

$2006 (PPUADDR) - Write only (2-byte sequence)
  - First write: High byte
  - Second write: Low byte

$2007 (PPUDATA) - Read/Write
  - VRAM data I/O

$4014 (OAMDMA) - Write only
  - DMA transfer page
```

### APU Register Access

```
$4000/$4004 - Square 1/2 Control
  - Envelope decay rate
  - Duty mode
  - Loop enable

$4001/$4005 - Square 1/2 Sweep
  - Sweep enable
  - Sweep period
  - Add/sub mode
  - Shift amount

$4002/$4006 - Square 1/2 Timer Low
$4003/$4007 - Square 1/2 Timer High/Length

$4008 - Triangle Control
  - Linear counter control
  - Linear counter load

$400A - Triangle Timer Low
$400B - Triangle Timer High/Length

$400C - Noise Control
  - Envelope decay rate
  - Loop enable

$400E - Noise Wavelength/Mode
  - Timer value
  - Random mode

$400F - Noise Length

$4010 - DMC Control
  - Play mode
  - Frequency

$4011 - DMC DAC
  - Delta counter load
  - DAC LSB

$4012 - DMC Address
  - Start address (shifted)

$4013 - DMC Length
  - Play length

$4015 - Channel Enable
  - Channel enable bits
  - Read: Returns channel status

$4017 - Frame Counter
  - Frame IRQ enable
  - Count sequence (4-step/5-step)
```

---

## Additional Notes

### PAL vs NTSC

**Clock Speeds:**
- NTSC: CPU 1.79 MHz, PPU 5.37 MHz
- PAL: CPU 1.66 MHz, PPU 4.99 MHz

**Frame Rates:**
- NTSC: 60 Hz (16.6ms per frame)
- PAL: 50 Hz (20ms per frame)

### Save States

To implement save states, serialize:
1. CPU state (registers, memory, flags)
2. PPU state (registers, VRAM, OAM, timers)
3. PAPU state (registers, channel states, frame counter)
4. Mapper state (bank registers, IRQ counters)
5. Controller state (shift register position)

### Performance Tips

1. **PPU Rendering**: Cache tiles in `ptTile` array to avoid re-decoding
2. **Nametable Mirroring**: Use lookup table for fast address translation
3. **Audio**: Precompute DAC tables, accumulate samples over multiple cycles
4. **CPU**: Use direct lookup for opcodes, avoid switch statement overhead

---

**Document Version:** 1.0
**Last Updated:** 2026-02-14
**Based on:** JSNES 1.2.1