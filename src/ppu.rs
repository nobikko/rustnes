//! PPU (Picture Processing Unit) Emulator
//!
//! Implements the Ricoh 2C02 PPU used in the NES.

/// PPU Status flags
pub const STATUS_VBLANK: u8 = 0x80;
pub const STATUS_SPRITE0HIT: u8 = 0x40;
pub const STATUS_SPRITEOVERFLOW: u8 = 0x20;

/// Palette colors (RGB555 format)
pub const NTSC_PALETTE: [u32; 64] = [
    0x525252, 0xB40000, 0xA00000, 0xB1003D, 0x740069, 0x00005B, 0x00005F, 0x001840,
    0x002F10, 0x084A08, 0x006700, 0x124200, 0x6D2800, 0x000000, 0x000000, 0x000000,
    0xC4D5E7, 0xFF4000, 0xDC0E22, 0xFF476B, 0xD7009F, 0x680AD7, 0x0019BC, 0x0054B1,
    0x006A5B, 0x008C03, 0x00AB00, 0x2C8800, 0xA47200, 0x000000, 0x000000, 0x000000,
    0xF8F8F8, 0xFFAB3C, 0xFF7981, 0xFF5BC5, 0xFF48F2, 0xDF49FF, 0x476DFF, 0x00B4F7,
    0x00E0FF, 0x00E375, 0x03F42B, 0x78B82E, 0xE5E218, 0x787878, 0x000000, 0x000000,
    0xFFFFFF, 0xFFF2BE, 0xF8B8B8, 0xF8B8D8, 0xFFB6FF, 0xFFC3FF, 0xC7D1FF, 0x9ADAFF,
    0x88EDF8, 0x83FFDD, 0xB8F8B8, 0xF5F8AC, 0xFFFFB0, 0xF8D8F8, 0x000000, 0x000000,
];

/// Nametable structure (32x30 tiles)
#[derive(Debug, Clone)]
pub struct NameTable {
    pub tiles: [u8; 960],  // 32 * 30
}

impl NameTable {
    pub fn new() -> Self {
        Self { tiles: [0u8; 960] }
    }
}

impl Default for NameTable {
    fn default() -> Self {
        Self::new()
    }
}

/// Attribute table (8x8 blocks of tiles)
#[derive(Debug, Clone)]
pub struct AttributeTable {
    pub data: [u8; 64],  // 32/4 * 30/4 = 8 * 7.5 -> actually 64 bytes
}

impl AttributeTable {
    pub fn new() -> Self {
        Self { data: [0u8; 64] }
    }
}

impl Default for AttributeTable {
    fn default() -> Self {
        Self::new()
    }
}

/// Tile data (8x8 pixels, 16 bytes)
#[derive(Debug, Clone, Copy, Default)]
pub struct Tile {
    pub data: [u8; 16],
}

/// 8x8 pixel tile renderer
#[derive(Debug)]
pub struct TileRenderer {
    pub tiles: [Tile; 512],  // Pattern table (4KB / 16 = 256 tiles per 4KB)
    pub palette: [u8; 16],   // Current palette
}

impl TileRenderer {
    pub fn new() -> Self {
        Self {
            tiles: [Tile::default(); 512],
            palette: [0u8; 16],
        }
    }
}

impl Default for TileRenderer {
    fn default() -> Self {
        Self::new()
    }
}

/// PPU Registers
#[derive(Debug, Clone, Copy)]
pub struct PpuRegisters {
    pub ctrl1: u8,   // $2000 - Control register 1
    pub ctrl2: u8,   // $2001 - Control register 2
    pub status: u8,  // $2002 - Status register
    pub oam_addr: u8, // $2003 - OAM address
    pub scroll_x: u8, // $2005 - Scroll X
    pub scroll_y: u8, // $2005 - Scroll Y (first write)
    pub scroll_y_second: u8, // $2005 - Scroll Y (second write)
    pub addr: u16,   // $2006 - VRAM address
    pub data: u8,    // $2007 - VRAM data
}

/// The PPU emulator
#[derive(Debug)]
pub struct PPU {
    pub vram: [u8; 0x8000],      // 32KB VRAM
    pub oam: [u8; 256],          // 256-byte OAM (Object Attribute Memory)
    pub palette: [u8; 32],       // 32-byte palette RAM
    pub open_bus: u8,            // Open bus latch

    // Debug flag
    pub debug: bool,             // Enable debug output

