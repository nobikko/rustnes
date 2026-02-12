//! PPU (Picture Processing Unit) implementation
//!
//! The NES PPU is responsible for rendering graphics.
//! Key specs:
//! - NTSC: 525 lines, ~60Hz (59.94Hz)
//! - PAL: 625 lines, ~50Hz (49.7Hz)
//! - NES uses 240 pixels wide, 224 lines visible
//! - Sprite height: 8 or 16 pixels (configurable)
//! - Background tile size: 8x8 pixels
//! - Palette: 54 colors (6 colors per palette, 8 palettes)

/// PPU memory map
pub const VRAM_SIZE: usize = 16384; // 16KB
pub const PALETTE_SIZE: usize = 32;  // 32 bytes (8 palettes x 4 colors each)
pub const OAM_SIZE: usize = 256;     // Object Attribute Memory

/// PPU registers
#[derive(Debug, Clone, Copy)]
pub enum PpuRegister {
    /// $2000 - PPUCTRL (Control)
    PpuCtrl,
    /// $2001 - PPUMASK (Mask)
    PpuMask,
    /// $2002 - PPUSTATUS (Status)
    PpuStatus,
    /// $2003 - OAMADDR (Sprite Address)
    OamAddr,
    /// $2004 - OAMDATA (Sprite Data)
    OamData,
    /// $2005 - PPUSCROLL (Scroll)
    PpuScroll,
    /// $2006 - PPUADDR (Address)
    PpuAddr,
    /// $2007 - PPUDATA (Data)
    PpuData,
}

/// PPU control flags
#[derive(Debug, Clone, Copy)]
pub struct PpuCtrl(u8);

impl PpuCtrl {
    pub const NMI_ENABLE: u8 = 0b10000000;
    pub const MASTER_SLAVE: u8 = 0b01000000;
    pub const SPRITE_SIZE: u8 = 0b00100000;
    pub const BG_PATTERN_TABLE: u8 = 0b00010000;
    pub const SPR_PATTERN_TABLE: u8 = 0b00001000;
    pub const VRAM_INC: u8 = 0b00000100;
    pub const NAMETABLE: u8 = 0b00000011;

    pub fn new(val: u8) -> Self {
        Self(val)
    }

    pub fn nmi_enable(&self) -> bool {
        (self.0 & Self::NMI_ENABLE) != 0
    }

    pub fn sprite_size(&self) -> bool {
        (self.0 & Self::SPRITE_SIZE) != 0
    }

    pub fn nametable(&self) -> u8 {
        self.0 & Self::NAMETABLE
    }
}

/// PPU status flags
#[derive(Debug, Clone, Copy)]
pub struct PpuStatus(u8);

impl PpuStatus {
    pub const VBLANK: u8 = 0b10000000;
    pub const SPRITE_ZERO_HIT: u8 = 0b01000000;
    pub const SPRITE_OVERFLOW: u8 = 0b00100000;

    pub fn new(val: u8) -> Self {
        Self(val)
    }

    pub fn vblank(&self) -> bool {
        (self.0 & Self::VBLANK) != 0
    }

    pub fn sprite_zero_hit(&self) -> bool {
        (self.0 & Self::SPRITE_ZERO_HIT) != 0
    }

    pub fn sprite_overflow(&self) -> bool {
        (self.0 & Self::SPRITE_OVERFLOW) != 0
    }
}

/// PPU render mask flags
#[derive(Debug, Clone, Copy)]
pub struct PpuMask(u8);

impl PpuMask {
    pub const GRAYSCALE: u8 = 0b10000000;
    pub const RENDER_BG: u8 = 0b00001000;     // Bit 3 - render background
    pub const RENDER_SPR: u8 = 0b00000100;    // Bit 2 - render sprites
    pub const RENDER_BG_LEFT: u8 = 0b00100000;  // Bit 5 - render background in left 8px
    pub const RENDER_SPR_LEFT: u8 = 0b00010000; // Bit 4 - render sprites in left 8px
    pub const EMPHASIZE_RED: u8 = 0b00000010;
    pub const EMPHASIZE_GREEN: u8 = 0b00000001;

