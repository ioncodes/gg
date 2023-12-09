use crate::memory::Memory;

pub(crate) struct IoRequest {
    pub(crate) port: u8,
    pub(crate) value: u8
}

// I/O: https://www.smspower.org/uploads/Development/smstech-20021112.txt

pub(crate) struct Bus {
    rom: Memory, // 0x0000 - 0xbfff
    ram: Memory, // 0xc000 - 0xffff
    io: Option<IoRequest>
}

impl Bus {
    pub(crate) fn new() -> Bus {
        Bus { 
            rom: Memory::new(0x1024 * 16, 0x0000),
            ram: Memory::new(0x1024 * 16, 0xc000),
            io: None
        }
    }

    pub(crate) fn push_io_request(&mut self, port: u8, value: u8) {
        self.io = Some(IoRequest { port, value });
    }

    pub(crate) fn pop_io_request(&mut self, port: u8) -> Option<u8> {
        let answer = self.io.take();
        if let Some(ref request) = answer {
            if request.port == port {
                return Some(request.value);
            }
        }
        None
    }

    // todo: bank mappers
    pub(crate) fn read(&self, mut address: u16) -> Result<u8, &str> {
        if address == 0xfffc || address == 0xfffd || address == 0xfffe || address == 0xffff {
            address = address - 0xe000;
        }

        if address >= 0x0000 && address < 0xc000 {
            return Ok(self.rom.read(address));
        }

        if address >= 0xc000 && address <= 0xffff {
            return Ok(self.ram.read(address));
        }

        Err("out of bounds")
    }

    pub(crate) fn read_word(&self, mut address: u16) -> Result<u16, &str> {
        if address == 0xfffc || address == 0xfffd || address == 0xfffe || address == 0xffff {
            address = address - 0xe000;
        }

        if address >= 0x0000 && address < 0xc000 {
            return Ok(self.rom.read_word(address));
        }

        if address >= 0xc000 && address <= 0xffff {
            return Ok(self.ram.read_word(address));
        }

        Err("out of bounds")
    }

    pub(crate) fn write(&mut self, mut address: u16, value: u8) -> Result<(), &str> {
        if address == 0xfffc || address == 0xfffd || address == 0xfffe || address == 0xffff {
            address = address - 0xe000;
        }

        if address >= 0x0000 && address < 0xc000 {
            return Ok(self.rom.write(address, value));
        }

        if address >= 0xc000 && address <= 0xffff {
            return Ok(self.ram.write(address, value));
        }

        Err("out of bounds")
    }

    pub(crate) fn write_word(&mut self, mut address: u16, value: u16) -> Result<(), &str> {
        if address == 0xfffc || address == 0xfffd || address == 0xfffe || address == 0xffff {
            address = address - 0xe000;
        }

        if address >= 0x0000 && address < 0xc000 {
            return Ok(self.rom.write_word(address, value));
        }

        if address >= 0xc000 && address <= 0xffff {
            return Ok(self.ram.write_word(address, value));
        }

        Err("out of bounds")
    }
}
