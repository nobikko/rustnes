//! Integration tests for the NES system

use nes_core::system::NesSystem;

#[test]
fn test_system_creation() {
    let system = NesSystem::new();
    assert_eq!(system.frame_count(), 0);
}

#[test]
fn test_system_reset() {
    let mut system = NesSystem::new();
    system.reset();
    assert_eq!(system.cpu().registers().pc, 0xFFFC);
}

#[test]
fn test_system_with_cartridge() {
    let prg_rom = vec![0xFF; 16384]; // 16KB
    let chr_rom = vec![0x00; 8192];  // 8KB
    let cartridge = nes_core::bus::SimpleCartridge::new(prg_rom, chr_rom);

    let mut system = NesSystem::new();
    system.load_simple_cartridge(cartridge);
    system.reset();

    assert_eq!(system.cpu().registers().pc, 0xFFFC);
}

#[test]
fn test_cpu_after_reset() {
    let mut system = NesSystem::new();
    system.reset();

    let cpu = system.cpu();
    assert_eq!(cpu.registers().a, 0);
    assert_eq!(cpu.registers().x, 0);
    assert_eq!(cpu.registers().y, 0);
    assert_eq!(cpu.registers().sp, 0xFD);
    assert_eq!(cpu.registers().pc, 0xFFFC);
}