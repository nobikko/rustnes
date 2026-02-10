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
    pub const GRayscale: u8 = 0b10000000;
    pub const RENDER_BG_LEFT: u8 = 0b00100000;
    pub const RENDER_SPR_LEFT: u8 = 0b00010000;
    pub const HIGHLIGHT_BG: u8 = 0b00001000;
    pub const HIGHLIGHT_SPR: u8 = 0b00000100;
    pub const EMPHASIZE_RED: u8 = 0b00000010;
    pub const EMPHASIZE_GREEN: u8 = 0b00000001;

    pub fn new(val: u8) -> Self {
        Self(val)
    }

    pub fn render_background(&self) -> bool {
        (self.0 & Self::RENDER_BG_LEFT) != 0
    }

    pub fn render_sprites(&self) -> bool {
        (self.0 & Self::RENDER_SPR_LEFT) != 0
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
                // Clear VBLANK flag on read
                self.status = PpuStatus::new(status & !PpuStatus::VBLANK);
                // Reset fine scroll
                self.scroll = 0;
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
                let value = self.vram[self.address as usize];
                // Update address for next read
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
                if self.dot < 256 {
                    // First write - X scroll (fine and coarse)
                    self.fine_scroll_x = value & 0x07;
                    self.coarse_x = (value >> 3) & 0x1F;
                } else {
                    // Second write - Y scroll
                    self.fine_y = value & 0x07;
                    self.coarse_y = (value >> 3) & 0x1F;
                }
            }
            // $2006 - PPUADDR
            0x2006 => {
                if self.dot < 256 {
                    // First write - high byte
                    self.address = (self.address & 0x00FF) | ((value as u16) << 8);
                } else {
                    // Second write - low byte
                    self.address = (self.address & 0xFF00) | (value as u16);
                }
            }
            // $2007 - PPUDATA
            0x2007 => {
                self.vram[self.address as usize] = value;
                // Update address for next write
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
}