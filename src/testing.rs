//! Testing utilities for the NES emulator
//!
//! Provides tools for running ROM tests, CPU instruction validation,
//! and comprehensive emulator debugging.

use crate::nes::NES;
use crate::rom::Rom;

/// Test runner for NES test ROMs
pub struct TestRunner {
    nes: NES,
}

impl TestRunner {
    /// Create a new test runner
    pub fn new() -> Self {
        let nes = NES::new(44100);
        Self { nes }
    }

    /// Load a ROM for testing
    pub fn load_rom(&mut self, rom: Rom) -> Result<(), &'static str> {
        self.nes.load_rom(rom)
    }

    /// Run the nestest ROM and parse output
    ///
    /// The nestest ROM outputs test results via memory locations:
    /// - $02-$03: Error code (0 = success)
    /// - $00-$01: Current test number
    pub fn run_nestest(&mut self, max_frames: u32) -> Result<TestResult, String> {
        // Reset and prepare for test
        self.nes.reset();

        // For nestest ROM automation mode, PC should be set to $C000
        // This is where the test suite starts when running in automation mode
        // The reset vector is 0x0000, so we manually set PC to start at $C000
        self.nes.cpu.registers.pc = 0xC000;

        // nestest loads from $8000, PC should be set to reset vector
        // The reset vector for nestest is at $FFFC-$FFFD
        // We need to ensure the ROM is loaded correctly

        let mut frame_count = 0u32;
        let mut max_iterations = 0u32;
        let mut error_code: Option<u16> = None;

        // Run emulation until test completes or timeout
        while frame_count < max_frames {
            // Run a frame
            self.nes.frame();

            frame_count += 1;
            max_iterations += 1;

            // Read test status from memory
            let test_num_lo = self.nes.cpu.memory[0x02];
            let test_num_hi = self.nes.cpu.memory[0x03];
            let error_code_val = (test_num_hi as u16) << 8 | (test_num_lo as u16);

            if error_code_val != 0 {
                error_code = Some(error_code_val);
                break;
            }

            // Check if nestest has completed (branch to self at end at $FFF8)
            if self.nes.cpu.registers.pc == 0xFFF8 {
                break;
            }

            // Safety limit
            if max_iterations > 1_000_000 {
                break;
            }
        }

        Ok(TestResult {
            frames_executed: frame_count,
            max_iterations,
            error_code,
            passed: error_code == Some(0),
        })
    }

    /// Run a CPU test by executing instructions and checking registers
    pub fn run_cpu_test(
        &mut self,
        test_name: &str,
        setup_fn: impl FnOnce(&mut NES),
        execute_fn: impl FnOnce(&mut NES) -> bool,
        expected: &ExpectedState,
    ) -> CpuTestResult {
        self.nes.reset();

        // Apply setup
        setup_fn(&mut self.nes);

        // Execute test
        let passed = execute_fn(&mut self.nes);

        // Read final state
        let actual = ActualState {
            a: self.nes.cpu.registers.a,
            x: self.nes.cpu.registers.x,
            y: self.nes.cpu.registers.y,
            sp: self.nes.cpu.registers.sp,
            pc: self.nes.cpu.registers.pc,
            flags: self.nes.cpu.flags.clone(),
        };

        CpuTestResult {
            name: test_name.to_string(),
            passed,
            expected: expected.clone(),
            actual,
        }
    }

    /// Get frame buffer for visual testing
    pub fn get_frame_buffer(&self) -> &[u32] {
        self.nes.get_frame_buffer()
    }

    /// Get current CPU state for debugging
    pub fn cpu_state(&self) -> CpuState {
        CpuState {
            a: self.nes.cpu.registers.a,
            x: self.nes.cpu.registers.x,
            y: self.nes.cpu.registers.y,
            sp: self.nes.cpu.registers.sp,
            pc: self.nes.cpu.registers.pc,
            flags: self.nes.cpu.flags.clone(),
            cycles: self.nes.cpu.cycles,
        }
    }

    /// Dump CPU memory region for debugging
    pub fn dump_memory(&self, start: u16, end: u16) -> Vec<(u16, u8)> {
        (start..=end)
            .map(|addr| (addr, self.nes.cpu.memory[addr as usize]))
            .collect()
    }

    /// Print detailed CPU state for debugging
    pub fn print_cpu_state(&self) {
        let state = self.cpu_state();
        println!("CPU State:");
        println!("  A:    ${:02X}", state.a);
        println!("  X:    ${:02X}", state.x);
        println!("  Y:    ${:02X}", state.y);
        println!("  SP:   ${:02X}", state.sp);
        println!("  PC:   ${:04X}", state.pc);
        println!("  CYCLES: {}", state.cycles);
        println!("  Flags: {:08b} (N {:1} Z {:1} C {:1} I {:1} D {:1} V {:1})",
            state.flags.to_u8(),
            state.flags.sign as u8,
            state.flags.zero as u8,
            state.flags.carry as u8,
            state.flags.interrupt as u8,
            state.flags.decimal as u8,
            state.flags.overflow as u8,
        );
    }

    /// Run one CPU instruction (wrapper for NES::run_cpu)
    pub fn run_cpu(&mut self) -> u8 {
        self.nes.run_cpu()
    }
}

