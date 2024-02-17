use crate::error::GgError;

pub(crate) trait Controller {
    fn read_io(&mut self, port: u8) -> Result<u8, GgError>; // in
    fn write_io(&mut self, port: u8, value: u8) -> Result<(), GgError>; // out
}
