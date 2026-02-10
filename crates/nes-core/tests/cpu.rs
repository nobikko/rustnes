//! CPU tests for the NES emulator

use nes_core::cpu::{Cpu, CpuError, StatusFlags, Opcode};

#[test]
fn test_cpu_reset() {
    let mut cpu = Cpu::new();
    cpu.reset();

    assert_eq!(cpu.registers().a, 0);
    assert_eq!(cpu.registers().x, 0);
    assert_eq!(cpu.registers().y, 0);
    assert_eq!(cpu.registers().sp, 0xFD);
    assert_eq!(cpu.registers().pc, 0xFFFC);
}

#[test]
fn test_status_flags() {
    let mut flags = StatusFlags::new(0xFF);
    assert!(flags.carry());
    assert!(flags.zero());
    assert!(flags.interrupt());
    assert!(flags.overflow());
    assert!(flags.negative());

    flags.set_carry(false);
    assert!(!flags.carry());

    flags.set_overflow(true);
    assert!(flags.overflow());
}

#[test]
fn test_opcode_variants_exist() {
    // Just verify all opcode variants exist
    let _ = Opcode::ADCImmediate;
    let _ = Opcode::BRKImplied;
    let _ = Opcode::NOPImplied;
    let _ = Opcode::LDAImmediate;
    let _ = Opcode::LDXImmediate;
    let _ = Opcode::LDYImmediate;
    let _ = Opcode::STAZeroPage;
    let _ = Opcode::STXZeroPage;
    let _ = Opcode::STYZeroPage;
}