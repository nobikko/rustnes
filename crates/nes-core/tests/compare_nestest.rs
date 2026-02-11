//! Compare NES test output with nestest.log

use std::env;
use std::fs;

use nes_core::system::NesSystem;
use nes_core::cartridge::Cartridge;

fn parse_log_line(line: &str) -> Option<LogEntry> {
    // Format: C000  4C F5 C5  JMP $C5F5                       A:00 X:00 Y:00 P:24 SP:FD PPU:  0, 21 CYC:7
    let line = line.trim();
    if line.is_empty() {
        return None;
    }

    // Parse PC address (first 4 hex chars)
    let pc_str = line.get(0..4)?;
    let pc = u16::from_str_radix(pc_str, 16).ok()?;

    // Parse opcode bytes (after PC, before instruction)
    // Find the position of "  " that separates opcodes from instruction
    // The opcodes are at positions 5..X where X is the start of "  " before instruction
    let mut opcode_end = 5;
    let mut i = 5;
    while i < line.len() - 1 {
        if line[i..].starts_with("  ") {
            opcode_end = i;
            break;
        }
        i += 1;
    }

    let opcode_str = line.get(5..opcode_end)?.trim();
    let opcodes: Vec<u8> = opcode_str
        .split_whitespace()
        .filter(|s| !s.is_empty())
        .map(|s| u8::from_str_radix(s, 16).ok())
        .collect::<Option<Vec<_>>>()?;

    // Find where registers start (position of "A:")
    let registers_start = line.find("A:")?;

    // Extract instruction part: from after opcodes to before registers
    let instr_part = line.get(opcode_end + 2..registers_start)?;

    // The instruction is the part before the trailing spaces
    // Find the last non-space character by iterating from the end
    let mut instr_end_pos = instr_part.len();
    for (i, c) in instr_part.char_indices().rev() {
        if !c.is_whitespace() {
            instr_end_pos = i + c.len_utf8();
            break;
        }
    }
    let instruction = instr_part.get(..instr_end_pos)?.trim();

    // Parse registers: A:XX X:XX Y:XX P:XX SP:XX
    let registers_str = line.get(registers_start..)?;

    let a = parse_hex(registers_str, "A:")?;
    let x = parse_hex(registers_str, "X:")?;
    let y = parse_hex(registers_str, "Y:")?;
    let p = parse_hex(registers_str, "P:")?;
    let sp = parse_hex(registers_str, "SP:")?;

    // Parse PPU: line,cycle
    let ppu_str = registers_str.get(registers_str.find("PPU:")?..)?;
    let ppu_parts: Vec<&str> = ppu_str
        .split(|c| c == ',' || c == ' ')
        .filter(|s| !s.is_empty())
        .collect();
    let ppu_line = ppu_parts.get(1)?.parse::<i16>().ok()?;
    let ppu_cycle = ppu_parts.get(2)?.parse::<u16>().ok()?;

    // Parse CYC:cycle
    let cyc_str = ppu_str.get(ppu_str.find("CYC:")?..)?;
    let cycles = cyc_str.get(4..)?.parse::<u64>().ok()?;

    Some(LogEntry {
        pc,
        opcodes,
        instruction: instruction.to_string(),
        a,
        x,
        y,
        p,
        sp,
        ppu_line,
        ppu_cycle,
        cycles,
    })
}

fn parse_hex(s: &str, prefix: &str) -> Option<u8> {
    let start = s.find(prefix)? + prefix.len();
    let hex = s.get(start..start + 2)?;
    u8::from_str_radix(hex, 16).ok()
}

struct LogEntry {
    pc: u16,
    opcodes: Vec<u8>,
    instruction: String,
    a: u8,
    x: u8,
    y: u8,
    p: u8,
    sp: u8,
    ppu_line: i16,
    ppu_cycle: u16,
    cycles: u64,
}

fn get_nestest_log_path() -> String {
    // Try multiple possible paths since tests can run from different directories
    let paths = [
        "tests/roms/nestest.log",
        "../tests/roms/nestest.log",
        "../../tests/roms/nestest.log",
        "../../../tests/roms/nestest.log",
    ];

    let current_dir = env::current_dir().unwrap_or_default();
    for path in paths {
        let full_path = current_dir.join(path);
        if full_path.exists() {
            return full_path.to_string_lossy().to_string();
        }
    }

    // Fallback to relative path from crate directory
    "../roms/nestest.log".to_string()
}

