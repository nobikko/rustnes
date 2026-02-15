# NES PPU (Picture Processing Unit) Specification

**Source References:**
- Ricoh 2C02/2C03/2C05 technical documentation
- https://www.tekepen.com/nes/ppu.html
- NES Dev Wiki (nesdev.com)

---

## Table of Contents

1. [PPU Architecture](#ppu-architecture)
2. [Memory Map](#memory-map)
3. [PPU Registers](#ppu-registers)
4. [Rendering Pipeline](#rendering-pipeline)
5. [Timing Specifications](#timing-specifications)
6. [Sprite Handling](#sprite-handling)
7. [Nametables and Attribute Tables](#nametables-and-attribute-tables)
8. [Pattern Tables and Tile Rendering](#pattern-tables-and-tile-rendering)
9. [Palette System](#palette-system)
10. [Scroll Registers and Raster Scrolling](#scroll-registers-and-raster-scrolling)
11. [NMI Generation and VBlank Handling](#nmi-generation-and-vblank-handling)
12. [Implementation Guide](#implementation-guide)

---

## PPU Architecture

### PPU Variants

| Variant | Clock | Usage | Notes |
|---------|-------|-------|-------|
| Ricoh 2C02 | 3.58 MHz (NTSC) | Standard NES | 2048 bytes VRAM, 64 bytes OAM |
| Ricoh 2C03 | 3.55 MHz (PAL) | PAL NES | Same as 2C02, different clock |
| Ricoh 2C05 | 3.58 MHz | MMC3-based NES | Includes MMC3 mapper controller |

### PPU Specifications

- **Transistors:** 2,400,000
- **VRAM:** 2 KiB (2048 bytes) built-in
- **OAM:** 64 bytes (64 sprites x 4 bytes) built-in
- **Shift Registers:** 16 8-bit shift registers for sprite rendering
- **Output:** 256x240 pixels at 60 Hz (NTSC) / 50 Hz (PAL)

### PPU Clocking

```
NTSC PPU Clock: 3.579545 MHz (13.5 MHz / 3.75)
PAL PPU Clock:  3.546895 MHz

CPU Clock: PPU Clock / 12
NTSC CPU: 1.789773 MHz
PAL CPU:  1.773448 MHz
```

---

## Memory Map

### VRAM Address Space ($0000-$1FFF)

```
$0000-$0FFF: Pattern Table 0 (4096 bytes)
             - 256 tiles at 16 bytes each
             - 8x8 pixel tiles, 2 bits per pixel
             - Default background pattern table

$1000-$1FFF: Pattern Table 1 (4096 bytes)
             - 256 tiles at 16 bytes each
             - Used when PPUCTRL bit 4/3 is set

$2000-$23FF: Nametable 0 (1024 bytes)
             - 32x30 tile indices
             - Attribute table at $23C0-$23FF

$2400-$27FF: Nametable 1 (1024 bytes)
             - Attribute table at $27C0-$27FF

$2800-$2BFF: Nametable 2 (1024 bytes)
             - Attribute table at $2BC0-$2BFF

$2C00-$2FFF: Nametable 3 (1024 bytes)
             - Attribute table at $2FC0-$2FFF

$3000-$3EFF: Mirrors of $2000-$2EFF
$3F00-$3F1F: Palette Memory (32 bytes)
$3F20-$3FFF: Mirrors of $3F00-$3F1F
```

### Nametable Structure

```
Each Nametable:
  Tile Indices: 32x30 = 960 bytes ($000-$3BF)
  Attribute Table: 64 bytes ($3C0-$3FF)

Attribute Table Layout (8x8 grid):
  Each byte controls 4x4 tile blocks:
  +----+----+----+----+
  | 0  | 1  | 2  | 3  |  Byte bits 7-6: Top-left
  +----+----+----+----+  Byte bits 5-4: Top-right
  | 4  | 5  | 6  | 7  |  Byte bits 3-2: Bottom-left
  +----+----+----+----+  Byte bits 1-0: Bottom-right
  | 8  | 9  | A  | B  |
  +----+----+----+----+
  | C  | D  | E  | F  |
  +----+----+----+----+

Each 4-bit nibble contains 2-bit palette index.
```

### Memory Mirroring

```
Nametable Mirroring (every $1000 bytes):
  $2000-$2FFF = $3000-$3FFF = $4000-$4FFF = ...

Palette Mirroring (every $20 bytes):
  $3F00-$3F1F = $3F20-$3F3F = $3F40-$3F5F = ...
```

---

## PPU Registers

### Register Map

| Address | Name | Type | Description |
|---------|------|------|-------------|
| $2000 | PPUCTRL | Write | PPU Control |
| $2001 | PPUSCRSEL | Write | PPU Mask / Scrolling |
| $2002 | PPUSTATUS | Read | PPU Status |
| $2003 | OAMADDR | Write | OAM Address |
| $2004 | OAMDATA | Read/Write | OAM Data |
| $2005 | PPUSCROLL | Write | PPU Scroll (Double Buffer) |
| $2006 | PPUADDR | Write | PPU Address (Double Buffer) |
| $2007 | PPUDATA | Read/Write | PPU Data |
| $4014 | OAMDMA | Write | OAM DMA Transfer |

---

### Register $2000 - PPUCTRL (Write-Only)

```
Bit 7 - VBA (Video Memory Increment Address)
        0 = Increment by 1 (horizontal)
        1 = Increment by 32 (vertical)

Bit 6 - NMI at VBlank
        0 = Disable NMI at VBlank
        1 = Enable NMI at VBlank (generated at end of scanline 241)

Bit 5 - Slave Mode (not used on NTSC)
        0 = Master mode (sprite priority)
        1 = Slave mode (background priority)

Bit 4 - Pattern Table Background
        0 = Pattern Table 0 at $0000 (background)
        1 = Pattern Table 1 at $1000 (background)

Bit 3 - Pattern Table Sprite
        0 = Pattern Table 0 at $0000 (sprites)
        1 = Pattern Table 1 at $1000 (sprites)

Bit 2 - Fine Y Scroll (bit 0)
        0 = Coarse Y = 0
        1 = Coarse Y = 1

Bit 1 - Fine Y Scroll (bit 1)
        0 = Coarse Y = 0
        1 = Coarse Y = 1

Bit 0 - Coarse X Scroll
        0 = Coarse X = 0
        1 = Coarse X = 1
```

**Register $2000 Bit Descriptions:**

| Bits | Field | Description |
|------|-------|-------------|
| 7 | VBA | VRAM address increment (1 or 32) |
| 6 | NMIEN | NMI enable on VBlank |
| 5 | SLV | Master/slave select |
| 4 | BGPN | Background pattern table base |
| 3 | SP8PN | Sprite pattern table base |
| 2-1 | Fine Y | Fine vertical scroll (0-3) |
| 0 | Coarse X | Coarse horizontal scroll (0-31) |

---

### Register $2001 - PPUSCRSEL (Write-Only)

```
Bit 7 - BG EX (Background Enable)
        0 = Disable background rendering
        1 = Enable background rendering

Bit 6 - SPR EX (Sprite Enable)
        0 = Disable sprite rendering
        1 = Enable sprite rendering

Bit 5 - BG CLIP (Background Clipping)
        0 = Show background at left 8 pixels (X=0-7)
        1 = Hide background at left 8 pixels (clipped)

Bit 4 - SPR CLIP (Sprite Clipping)
        0 = Show sprites at left 8 pixels (X=0-7)
        1 = Hide sprites at left 8 pixels (clipped)

Bit 3 - GR (Greyscale)
        0 = Normal color output
        1 = Greyscale mode (reduce color saturation)

Bit 2 - R (Red intensify)
        1 = Intensify red component

Bit 1 - G (Green intensify)
        1 = Intensify green component

Bit 0 - B (Blue intensify)
        1 = Intensify blue component
```

**Rendering Control:**

| BG EX | SPR EX | Result |
|-------|--------|--------|
| 0 | 0 | Nothing rendered |
| 1 | 0 | Background only |
| 0 | 1 | Sprites only |
| 1 | 1 | Both background and sprites |

---

### Register $2002 - PPUSTATUS (Read-Only)

```
Bit 7 - V (Vertical Blank)
        1 = PPU is in VBlank period (scanlines 241-260)
        Read clears this flag (auto-clears on read)

Bit 6 - S (Sprite 0 Hit)
        1 = Sprite 0 has hit the visible area
        Read clears this flag (auto-clears on read)

Bit 5 - W (Write Protect)
        1 = PPU write protected (scanlines 0-239, rendering active)
        0 = PPU can be written to

Bit 4-0 - Fine Y
          Current fine Y position within the tile
          Updated during rendering
```

**Flag Clearing:**

| Action | VBlank | Sprite 0 Hit |
|--------|--------|--------------|
| Read $2002 | Cleared | Cleared |
| Dot 1 of scanline 241 | Cleared | Cleared |
| Dot 1 of scanline 261 | Set | Reset |

---

### Register $2003 - OAMADDR (Write-Only)

```
8-bit address for OAM access
Sets the initial OAM address for subsequent $2004 accesses

Writing to $2003:
- Sets OAMADDR to the written value
- OAMADDR increments after each $2004 write
```

---

### Register $2004 - OAMDATA (Read/Write)

```
OAM data port
Writes: Writes byte to OAM at current OAMADDR, increments OAMADDR
Reads: Returns byte from OAM at current OAMADDR (no increment)
```

**OAM Structure (64 sprites, 4 bytes each):**

```
Sprite n (n = 0-63):
  Byte 0: Y (Vertical position - 1 line above visible area)
  Byte 1: Tile Index (pattern table index)
  Byte 2: Attributes (flags)
  Byte 3: X (Horizontal position)
```

**Sprite Attribute Byte:**

```
Bit 7 - Priority (P)
        0 = Sprite in front of background
        1 = Sprite behind background

Bit 6 - Fine Y Flip (F)
        0 = Normal vertical orientation
        1 = Flip sprite vertically

Bit 5 - Horizontal Flip (H)
        0 = Normal horizontal orientation
        1 = Flip sprite horizontally

Bit 4 - Vertical Flip (V)
        0 = Normal vertical orientation
        1 = Flip sprite vertically (note: F and V are often confused)

Bit 3-0 - Palette (P)
          Sprite palette number (0-3, uses $3F10-$3F1F)
```

---

### Register $2005 - PPUSCROLL (Write-Only, Double Buffer)

```
First write (even): Horizontal scroll (fine X + coarse X)
  Bits 7-3: Coarse X (0-31)
  Bits 2-0: Fine X (0-7)

Second write (odd): Vertical scroll (fine Y + coarse Y)
  Bits 7-3: Coarse Y (0-29, attribute table row)
  Bits 2-0: Fine Y (0-7)
```

**Double Buffer Behavior:**

```
Write 1: Horizontal scroll buffer = (coarse X << 3) | fine X
Write 2: Vertical scroll buffer = (coarse Y << 3) | fine Y

Buffers are applied at start of rendering.
```

---

### Register $2006 - PPUADDR (Write-Only, Double Buffer)

```
Sets the PPU memory address for $2007 reads/writes

First write (even): High byte of address
  Bits 7-0: A15-A8 (upper address bits)

Second write (odd): Low byte of address
  Bits 7-0: A7-A0 (lower address bits)
```

**Address Calculation:**

```
Address = (High Byte << 8) | Low Byte
Range: $0000-$3FFF (14-bit address)
```

---

### Register $2007 - PPUDATA (Read/Write)

```
PPU memory data port
Reads: Returns byte from VRAM at current PPUADDR, increments PPUADDR
Writes: Writes byte to VRAM at current PPUADDR, increments PPUADDR

Address increment based on PPUCTRL bit 7:
  0 = Increment by 1 (horizontal)
  1 = Increment by 32 (vertical)
```

**VRAM Read During Rendering:**

```
During visible rendering, $2007 reads from:
  $2000 + (PPUADDR & $3FFF)

Outside rendering, reads from:
  PPUADDR directly
```

---

### Register $4014 - OAMDMA (Write-Only)

```
OAM Direct Memory Access
Writing a page address ($XX00) initiates DMA transfer

Transfer: 256 bytes from CPU page $XX00-$XXFF to OAM
Duration: 512 CPU cycles (512/12 = ~42.67 PPU cycles)

Writing $4014:
  - Transfers byte value to OAMADDR
  - Initiates 256-byte transfer from $XX00-$XXFF
  - Blocks CPU during transfer
```

---

## Rendering Pipeline

### Dot Cycle Breakdown (341 dots per scanline)

```
NTSC PPU Rendering Timeline:

Dot Cycle    Action
-----------  ---------------------------------------
0-255        Visible rendering (left side of screen)
256-320      Horizontal blank (pre-render)
321-336      Visible rendering (right side, nametable wrap)
337-340      Horizontal blank (post-render, pre-vsync)
```

### Scanline Timeline

```
Scanline      Action
-----------   ----------------------------------------
0-239         Rendering active (240 visible scanlines)
240           Last render scanline (bottom of screen)
241           VBlank start (dot 257), NMI asserted if enabled
242-260       VBlank period (NMI active)
261           VBlank end, NMI deasserted, new frame starts
262           Second field (interlaced mode - optional)
```

### Background Rendering Pipeline

```
Background Rendering Steps:

1. Coarse X/Y incremented every 8 dots
2. Attribute table fetched every 32 dots
3. Pattern table data fetched for tile rendering
4. Palette lookup and color output
```

**Background Fetching:**

```
Every 8 dots:
  - Fetch nametable byte (tile index)
  - Fetch attribute table byte (palette info)
  - Fetch pattern table bytes (low plane)
  - Fetch pattern table bytes (high plane)
```

### Sprite Rendering Pipeline

```
Sprite Rendering Steps:

1. OAM search phase (cycles 0-64): Find visible sprites
2. Sprite evaluation: Check Y position, priority
3. Tile data fetch: Get pattern data from pattern table
4. Pixel composition: Combine sprite with background
```

**Sprite Search Algorithm:**

```
1. Search OAM for sprites with Y position matching current scanline
2. Up to 8 sprites can be rendered per scanline
3. If more than 8, the "last 8" are selected (implementation dependent)
4. This causes the "sprite starvation" effect
```

---

## Timing Specifications

### NTSC Timing

```
PPU Clock: 3.579545 MHz (13.5 MHz / 3.75)

Per Scanline:
  Total dots: 341
  Horizontal blank: ~84 dots
  Visible rendering: ~257 dots
  Scanline duration: ~95.2 microseconds

Per Frame:
  Scanlines: 262
  Frame duration: ~16.67 milliseconds
  Frame rate: ~60.09 Hz
```

### PAL Timing

```
PPU Clock: 3.546895 MHz

Per Scanline:
  Total dots: 341
  Scanline duration: ~96.5 microseconds

Per Frame:
  Scanlines: 312 (typically 262 visible + 50 hidden)
  Frame duration: ~16.67 milliseconds
  Frame rate: ~50.0 Hz
```

### VBlank Timing

```
VBlank starts: End of scanline 241 (after dot 257)
VBlank ends:   Start of scanline 261

VBlank duration:
  NTSC: Scanlines 241-260 = 20 scanlines = ~1.9 ms
  PAL:  Scanlines 241-311 = 70+ scanlines = ~6.7 ms
```

### NMI Timing

```
NMI assertion: Dot 1 of scanline 241
NMI deassertion: Start of scanline 261

CPU must service NMI within 7-8 cycles after NMI is asserted.
```

### DMA Timing

```
OAMDMA at $4014:
  - Takes 512 CPU cycles
  - Cannot be interrupted
  - Blocks CPU during transfer
  - Duration: 512/12 = ~42.67 PPU cycles
```

---

## Sprite Handling

### OAM Structure

```
64 sprites x 4 bytes = 256 bytes total OAM

Sprite Layout:
  Offset 0: Y - Vertical position (0-255)
            Sprite appears one line above this position

  Offset 1: Tile Index - Pattern table tile number (0-255)

  Offset 2: Attributes:
    Bit 7: Priority (0=front, 1=behind background)
    Bit 6: Fine Y Flip
    Bit 5: Horizontal Flip
    Bit 4: Vertical Flip (some implementations)
    Bit 3-0: Palette (0-3)

  Offset 3: X - Horizontal position (0-254)
            Sprite appears at this X position
```

### Sprite 0 Hit Detection

**Conditions for Sprite 0 Hit:**

1. Sprite 0 is enabled and visible
2. Sprite 0's pixels overlap with background pixels
3. Background rendering is enabled
4. Pixel is not in the left clipping region

**Sprite 0 Hit Register Bit:**

- Set during visible rendering when conditions are met
- Read of $2002 clears the bit
- Used for game timing (raster timing)

### Sprite Starvation

```
When more than 8 sprites appear on the same scanline:
- The PPU selects the "last 8" sprites
- This causes the "sprite starvation" effect
- First sprites in OAM order may disappear
```

---

## Nametables and Attribute Tables

### Nametable Layout

```
Nametable 0: $2000-$23BF (tile indices)
             $23C0-$23FF (attribute table)

Nametable structure (32x30 tiles):
  +--------+--------+--------+--------+
  | $2000  | $2100  | $2200  | $2300  |
  | tiles  | tiles  | tiles  | tiles  |
  +--------+--------+--------+--------+
  | $2000  | $2100  | $2200  | $2300  |
  | tiles  | tiles  | tiles  | tiles  |
  +--------+--------+--------+--------+
  (30 rows total)
```

### Attribute Table Structure

```
Attribute Table (64 bytes = 8x8 grid):
  Each byte controls 4x4 tile blocks:
  +----+----+----+----+
  | 0  | 1  | 2  | 3  |  Byte bits 7-6: Top-left
  +----+----+----+----+  Byte bits 5-4: Top-right
  | 4  | 5  | 6  | 7  |  Byte bits 3-2: Bottom-left
  +----+----+----+----+  Byte bits 1-0: Bottom-right
  | 8  | 9  | A  | B  |
  +----+----+----+----+
  | C  | D  | E  | F  |
  +----+----+----+----+

  Each nibble (4 bits) selects 2-bit palette index.
```

### Nametable Mirroring

```
Horizontal mirroring:
  Nametables 0/1 side by side
  Nametables 2/3 side by side

Vertical mirroring:
  Nametables 0/2 stacked
  Nametables 1/3 stacked

4-screen mode (some mappers):
  Separate VRAM for each nametable
```

---

## Pattern Tables and Tile Rendering

### Pattern Table Structure

```
Each pattern table: 4096 bytes = 256 tiles x 16 bytes per tile

16-byte tile layout (16x16 pixels, 2bpp):
  Bytes 0-7:  Low plane (bit 0 of each pixel)
  Bytes 8-15: High plane (bit 1 of each pixel)

Each row of 8 pixels is stored as:
  Low plane byte:  X7 X6 X5 X4 X3 X2 X1 X0
  High plane byte: X7 X6 X5 X4 X3 X2 X1 X0
  (bit 7 = leftmost pixel)
```

### Tile Rendering Process

```
1. Fetch tile index from nametable
2. Calculate pattern table address:
   Address = PatternTableBase + (TileIndex * 16)
3. Fetch low and high plane bytes for each row
4. Combine planes to create 8 pixels:
   Pixel = (High << 1) | Low (for each bit position)
5. Apply palette index from attribute table
6. Output color from palette
```

### Pattern Table Selection

```
PPUCTRL bit 4: Background pattern table
PPUCTRL bit 3: Sprite pattern table

$0000: Pattern Table 0
$1000: Pattern Table 1
```

---

## Palette System

### Palette Memory ($3F00-$3F1F)

```
Background Palettes ($3F00-$3F0F):
  $3F00: Background palette 0, color 0 (transparent)
  $3F01: Background palette 0, color 1
  $3F02: Background palette 0, color 2
  $3F03: Background palette 0, color 3
  $3F04: Background palette 1, color 0
  ...
  $3F0C: Background palette 3, color 0
  $3F0D: Background palette 3, color 1
  $3F0E: Background palette 3, color 2
  $3F0F: Background palette 3, color 3

Sprite Palettes ($3F10-$3F1F):
  $3F10: Sprite palette 0, color 0 (transparent)
  $3F11: Sprite palette 0, color 1
  ...
  $3F1C: Sprite palette 3, color 0
  $3F1D: Sprite palette 3, color 1
  $3F1E: Sprite palette 3, color 2
  $3F1F: Sprite palette 3, color 3
```

### RGB Color Values (NTSC NES)

The NES uses a custom RGB palette with the following values:

```
RGB Palette (approximate):

$00: RGB(0,  0,   0)   - Black (transparent)
$01: RGB(0,  0,   85)  - Dark blue
$02: RGB(0,  0,   170) - Blue
$03: RGB(0,  85,  0)   - Dark green
$04: RGB(0,  85,  85)  - Teal
$05: RGB(0,  85,  170) - Light blue
$06: RGB(0,  170, 0)   - Green
$07: RGB(0,  170, 85)  - Cyan
$08: RGB(0,  170, 170) - Light cyan
$09: RGB(85, 0,   0)   - Dark red
$0A: RGB(85, 0,   85)  - Purple
$0B: RGB(85, 0,   170) - Violet
$0C: RGB(85, 85,  0)   - Brown
$0D: RGB(85, 85,  85)  - Dark gray
$0E: RGB(85, 85,  170) - Light purple
$0F: RGB(85, 170, 0)   - Olive
$10: RGB(85, 170, 85)  - Light green
$11: RGB(85, 170, 170) - Light teal
$12: RGB(170, 0,   0)  - Red
$13: RGB(170, 0,   85) - Pink
$14: RGB(170, 0,   170) - Magenta
$15: RGB(170, 85,  0)  - Orange
$16: RGB(170, 85,  85) - Light red
$17: RGB(170, 85,  170) - Light pink
$18: RGB(170, 170, 0)  - Yellow
$19: RGB(170, 170, 85) - Light orange
$1A: RGB(170, 170, 170) - Light yellow
$1B: RGB(255, 0,   0)  - Bright red
$1C: RGB(255, 0,   85) - Bright pink
$1D: RGB(255, 0,   170) - Bright magenta
$1E: RGB(255, 85,  0)  - Bright orange
$1F: RGB(255, 85,  85) - Bright pink
```

### Intensify Bits

```
PPUSCRSEL bits 2-0 affect color saturation:

Bit 2 (R): Red intensify
Bit 1 (G): Green intensify
Bit 0 (B): Blue intensify

When set, the corresponding color component is intensified.
```

---

## Scroll Registers and Raster Scrolling

### Scroll Register Structure

```
PPUCTRL ($2000) contains coarse X and fine Y scroll bits:
  Bit 0: Coarse X (0-31)
  Bit 1: Coarse Y bit 0 (0-1)
  Bit 2: Fine Y (0-7)

PPUSCROLL ($2005) double-buffered:
  First write: Fine X (bits 2-0) + Coarse X (bits 7-3)
  Second write: Fine Y (bits 2-0) + Coarse Y (bits 7-3)

Internal PPU scroll registers:
  Fine X: 3 bits (0-7)
  Coarse X: 5 bits (0-31)
  Coarse Y: 5 bits (0-29)
```

### Scroll Update Timing

```
Scroll updates occur:
1. When $2005 is written (double-buffered)
2. At start of each frame (from PPUCTRL)
3. When $2006 is written (absolute addressing)
```

### Raster Scrolling (Wavy Screen Effect)

**Implementation requires:**

1. Track current scanline
2. Write to $2005 at specific dots to change scroll
3. Must be done during VBlank or horizontal blank
4. Timing must be precise (dot-accurate)

**Raster Scrolling Example:**

```
At the start of each scanline (after dot 257):
- Write new scroll values to $2005
- This creates effects like:
  - Wavy water
  - Shaking screens
  - Parallax scrolling
```

### Scroll Register Summary

```
After reset, scroll registers:
  Fine X = 0
  Coarse X = 0
  Coarse Y = 0
  Fine Y = 0

When PPU is enabled:
  - Coarse X increments every 8 dots
  - When Coarse X > 31, reset to 0 and increment Coarse Y
  - When Coarse Y > 29, reset to 0 and wrap nametable
  - Fine Y increments when Coarse Y wraps
```

---

## NMI Generation and VBlank Handling

### VBlank Period

```
NTSC:
  VBlank starts: End of scanline 241 (after dot 257)
  VBlank ends:   Start of scanline 261
  Duration:      20 scanlines

PAL:
  VBlank starts: End of scanline 241
  VBlank ends:   Start of scanline 312 (or 262)
  Duration:      70+ scanlines
```

### NMI (Non-Maskable Interrupt)

```
PPUCTRL bit 6 (NMI enable):
  0: NMI disabled
  1: NMI enabled at VBlank

NMI assertion:
  - Asserted at dot 1 of scanline 241
  - Deasserted at the start of scanline 261
  - CPU vector: $FFFA-$FFFB

NMI timing (6502 CPU):
  - CPU finishes current instruction
  - Pushes PC and P to stack
  - Jumps to $FFFA-$FFFB
  - Must service within 7-8 cycles
```

### PPUSTATUS Flag Handling

```
Bit 7 (V - VBlank):
  - Set at start of VBlank
  - Cleared on read of $2002
  - Also cleared at dot 1 of scanline 241 (before VBlank)

Bit 6 (S - Sprite 0 Hit):
  - Set when Sprite 0 hits visible area
  - Cleared on read of $2002
  - Also cleared at dot 1 of scanline 241
```

### VBlank Handler Pattern

```
NMI Handler:
  1. Save CPU registers
  2. Read $2002 (clear VBlank and Sprite 0 flags)
  3. Read $4016 (joystick strobe)
  4. Update game state
  5. Update PPU (VRAM, OAM)
  6. Restore CPU registers
  7. RTI (Return from Interrupt)
```

### PPU Power-On State

```
After reset, PPU state:
  - Rendering disabled (PPUSCRSEL = $00)
  - NMI disabled (PPUCTRL = $00)
  - Scroll: Fine X=0, Coarse X=0, Coarse Y=0, Fine Y=0
  - VRAM address: $0000
  - OAM address: $00
  - OAMDMA not active

First few frames:
  - Frames 1-2: No rendering (PPU warming up)
  - Frame 3: Normal rendering begins
```

### VBlank Timing Diagram

```
Scanline Timeline (NTSC):

  0-239   : Rendering active
  240     : Last render scanline (bottom of screen)
  241     : VBlank start (dot 257), NMI asserted
  242-260 : VBlank period (NMI active)
  261     : VBlank end, NMI deasserted, new frame starts
  262     : Second field (interlaced mode)
```

---

## Implementation Guide

### Core PPU Emulator Structure

```javascript
class PPU {
  constructor() {
    // Memory
    this.vram = new Uint8Array(2048);     // 2KB VRAM
    this.oam = new Uint8Array(256);       // 256-byte OAM
    this.palette = new Uint8Array(32);    // 32-byte palette

    // Registers
    this.ppuctrl = 0;   // $2000
    this.ppuMask = 0;   // $2001
    this.ppuStatus = 0; // $2002
    this.oamAddr = 0;   // $2003
    this.oamData = 0;   // $2004
    this.ppuAddr = 0;   // $2006 double buffer
    this.ppuScroll = 0; // $2005 double buffer

    // Internal state
    this.currentAddr = 0; // Current VRAM address
    this.v = 0;           // Vertical position
    this.h = 0;           // Horizontal position
    this.inVBlank = false; // VBlank flag
    this.sprite0Hit = false; // Sprite 0 hit flag
    this.firstWrite = false; // Double buffer state

    // Rendering buffer
    this.frameBuffer = new Uint32Array(256 * 240);
  }

  // Step PPU for N cycles
  step(cycles) {
    for (let i = 0; i < cycles; i++) {
      this.dotCycle++;
      if (this.dotCycle >= 341) {
        this.dotCycle = 0;
        this.scanline++;
        if (this.scanline >= 262) {
          this.scanline = 0;
          this.inVBlank = false;
        }
      }
      this.renderDot();
    }
  }

  // Read from PPU registers
  read(addr) {
    switch (addr & 0x2007) {
      case 0x2002:
        return this.readStatus();
      case 0x2004:
        return this.readOAM();
      case 0x2007:
        return this.readVRAM();
      default:
        return this.dataBus; // Open bus
    }
  }

  // Write to PPU registers
  write(addr, value) {
    switch (addr & 0x2007) {
      case 0x2000:
        this.writeCtrl(value);
        break;
      case 0x2001:
        this.writeMask(value);
        break;
      case 0x2003:
        this.writeOAMAddr(value);
        break;
      case 0x2004:
        this.writeOAM(value);
        break;
      case 0x2005:
        this.writeScroll(value);
        break;
      case 0x2006:
        this.writeAddr(value);
        break;
      case 0x2007:
        this.writeVRAM(value);
        break;
    }
  }

  // OAM DMA transfer
  dmaTransfer(page) {
    // Transfer 256 bytes from CPU page to OAM
    // Takes 512 CPU cycles
  }

  // NMI request check
  nmiRequested() {
    return (this.ppuctrl & 0x80) && this.inVBlank;
  }

  // Get rendered frame
  getFrame() {
    return this.frameBuffer;
  }
}
```

### Rendering Loop

```javascript
for (let frame = 0; frame < totalFrames; frame++) {
  for (let scanline = 0; scanline < 262; scanline++) {
    for (let dot = 0; dot < 341; dot++) {
      if (dot < 256) {
        renderLeftSide();
      } else if (dot < 320) {
        horizontalBlank();
      } else if (dot < 336) {
        renderRightSide();
      } else {
        postRenderBlank();
      }
    }

    if (scanline === 241) {
      inVBlank = true;
      if (NMI enabled) assertNMI();
    }

    if (scanline === 261) {
      inVBlank = false;
      resetFrame();
    }
  }
}
```

### Memory Access During Rendering

```
VRAM access timing:
- Background fetch: Every 8 dots
- Sprite fetch: Every 8 dots (when enabled)
- VRAM reads/writes: Only during VBlank or blank periods

PPU register access:
- $2000, $2001, $2005, $2006: Any time (double buffered)
- $2002: Read-only, any time
- $2003, $2004: Any time
- $2007: During rendering, returns buffered data
- $4014: Any time (DMA)
```

### Key Implementation Notes

1. **Double Buffering**: $2005 and $2006 use double buffers - first write goes to buffer, second write applies
2. **Open Bus**: Unread PPU register values latch on the data bus
3. **VRAM Write Protection**: Writes during rendering (scanlines 0-239) may be ignored or delayed
4. **Sprite 0 Hit**: Must be checked during visible rendering only
5. **VBlank Window**: Only write to VRAM during VBlank (scanlines 241-260)
6. **PAL Timing**: Longer VBlank period requires different handling

---

## References

1. **Ricoh 2C02 Technical Documentation**
2. **https://www.tekepen.com/nes/ppu.html**
3. **NES Dev Wiki (nesdev.com)**
4. **JSNES Implementation (github.com/bfirsh/jsnes)**

---

**Document Version:** 1.0
**Last Updated:** 2026-02-14
**Based on:** Ricoh 2C02/2C03/2C05, tekepen.com, JSNES