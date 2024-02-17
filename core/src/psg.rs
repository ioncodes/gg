use crate::error::GgError;
use crate::io::Controller;

pub struct Psg {}

impl Psg {
    pub(crate) fn new() -> Psg {
        Psg {}
    }
}

impl Controller for Psg {
    fn read_io(&mut self, _port: u8) -> Result<u8, GgError> {
        Ok(0)
    }

    fn write_io(&mut self, _port: u8, _value: u8) -> Result<(), GgError> {
        Ok(())
    }
}