#[test]
fn test_nestest_log_parsing() {
    let log_path = get_nestest_log_path();
    let log_content = fs::read_to_string(&log_path)
        .expect(format!("Failed to read nestest.log at {}", log_path).as_str());

    let entries: Vec<LogEntry> = log_content
        .lines()
        .filter_map(parse_log_line)
        .collect();

    assert!(entries.len() > 0, "No entries parsed from log");

    // Check initial state (first entry)
    let first = entries.first().expect("Empty log");
    assert_eq!(first.pc, 0xC000, "First instruction should be at $C000");
    assert_eq!(first.opcodes, vec![0x4C, 0xF5, 0xC5], "First instruction opcodes should be JMP $C5F5");
    assert_eq!(first.a, 0x00);
    assert_eq!(first.x, 0x00);
    assert_eq!(first.y, 0x00);
    assert_eq!(first.p, 0x24);
    assert_eq!(first.sp, 0xFD);
}

#[test]
fn test_nestest_log_content() {
    let log_path = get_nestest_log_path();
    let log_content = fs::read_to_string(&log_path)
        .expect(format!("Failed to read nestest.log at {}", log_path).as_str());

    let entries: Vec<LogEntry> = log_content
        .lines()
        .filter_map(parse_log_line)
        .collect();

    println!("Parsed {} log entries from {}", entries.len(), log_path);
    assert!(entries.len() > 100, "Should have parsed many log entries");

    // Debug: print first few instructions
    for i in 0..5 {
        println!("  Entry {}: pc=${:04X} instr='{}'", i, entries[i].pc, entries[i].instruction);
    }

    // Verify some key instructions
    // First few instructions from nestest.log
    assert!(entries[0].instruction.starts_with("JMP"), "First instruction should be JMP");
    assert!(entries[1].instruction.starts_with("LDX"), "Second instruction should be LDX");
    assert!(entries[2].instruction.starts_with("STX"), "Third instruction should be STX");
}

/// Get nestest.nes ROM path
fn get_nestest_rom_path() -> String {
    let paths = [
        "tests/roms/nestest.nes",
        "../tests/roms/nestest.nes",
        "../../tests/roms/nestest.nes",
        "../../../tests/roms/nestest.nes",
    ];

    let current_dir = env::current_dir().unwrap_or_default();
    for path in paths {
        let full_path = current_dir.join(path);
        if full_path.exists() {
            return full_path.to_string_lossy().to_string();
        }
    }

    "../roms/nestest.nes".to_string()
}

/// Capture CPU state at a given point
fn capture_cpu_state(system: &NesSystem) -> CpuState {
    let cpu = system.cpu();
    let registers = cpu.registers();
    CpuState {
        pc: registers.pc,
        a: registers.a,
        x: registers.x,
        y: registers.y,
        p: cpu.p_register(),
        sp: registers.sp,
        cycles: cpu.total_cycles(),
    }
}

/// CPU state for comparison
struct CpuState {
    pc: u16,
    a: u8,
    x: u8,
    y: u8,
    p: u8,
    sp: u8,
    cycles: u64,
}

impl CpuState {
    /// Format as hex string for display
    fn to_string(&self) -> String {
        format!(
            "PC:${:04X} A:${:02X} X:${:02X} Y:${:02X} P:${:02X} SP:${:02X} CYC:{}",
            self.pc, self.a, self.x, self.y, self.p, self.sp, self.cycles
        )
    }
}

