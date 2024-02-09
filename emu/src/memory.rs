use std::ops::*;

pub struct Memory<T> {
    pub buffer: Vec<u8>,
    pub(crate) base_address: T,
}

impl<T> Memory<T> 
where T: Add<Output = T> + Sub<Output = T> + Into<usize> + From<u16> + Copy
{
    pub(crate) fn new(size: usize, base_address: T) -> Memory<T> {
        Memory {
            buffer: vec![0; size],
            base_address,
        }
    }

    pub fn read(&self, address: T) -> u8 {
        self.buffer[(address - self.base_address).into()]
    }

    pub fn read_word(&self, address: T) -> u16 {
        let low = self.read(address);
        let high = self.read(address + 1.into());
        ((high as u16) << 8) | (low as u16)
    }

    pub fn write(&mut self, address: T, value: u8) {
        self.buffer[(address - self.base_address).into()] = value;
    }

    pub fn write_word(&mut self, address: T, value: u16) {
        let low = (value & 0xff) as u8;
        let high = ((value >> 8) & 0xff) as u8;
        self.write(address, low);
        self.write(address + 1.into(), high);
    }

    pub(crate) fn resize(&mut self, size: usize) {
        self.buffer.resize(size, 0);
    }
}
