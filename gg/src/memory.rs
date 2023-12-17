pub(crate) struct Memory {
    pub(crate) buffer: Vec<u8>,
    pub(crate) base_address: u16,
}

impl Memory {
    pub(crate) fn new(size: usize, base_address: u16) -> Memory {
        Memory {
            buffer: vec![0; size],
            base_address,
        }
    }

    pub(crate) fn read(&self, address: u16) -> u8 {
        self.buffer[(address - self.base_address) as usize]
    }

    pub(crate) fn read_word(&self, address: u16) -> u16 {
        let low = self.read(address);
        let high = self.read(address + 1);
        ((low as u16) << 8) | (high as u16)
    }

    pub(crate) fn write(&mut self, address: u16, value: u8) {
        self.buffer[(address - self.base_address) as usize] = value;
    }

    pub(crate) fn write_word(&mut self, address: u16, value: u16) {
        let low = (value & 0xff) as u8;
        let high = ((value >> 8) & 0xff) as u8;
        self.write(address, high);
        self.write(address + 1, low);
    }

    pub(crate) fn copy(&mut self, address: u16, data: &[u8]) {
        for i in 0..data.len() {
            self.write(address + i as u16, data[i]);
        }
    }
}
