//! APU (Audio Processing Unit) Emulator
//!
//! Implements the Ricoh 2A03 APU with 5 channels:
//! - 2 Square wave channels
//! - 1 Triangle wave channel
//! - 1 Noise channel
//! - 1 DMC (Delta Modulation Channel)

/// APU registers
pub const REGSquare1_CTRL: u16 = 0x4000;
pub const REGSquare1_SWEEP: u16 = 0x4001;
pub const REGSquare1_FREQ_LOW: u16 = 0x4002;
pub const REGSquare1_FREQ_HIGH: u16 = 0x4003;
pub const REGSquare2_CTRL: u16 = 0x4004;
pub const REGSquare2_SWEEP: u16 = 0x4005;
pub const REGSquare2_FREQ_LOW: u16 = 0x4006;
pub const REGSquare2_FREQ_HIGH: u16 = 0x4007;
pub const REGTriangle_CTRL: u16 = 0x4008;
pub const REGTriangle_FREQ_LOW: u16 = 0x400A;
pub const REGTriangle_FREQ_HIGH: u16 = 0x400B;
pub const REGNoise_CTRL: u16 = 0x400C;
pub const REGNoise_FREQ: u16 = 0x400E;
pub const REGNoise_LENGTH: u16 = 0x400F;
pub const REGDMC_CTRL: u16 = 0x4010;
pub const REGDMC_DAC: u16 = 0x4011;
pub const REGDMC_START: u16 = 0x4012;
pub const REGDMC_LENGTH: u16 = 0x4013;
pub const REGCHANNEL_ENABLE: u16 = 0x4015;
pub const REGFRAME_COUNTER: u16 = 0x4017;

/// Square wave channel
#[derive(Debug)]
pub struct SquareChannel {
    pub enabled: bool,
    pub duty_cycle: u8,       // 0-3
    pub duty_position: u8,    // Current position in duty cycle
    pub envelope_loop: bool,  // Loop envelope
    pub envelope_constant: bool, // Constant volume
    pub envelope_period: u8,  // Envelope period (0-15)
    pub envelope_counter: u8, // Envelope counter
    pub envelope_volume: u8,  // Envelope volume (0-15)

    pub sweep_enabled: bool,  // Sweep enabled
    pub sweep_period: u8,     // Sweep period (0-7)
    pub sweep_direction: bool, // Sweep direction (0=add, 1=subtract)
    pub sweep_shift: u8,      // Sweep shift amount (0-7)
    pub sweep_counter: u8,    // Sweep counter

    pub length_counter: u8,   // Length counter (0-63)
    pub length_enabled: bool, // Length counter enabled

    pub timer_low: u8,        // Timer low byte
    pub timer_high: u8,       // Timer high byte (3 bits)
    pub timer_period: u16,    // Timer period
    pub timer_counter: u16,   // Timer counter

    pub output: i32,          // Current output sample
}

impl SquareChannel {
    pub fn new() -> Self {
        Self {
            enabled: false,
            duty_cycle: 0,
            duty_position: 0,
            envelope_loop: false,
            envelope_constant: false,
            envelope_period: 0,
            envelope_counter: 0,
            envelope_volume: 0,

            sweep_enabled: false,
            sweep_period: 0,
            sweep_direction: false,
            sweep_shift: 0,
            sweep_counter: 0,

            length_counter: 0,
            length_enabled: false,

            timer_low: 0,
            timer_high: 0,
            timer_period: 0,
            timer_counter: 0,

            output: 0,
        }
    }

    pub fn reset(&mut self) {
        self.enabled = false;
        self.duty_position = 0;
        self.envelope_counter = 0;
        self.envelope_volume = 0;
        self.sweep_counter = 0;
        self.length_counter = 0;
        self.timer_counter = 0;
        self.output = 0;
    }

    pub fn set_ctrl(&mut self, value: u8) {
        self.duty_cycle = (value >> 6) & 0x03;
        self.envelope_loop = (value & 0x08) != 0;
        self.envelope_constant = (value & 0x10) == 0;
        self.envelope_period = value & 0x0F;
        self.length_enabled = (value & 0x20) != 0;
    }