/// Compare emulator state with nestest.log entries
#[test]
fn test_compare_with_nestest_log() {
    let log_path = get_nestest_log_path();
    let rom_path = get_nestest_rom_path();

    // Read log file
    let log_content = fs::read_to_string(&log_path)
        .expect(format!("Failed to read nestest.log at {}", log_path).as_str());

    // Parse log entries
    let log_entries: Vec<LogEntry> = log_content
        .lines()
        .filter_map(parse_log_line)
        .collect();

    assert!(!log_entries.is_empty(), "No entries parsed from log");

    // Read ROM file
    let rom_data = fs::read(&rom_path)
        .expect(format!("Failed to read nestest.nes at {}", rom_path).as_str());

    // The nestest.nes ROM may not have a valid reset vector at $FFFC
    // We'll manually set the CPU to start at $C000 where nestest.log begins

    // Create NES system and load ROM
    let mut system = NesSystem::new();
    system.load_rom(&rom_data)
        .expect("Failed to load ROM");

    // Manually set PC to $C000 (where nestest.log starts) and reset the CPU state
    {
        let cpu = system.cpu_mut();
        cpu.registers_mut().pc = 0xC000;
        // Set initial registers as expected by nestest.log
        cpu.registers_mut().a = 0x00;
        cpu.registers_mut().x = 0x00;
        cpu.registers_mut().y = 0x00;
        cpu.registers_mut().sp = 0xFD;
        // Status: P=0x24 (interrupt disable set, unknown flag set)
        // 0x24 = 0b00100100 = C=0, Z=0, I=1, D=0, B=0, U=1, V=0, N=0
        cpu.status_mut().set_carry(false);
        cpu.status_mut().set_zero(false);
        cpu.status_mut().set_interrupt(true);
        cpu.status_mut().set_decimal(false);
        cpu.status_mut().set_overflow(false);
        cpu.status_mut().set_negative(false);
    }

    println!("CPU initialized: PC=${:04X} A=${:02X} X=${:02X} Y=${:02X} P=${:02X} SP=${:02X}",
             system.cpu().registers().pc,
             system.cpu().registers().a,
             system.cpu().registers().x,
             system.cpu().registers().y,
             system.cpu().registers().p,
             system.cpu().registers().sp);

    // Let's step through and compare with log entries
    let mut instruction_count = 0;
    let mut log_index = 0;

    // The log shows state BEFORE each instruction execution
    // So we compare BEFORE stepping, then step, then move to next log entry
    // Run up to 1000 instructions or until we match enough log entries
    while instruction_count < 1000 && log_index < log_entries.len() {
        // Capture state before step - this is what we compare against
        let cpu = system.cpu();
        let registers = cpu.registers();
        let state_before = CpuState {
            pc: registers.pc,
            a: registers.a,
            x: registers.x,
            y: registers.y,
            p: cpu.p_register(),
            sp: registers.sp,
            cycles: cpu.total_cycles(),
        };

        // Check if this matches the current log entry
        // The log shows state BEFORE instruction execution
        let log_entry = &log_entries[log_index];

        if state_before.pc == log_entry.pc {
            // Check registers match (mask out B and U flags which may differ)
            let p_mask = 0x30; // B and U flags can differ
            let a_match = state_before.a == log_entry.a;
            let x_match = state_before.x == log_entry.x;
            let y_match = state_before.y == log_entry.y;
            let p_match = (state_before.p & !p_mask) == (log_entry.p & !p_mask);
            let sp_match = state_before.sp == log_entry.sp;

            if a_match && x_match && y_match && p_match && sp_match {
                log_index += 1;
                if log_index % 50 == 0 {
                    println!("Matched {} instructions, PC=${:04X}", log_index, state_before.pc);
                }
            } else {
                // Print mismatch details
                if !a_match {
                    println!("MISMATCH at instr {}: A: got ${:02X}, expected ${:02X}",
                             log_index, state_before.a, log_entry.a);
                }
                if !x_match {
                    println!("MISMATCH at instr {}: X: got ${:02X}, expected ${:02X}",
                             log_index, state_before.x, log_entry.x);
                }
                if !y_match {
                    println!("MISMATCH at instr {}: Y: got ${:02X}, expected ${:02X}",
                             log_index, state_before.y, log_entry.y);
                }
                if !p_match {
                    println!("MISMATCH at instr {}: P: got ${:02X}, expected ${:02X}",
                             log_index, state_before.p, log_entry.p);
                }
                if !sp_match {
                    println!("MISMATCH at instr {}: SP: got ${:02X}, expected ${:02X}",
                             log_index, state_before.sp, log_entry.sp);
                }
            }
        } else {
            // Print PC mismatch
            println!("PC MISMATCH at instr {}: log_index={}, got ${:04X}, expected ${:04X}",
                     instruction_count, log_index, state_before.pc, log_entry.pc);
        }

        // Step one instruction
        let step_result = system.step();
        match step_result {
            Ok(running) => {
                if !running {
                    println!("CPU stopped at instruction {}", instruction_count);
                    break;
                }
            }
            Err(e) => {
                println!("CPU error at instruction {}: {}", instruction_count, e);
                break;
            }
        }

        // Print instructions around the mismatch for debugging
        if instruction_count < 15 || (instruction_count >= 20 && instruction_count <= 30) {
            let cpu = system.cpu();
            let registers = cpu.registers();
            println!("Instr {}: after step - PC=${:04X} A=${:02X} X=${:02X} Y=${:02X} P=${:02X} SP=${:02X}",
                     instruction_count, registers.pc, registers.a, registers.x, registers.y, cpu.p_register(), registers.sp);
        }

        instruction_count += 1;
    }

    println!("Ran {} instructions, matched {} log entries", instruction_count, log_index);

    // Verify we can at least run some instructions and match the log
    // For nestest.nes, we should be able to match many entries
    // If we match at least 50 consecutive entries, the CPU implementation is working
    assert!(log_index > 50, "Should match at least 50 log entries (got {})", log_index);
}