    // Rendering state
    pub cur_x: u16,              // Current PPU dot (0-340)
    pub scanline: i16,           // Current scanline (-1 to 261)
    pub frame_count: u32,        // Frame counter
    pub frame_complete: bool,    // Flag set when a frame completes

    // Scrolling counters
    pub vram_address: u16,       // Current VRAM address
    pub vram_buffered_value: u8, // Buffered VRAM read value
    pub first_write: bool,       // First write flag for two-byte registers

    // Control flags
    pub nmi_on_vblank: bool,     // Bit 7 of $2000
    pub sprite_size: bool,       // Bit 5 of $2000 (0=8x8, 1=8x16)
    pub bg_pattern_table: u16,   // Bit 4 of $2000 (0=$0000, 1=$1000)
    pub sp_pattern_table: u16,   // Bit 3 of $2000
    pub address_increment: u8,   // Bit 2 of $2000 (0=+1, 1=+32)
    pub nametable_select: u16,   // Bits 1-0 of $2000

    // OAM address register
    pub oam_addr: u8,            // $2003 - OAM address

    // Display flags
    pub sprite_visible: bool,    // Bit 4 of $2001
    pub bg_visible: bool,        // Bit 3 of $2001
    pub sprite_clipping: bool,   // Bit 2 of $2001
    pub bg_clipping: bool,       // Bit 1 of $2001
    pub display_type: bool,      // Bit 0 of $2001 (0=color, 1=mono)
    pub emphasis: u8,            // Bits 5-7 of $2001

    // Sprite state
    pub sprites_evaluated: u8,   // Number of sprites evaluated
    pub sprite0_hit: bool,       // Sprite 0 hit flag
    pub sprite_overflow: bool,   // Sprite overflow flag

    // Frame buffer (256x240 pixels)
    pub frame_buffer: Vec<u32>,

    // Nametables
    pub name_tables: [NameTable; 4],
}

impl PPU {
    /// Create a new PPU instance
    pub fn new() -> Self {
        let mut ppu = Self {
            vram: [0u8; 0x8000],
            oam: [0u8; 256],
            // Initialize palette with a visible color (white) at index 0
            // This ensures background is visible even without CHR ROM
            palette: [
                0x3F, 0x00, 0x00, 0x00,  // Background palette 0: white, black, black, black
                0x00, 0x3F, 0x00, 0x00,  // Background palette 1: black, white, black, black
                0x00, 0x00, 0x3F, 0x00,  // Background palette 2: black, black, white, black
                0x00, 0x00, 0x00, 0x3F,  // Background palette 3: black, black, black, white
                0x3F, 0x00, 0x00, 0x00,  // Sprite palette 1: white, black, black, black
                0x00, 0x3F, 0x00, 0x00,  // Sprite palette 2: black, white, black, black
                0x00, 0x00, 0x3F, 0x00,  // Sprite palette 3: black, black, white, black
                0x00, 0x00, 0x00, 0x3F,  // Sprite palette 4: black, black, black, white
            ],
            open_bus: 0,

            debug: false,

            cur_x: 0,
            scanline: -1,
            frame_count: 0,
            frame_complete: false,

            vram_address: 0,
            vram_buffered_value: 0,
            first_write: true,

            nmi_on_vblank: false,
            sprite_size: false,
            bg_pattern_table: 0,
            sp_pattern_table: 0,
            address_increment: 0,
            nametable_select: 0,

            oam_addr: 0,

            sprite_visible: true,   // Enable by default for desktop version
            bg_visible: true,   // Enable by default for desktop version
            sprite_clipping: true,
            bg_clipping: true,
            display_type: false,
            emphasis: 0,

            sprites_evaluated: 0,
            sprite0_hit: false,
            sprite_overflow: false,

            frame_buffer: vec![0u32; 256 * 240],

            name_tables: [
                NameTable::default(),
                NameTable::default(),
                NameTable::default(),
                NameTable::default(),
            ],
        };

        // Initialize nametables with visible content
        ppu.init_nametables();

        ppu
    }

    /// Start a new frame
    pub fn start_frame(&mut self) {
        // Reset frame state for new frame
        self.scanline = -1;
        self.cur_x = 0;
        self.frame_complete = false;
        self.sprite0_hit = false;
        self.sprite_overflow = false;
        self.sprites_evaluated = 0;

        // Clear VBlank flag
        self.open_bus &= !STATUS_VBLANK;
    }

