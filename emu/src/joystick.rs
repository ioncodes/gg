use crate::error::GgError;
use crate::io;

pub(crate) const JOYSTICK_AB_PORT: u8 = 0xdc;
pub(crate) const JOYSTICK_B_MISC_PORT: u8 = 0xdd;

#[derive(PartialEq)]
pub enum JoystickPort {
    Player1,
    Player2,
}

pub struct Joystick {
    pub(crate) port: JoystickPort,
}

impl Joystick {
    pub fn new(port: JoystickPort) -> Joystick {
        Joystick { port }
    }
}

impl io::Controller for Joystick {
    fn read_io(&self, port: u8) -> Result<u8, GgError> {
        match (port, &self.port) {
            (JOYSTICK_AB_PORT, JoystickPort::Player1) => return Ok(0b1111_1110),
            (JOYSTICK_B_MISC_PORT, JoystickPort::Player2) => return Ok(0b1111_1110),
            _ => {}
        }

        Err(GgError::IoControllerInvalidPort)
    }

    fn write_io(&mut self, _port: u8, _value: u8) -> Result<(), GgError> {
        Err(GgError::IoControllerInvalidPort)
    }
}