    pub fn new(val: u8) -> Self {
        Self(val)
    }

    pub fn render_background(&self) -> bool {
        // Background rendering is enabled if bit 3 (RENDER_BG) or bit 5 (RENDER_BG_LEFT) is set
        (self.0 & (Self::RENDER_BG | Self::RENDER_BG_LEFT)) != 0
    }

    pub fn render_sprites(&self) -> bool {
        // Sprite rendering is enabled if bit 2 (RENDER_SPR) or bit 4 (RENDER_SPR_LEFT) is set
        (self.0 & (Self::RENDER_SPR | Self::RENDER_SPR_LEFT)) != 0
    }
}

/// PPU internal state
#[derive(Debug, Clone)]
pub struct Ppu {
    /// VRAM (16KB)
    vram: [u8; VRAM_SIZE],
    /// Palette memory (32 bytes)
    palette: [u8; PALETTE_SIZE],
    /// OAM (256 bytes)
    oam: [u8; OAM_SIZE],
    /// PPU registers
    control: PpuCtrl,
    mask: PpuMask,
    status: PpuStatus,
    oam_addr: u8,
    scroll: u16,
    address: u16,
    /// Fine scrollX (bits 0-2)
    fine_scroll_x: u8,
    /// Coarse X (bits 3-7)
    coarse_x: u8,
    /// Coarse Y (bits 8-12)
    coarse_y: u8,
    /// Fine Y (bits 13-15)
    fine_y: u8,
    /// Nametable select
    nametable: u8,
    /// Read buffer (for PPUDATA)
    read_buffer: u8,
    /// Sprite zero detected
    sprite_zero_detected: bool,
    /// Sprite overflow detected
    sprite_overflow_detected: bool,
    /// Dot position (0-340)
    dot: u16,
    /// Scanline position (-1 to 261, -1 is pre-render)
    scanline: i16,
    /// Frame complete flag
    frame_complete: bool,
    /// CHR ROM data for pattern tables (8KB typical)
    chr_rom: Vec<u8>,
    /// Write toggle for PPUSCROLL and PPUADDR
    write_toggle: bool,
}

impl Ppu {
    /// Create a new PPU instance
    pub fn new() -> Self {
        Self {
            vram: [0; VRAM_SIZE],
            palette: [0; PALETTE_SIZE],
            oam: [0; OAM_SIZE],
            control: PpuCtrl::new(0),
            mask: PpuMask::new(0),
            status: PpuStatus::new(0),
            oam_addr: 0,
            scroll: 0,
            address: 0,
            fine_scroll_x: 0,
            coarse_x: 0,
            coarse_y: 0,
            fine_y: 0,
            nametable: 0,
            read_buffer: 0,
            sprite_zero_detected: false,
            sprite_overflow_detected: false,
            dot: 0,
            scanline: -1,
            frame_complete: false,
            write_toggle: false,
            chr_rom: vec![0; 8192], // Default 8KB CHR ROM
        }
    }

    /// Set the CHR ROM data for pattern tables
    /// Also loads the palette data from the second 4KB bank (offset 4096)
    pub fn set_chr_rom(&mut self, chr_rom: Vec<u8>) {
        self.chr_rom = chr_rom;
        // Load palette data from offset 4096 (second 4KB bank of CHR ROM)
        // The palette is 32 bytes (8 palettes x 4 colors)
        let palette_start = 4096;
        let palette_end = palette_start + PALETTE_SIZE;
        if palette_end <= self.chr_rom.len() {
            for i in 0..PALETTE_SIZE {
                self.palette[i] = self.chr_rom[palette_start + i];
            }
        }
    }

    /// Reset the PPU
    pub fn reset(&mut self) {
        self.vram = [0; VRAM_SIZE];
        self.palette = [0; PALETTE_SIZE];
        self.oam = [0; OAM_SIZE];
        self.control = PpuCtrl::new(0);
        self.mask = PpuMask::new(0);
        self.status = PpuStatus::new(0);
        self.oam_addr = 0;
        self.scroll = 0;
        self.address = 0;
        self.fine_scroll_x = 0;
        self.coarse_x = 0;
        self.coarse_y = 0;
        self.fine_y = 0;
        self.nametable = 0;
        self.read_buffer = 0;
        self.sprite_zero_detected = false;
        self.sprite_overflow_detected = false;
        self.dot = 0;
        self.scanline = -1;
        self.frame_complete = false;
        self.write_toggle = false;
        // Keep chr_rom intact
    }