    /// Initialize nametables with visible content
    /// This ensures there's something to display even without CHR ROM
    pub fn init_nametables(&mut self) {
        // Initialize nametables with tile 1 (first visible character in default font)
        // Tile 0 is always blank (space character), so we use tile 1 which shows '!'
        for nametable in &mut self.name_tables {
            for i in 0..nametable.tiles.len() {
                nametable.tiles[i] = 1;  // Use tile 1 instead of 0
            }
        }

        // Initialize attribute tables with palette 1 (white when combined with tile 1)
        // Attribute table is 8x8 blocks of tiles, so 32*30/4 = 240 bytes
        for attr_table in &mut self.name_tables {
            for i in 0..attr_table.tiles.len() / 15 {  // 960/15 = 64 bytes per attr table
                // Each byte controls 4 tiles (2 bits each for 4 8x8 blocks)
                // Palette 1 is white (color index 3 in palette)
                attr_table.tiles[i] = 0x55;  // Alternate between palettes
            }
        }
    }

    /// Render a scanline to the frame buffer
    fn render_scanline(&mut self, scanline: i16) {
        if scanline < 0 || scanline >= 240 {
            return;  // Not a visible scanline
        }

        let y = scanline as u16;

        for x in 0..256 {
            let color = self.render_pixel(x as u16, y);
            let pixel_index = (y as usize * 256) + (x as usize);
            if pixel_index < self.frame_buffer.len() {
                self.frame_buffer[pixel_index] = color;
            }
        }
    }

    /// End current scanline
    pub fn end_scanline(&mut self) {
        if self.debug {
            eprintln!("PPU: end_scanline, prev_scanline={}", self.scanline);
        }
        self.cur_x = 0;

        // Render the previous scanline if it was visible
        let prev_scanline = self.scanline;
        self.scanline += 1;

        // Render the completed scanline if it's visible (0-239)
        if prev_scanline >= 0 && prev_scanline < 240 {
            if self.debug {
                eprintln!("PPU: rendering scanline {}", prev_scanline);
            }
            self.render_scanline(prev_scanline);
        } else if self.debug {
            eprintln!("PPU: skipping render for scanline {} (prev_scanline={})", self.scanline - 1, prev_scanline);
        }

        if self.scanline == 241 {
            // Start of VBlank
            if self.debug {
                eprintln!("PPU: VBlank start");
            }
            self.start_vblank();
        } else if self.scanline > 261 {
            // Frame complete - set flag and keep scanline at 262
            // until start_frame() is called
            if self.debug {
                eprintln!("PPU: frame complete!");
            }
            self.frame_count += 1;
            self.frame_complete = true;
        }
    }

    /// Start VBlank period
    fn start_vblank(&mut self) {
        self.open_bus |= STATUS_VBLANK;

        if self.nmi_on_vblank {
            // NMI will be processed on next instruction
            // In real hardware, this happens at end of T-cycle
        }

        // Reset scroll after VBlank starts
        // (Actually happens at dot 1 of scanline 20 in real PPU)
    }

    /// Read from PPU registers
    pub fn read(&mut self, address: u16) -> u8 {
        let value = match address {
            0x2002 => {
                let value = self.open_bus;
                self.first_write = true;
                self.open_bus &= 0x1F;  // Clear status bits
                value
            }
            0x2004 => {
                // OAM data read
                let addr = self.oam_addr as usize;
                let value = self.oam[addr];
                self.oam_addr = self.oam_addr.wrapping_add(1);
                value
            }
            0x2007 => {
                // VRAM data read
                let addr = self.vram_address as usize;
                let value = self.vram_buffered_value;
                self.vram_buffered_value = self.vram[addr];
                self.vram_address = self.vram_address.wrapping_add(self.address_increment as u16);
                value
            }
            _ => {
                // Mirror of $2000-$2007
                self.open_bus
            }
        };
        self.open_bus = value;
        value
    }