    pub fn set_sweep(&mut self, value: u8) {
        self.sweep_enabled = (value & 0x80) != 0;
        self.sweep_period = (value >> 4) & 0x07;
        self.sweep_direction = (value & 0x08) != 0;
        self.sweep_shift = value & 0x07;
    }

    pub fn set_freq_low(&mut self, value: u8) {
        self.timer_low = value;
        self.update_timer();
    }

    pub fn set_freq_high(&mut self, value: u8) {
        self.timer_high = value & 0x07;
        self.length_counter = ((value >> 3) & 0x1F) as u8;
        self.update_timer();

        if self.envelope_loop || self.envelope_constant {
            self.envelope_counter = self.envelope_period;
            self.envelope_volume = self.envelope_period;
        }

        // Reload length counter if enabled
        if self.length_enabled && self.length_counter == 0 {
            self.length_counter = 64;
        }
    }

    fn update_timer(&mut self) {
        let period = (self.timer_high as u16) << 8 | (self.timer_low as u16);
        self.timer_period = period + 1;
    }

    pub fn clock_length(&mut self) {
        if self.length_enabled && self.length_counter > 0 {
            self.length_counter -= 1;
        }
    }

    pub fn clock_envelope(&mut self) {
        if self.envelope_constant {
            self.envelope_volume = self.envelope_period;
        } else {
            if self.envelope_counter == 0 {
                self.envelope_counter = self.envelope_period;
                if self.envelope_volume > 0 {
                    self.envelope_volume -= 1;
                } else if self.envelope_loop {
                    self.envelope_volume = 15;
                }
            } else {
                self.envelope_counter -= 1;
            }
        }
    }

    pub fn clock_sweep(&mut self) {
        if !self.sweep_enabled || self.sweep_period == 0 {
            return;
        }

        if self.sweep_counter == 0 {
            self.sweep_counter = self.sweep_period;

            let shift_amount = self.sweep_shift as i16;
            let change = (self.timer_period >> shift_amount) as i16;

            if self.sweep_direction {
                // Subtract
                let new_period = self.timer_period as i16 - change;
                if new_period >= 0 {
                    self.timer_period = new_period as u16;
                }
            } else {
                // Add
                let new_period = self.timer_period as i16 + change;
                if new_period <= 0x7FF {
                    self.timer_period = new_period as u16;
                }
            }
        } else {
            self.sweep_counter -= 1;
        }
    }

    pub fn update_output(&mut self) {
        if self.timer_counter == 0 {
            self.timer_counter = self.timer_period;

            // Duty cycle pattern
            let pattern = match self.duty_cycle {
                0 => [1, 0, 0, 0, 0, 0, 0, 0],  // 1/8
                1 => [1, 1, 0, 0, 0, 0, 0, 0],  // 2/8
                2 => [1, 1, 1, 1, 0, 0, 0, 0],  // 4/8
                3 => [1, 1, 1, 1, 1, 1, 1, 0],  // 8/8 (negative)
                _ => [0; 8],
            };

            self.duty_position = (self.duty_position + 1) % 8;
            let output = pattern[self.duty_position as usize];

            // Output is envelope_volume * output
            if output == 1 {
                self.output = self.envelope_volume as i32;
            } else {
                self.output = 0;
            }
        } else {
            self.timer_counter -= 1;
        }
    }

    pub fn get_output(&self) -> i32 {
        if !self.enabled || self.length_counter == 0 {
            0
        } else {
            self.output
        }
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
}

impl Default for SquareChannel {
    fn default() -> Self {
        Self::new()
    }
}

/// Triangle wave channel
#[derive(Debug)]
pub struct TriangleChannel {
    pub enabled: bool,
    pub linear_counter_control: bool,  // Bit 7 of $4008
    pub linear_counter_load: u8,       // Bits 6-0 of $4008
    pub linear_counter: u8,

    pub length_counter: u8,
    pub length_enabled: bool,