    /// Step the PPU by one cycle
    pub fn step(&mut self) {
        self.dot += 1;

        if self.dot > 340 {
            self.dot = 0;
            self.scanline += 1;

            if self.scanline > 261 {
                self.scanline = -1;
                self.frame_complete = true;
                // Clear VBLANK flag at end of frame
                self.status = PpuStatus::new(self.status.0 & !PpuStatus::VBLANK);
            }
        }

        // Handle scanline-specific behavior
        self.handle_scanline();
    }

    /// Handle behavior for specific scanlines
    fn handle_scanline(&mut self) {
        match self.scanline {
            -1 => {
                // Pre-render scanline
                if self.dot == 1 {
                    // Clear VBLANK at start of pre-render
                    self.status = PpuStatus::new(self.status.0 & !PpuStatus::VBLANK);
                }
            }
            0..=239 => {
                // Visible scanlines
            }
            241 => {
                // VBLANK starts
                self.status = PpuStatus::new(self.status.0 | PpuStatus::VBLANK);
                // Clear sprite zero and overflow flags
                self.status = PpuStatus::new(self.status.0 & !(PpuStatus::SPRITE_ZERO_HIT | PpuStatus::SPRITE_OVERFLOW));
                self.sprite_zero_detected = false;
                self.sprite_overflow_detected = false;
            }
            242..=260 => {
                // Post-render scanlines
            }
            _ => {}
        }
    }

    /// Read from PPU memory map
    pub fn read(&mut self, address: u16) -> u8 {
        match address {
            // $2000 - PPUCTRL
            0x2000 => self.control.0,
            // $2001 - PPUMASK
            0x2001 => self.mask.0,
            // $2002 - PPUSTATUS
            0x2002 => {
                let status = self.status.0;
                // Clear VBLANK flag on read (VBLANK is set at scanline 241)
                // The VBLANK flag is cleared by reading PPUSTATUS
                self.status = PpuStatus::new(status & !PpuStatus::VBLANK);
                // Reset fine scroll bits (not full scroll register)
                self.fine_scroll_x = 0;
                self.fine_y = 0;
                // Return the read buffer on first read after address increment
                status
            }
            // $2003 - OAMADDR (read only returns OAMDATA after write)
            0x2003 => self.oam_addr,
            // $2004 - OAMDATA
            0x2004 => self.oam[self.oam_addr as usize],
            // $2005 - PPUSCROLL (read only returns last written value)
            0x2005 => {
                // Returns scroll register (not really implemented fully yet)
                0
            }
            // $2006 - PPUADDR (read only returns last written value)
            0x2006 => {
                // High byte first, then low byte
                ((self.address >> 8) & 0xFF) as u8
            }
            // $2007 - PPUDATA
            0x2007 => {
                // First read after address set returns the read buffer (previous VRAM content)
                // Second read returns current VRAM content and updates read buffer
                let value = self.read_buffer;
                self.read_buffer = self.vram[self.address as usize];
                // Update address for next access
                let increment = if (self.control.0 & PpuCtrl::VRAM_INC) != 0 { 32 } else { 1 };
                self.address = self.address.wrapping_add(increment as u16);
                value
            }
            _ => 0,
        }
    }

