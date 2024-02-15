use log::info;

use crate::{error::GgError, io::Controller};

pub(crate) struct DebugConsole;

impl Controller for DebugConsole {
    fn read_io(&self, _port: u8) -> Result<u8, GgError> {
        Err(GgError::IoControllerInvalidPort)
    }

    fn write_io(&mut self, port: u8, value: u8) -> Result<(), GgError> {
        match port {
            0xfc => info!("Debug console (FC): {}", value),
            0xfd => info!("Debug console (FD): {}", value),
            _ => return Err(GgError::IoControllerInvalidPort),
        }

        Ok(())
    }
}