    pub timer_low: u8,
    pub timer_high: u8,
    pub timer_period: u16,
    pub timer_counter: u16,

    pub output: i32,
}

impl TriangleChannel {
    pub fn new() -> Self {
        Self {
            enabled: false,
            linear_counter_control: false,
            linear_counter_load: 0,
            linear_counter: 0,
            length_counter: 0,
            length_enabled: false,
            timer_low: 0,
            timer_high: 0,
            timer_period: 0,
            timer_counter: 0,
            output: 0,
        }
    }

    pub fn reset(&mut self) {
        self.linear_counter = 0;
        self.length_counter = 0;
        self.timer_counter = 0;
        self.output = 0;
    }

    pub fn set_ctrl(&mut self, value: u8) {
        self.linear_counter_control = (value & 0x80) != 0;
        self.linear_counter_load = value & 0x7F;
        self.length_enabled = (value & 0x80) == 0;
    }

    pub fn set_freq_low(&mut self, value: u8) {
        self.timer_low = value;
        self.update_timer();
    }

    pub fn set_freq_high(&mut self, value: u8) {
        self.timer_high = value & 0x07;
        self.length_counter = ((value >> 3) & 0x1F) as u8;
        self.update_timer();

        if self.linear_counter_control {
            self.linear_counter = self.linear_counter_load;
        }
    }

    fn update_timer(&mut self) {
        let period = (self.timer_high as u16) << 8 | (self.timer_low as u16);
        self.timer_period = period + 1;
    }

    pub fn clock_length(&mut self) {
        if self.length_enabled && self.length_counter > 0 {
            self.length_counter -= 1;
        }
    }

    pub fn clock_linear(&mut self) {
        if self.linear_counter_control {
            self.linear_counter = self.linear_counter_load;
        } else if self.linear_counter > 0 {
            self.linear_counter -= 1;
        }
    }

    pub fn update_output(&mut self) {
        if self.timer_counter == 0 {
            self.timer_counter = self.timer_period;

            // Triangle wave output (0-15)
            if self.linear_counter > 0 && self.length_counter > 0 {
                // Simple triangle pattern: 0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,14,13,...
                // For simplicity, use a basic pattern
                self.output = ((self.timer_counter & 0x0F) as i32) - 8;
            } else {
                self.output = 0;
            }
        } else {
            self.timer_counter -= 1;
        }
    }

    pub fn get_output(&self) -> i32 {
        if !self.enabled || self.length_counter == 0 || self.linear_counter == 0 {
            0
        } else {
            self.output
        }
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
}

impl Default for TriangleChannel {
    fn default() -> Self {
        Self::new()
    }
}

/// Noise channel
#[derive(Debug)]
pub struct NoiseChannel {
    pub enabled: bool,
    pub envelope_loop: bool,
    pub envelope_constant: bool,
    pub envelope_period: u8,
    pub envelope_counter: u8,
    pub envelope_volume: u8,

    pub length_counter: u8,
    pub length_enabled: bool,

    pub noise_mode: bool,       // 0=7-stage, 1=15-stage
    pub noise_period_index: u8, // Frequency table index

    pub noise_shift: u32,       // Shift register
    pub noise_counter: u32,

    pub output: i32,
}

impl NoiseChannel {
    // Wavelength table (based on 1.79 MHz clock)
    const WAVELENGTHS: [u16; 16] = [
        4, 8, 16, 32, 64, 96, 128, 160, 202, 254, 380, 508, 762, 1016, 2034, 4068,
    ];

    pub fn new() -> Self {
        Self {
            enabled: false,
            envelope_loop: false,
            envelope_constant: false,
            envelope_period: 0,
            envelope_counter: 0,
            envelope_volume: 0,
            length_counter: 0,
            length_enabled: false,
            noise_mode: false,
            noise_period_index: 0,
            noise_shift: 0x7F,
            noise_counter: 0,
            output: 0,
        }
    }