    /// Write to PPU memory map
    pub fn write(&mut self, address: u16, value: u8) {
        match address {
            // $2000 - PPUCTRL
            0x2000 => {
                self.control = PpuCtrl::new(value);
                // Update nametable from control
                self.nametable = self.control.nametable();
            }
            // $2001 - PPUMASK
            0x2001 => {
                self.mask = PpuMask::new(value);
            }
            // $2002 - PPUSTATUS (write has no effect)
            0x2002 => {}
            // $2003 - OAMADDR
            0x2003 => {
                self.oam_addr = value;
            }
            // $2004 - OAMDATA
            0x2004 => {
                self.oam[self.oam_addr as usize] = value;
                self.oam_addr = self.oam_addr.wrapping_add(1);
            }
            // $2005 - PPUSCROLL
            0x2005 => {
                if !self.write_toggle {
                    // First write - X scroll (fine and coarse)
                    // X fine scroll is bits 0-2, X coarse scroll is bits 3-7
                    self.fine_scroll_x = value & 0x07;
                    self.coarse_x = (value >> 3) & 0x1F;
                    self.write_toggle = true;
                } else {
                    // Second write - Y scroll (fine and coarse)
                    self.fine_y = value & 0x07;
                    self.coarse_y = (value >> 3) & 0x1F;
                    self.write_toggle = false;
                }
            }
            // $2006 - PPUADDR
            0x2006 => {
                if !self.write_toggle {
                    // First write - high byte (bits 15-8 of address)
                    // The high 2 bits (15-14) select the nametable
                    // Bits 13-0 are the tile offset within the nametable
                    self.address = (self.address & 0x00FF) | ((value as u16) & 0x3F) << 8;
                    self.write_toggle = true;
                } else {
                    // Second write - low byte (bits 7-0 of address)
                    self.address = (self.address & 0xFF00) | (value as u16);
                    self.write_toggle = false;
                }
            }
            // $2007 - PPUDATA
            0x2007 => {
                self.vram[self.address as usize] = value;
                // Write also updates the read buffer with the value being written
                self.read_buffer = value;
                // Update address for next access
                let increment = if (self.control.0 & PpuCtrl::VRAM_INC) != 0 { 32 } else { 1 };
                self.address = self.address.wrapping_add(increment as u16);
            }
            _ => {}
        }
    }

    /// Get PPU status
    pub fn status(&self) -> &PpuStatus {
        &self.status
    }

    /// Get PPU mask
    pub fn mask(&self) -> &PpuMask {
        &self.mask
    }

    /// Get PPU control
    pub fn control(&self) -> &PpuCtrl {
        &self.control
    }

    /// Get raw PPU mask value (for debugging)
    pub fn mask_value(&self) -> u8 {
        self.mask.0
    }

    /// Get raw PPU status value (for debugging)
    pub fn status_value(&self) -> u8 {
        self.status.0
    }

    /// Get raw PPU control value (for debugging)
    pub fn control_value(&self) -> u8 {
        self.control.0
    }

    /// Check if VBLANK is active
    pub fn in_vblank(&self) -> bool {
        self.status.vblank()
    }

    /// Get current scanline
    pub fn scanline(&self) -> i16 {
        self.scanline
    }

    /// Get current dot
    pub fn dot(&self) -> u16 {
        self.dot
    }

    /// Get palette entry (4 bytes per palette: 4 colors)
    /// Returns the palette index for background/sprites
    /// Each byte contains two 4-bit color indices
    pub fn get_palette(&self, palette_idx: usize) -> [u8; 4] {
        if palette_idx >= 8 {
            return [0; 4];
        }
        let base = palette_idx * 4;
        let c0 = self.palette[base] & 0x0F;
        let c1 = (self.palette[base] >> 4) & 0x0F;
        let c2 = self.palette[base + 1] & 0x0F;
        let c3 = (self.palette[base + 1] >> 4) & 0x0F;
        [c0 as u8, c1 as u8, c2 as u8, c3 as u8]
    }

    /// Get the palette byte at the given index (for direct access)
    pub fn get_palette_byte(&self, byte_idx: usize) -> u8 {
        if byte_idx < PALETTE_SIZE {
            self.palette[byte_idx]
        } else {
            0
        }
    }

