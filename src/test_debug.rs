use rust_nes_emulator::{NES, Rom};

fn main() {
    let mut nes = NES::new(44100);
    
    // Enable debug mode
    nes.debug = true;
    
    // Load AccuracyCoin.nes
    match Rom::load_from_file("AccuracyCoin.nes") {
        Ok(rom) => {
            if nes.load_rom(rom).is_ok() {
                println!("ROM loaded successfully");
                
                // Run a few frames
                for i in 0..3 {
                    println!("\n--- Frame {} ---", i);
                    nes.frame();
                    println!("Frame count: {}", nes.frame_count);
                    println!("PPU frame_complete: {}", nes.ppu.frame_complete);
                    println!("PPU scanline: {}", nes.ppu.scanline);
                    println!("PPU bg_visible: {}", nes.ppu.bg_visible);
                    println!("PPU sprite_visible: {}", nes.ppu.sprite_visible);
                    
                    // Count non-black pixels
                    let non_black: usize = nes.ppu.frame_buffer.iter().filter(|&&p| p != 0).count();
                    println!("Non-black pixels: {}", non_black);
                }
            }
        }
        Err(e) => println!("Load error: {}", e),
    }
}