    pub fn reset(&mut self) {
        self.envelope_counter = 0;
        self.envelope_volume = 0;
        self.length_counter = 0;
        self.noise_shift = 0x7F;
        self.noise_counter = 0;
        self.output = 0;
    }

    pub fn set_ctrl(&mut self, value: u8) {
        self.envelope_loop = (value & 0x08) != 0;
        self.envelope_constant = (value & 0x10) == 0;
        self.envelope_period = value & 0x0F;
        self.length_enabled = (value & 0x20) != 0;
    }

    pub fn set_freq(&mut self, value: u8) {
        self.noise_mode = (value & 0x80) != 0;
        self.noise_period_index = value & 0x0F;
    }

    pub fn set_length(&mut self, value: u8) {
        self.length_counter = ((value >> 3) & 0x1F) as u8;
    }

    pub fn clock_length(&mut self) {
        if self.length_enabled && self.length_counter > 0 {
            self.length_counter -= 1;
        }
    }

    pub fn clock_envelope(&mut self) {
        if self.envelope_constant {
            self.envelope_volume = self.envelope_period;
        } else {
            if self.envelope_counter == 0 {
                self.envelope_counter = self.envelope_period;
                if self.envelope_volume > 0 {
                    self.envelope_volume -= 1;
                } else if self.envelope_loop {
                    self.envelope_volume = 15;
                }
            } else {
                self.envelope_counter -= 1;
            }
        }
    }

    pub fn update_output(&mut self) {
        let period = Self::WAVELENGTHS[self.noise_period_index as usize] as u32;

        if self.noise_counter == 0 {
            self.noise_counter = period;

            // Generate noise
            let feedback = if self.noise_mode {
                // 15-stage: bit 14 XOR bit 13
                ((self.noise_shift >> 14) & 1) ^ ((self.noise_shift >> 13) & 1)
            } else {
                // 7-stage: bit 6 XOR bit 5
                ((self.noise_shift >> 6) & 1) ^ ((self.noise_shift >> 5) & 1)
            };

            let new_bit = feedback;
            self.noise_shift = (self.noise_shift << 1) | new_bit;

            // Output is noise shifted bit 0
            let output = (self.noise_shift & 1) as i32;
            if output == 0 {
                self.output = self.envelope_volume as i32;
            } else {
                self.output = 0;
            }
        } else {
            self.noise_counter -= 1;
        }
    }

    pub fn get_output(&self) -> i32 {
        if !self.enabled || self.length_counter == 0 {
            0
        } else {
            self.output
        }
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
}

impl Default for NoiseChannel {
    fn default() -> Self {
        Self::new()
    }
}

/// DMC (Delta Modulation Channel)
#[derive(Debug)]
pub struct DmcChannel {
    pub enabled: bool,
    pub play_mode: u8,      // 00=normal, 01=loop, 10=IRQ on completion
    pub frequency_index: u8,

    pub sample_address: u16,  // $C000 + (value << 6)
    pub sample_length: u16,   // (value + 1) * 16 bytes

    pub dac_latch: u8,      // 6-bit DAC
    pub delta_counter: u8,  // 6-bit delta

    pub sample_buffer: u8,
    pub sample_bit_count: u8,

    pub sample_address_counter: u16,
    pub sample_length_counter: u16,

    pub output: i32,
}

impl DmcChannel {
    // DMC frequency table (based on 1.79 MHz clock)
    const FREQ_DIVISORS: [u32; 16] = [
        537472, 478016, 424704, 399872, 357824, 318720, 298944, 279168,
        260608, 239008, 212352, 199936, 178912, 159360, 149472, 134848,
    ];

    pub fn new() -> Self {
        Self {
            enabled: false,
            play_mode: 0,
            frequency_index: 0,
            sample_address: 0,
            sample_length: 0,
            dac_latch: 0,
            delta_counter: 0,
            sample_buffer: 0,
            sample_bit_count: 0,
            sample_address_counter: 0,
            sample_length_counter: 0,
            output: 0,
        }
    }

    pub fn reset(&mut self) {
        self.dac_latch = 0;
        self.delta_counter = 0;
        self.sample_buffer = 0;
        self.sample_bit_count = 0;
        self.output = 0;
    }

