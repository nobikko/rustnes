//! APU (Audio Processing Unit) implementation
//!
//! The NES APU has five channels:
//! - Pulse 1 (4-bit pulse wave)
//! - Pulse 2 (4-bit pulse wave)
//! - Triangle (linear noise)
//! - Noise (shift register)
//! - DMC (delta modulation channel)
//!
//! For now, this is a stub with timing hooks that can be expanded later.

/// APU register map
pub const APU_REGISTER_COUNT: usize = 24;

/// APU registers
#[derive(Debug, Clone, Copy)]
pub enum ApuRegister {
    /// $4000 - Pulse 1 control
    Pulse1Ctrl,
    /// $4001 - Pulse 1 sweep
    Pulse1Sweep,
    /// $4002 - Pulse 1 timer low
    Pulse1TimerLow,
    /// $4003 - Pulse 1 timer high
    Pulse1TimerHigh,
    /// $4004 - Pulse 2 control
    Pulse2Ctrl,
    /// $4005 - Pulse 2 sweep
    Pulse2Sweep,
    /// $4006 - Pulse 2 timer low
    Pulse2TimerLow,
    /// $4007 - Pulse 2 timer high
    Pulse2TimerHigh,
    /// $4008 - Triangle control
    TriangleCtrl,
    /// $400A - Triangle linear counter
    TriangleLinear,
    /// $400C - Triangle timer low
    TriangleTimerLow,
    /// $400D - Triangle timer high
    TriangleTimerHigh,
    /// $400E - Noise control
    NoiseCtrl,
    /// $4010 - Noise length table
    NoiseLength,
    /// $4012 - Noise timer low
    NoiseTimerLow,
    /// $4013 - Noise timer high
    NoiseTimerHigh,
    /// $4014 - DMC control
    DmcCtrl,
    /// $4015 - DMC timer
    DmcTimer,
    /// $4016 - Controller 1
    Controller1,
    /// $4017 - Controller 2
    Controller2,
}

/// APU state
#[derive(Debug, Clone)]
pub struct Apu {
    /// APU registers
    registers: [u8; APU_REGISTER_COUNT],
    /// Cycle counter for timing
    cycle_count: u64,
    /// Frame counter state
    frame_counter: u8,
    /// Frame counter increment period
    frame_period: u8,
}

impl Apu {
    /// Create a new APU instance
    pub fn new() -> Self {
        Self {
            registers: [0; APU_REGISTER_COUNT],
            cycle_count: 0,
            frame_counter: 0,
            frame_period: 0,
        }
    }

    /// Reset the APU
    pub fn reset(&mut self) {
        self.registers = [0; APU_REGISTER_COUNT];
        self.cycle_count = 0;
        self.frame_counter = 0;
        self.frame_period = 0;
    }

    /// Step the APU by the given number of cycles
    pub fn step(&mut self, cycles: u8) {
        self.cycle_count += cycles as u64;
        self.frame_counter = self.frame_counter.wrapping_add(cycles);
    }

    /// Get the current cycle count
    pub fn cycle_count(&self) -> u64 {
        self.cycle_count
    }

    /// Read an APU register
    pub fn read(&self, address: u16) -> u8 {
        let offset = (address - 0x4000) as usize;
        if offset < APU_REGISTER_COUNT {
            self.registers[offset]
        } else {
            0
        }
    }

    /// Write to an APU register
    pub fn write(&mut self, address: u16, value: u8) {
        let offset = (address - 0x4000) as usize;
        if offset < APU_REGISTER_COUNT {
            self.registers[offset] = value;
        }
    }

    /// Get the duration of a frame in CPU cycles
    pub fn frame_duration(&self) -> u64 {
        // NTSC: 29780 cycles per frame (59.94Hz)
        // PAL: 33240 cycles per frame (50Hz)
        29780
    }

    /// Get the duration of a half-frame
    pub fn half_frame_duration(&self) -> u64 {
        self.frame_duration() / 2
    }
}

impl Default for Apu {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apu_reset() {
        let mut apu = Apu::new();
        apu.reset();
        assert_eq!(apu.cycle_count(), 0);
    }

    #[test]
    fn test_apu_step() {
        let mut apu = Apu::new();
        apu.step(100);
        assert_eq!(apu.cycle_count(), 100);
    }

    #[test]
    fn test_apu_read_write() {
        let mut apu = Apu::new();
        apu.write(0x4000, 0x42);
        assert_eq!(apu.read(0x4000), 0x42);
    }
}