    /// Render a scanline to a framebuffer
    /// framebuffer should be sized for at least `width` * 3 bytes per pixel (RGB)
    /// Returns the number of bytes written to the framebuffer
    pub fn render_scanline(&self, scanline: usize, framebuffer: &mut [u8], width: usize) {
        if scanline >= 240 || framebuffer.len() < width * 3 {
            return;
        }

        // Get the render mask
        let render_bg = self.mask.render_background();
        let render_sprites = self.mask.render_sprites();

        // NES color palette (6-bit values converted to 8-bit RGB)
        // These are the standard NES palettes (64 colors)
        let palette_table: [(u8, u8, u8); 64] = [
            (84, 84, 84), (0, 30, 116), (8, 22, 147), (48, 12, 154), (92, 4, 121), (136, 6, 85), (147, 22, 34), (132, 48, 0), (76, 84, 0), (12, 102, 0), (0, 120, 44), (0, 106, 132), (0, 84, 136), (0, 0, 0), (0, 0, 0), (0, 0, 0),
            (160, 160, 160), (0, 70, 196), (48, 92, 255), (92, 70, 255), (136, 58, 255), (196, 78, 255), (204, 92, 204), (255, 114, 136), (255, 147, 84), (255, 173, 0), (216, 196, 0), (120, 214, 0), (0, 230, 116), (0, 196, 214), (0, 160, 255), (0, 0, 0),
            (255, 255, 255), (48, 152, 255), (120, 147, 255), (176, 138, 255), (220, 132, 255), (255, 152, 255), (255, 165, 214), (255, 188, 160), (255, 214, 136), (255, 234, 120), (255, 255, 160), (188, 255, 160), (120, 255, 188), (120, 255, 255), (120, 214, 255), (84, 84, 255),
            (255, 255, 255), (166, 230, 255), (188, 220, 255), (204, 214, 255), (214, 204, 255), (220, 204, 255), (214, 208, 230), (220, 214, 204), (234, 220, 196), (255, 230, 188), (240, 234, 196), (214, 240, 196), (188, 244, 214), (188, 244, 230), (188, 230, 244), (176, 176, 255),
        ];

        // Calculate pattern table bases
        let bg_pattern_table_base = if (self.control.0 & PpuCtrl::BG_PATTERN_TABLE) != 0 { 4096 } else { 0 };
        let sprite_pattern_table_base = if (self.control.0 & PpuCtrl::SPR_PATTERN_TABLE) != 0 { 4096 } else { 0 };

        // Helper function to get a pixel from a tile
        // Reads 2 bytes from pattern table and extracts the pixel color index
        let get_tile_pixel = |tile_idx: u8, pixel_x: u8, pixel_y: u8, pattern_base: usize, chr_rom: &[u8]| -> u8 {
            // Pattern table entry: each tile is 16 bytes (8x8 pixels, 2 bits per pixel)
            let tile_base = (tile_idx as usize) * 16;

            // Get the two planes (bit 0 and bit 1 for each pixel)
            let plane0_addr = pattern_base + tile_base + (pixel_y as usize);
            let plane1_addr = pattern_base + tile_base + 8 + (pixel_y as usize);

            if plane0_addr >= chr_rom.len() {
                return 0;
            }

            let plane0 = chr_rom[plane0_addr];
            let plane1 = if plane1_addr < chr_rom.len() { chr_rom[plane1_addr] } else { 0 };

            // Extract the pixel bit from each plane (bit 7 is leftmost)
            let bit = 7 - (pixel_x as usize);
            let bit0 = (plane0 >> bit) & 1;
            let bit1 = (plane1 >> bit) & 1;

            // Combine to get color index (2 bits = 0-3)
            (bit1 << 1) | bit0
        };

        // Helper to get palette index from attribute table
        // Attribute table is at $23C0-$2FFF (256 bytes, covers 32x30 tile area)
        // Each byte controls a 4x4 tile block
        // Bits 0-1: upper-left quadrant, Bits 2-3: upper-right, Bits 4-5: lower-left, Bits 6-7: lower-right
        let get_attr_palette = |attr: u8, tile_x: u8, tile_y: u8| -> u8 {
            // Determine which quadrant this tile is in within its 4x4 block
            let x_in_block = tile_x % 4;
            let y_in_block = tile_y % 4;

            let shift = if x_in_block < 2 {
                if y_in_block < 2 { 0 } else { 4 }
            } else {
                if y_in_block < 2 { 2 } else { 6 }
            };

            ((attr >> shift) & 0x03) as u8
        };

        // Get the background nametable base address
        // Nametables are at $2000-$23FF in VRAM
        // With mirroring: nametable 0 = $2000-$23FF, nametable 1 = $2400-$27FF, etc.
        let nametable_base = (0x2000 + (self.nametable as usize) * 1024) as usize;

        // Attribute table is at $23C0-$2FFF (32 bytes per nametable)
        let attr_table_base = nametable_base + 960; // $23C0 - $2000 = 0x3C0 = 960

        // Render background
        for x in 0..width.min(256) {
            // Calculate scroll position
            let fine_x = self.fine_scroll_x as i32;
            let coarse_x = self.coarse_x as i32;
            let coarse_y = self.coarse_y as i32;

            // Calculate the tile X position accounting for fine scroll
            // The fine scroll tells us how many pixels into the tile to start
            let tile_x = if x as i32 >= fine_x {
                coarse_x + ((x as i32 - fine_x) / 8)
            } else {
                // Wrapping to the other side of the screen
                coarse_x + 32 - ((fine_x - x as i32) / 8)
            } % 32;

            // Calculate pixel position within the tile (0-7)
            let pixel_x = if x as i32 >= fine_x {
                (x as i32 - fine_x) % 8
            } else {
                8 - (fine_x - x as i32) % 8
            } as u8;

            // Calculate tile Y position
            // For Y, we need to account for the current scanline relative to the coarse Y
            let scanline_in_tile = scanline as i32 % 8;
            let tile_y = (coarse_y + (scanline as i32 / 8)) % 32;

            let color_idx = if render_bg {
                // Calculate nametable address for this tile
                let nametable_addr = nametable_base + (tile_y as usize) * 32 + (tile_x as usize);

                if nametable_addr >= self.vram.len() {
                    0
                } else {
                    let tile_idx = self.vram[nametable_addr];

                    // Get the attribute table byte for this tile
                    // Attribute table is organized in 4x4 tile blocks
                    let attr_tile_x = tile_x as u8 / 4;
                    let attr_tile_y = tile_y as u8 / 4;
                    let attr_addr = attr_table_base + (attr_tile_y as usize) * 8 + (attr_tile_x as usize);

                    let palette_select = if attr_addr < self.vram.len() {
                        let attr = self.vram[attr_addr];
                        get_attr_palette(attr, tile_x as u8, tile_y as u8)
                    } else {
                        0
                    };

                    // Get the actual color from the tile using the pattern table
                    let pixel_y = scanline_in_tile as u8;

                    // Get color index from pattern table (0-3)
                    let color = get_tile_pixel(tile_idx, pixel_x, pixel_y, bg_pattern_table_base, &self.chr_rom);

                    // Use palette to get final color index (0-63)
                    if color > 0 {
                        palette_select * 4 + color
                    } else {
                        0 // Background color (palette index 0 of selected palette)
                    }
                }
            } else if render_sprites {
                // Simple sprite rendering - check OAM for sprites on this scanline
                let mut sprite_color: u8 = 0;
                let mut sprite_found = false;

                for sprite_idx in 0..64 {
                    let oam_base = (sprite_idx as usize) * 4;

                    if oam_base + 3 >= self.oam.len() {
                        break;
                    }

                    let sprite_y = self.oam[oam_base] as i32;
                    let tile_idx = self.oam[oam_base + 1];
                    let flags = self.oam[oam_base + 2];
                    let sprite_x = self.oam[oam_base + 3] as i32;

                    // Check if sprite is on this scanline
                    let sprite_height = if self.control.sprite_size() { 16 } else { 8 };

                    // Sprite is visible if: sprite_y <= scanline < sprite_y + sprite_height
                    // Sprite Y position is offset by 1 (sprite at Y=0 is drawn at scanline 1)
                    if (scanline as i32) >= (sprite_y + 1) && (scanline as i32) < (sprite_y + 1 + sprite_height as i32) {
                        let pixel_y = (scanline as i32 - (sprite_y + 1)) as u8;

                        if self.control.sprite_size() {
                            // 16x16 sprite - two tiles stacked vertically
                            let tile_row = if pixel_y >= 8 { 1 } else { 0 };
                            let actual_y = if (flags & 0x80) != 0 { 7 - pixel_y % 8 } else { pixel_y % 8 };

                            let actual_tile = if (flags & 0x40) != 0 {
                                // Flipped vertically
                                tile_idx as u16 + (1 - tile_row) * 2
                            } else {
                                tile_idx as u16 + tile_row as u16
                            };

                            let pixel_x = x as i32 - sprite_x;
                            if pixel_x >= 0 && pixel_x < 8 {
                                let color = get_tile_pixel(actual_tile as u8, pixel_x as u8, actual_y as u8, sprite_pattern_table_base, &self.chr_rom);
                                if color > 0 {
                                    // Sprite palette is in bits 4-5 of flags
                                    let sprite_palette = ((flags >> 4) as usize & 3) * 4;
                                    sprite_color = ((sprite_palette + color as usize) as u8).min(63);
                                    sprite_found = true;
                                    break;
                                }
                            }
                        } else {
                            // 8x8 sprite
                            let pixel_x = x as i32 - sprite_x;

                            if pixel_x >= 0 && pixel_x < 8 {
                                let actual_y = if (flags & 0x80) != 0 { 7 - pixel_y } else { pixel_y };
                                let actual_tile = if (flags & 0x40) != 0 { tile_idx + 1 } else { tile_idx };

                                let color = get_tile_pixel(actual_tile, pixel_x as u8, actual_y as u8, sprite_pattern_table_base, &self.chr_rom);

                                if color > 0 {
                                    let sprite_palette = ((flags >> 4) as usize & 3) * 4;
                                    sprite_color = (sprite_palette as u8 + color).min(63);
                                    sprite_found = true;
                                    break;
                                }
                            }
                        }
                    }
                }
                if sprite_found {
                    sprite_color
                } else {
                    0  // Black when nothing rendered
                }
            } else {
                0  // Black when nothing rendered
            };

            let rgb = palette_table[color_idx as usize % palette_table.len()];
            let idx = x * 3;
            framebuffer[idx] = rgb.0;
            framebuffer[idx + 1] = rgb.1;
            framebuffer[idx + 2] = rgb.2;
        }
    }
}