    pub fn set_ctrl(&mut self, value: u8) {
        self.play_mode = (value >> 6) & 0x03;
        self.frequency_index = value & 0x0F;
    }

    pub fn set_dac(&mut self, value: u8) {
        self.dac_latch = value & 0x7F;
        self.delta_counter = self.dac_latch;
    }

    pub fn set_address(&mut self, value: u8) {
        self.sample_address = 0xC000 | ((value as u16) << 6);
    }

    pub fn set_length(&mut self, value: u8) {
        self.sample_length = ((value as u16) + 1) * 16;
    }

    pub fn start_sample(&mut self) {
        if self.sample_length > 0 {
            self.sample_address_counter = self.sample_address;
            self.sample_length_counter = self.sample_length;
            self.sample_bit_count = 0;
            self.enabled = true;
        }
    }

    pub fn clock_sample(&mut self) {
        if self.sample_bit_count == 0 && self.sample_length_counter > 0 {
            // Load next byte from ROM
            self.sample_buffer = 0;  // Would read from memory
            self.sample_bit_count = 8;
        }

        if self.sample_bit_count > 0 {
            let bit = (self.sample_buffer >> (self.sample_bit_count - 1)) & 1;

            if bit == 0 {
                if self.delta_counter >= 4 {
                    self.delta_counter -= 4;
                }
            } else {
                if self.delta_counter <= 123 {
                    self.delta_counter += 4;
                }
            }

            self.dac_latch = self.delta_counter;
            self.output = self.delta_counter as i32;

            self.sample_bit_count -= 1;
            self.sample_length_counter -= 1;

            if self.sample_length_counter == 0 && self.play_mode != 0 {
                // Re-start sample if looping
                self.sample_length_counter = self.sample_length;
            } else if self.sample_length_counter == 0 {
                self.enabled = false;
            }
        }
    }

    pub fn get_output(&self) -> i32 {
        if !self.enabled {
            0
        } else {
            self.output
        }
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
}

impl Default for DmcChannel {
    fn default() -> Self {
        Self::new()
    }
}

/// Frame counter for APU
#[derive(Debug)]
pub struct FrameCounter {
    pub cycle_counter: u64,
    pub step: u8,
    pub count_sequence: u8,  // 0=4-step, 1=5-step
    pub irq_enabled: bool,
    pub irq_pending: bool,
}

impl FrameCounter {
    // NTSC cycle counts for frame steps
    const FRAME_STEPS_4: [u64; 4] = [7457, 14913, 22371, 29829];
    const FRAME_STEPS_5: [u64; 5] = [7457, 14913, 22371, 29829, 37281];

    pub fn new() -> Self {
        Self {
            cycle_counter: 0,
            step: 0,
            count_sequence: 0,
            irq_enabled: false,
            irq_pending: false,
        }
    }

    pub fn reset(&mut self) {
        self.cycle_counter = 0;
        self.step = 0;
        self.irq_pending = false;
    }

    pub fn clock(&mut self, cycles: u64) {
        self.cycle_counter += cycles;

        if self.count_sequence == 0 {
            // 4-step sequence
            if self.step < 4 {
                let next_step = Self::FRAME_STEPS_4[self.step as usize];
                if self.cycle_counter >= next_step {
                    self.step += 1;
                    if self.step == 4 {
                        self.step = 0;
                        self.cycle_counter = 0;
                    }
                }
            }
        } else {
            // 5-step sequence
            if self.step < 5 {
                let next_step = Self::FRAME_STEPS_5[self.step as usize];
                if self.cycle_counter >= next_step {
                    self.step += 1;
                    if self.step == 5 {
                        self.step = 0;
                        self.cycle_counter = 0;
                        self.irq_pending = true;
                    }
                }
            }
        }
    }

    pub fn get_step(&self) -> u8 {
        self.step
    }

    pub fn set_sequence(&mut self, sequence: u8) {
        self.count_sequence = sequence;
    }

