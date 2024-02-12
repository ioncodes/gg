use crate::{error::GgError, io};

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
            (0xdd, JoystickPort::Player2) => return Ok(0b1111_1111),
            (0xdc, JoystickPort::Player1) => return Ok(0b1111_1111),
            _ => {}
        }

        Err(GgError::IoControllerInvalidPort)
    }

    fn write_io(&mut self, _port: u8, _value: u8) -> Result<(), GgError> {
        Err(GgError::IoControllerInvalidPort)
    }
}