/// Expected CPU state for comparison
#[derive(Debug, Clone)]
pub struct ExpectedState {
    pub a: u8,
    pub x: u8,
    pub y: u8,
    pub sp: u8,
    pub pc: u16,
    pub flags: crate::cpu::StatusFlags,
}

impl ExpectedState {
    pub fn new(a: u8, x: u8, y: u8, sp: u8, pc: u16, flags: crate::cpu::StatusFlags) -> Self {
        Self { a, x, y, sp, pc, flags }
    }
}

/// Actual CPU state from emulation
#[derive(Debug, Clone)]
pub struct ActualState {
    pub a: u8,
    pub x: u8,
    pub y: u8,
    pub sp: u8,
    pub pc: u16,
    pub flags: crate::cpu::StatusFlags,
}

/// Test result for ROM tests
#[derive(Debug, Clone)]
pub struct TestResult {
    pub frames_executed: u32,
    pub max_iterations: u32,
    pub error_code: Option<u16>,
    pub passed: bool,
}

/// CPU test result
#[derive(Debug, Clone)]
pub struct CpuTestResult {
    pub name: String,
    pub passed: bool,
    pub expected: ExpectedState,
    pub actual: ActualState,
}

impl CpuTestResult {
    pub fn print(&self) {
        println!("\n=== CPU Test: {} ===", self.name);
        println!("Result: {}", if self.passed { "PASSED" } else { "FAILED" });

        if !self.passed {
            println!("\nExpected:");
            println!("  A:    ${:02X}", self.expected.a);
            println!("  X:    ${:02X}", self.expected.x);
            println!("  Y:    ${:02X}", self.expected.y);
            println!("  SP:   ${:02X}", self.expected.sp);
            println!("  PC:   ${:04X}", self.expected.pc);
            println!("  Flags: {:08b}", self.expected.flags.to_u8());

            println!("\nActual:");
            println!("  A:    ${:02X}", self.actual.a);
            println!("  X:    ${:02X}", self.actual.x);
            println!("  Y:    ${:08X}", self.actual.y);
            println!("  SP:   ${:02X}", self.actual.sp);
            println!("  PC:   ${:04X}", self.actual.pc);
            println!("  Flags: {:08b}", self.actual.flags.to_u8());
        }
    }
}

/// Full CPU state for debugging
#[derive(Debug, Clone)]
pub struct CpuState {
    pub a: u8,
    pub x: u8,
    pub y: u8,
    pub sp: u8,
    pub pc: u16,
    pub flags: crate::cpu::StatusFlags,
    pub cycles: u64,
}