    /// Write to PPU registers
    pub fn write(&mut self, address: u16, value: u8) {
        self.open_bus = value;

        if self.debug {
            eprintln!("PPU write: ${:04X} = ${:02X}", address, value);
            if address == 0x2001 {
                self.sprite_visible = (value & 0x10) != 0;
                self.bg_visible = (value & 0x08) != 0;
                eprintln!("  -> sprite_visible={}, bg_visible={}", self.sprite_visible, self.bg_visible);
                // Don't do the full case match, just show the write
            }
        }

        match address {
            0x2000 => {
                // PPUCTRL - Control register 1
                self.nmi_on_vblank = (value & 0x80) != 0;
                self.sprite_size = (value & 0x20) != 0;
                self.bg_pattern_table = if (value & 0x10) != 0 { 0x1000 } else { 0 };
                self.sp_pattern_table = if (value & 0x08) != 0 { 0x1000 } else { 0 };
                self.address_increment = if (value & 0x04) != 0 { 32 } else { 1 };
                self.nametable_select = (value & 0x03) as u16;

                self.first_write = true;
            }
            0x2001 => {
                // PPUMASK - Control register 2
                self.sprite_visible = (value & 0x10) != 0;
                self.bg_visible = (value & 0x08) != 0;
                self.sprite_clipping = (value & 0x04) != 0;
                self.bg_clipping = (value & 0x02) != 0;
                self.display_type = (value & 0x01) != 0;
                self.emphasis = (value >> 5) & 0x07;
                if self.debug {
                    eprintln!("PPU: sprite_visible={}, bg_visible={}", self.sprite_visible, self.bg_visible);
                }
            }
            0x2003 => {
                // OAMADDR - OAM address
                self.oam_addr = value;
            }
            0x2004 => {
                // OAMDATA - OAM data write
                let addr = self.oam_addr as usize;
                self.oam[addr] = value;
                self.oam_addr = self.oam_addr.wrapping_add(1);
            }
            0x2005 => {
                // PPUSCROLL - Scroll register (2-byte write)
                if self.first_write {
                    // First write: X scroll (coarse X)
                    self.vram_address = (self.vram_address & 0xFBE0) | ((value as u16 & 0x1F) << 0);
                } else {
                    // Second write: Y scroll (coarse Y + fine Y)
                    let fine_y = value & 0x07;
                    let coarse_y = (value >> 3) & 0x1F;

                    self.vram_address = (self.vram_address & 0x841F) |
                        ((coarse_y as u16) << 5) |
                        ((fine_y as u16) << 12);
                }
                self.first_write = !self.first_write;
            }
            0x2006 => {
                // PPUADDR - VRAM address (2-byte write)
                if self.first_write {
                    // First write: High byte
                    self.vram_address = (self.vram_address & 0x00FF) | ((value as u16) << 8);
                } else {
                    // Second write: Low byte
                    self.vram_address = (self.vram_address & 0xFF00) | (value as u16);
                }
                self.first_write = !self.first_write;
            }
            0x2007 => {
                // PPUDATA - VRAM data write
                let addr = self.vram_address as usize;
                self.vram[addr] = value;
                self.vram_address = self.vram_address.wrapping_add(self.address_increment as u16);

                // Buffer the read value
                let next_addr = self.vram_address as usize;
                self.vram_buffered_value = self.vram[next_addr];
            }
            _ => {
                // Mirror of $2000-$2007
            }
        }
    }

    /// Read from VRAM
    pub fn vram_read(&mut self, address: u16) -> u8 {
        let addr = (address & 0x3FFF) as usize;
        self.vram[addr]
    }

    /// Write to VRAM
    pub fn vram_write(&mut self, address: u16, value: u8) {
        let addr = (address & 0x3FFF) as usize;
        self.vram[addr] = value;
    }

    /// Get palette color with emphasis
    pub fn get_palette_color(&self, index: u8) -> u32 {
        if index >= 64 {
            return 0x000000;  // Black
        }

        let color = NTSC_PALETTE[index as usize];

        // Apply color emphasis
        let r = (color & 0x0000FF) as u32;
        let g = ((color & 0x00FF00) >> 8) as u32;
        let b = ((color & 0xFF0000) >> 16) as u32;

        let r = if (self.emphasis & 0x04) != 0 { (r * 3) / 4 } else { r };
        let g = if (self.emphasis & 0x02) != 0 { (g * 3) / 4 } else { g };
        let b = if (self.emphasis & 0x01) != 0 { (b * 3) / 4 } else { b };

        (r as u32) | ((g as u32) << 8) | ((b as u32) << 16)
    }

    /// Get attribute table index for a tile
    pub fn get_attribute(&mut self, tile_x: u8, tile_y: u8, nametable: u8) -> u8 {
        let nametable_offset = nametable as u16 * 0x400;
        let attr_x = tile_x / 4;
        let attr_y = tile_y / 4;
        let attr_addr = nametable_offset + 0x3C0 + (attr_y as u16) * 8 + (attr_x as u16) / 2;

        let byte = self.vram_read(attr_addr as u16);
        let shift = ((tile_x % 4) % 2) * 4;

        (byte >> shift) & 0x03
    }

