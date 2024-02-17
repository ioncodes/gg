use log::debug;

use crate::error::GgError;
use crate::io;

pub(crate) const JOYSTICK_START_PORT: u8 = 0x00;
pub(crate) const JOYSTICK_AB_PORT: u8 = 0xdc;
pub(crate) const JOYSTICK_B_MISC_PORT: u8 = 0xdd;

pub const JOYSTICK_A_UP_MASK: u8 = 0b0000_0001;
pub const JOYSTICK_A_DOWN_MASK: u8 = 0b0000_0010;
pub const JOYSTICK_A_LEFT_MASK: u8 = 0b0000_0100;
pub const JOYSTICK_A_RIGHT_MASK: u8 = 0b0000_1000;
pub const JOYSTICK_A_BUTTON1_MASK: u8 = 0b0001_0000;
pub const JOYSTICK_A_BUTTON2_MASK: u8 = 0b0010_0000;
pub const JOYSTICK_A_START_MASK: u8 = 0b1000_0000;

#[derive(PartialEq)]
pub enum JoystickPort {
    Player1,
    Player2,
}

pub struct Joystick {
    pub(crate) port: JoystickPort,
    input: u8,
    start: bool,
}

impl Joystick {
    pub fn new(port: JoystickPort) -> Joystick {
        Joystick {
            port,
            input: 0b1111_1111,
            start: false,
        }
    }

    pub fn set_input_up(&mut self, up: bool) {
        self.input = if up {
            self.input & !JOYSTICK_A_UP_MASK
        } else {
            self.input | JOYSTICK_A_UP_MASK
        };
    }

    pub fn set_input_down(&mut self, down: bool) {
        self.input = if down {
            self.input & !JOYSTICK_A_DOWN_MASK
        } else {
            self.input | JOYSTICK_A_DOWN_MASK
        };
    }

    pub fn set_input_left(&mut self, left: bool) {
        self.input = if left {
            self.input & !JOYSTICK_A_LEFT_MASK
        } else {
            self.input | JOYSTICK_A_LEFT_MASK
        };
    }

    pub fn set_input_right(&mut self, right: bool) {
        self.input = if right {
            self.input & !JOYSTICK_A_RIGHT_MASK
        } else {
            self.input | JOYSTICK_A_RIGHT_MASK
        };
    }

    pub fn set_input_button1(&mut self, button1: bool) {
        self.input = if button1 {
            self.input & !JOYSTICK_A_BUTTON1_MASK
        } else {
            self.input | JOYSTICK_A_BUTTON1_MASK
        };
    }

    pub fn set_input_button2(&mut self, button2: bool) {
        self.input = if button2 {
            self.input & !JOYSTICK_A_BUTTON2_MASK
        } else {
            self.input | JOYSTICK_A_BUTTON2_MASK
        };
    }

    pub fn set_start(&mut self, start: bool) {
        self.start = start;
    }
}

impl io::Controller for Joystick {
    fn read_io(&mut self, port: u8) -> Result<u8, GgError> {
        match port {
            JOYSTICK_AB_PORT => {
                if self.port == JoystickPort::Player1 {
                    debug!("Reading joystick A/B port: {:08b}", self.input);
                    return Ok(self.input);
                }

                return Err(GgError::IoControllerInvalidPort);
            }
            JOYSTICK_B_MISC_PORT => {
                if self.port == JoystickPort::Player2 {
                    debug!("Reading joystick B/misc port: {:08b}", self.input);
                    return Ok(self.input);
                }

                return Err(GgError::IoControllerInvalidPort);
            }
            JOYSTICK_START_PORT => {
                debug!("Reading START port: {}", self.start);
                return Ok(if self.start { 0b0111_1111 } else { 0b1111_1111 });
            }
            _ => {}
        }

        Err(GgError::IoControllerInvalidPort)
    }

    fn write_io(&mut self, _port: u8, _value: u8) -> Result<(), GgError> {
        Err(GgError::IoControllerInvalidPort)
    }
}