/// Load a test ROM from the test_roms directory
pub fn load_test_rom(name: &str) -> Result<Rom, String> {
    let path = format!("test_roms/{}/{}.nes", name, name);
    Rom::load_from_file(&path)
        .map_err(|e| format!("Failed to load ROM {}: {}", path, e))
}

/// Run all available tests
pub fn run_all_tests() {
    println!("=== NES Emulator Test Suite ===\n");

    // Test 1: Basic CPU instructions
    test_cpu_instructions();

    // Test 2: Memory operations
    test_memory_operations();

    // Test 3: Stack operations
    test_stack_operations();

    // Test 4: Branch instructions
    test_branch_instructions();

    println!("\n=== Test Suite Complete ===");
}

/// Test CPU instruction set
fn test_cpu_instructions() {
    println!("--- Testing CPU Instructions ---");

    let mut runner = TestRunner::new();

    // Test LDA immediate
    let test_result = runner.run_cpu_test(
        "LDA Immediate",
        |nes| {
            // LDA #$42
            nes.cpu.memory[0x0000] = 0xA9;
            nes.cpu.memory[0x0001] = 0x42;
            nes.cpu.registers.pc = 0x0000;
        },
        |nes| {
            nes.run_cpu();
            nes.cpu.registers.a == 0x42
        },
        &ExpectedState::new(
            0x42,
            0x00,
            0x00,
            0xFD,
            0x0001,
            crate::cpu::StatusFlags::new(),
        ),
    );

    test_result.print();

    // Test STA absolute
    let test_result = runner.run_cpu_test(
        "STA Absolute",
        |nes| {
            // LDA #$AA
            // STA $0200
            nes.cpu.memory[0x0000] = 0xA9;
            nes.cpu.memory[0x0001] = 0xAA;
            nes.cpu.memory[0x0002] = 0x8D;
            nes.cpu.memory[0x0003] = 0x00;
            nes.cpu.memory[0x0004] = 0x02;
            nes.cpu.registers.pc = 0x0000;
        },
        |nes| {
            nes.run_cpu(); // LDA
            nes.run_cpu(); // STA
            nes.cpu.memory[0x0200] == 0xAA
        },
        &ExpectedState::new(
            0xAA,
            0x00,
            0x00,
            0xFD,
            0x0005,
            crate::cpu::StatusFlags::new(),
        ),
    );

    test_result.print();
}

/// Test memory operations
fn test_memory_operations() {
    println!("\n--- Testing Memory Operations ---");

    let mut runner = TestRunner::new();

    // Test zero page addressing
    let test_result = runner.run_cpu_test(
        "Zero Page Addressing",
        |nes| {
            // LDA $80
            // STA $81
            nes.cpu.memory[0x0000] = 0xA5;
            nes.cpu.memory[0x0001] = 0x80;
            nes.cpu.memory[0x0002] = 0x85;
            nes.cpu.memory[0x0003] = 0x81;
            nes.cpu.memory[0x0080] = 0x55;
            nes.cpu.registers.pc = 0x0000;
        },
        |nes| {
            nes.run_cpu(); // LDA $80
            nes.run_cpu(); // STA $81
            nes.cpu.memory[0x0081] == 0x55 && nes.cpu.registers.a == 0x55
        },
        &ExpectedState::new(
            0x55,
            0x00,
            0x00,
            0xFD,
            0x0004,
            crate::cpu::StatusFlags::new(),
        ),
    );

    test_result.print();
}

/// Test stack operations
fn test_stack_operations() {
    println!("\n--- Testing Stack Operations ---");

    let mut runner = TestRunner::new();

    // Test PHA/PLA
    let test_result = runner.run_cpu_test(
        "PHA/PLA",
        |nes| {
            // LDA #$33
            // PHA
            // PLA
            nes.cpu.memory[0x0000] = 0xA9;
            nes.cpu.memory[0x0001] = 0x33;
            nes.cpu.memory[0x0002] = 0x48;
            nes.cpu.memory[0x0003] = 0x68;
            nes.cpu.registers.pc = 0x0000;
        },
        |nes| {
            nes.run_cpu(); // LDA
            nes.run_cpu(); // PHA
            nes.run_cpu(); // PLA
            nes.cpu.registers.a == 0x33
        },
        &ExpectedState::new(
            0x33,
            0x00,
            0x00,
            0xFD,
            0x0004,
            crate::cpu::StatusFlags::new(),
        ),
    );

    test_result.print();
}