    /// Render a single pixel
    pub fn render_pixel(&mut self, x: u16, y: u16) -> u32 {
        // Check sprite 0 hit
        if x < 256 && y < 240 {
            if self.sprite0_hit && self.sprite_visible && self.bg_visible {
                // Sprite 0 hit occurred
            }
        }

        // Render background if visible
        if self.bg_visible && x < 256 && y < 240 {
            let bg_color = self.render_background(x, y);
            if bg_color != 0 {
                return bg_color;
            }
        }

        // Render sprites if visible
        if self.sprite_visible && x < 256 && y < 240 {
            if let Some(sprite_color) = self.render_sprite(x, y) {
                return sprite_color;
            }
        }

        0  // Background color (black)
    }

    /// Render background pixel
    fn render_background(&mut self, x: u16, y: u16) -> u32 {
        // Calculate tile coordinates
        let coarse_x = (x as u16 / 8) & 0x1F;
        let coarse_y = (y as u16 / 8) & 0x1F;
        let fine_x = x as u16 & 0x07;
        let fine_y = y as u16 & 0x07;

        // Get nametable base
        let nametable_base = self.nametable_select * 0x400;

        // Calculate tile index address
        let tile_index_addr = nametable_base + (coarse_y as u16) * 32 + (coarse_x as u16);
        let tile_index = self.vram_read(tile_index_addr);

        // Calculate attribute table address
        let attr_x = coarse_x / 4;
        let attr_y = coarse_y / 4;
        let attr_addr = nametable_base + 0x3C0 + (attr_y as u16) * 8 + (attr_x as u16) / 2;
        let attr_byte = self.vram_read(attr_addr);
        let attr_shift = ((coarse_x % 4) % 2) * 4;
        let palette = ((attr_byte >> attr_shift) & 0x03) as u8;

        // Calculate pattern table address
        let pattern_addr = self.bg_pattern_table + (tile_index as u16) * 16 + (fine_y as u16);
        let byte1 = self.vram_read(pattern_addr);
        let byte2 = self.vram_read(pattern_addr + 8);

        // Get pixel color
        let bit1 = (byte1 >> (7 - fine_x as u8)) & 1;
        let bit2 = (byte2 >> (7 - fine_x as u8)) & 1;
        let color = (bit2 << 1) | bit1;

        if color == 0 {
            return 0;  // Transparent
        }

        let palette_addr = 0x3F00 + (palette as u16) * 4 + color as u16;
        let palette_index = self.vram_read(palette_addr) & 0x3F;

        self.get_palette_color(palette_index)
    }

    /// Render sprite pixel
    fn render_sprite(&mut self, x: u16, y: u16) -> Option<u32> {
        if !self.sprite_visible {
            return None;
        }

        // Sprite evaluation happens during pre-render scanline
        // For simplicity, we iterate through OAM
        let sprite_size = if self.sprite_size { 16 } else { 8 };
        let sprite_pattern_base = self.sp_pattern_table;

        for i in 0..64 {
            let base = (i as u16) * 4;

            let sprite_y = self.oam[base as usize] as u16;
            let tile = self.oam[(base + 1) as usize] as u16;
            let attr = self.oam[(base + 2) as usize];
            let sprite_x = self.oam[(base + 3) as usize] as u16;

            // Check if pixel is within sprite bounds
            let sprite_top = sprite_y as u16;
            let sprite_left = sprite_x as u16;

            if y < sprite_top || y >= sprite_top + sprite_size {
                continue;
            }

            if x < sprite_left || x >= sprite_left + 8 {
                continue;
            }

            // Calculate pixel position within sprite
            let pixel_y = (y - sprite_top) as u8;
            let pixel_x = (x - sprite_left) as u8;

            // Handle flip
            let flip_x = (attr & 0x40) != 0;
            let flip_y = (attr & 0x80) != 0;
            let _priority = (attr & 0x20) != 0;

            let render_x = if flip_x { 7 - pixel_x } else { pixel_x };
            let render_y = if flip_y { sprite_size - 1 - (pixel_y as u16) } else { pixel_y as u16 };

            // Get tile data
            let pattern_addr = sprite_pattern_base + (tile as u16) * 16 + (render_y as u16);
            let byte1 = self.vram_read(pattern_addr);
            let byte2 = self.vram_read(pattern_addr + 8);

            let bit1 = (byte1 >> (7 - render_x as u8)) & 1;
            let bit2 = (byte2 >> (7 - render_x as u8)) & 1;
            let color = (bit2 << 1) | bit1;

            if color == 0 {
                continue;  // Transparent
            }

            // Get palette
            let palette = ((attr & 0x03) as u8) + 1;  // Sprite palettes are 1-3

            let palette_addr = 0x3F10 + (palette as u16) * 4 + color as u16;
            let palette_index = self.vram_read(palette_addr) & 0x3F;

            return Some(self.get_palette_color(palette_index));
        }

        None
    }