    pub fn set_irq_enabled(&mut self, enabled: bool) {
        self.irq_enabled = enabled;
    }

    pub fn is_irq_pending(&self) -> bool {
        self.irq_pending
    }

    pub fn clear_irq(&mut self) {
        self.irq_pending = false;
    }
}

impl Default for FrameCounter {
    fn default() -> Self {
        Self::new()
    }
}

/// APU emulator
pub struct APU {
    pub square1: SquareChannel,
    pub square2: SquareChannel,
    pub triangle: TriangleChannel,
    pub noise: NoiseChannel,
    pub dmc: DmcChannel,
    pub frame_counter: FrameCounter,

    // Channel enable/disable
    pub channel_enabled: [bool; 5],

    // Audio output
    pub on_audio_sample: Option<Box<dyn Fn(i32, i32) + Send + Sync>>,
    pub sample_rate: u32,
    pub master_volume: f32,

    // Sample accumulator
    pub sample_counter: u64,
    pub sample_buffer: i32,
}

impl APU {
    /// Create a new APU instance
    pub fn new(sample_rate: u32) -> Self {
        Self {
            square1: SquareChannel::new(),
            square2: SquareChannel::new(),
            triangle: TriangleChannel::new(),
            noise: NoiseChannel::new(),
            dmc: DmcChannel::new(),
            frame_counter: FrameCounter::new(),

            channel_enabled: [false; 5],

            on_audio_sample: None,
            sample_rate,
            master_volume: 1.0,

            sample_counter: 0,
            sample_buffer: 0,
        }
    }

    pub fn reset(&mut self) {
        self.square1.reset();
        self.square2.reset();
        self.triangle.reset();
        self.noise.reset();
        self.dmc.reset();
        self.frame_counter.reset();

        for ch in self.channel_enabled.iter_mut() {
            *ch = false;
        }
    }

    /// Read from APU registers
    pub fn read(&mut self, address: u16) -> u8 {
        match address {
            0x4015 => {
                // Channel enable/status
                let mut value = 0u8;
                value |= if self.square1.length_counter > 0 { 0x01 } else { 0 };
                value |= if self.square2.length_counter > 0 { 0x02 } else { 0 };
                value |= if self.triangle.length_counter > 0 { 0x04 } else { 0 };
                value |= if self.noise.length_counter > 0 { 0x08 } else { 0 };
                value |= if self.dmc.sample_length_counter > 0 { 0x10 } else { 0 };
                value |= if self.frame_counter.is_irq_pending() { 0x40 } else { 0 };
                value |= if self.frame_counter.irq_enabled { 0x80 } else { 0 };
                value
            }
            0x4017 => {
                // Frame counter
                let value = self.frame_counter.get_step() as u8;
                self.frame_counter.clear_irq();
                value
            }
            _ => 0,
        }
    }

    /// Write to APU registers
    pub fn write(&mut self, address: u16, value: u8) {
        match address {
            0x4000 => self.square1.set_ctrl(value),
            0x4001 => self.square1.set_sweep(value),
            0x4002 => self.square1.set_freq_low(value),
            0x4003 => self.square1.set_freq_high(value),
            0x4004 => self.square2.set_ctrl(value),
            0x4005 => self.square2.set_sweep(value),
            0x4006 => self.square2.set_freq_low(value),
            0x4007 => self.square2.set_freq_high(value),
            0x4008 => self.triangle.set_ctrl(value),
            0x400A => self.triangle.set_freq_low(value),
            0x400B => self.triangle.set_freq_high(value),
            0x400C => self.noise.set_ctrl(value),
            0x400E => self.noise.set_freq(value),
            0x400F => self.noise.set_length(value),
            0x4010 => self.dmc.set_ctrl(value),
            0x4011 => self.dmc.set_dac(value),
            0x4012 => self.dmc.set_address(value),
            0x4013 => self.dmc.set_length(value),
            0x4015 => {
                // Channel enable
                self.channel_enabled[0] = (value & 0x01) != 0;
                self.channel_enabled[1] = (value & 0x02) != 0;
                self.channel_enabled[2] = (value & 0x04) != 0;
                self.channel_enabled[3] = (value & 0x08) != 0;
                self.channel_enabled[4] = (value & 0x10) != 0;

                self.square1.set_enabled(self.channel_enabled[0]);
                self.square2.set_enabled(self.channel_enabled[1]);
                self.triangle.set_enabled(self.channel_enabled[2]);
                self.noise.set_enabled(self.channel_enabled[3]);
                self.dmc.set_enabled(self.channel_enabled[4]);

                // Start DMC if enabled
                if (value & 0x10) != 0 && self.dmc.sample_length_counter == 0 {
                    self.dmc.start_sample();
                }

                // Reset DMC length if bit 4 is set
                if (value & 0x20) != 0 {
                    self.dmc.sample_length_counter = self.dmc.sample_length;
                }
            }
            0x4017 => {
                // Frame counter
                self.frame_counter.set_sequence((value >> 7) & 0x01);
                self.frame_counter.set_irq_enabled((value & 0x40) != 0);
            }
            _ => {}
        }
    }