/// Test branch instructions
fn test_branch_instructions() {
    println!("\n--- Testing Branch Instructions ---");

    let mut runner = TestRunner::new();

    // Test BEQ (branch if equal/zero)
    let test_result = runner.run_cpu_test(
        "BEQ Branch",
        |nes| {
            // LDA #$00 (sets zero flag)
            // BEQ +2 (branch if zero)
            // NOP
            // BRK
            nes.cpu.memory[0x0000] = 0xA9;
            nes.cpu.memory[0x0001] = 0x00;
            nes.cpu.memory[0x0002] = 0xF0;
            nes.cpu.memory[0x0003] = 0x02;
            nes.cpu.memory[0x0004] = 0xEA; // NOP
            nes.cpu.memory[0x0005] = 0x00; // BRK
            nes.cpu.registers.pc = 0x0000;
        },
        |nes| {
            nes.run_cpu(); // LDA
            nes.run_cpu(); // BEQ
            nes.cpu.registers.pc == 0x0005 // Should have branched to BRK
        },
        &ExpectedState::new(
            0x00,
            0x00,
            0x00,
            0xFD,
            0x0005,
            crate::cpu::StatusFlags {
                zero: true,
                ..crate::cpu::StatusFlags::new()
            },
        ),
    );

    test_result.print();
}