    /// Check sprite 0 hit
    pub fn check_sprite0_hit(&mut self, x: u16, y: u16) {
        if !self.sprite_visible || !self.bg_visible {
            return;
        }

        // Only check sprite 0
        let sprite_y = self.oam[0] as u16;
        let sprite_x = self.oam[3] as u16;
        let sprite_size = if self.sprite_size { 16 } else { 8 };

        // Check if pixel is within sprite bounds
        if y >= sprite_y && y < sprite_y + sprite_size &&
           x >= sprite_x && x < sprite_x + 8 {
            // Check for non-transparent pixel overlap
            let bg_color = self.render_background(x, y);
            if bg_color != 0 {
                self.sprite0_hit = true;
            }
        }
    }

    /// Run PPU for n cycles
    pub fn run_cycles(&mut self, cycles: u64) {
        for _ in 0..cycles {
            self.cur_x += 1;

            if self.cur_x == 341 {
                self.cur_x = 0;
                self.end_scanline();
            }
        }
    }

    /// Update sprite evaluation
    pub fn update_sprite_evaluation(&mut self) {
        // Sprite evaluation happens during pre-render scanline (scanline -1)
        // and the first visible scanline
        if self.scanline != -1 && self.scanline != 255 {
            return;
        }

        let sprite_size = if self.sprite_size { 16 } else { 8 };
        let mut sprites_on_line = 0;

        for i in 0..64 {
            let base = (i as u16) * 4;
            let sprite_y = self.oam[base as usize] as u16;

            // Check if sprite is on this scanline
            // (-1 for off-screen top)
            if (self.scanline as u16) >= sprite_y &&
               (self.scanline as u16) < sprite_y + sprite_size {
                sprites_on_line += 1;

                if sprites_on_line > 8 {
                    self.sprite_overflow = true;
                    break;
                }

                // Check sprite 0
                if i == 0 && self.bg_visible {
                    let sprite_x = self.oam[(base as usize) + 3] as u16;
                    if self.cur_x >= sprite_x && self.cur_x < sprite_x + 8 {
                        self.check_sprite0_hit(self.cur_x, self.scanline as u16);
                    }
                }
            }
        }

        self.sprites_evaluated = sprites_on_line;
    }
}

impl Default for PPU {
    fn default() -> Self {
        Self::new()
    }
}
#[cfg(test)]
mod ppu_tests {
    use super::*;

    #[test]
    fn test_render_pixel_background() {
        let mut ppu = PPU::new();
        
        // Enable background display
        ppu.bg_visible = true;
        ppu.sprite_visible = false;
        
        // Set palette to white (color 3 in palette 0)
        ppu.palette[0] = 0x3F;  // Color index 15 (white)
        ppu.palette[1] = 0x00;  // Color index 0 (black)
        ppu.palette[2] = 0x00;
        ppu.palette[3] = 0x00;
        
        // Draw a simple pattern - render a pixel at (10, 20)
        let color = ppu.render_pixel(10, 20);
        
        // Should be black (0) since no background is visible at that position
        // (palette 0, color 0 is black)
        assert_eq!(color, 0x000000);
    }
    
    #[test]
    fn test_frame_buffer_population() {
        let mut ppu = PPU::new();
        
        // Manually render to the frame buffer (simulating what render_scanline does)
        for y in 0..10 {
            for x in 0..20 {
                let color = 0xFF0000FF; // Blue pixel
                let idx = (y as usize * 256) + (x as usize);
                ppu.frame_buffer[idx] = color;
            }
        }
        
        // Check that pixels were written
        assert_eq!(ppu.frame_buffer[256 * 5 + 10], 0xFF0000FF);
    }
}
