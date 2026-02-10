//! PPU tests for the NES emulator

use nes_core::ppu::Ppu;

#[test]
fn test_ppu_reset() {
    let mut ppu = Ppu::new();
    ppu.reset();

    assert_eq!(ppu.scanline(), -1);
    assert_eq!(ppu.dot(), 0);
}

#[test]
fn test_ppu_vblank() {
    let mut ppu = Ppu::new();
    // Force VBLANK state
    ppu.step(); // Advance to first scanline

    // VBLANK should not be set initially
    assert!(!ppu.status().vblank());
}