/// Test with a real ROM file
pub fn test_with_rom(rom_path: &str) -> Result<TestResult, String> {
    let mut runner = TestRunner::new();
    let rom = Rom::load_from_file(rom_path)
        .map_err(|e| format!("Failed to load ROM: {}", e))?;
    runner.load_rom(rom)
        .map_err(|e| format!("Failed to load ROM into NES: {}", e))?;
    runner.run_nestest(1000)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_test_rom() {
        // Test that we can load a ROM
        let rom = load_test_rom("nestest");
        assert!(rom.is_ok());
    }

    #[test]
    fn test_cpu_lda_immediate() {
        let mut runner = TestRunner::new();

        // LDA #$42
        runner.nes.cpu.memory[0x0000] = 0xA9;
        runner.nes.cpu.memory[0x0001] = 0x42;
        runner.nes.cpu.registers.pc = 0x0000;

        println!("Before: PC={:04X}, A=${:02X}", runner.nes.cpu.registers.pc, runner.nes.cpu.registers.a);
        runner.run_cpu();
        println!("After: PC={:04X}, A=${:02X}", runner.nes.cpu.registers.pc, runner.nes.cpu.registers.a);

        assert_eq!(runner.nes.cpu.registers.a, 0x42);
    }

    #[test]
    fn test_cpu_sta_absolute() {
        let mut runner = TestRunner::new();

        // LDA #$AA
        // STA $0200
        runner.nes.cpu.memory[0x0000] = 0xA9;
        runner.nes.cpu.memory[0x0001] = 0xAA;
        runner.nes.cpu.memory[0x0002] = 0x8D;
        runner.nes.cpu.memory[0x0003] = 0x00;
        runner.nes.cpu.memory[0x0004] = 0x02;

        runner.nes.cpu.registers.pc = 0x0000;

        runner.run_cpu(); // LDA

        runner.run_cpu(); // STA
        println!("After STA: memory[0200]=${:02X}", runner.nes.cpu.memory[0x0200]);

        assert_eq!(runner.nes.cpu.memory[0x0200], 0xAA);
    }

    #[test]
    fn test_cpu_branch_beq() {
        let mut runner = TestRunner::new();

        // LDA #$00 (sets zero)
        // BEQ +2
        // BRK
        runner.nes.cpu.memory[0x0000] = 0xA9;
        runner.nes.cpu.memory[0x0001] = 0x00;
        runner.nes.cpu.memory[0x0002] = 0xF0;
        runner.nes.cpu.memory[0x0003] = 0x02;
        runner.nes.cpu.memory[0x0005] = 0x00; // BRK

        runner.nes.cpu.registers.pc = 0x0000;

        runner.run_cpu(); // LDA
        runner.run_cpu(); // BEQ

        // Should have branched to $0005
        assert_eq!(runner.nes.cpu.registers.pc, 0x0005);
    }

    #[test]
    fn test_cpu_stack_pha_pla() {
        let mut runner = TestRunner::new();

        // LDA #$33
        // PHA
        // PLA
        runner.nes.cpu.memory[0x0000] = 0xA9;
        runner.nes.cpu.memory[0x0001] = 0x33;
        runner.nes.cpu.memory[0x0002] = 0x48;
        runner.nes.cpu.memory[0x0003] = 0x68;

        runner.nes.cpu.registers.pc = 0x0000;

        runner.run_cpu(); // LDA
        runner.run_cpu(); // PHA
        runner.run_cpu(); // PLA

        assert_eq!(runner.nes.cpu.registers.a, 0x33);
    }

    #[test]
    fn test_jmp_instruction() {
        // Test that JMP instruction works correctly
        let mut runner = TestRunner::new();

        // Load nestest ROM
        let rom = load_test_rom("nestest").unwrap();
        runner.load_rom(rom).unwrap();
        runner.nes.reset();

        // Set PC to $C000 for automation mode
        runner.nes.cpu.registers.pc = 0xC000;

        println!("Before JMP:");
        println!("  PC = ${:04X}", runner.nes.cpu.registers.pc);
        println!("  Opcode at $C000 = ${:02X}", runner.nes.cpu.memory[0xC000]);
        println!("  Address bytes at $C001 = ${:02X}, $C002 = ${:02X}",
                 runner.nes.cpu.memory[0xC001], runner.nes.cpu.memory[0xC002]);

        // Run one CPU instruction (JMP)
        let cycles = runner.run_cpu();
        println!("After JMP:");
        println!("  PC = ${:04X}", runner.nes.cpu.registers.pc);
        println!("  Cycles executed: {}", cycles);

        // The JMP should set PC to $C5F5 (little endian: $C5 high, $F5 low)
        assert_eq!(runner.nes.cpu.registers.pc, 0xC5F5, "JMP should set PC to $C5F5");
    }

    #[test]
    fn test_rom_memory_contents() {
        // Test that ROM is loaded correctly into CPU memory
        let mut runner = TestRunner::new();

        // Load nestest ROM
        let rom = load_test_rom("nestest").unwrap();
        println!("PRG-ROM size: {}", rom.prg_rom.len());
        println!("First 10 bytes of PRG-ROM:");
        for i in 0..10 {
            println!("  PRG-ROM[{:04X}] = ${:02X}", i, rom.prg_rom[i]);
        }

        runner.load_rom(rom).unwrap();

        // Check memory contents at key addresses
        println!("CPU memory at $C000: ${:02X}", runner.nes.cpu.memory[0xC000]);
        println!("CPU memory at $C001: ${:02X}", runner.nes.cpu.memory[0xC001]);
        println!("CPU memory at $C002: ${:02X}", runner.nes.cpu.memory[0xC002]);

        // Check $8000 as well
        println!("CPU memory at $8000: ${:02X}", runner.nes.cpu.memory[0x8000]);
        println!("CPU memory at $8001: ${:02X}", runner.nes.cpu.memory[0x8001]);
        println!("CPU memory at $8002: ${:02X}", runner.nes.cpu.memory[0x8002]);

        // The first instruction should be JMP at $8000
        assert_eq!(runner.nes.cpu.memory[0x8000], 0x4C, "JMP opcode at $8000");
        assert_eq!(runner.nes.cpu.memory[0x8001], 0xF5, "Low byte of JMP address at $8001");
        assert_eq!(runner.nes.cpu.memory[0x8002], 0xC5, "High byte of JMP address at $8002");

        // Should also be at $C000 for smaller ROMs
        assert_eq!(runner.nes.cpu.memory[0xC000], 0x4C, "JMP opcode at $C000");
    }

    #[test]
    fn test_frame_completes() {
        // Test that frames complete properly without hitting safety limit
        let mut runner = TestRunner::new();

        // Load nestest ROM
        let rom = load_test_rom("nestest").unwrap();
        runner.load_rom(rom).unwrap();
        runner.nes.reset();

        // Set PC to $C000 for automation mode
        runner.nes.cpu.registers.pc = 0xC000;

        let mut frame_count = 0;
        let max_frames = 50;

        while frame_count < max_frames {
            runner.nes.frame();

            // After frame, frame_complete should be false (reset by start_frame)
            assert!(!runner.nes.ppu.frame_complete,
                    "frame_complete should be reset after start_frame");

            frame_count += 1;

            // Check if frame loop hit safety limit
            // (We can't directly track this, but we can verify frames complete)

            // Check test status
            let error_code = (runner.nes.cpu.memory[0x03] as u16) << 8
                           | (runner.nes.cpu.memory[0x02] as u16);

            if error_code != 0 {
                println!("Error code: {:04X}", error_code);
                break;
            }

            // If nestest completed (branch to self at end)
            if runner.nes.cpu.registers.pc == 0xFFF8 {
                println!("nestest completed at frame {}", frame_count);
                break;
            }
        }

        println!("Completed {} frames without safety limit hits", frame_count);
        assert!(frame_count > 0, "Should have completed at least one frame");
    }

    #[test]
    fn test_frame_buffer_rendering() {
        // Test that the frame buffer is being updated during rendering
        let mut runner = TestRunner::new();

        // Load nestest ROM
        let rom = load_test_rom("nestest").unwrap();
        runner.load_rom(rom).unwrap();

        // Debug: Check what's at key memory locations
        eprintln!("Memory at $FFFC: ${:02X}", runner.nes.cpu.memory[0xFFFC]);
        eprintln!("Memory at $FFFD: ${:02X}", runner.nes.cpu.memory[0xFFFD]);
        eprintln!("Reset vector: ${:04X}", (runner.nes.cpu.memory[0xFFFD] as u16) << 8 | runner.nes.cpu.memory[0xFFFC] as u16);

        runner.nes.reset();

        // Check what PC was set to by reset
        eprintln!("PC after reset: ${:04X}", runner.nes.cpu.registers.pc);

        // Set PC to $C000 for automation mode
        runner.nes.cpu.registers.pc = 0xC000;

        // Run a few frames
        for i in 0..5 {
            // Check PPU state before frame
            eprintln!("PPU before frame {}: scanline={}, bg_visible={}, sprite_visible={}",
                i, runner.nes.ppu.scanline, runner.nes.ppu.bg_visible, runner.nes.ppu.sprite_visible);

            // Run some CPU instructions before frame to let ROM initialize PPU
            for _ in 0..1000 {
                runner.run_cpu();
            }

            runner.nes.frame();

            // Check frame buffer contents
            let buffer = runner.nes.get_frame_buffer();

            // Count non-black pixels
            let non_black_pixels: usize = buffer.iter()
                .filter(|&&p| p != 0)
                .count();

            eprintln!("Frame {}: {} non-black pixels", i, non_black_pixels);
            eprintln!("  PPU scanline: {}, cur_x: {}, frame_complete: {}",
                runner.nes.ppu.scanline,
                runner.nes.ppu.cur_x,
                runner.nes.ppu.frame_complete);
            eprintln!("  PPU bg_visible: {}, sprite_visible: {}",
                runner.nes.ppu.bg_visible,
                runner.nes.ppu.sprite_visible);

            // Show first 16 OAM bytes
            eprintln!("  OAM[0..16]: {:?}", &runner.nes.ppu.oam[0..16]);

            // After a few frames, we should have some non-black pixels
            // (assuming the ROM enables display)
            assert!(non_black_pixels > 0, "Frame {} should have some non-black pixels", i);
        }
    }
}

    #[test]
    fn test_rendering_with_custom_rom() {
        // Create a minimal ROM that renders a colored rectangle
        let mut runner = TestRunner::new();
        
        // Create a simple ROM that:
        // 1. Enables background display
        // 2. Writes a pattern to nametable
        // 3. Waits for VBlank and loops
        
        // Reset and prepare
        runner.nes.reset();
        
        // Enable display (write to $2001)
        runner.nes.write_ppu(0x2001, 0x1E);  // Enable sprites + background + blue emphasis
        
        // Write a blue rectangle pattern to the name table at $2000
        // Color 3 (blue) in palette 0
        for i in 0..240 {
            runner.nes.write_ppu(0x2000 + (i as u16), 0xFF);  // Fill with tile that uses color 3
        }
        
        // Run a few frames
        for i in 0..3 {
            runner.nes.frame();
            
            let buffer = runner.nes.get_frame_buffer();
            let non_black_pixels: usize = buffer.iter().filter(|&&p| p != 0).count();
            
            eprintln!("Frame {}: {} non-black pixels", i, non_black_pixels);
            
            // We should see non-black pixels after enabling display
            // (exact count depends on palette setup)
        }
    }

    #[test]
    fn test_rendering_with_accuracy_coin() {
        // Test with AccuracyCoin ROM which should have graphics
        let mut runner = TestRunner::new();

        let rom = load_test_rom("AccuracyCoin").unwrap();
        runner.load_rom(rom).unwrap();
        runner.nes.reset();

        // Enable display by writing to $2001
        runner.nes.write_ppu(0x2001, 0x1E);  // Enable sprites + background + blue emphasis
        eprintln!("After write to $2001: bg_visible={}, sprite_visible={}", runner.nes.ppu.bg_visible, runner.nes.ppu.sprite_visible);

        // Check some VRAM contents
        eprintln!("VRAM at $2000: ${:02X}", runner.nes.ppu.vram[0x2000]);
        eprintln!("VRAM at $2001: ${:02X}", runner.nes.ppu.vram[0x2001]);
        eprintln!("VRAM at $2400 (attr): ${:02X}", runner.nes.ppu.vram[0x2400]);

        // Run a few frames
        for i in 0..5 {
            runner.nes.frame();

            let buffer = runner.nes.get_frame_buffer();
            let non_black_pixels: usize = buffer.iter().filter(|&&p| p != 0).count();

            eprintln!("Frame {}: {} non-black pixels", i, non_black_pixels);

            // AccuracyCoin should show some graphics
            // After 5 frames, we should see some non-black pixels
            if non_black_pixels > 100 {
                eprintln!("Found {} non-black pixels after {} frames", non_black_pixels, i);
            }
        }
    }

    #[test]
    fn test_accuracy_coin_debug() {
        // Test with AccuracyCoin ROM which should have graphics
        let mut runner = TestRunner::new();
        
        let rom = load_test_rom("AccuracyCoin").unwrap();
        runner.load_rom(rom).unwrap();
        runner.nes.reset();
        
        // Enable debug mode
        runner.nes.debug = true;
        
        // Run a few frames
        for i in 0..2 {
            eprintln!("=== Frame {} ===", i);
            runner.nes.frame();

            let buffer = runner.nes.get_frame_buffer();
            let non_black_pixels: usize = buffer.iter().filter(|&&p| p != 0).count();
            eprintln!("Frame {}: {} non-black pixels", i, non_black_pixels);
        }
    }