    /// Clock frame counter
    pub fn clock_frame_counter(&mut self, cycles: u64) {
        self.frame_counter.clock(cycles);
    }

    /// Clock quarter frame operations
    pub fn clock_quarter_frame(&mut self) {
        self.square1.clock_envelope();
        self.square2.clock_envelope();
        self.triangle.clock_linear();
        self.noise.clock_envelope();
    }

    /// Clock half frame operations
    pub fn clock_half_frame(&mut self) {
        self.square1.clock_length();
        self.square2.clock_length();
        self.triangle.clock_length();
        self.noise.clock_length();

        self.square1.clock_sweep();
        self.square2.clock_sweep();
    }

    /// Clock frame steps based on sequence
    pub fn clock_frame_step(&mut self) {
        let step = self.frame_counter.get_step();

        match self.frame_counter.count_sequence {
            0 => {
                // 4-step sequence
                match step {
                    0 => self.clock_quarter_frame(),
                    1 => self.clock_half_frame(),
                    2 => self.clock_quarter_frame(),
                    3 => self.clock_half_frame(),
                    _ => {}
                }
            }
            1 => {
                // 5-step sequence
                match step {
                    0 => self.clock_quarter_frame(),
                    1 => self.clock_half_frame(),
                    2 => self.clock_quarter_frame(),
                    3 => self.clock_half_frame(),
                    4 => {
                        self.clock_quarter_frame();
                        self.clock_half_frame();
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    /// Update all channels
    pub fn update_channels(&mut self) {
        if self.channel_enabled[0] {
            self.square1.update_output();
        }
        if self.channel_enabled[1] {
            self.square2.update_output();
        }
        if self.channel_enabled[2] {
            self.triangle.update_output();
        }
        if self.channel_enabled[3] {
            self.noise.update_output();
        }
        if self.channel_enabled[4] {
            self.dmc.clock_sample();
        }
    }

    /// Calculate output sample
    pub fn get_output(&self) -> (i32, i32) {
        let sq1 = self.square1.get_output();
        let sq2 = self.square2.get_output();
        let tri = self.triangle.get_output();
        let noise = self.noise.get_output();
        let dmc = self.dmc.get_output();

        // Simple mixing (real APU has more complex DAC tables)
        let left = (sq1 + sq2 + tri + noise + dmc) as i32;
        let right = left;  // Mono output for simplicity

        (left, right)
    }

    /// Generate audio sample
    pub fn generate_sample(&mut self) -> Option<(i32, i32)> {
        self.update_channels();

        let (left, right) = self.get_output();

        // Apply volume
        let left = (left as f32 * self.master_volume) as i32;
        let right = (right as f32 * self.master_volume) as i32;

        // Call callback if registered
        if let Some(callback) = &self.on_audio_sample {
            callback(left, right);
        }

        Some((left, right))
    }
}

impl Default for APU {
    fn default() -> Self {
        Self::new(44100)
    }
}