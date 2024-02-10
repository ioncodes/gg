use crate::{error::GgError, io};

#[derive(PartialEq)]
pub enum ControllerPort {
    Player1,
    Player2,
}

pub struct Controller {
    pub(crate) port: ControllerPort,
}

impl Controller {
    pub fn new(port: ControllerPort) -> Controller {
        Controller { port }
    }
}

impl io::Controller for Controller {
    fn read_io(&self, port: u8) -> Result<u8, GgError> {
        match (port, &self.port) {
            (0xdd, ControllerPort::Player2) => return Ok(0x00),
            (0xdc, ControllerPort::Player1) => return Ok(0x00),
            _ => {}
        }

        Err(GgError::IoControllerInvalidPort)
    }

    fn write_io(&mut self, _port: u8, _value: u8) -> Result<(), GgError> {
        Err(GgError::IoControllerInvalidPort)
    }
}
