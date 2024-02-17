use log::debug;

use crate::error::GgError;
use crate::io::Controller;

pub(crate) const CONTROL_PORT: u8 = 0xfc;
pub(crate) const DATA_PORT: u8 = 0xfd;

pub struct DebugConsole {
    pub buffer: String,
}

impl DebugConsole {
    pub fn new() -> DebugConsole {
        DebugConsole { buffer: String::new() }
    }
}

impl Controller for DebugConsole {
    fn read_io(&mut self, _port: u8) -> Result<u8, GgError> {
        Err(GgError::IoControllerInvalidPort)
    }

    fn write_io(&mut self, port: u8, value: u8) -> Result<(), GgError> {
        match port {
            CONTROL_PORT => {
                debug!("Debug console I/O control received: {}", value);
                if value == 0x02 {
                    self.buffer.clear();
                }
            }
            DATA_PORT => {
                debug!("Debug console I/O data received: {}", value);
                self.buffer.push(value as char);
            }
            _ => return Err(GgError::IoControllerInvalidPort),
        }

        Ok(())
    }
}
