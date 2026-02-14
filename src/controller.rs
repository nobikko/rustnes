//! Controller input handling

/// Button constants
pub const BUTTON_A: u8 = 0;
pub const BUTTON_B: u8 = 1;
pub const BUTTON_SELECT: u8 = 2;
pub const BUTTON_START: u8 = 3;
pub const BUTTON_UP: u8 = 4;
pub const BUTTON_DOWN: u8 = 5;
pub const BUTTON_LEFT: u8 = 6;
pub const BUTTON_RIGHT: u8 = 7;

/// Button states
pub const BUTTON_UP_STATE: u8 = 0x40;
pub const BUTTON_DOWN_STATE: u8 = 0x41;

/// Controller type
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ControllerType {
    Standard = 0,
    Zapper = 1,
    PowerPaddle = 2,
    Ardundrum = 3,
}

/// Standard NES controller
#[derive(Debug)]
pub struct StandardController {
    pub buttons: [u8; 8],
    pub strobe: bool,
    pub strobe_state: u8,
}

impl StandardController {
    pub fn new() -> Self {
        let mut buttons = [BUTTON_UP_STATE; 8];
        buttons[BUTTON_A as usize] = BUTTON_UP_STATE;
        buttons[BUTTON_B as usize] = BUTTON_UP_STATE;
        buttons[BUTTON_SELECT as usize] = BUTTON_UP_STATE;
        buttons[BUTTON_START as usize] = BUTTON_UP_STATE;
        buttons[BUTTON_UP as usize] = BUTTON_UP_STATE;
        buttons[BUTTON_DOWN as usize] = BUTTON_UP_STATE;
        buttons[BUTTON_LEFT as usize] = BUTTON_UP_STATE;
        buttons[BUTTON_RIGHT as usize] = BUTTON_UP_STATE;

        Self {
            buttons,
            strobe: false,
            strobe_state: 0,
        }
    }

    pub fn button_down(&mut self, button: u8) {
        if (button as usize) < 8 {
            self.buttons[button as usize] = BUTTON_DOWN_STATE;
        }
    }

    pub fn button_up(&mut self, button: u8) {
        if (button as usize) < 8 {
            self.buttons[button as usize] = BUTTON_UP_STATE;
        }
    }

    pub fn strobe_write(&mut self, value: u8) {
        if (value & 0x01) != 0 {
            // Strobe enabled
            self.strobe = true;
            self.strobe_state = 0;
        } else {
            // Strobe disabled
            self.strobe = false;
        }
    }

    pub fn read(&mut self) -> u8 {
        if self.strobe {
            // Return button state when strobe is active
            self.buttons[0]
        } else {
            // Shift register mode
            let value = self.buttons[self.strobe_state as usize];
            self.strobe_state = (self.strobe_state + 1) & 0x07;
            value
        }
    }
}

impl Default for StandardController {
    fn default() -> Self {
        Self::new()
    }
}

/// Zapper light gun
#[derive(Debug)]
pub struct ZapperController {
    pub x: u8,
    pub y: u8,
    pub trigger: bool,
    pub light_sensor: bool,
    pub strobe_state: u8,
}

impl ZapperController {
    pub fn new() -> Self {
        Self {
            x: 0,
            y: 0,
            trigger: false,
            light_sensor: true,  // Default to non-white
            strobe_state: 0,
        }
    }

    pub fn set_position(&mut self, x: u8, y: u8) {
        self.x = x;
        self.y = y;
    }

    pub fn trigger_down(&mut self) {
        self.trigger = true;
    }

    pub fn trigger_up(&mut self) {
        self.trigger = false;
    }

    pub fn set_light_state(&mut self, is_white: bool) {
        self.light_sensor = !is_white;  // 0 = white, 1 = non-white
    }

    pub fn strobe_write(&mut self, value: u8) {
        if (value & 0x01) != 0 {
            self.strobe_state = 0;
        }
    }

    pub fn read(&self) -> u8 {
        let mut value = if self.light_sensor { 0x10 } else { 0 };
        if self.trigger {
            value |= 0x08;
        }
        value |= self.strobe_state;
        value
    }
}

impl Default for ZapperController {
    fn default() -> Self {
        Self::new()
    }
}

/// Controller ports
#[derive(Debug)]
pub struct ControllerPorts {
    pub port1: StandardController,
    pub port2: StandardController,
    pub port1_type: ControllerType,
    pub port2_type: ControllerType,
}

impl ControllerPorts {
    pub fn new() -> Self {
        Self {
            port1: StandardController::new(),
            port2: StandardController::new(),
            port1_type: ControllerType::Standard,
            port2_type: ControllerType::Standard,
        }
    }

    pub fn strobe1_write(&mut self, value: u8) {
        self.port1.strobe_write(value);
    }

    pub fn strobe2_write(&mut self, value: u8) {
        self.port2.strobe_write(value);
    }

    pub fn read1(&mut self) -> u8 {
        self.port1.read()
    }

    pub fn read2(&mut self) -> u8 {
        self.port2.read()
    }

    pub fn button1_down(&mut self, button: u8) {
        self.port1.button_down(button);
    }

    pub fn button1_up(&mut self, button: u8) {
        self.port1.button_up(button);
    }

    pub fn button2_down(&mut self, button: u8) {
        self.port2.button_down(button);
    }

    pub fn button2_up(&mut self, button: u8) {
        self.port2.button_up(button);
    }

    pub fn set_controller_type(&mut self, port: u8, ty: ControllerType) {
        match port {
            1 => self.port1_type = ty,
            2 => self.port2_type = ty,
            _ => {}
        }
    }
}

impl Default for ControllerPorts {
    fn default() -> Self {
        Self::new()
    }
}