impl Default for Ppu {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ppu_reset() {
        let mut ppu = Ppu::new();
        ppu.reset();
        assert_eq!(ppu.scanline, -1);
        assert_eq!(ppu.dot, 0);
    }

    #[test]
    fn test_ppu_status_read() {
        let mut ppu = Ppu::new();
        // Set VBLANK flag
        ppu.status = PpuStatus::new(PpuStatus::VBLANK);

        let _ = ppu.read(0x2002); // Read PPUSTATUS
        // VBLANK should be cleared on read
        assert!(!ppu.status.vblank());
    }

    #[test]
    fn test_ppu_set_chr_rom_loads_palette() {
        let mut ppu = Ppu::new();

        // Create a CHR ROM with known palette data
        let mut chr_rom = vec![0; 8192];
        // Set first palette (background) to specific values
        chr_rom[4096] = 0x12; // Palette byte 0
        chr_rom[4097] = 0x34; // Palette byte 1
        chr_rom[4098] = 0x56; // Palette byte 2
        chr_rom[4099] = 0x78; // Palette byte 3

        ppu.set_chr_rom(chr_rom);

        // Verify palette was loaded
        assert_eq!(ppu.get_palette_byte(0), 0x12);
        assert_eq!(ppu.get_palette_byte(1), 0x34);
        assert_eq!(ppu.get_palette_byte(2), 0x56);
        assert_eq!(ppu.get_palette_byte(3), 0x78);
    }

    #[test]
    fn test_ppu_get_palette() {
        let mut ppu = Ppu::new();

        // Set up palette with known values
        ppu.palette[0] = 0x01; // Colors: 1, 0
        ppu.palette[1] = 0x23; // Colors: 3, 2
        ppu.palette[2] = 0x45; // Colors: 5, 4
        ppu.palette[3] = 0x67; // Colors: 7, 6

        let palette = ppu.get_palette(0);
        assert_eq!(palette, [1, 0, 3, 2]);
    }
}