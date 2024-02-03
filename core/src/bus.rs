use crate::error::GgError;
use crate::io::{IoBus, IoMode};
use crate::memory::Memory;

pub(crate) struct Bus {
    pub(crate) rom: Memory,      // 0x0000 - 0xbfff
    pub(crate) ram: Memory,      // 0xc000 - 0xffff
    pub(crate) bios_rom: Memory, // only for BIOS. enabled on startup, disabled by end of BIOS
    pub(crate) bios_enabled: bool,
    pub(crate) io: IoBus,
}

impl Bus {
    pub(crate) fn new() -> Bus {
        Bus {
            rom: Memory::new(0x1024 * 16, 0x0000),
            ram: Memory::new(0x1024 * 16, 0xc000),
            bios_rom: Memory::new(0x400, 0x0000),
            bios_enabled: true,
            io: IoBus::new(),
        }
    }

    pub(crate) fn process_io(&mut self) {
        /*
            Port $3E : Memory control
            D7 : Expansion slot enable (1= disabled, 0= enabled)
            D6 : Cartridge slot enable (1= disabled, 0= enabled)
            D5 : Card slot disabled (1= disabled, 0= enabled)
            D4 : Work RAM disabled (1= disabled, 0= enabled)
            D3 : BIOS ROM disabled (1= disabled, 0= enabled)
            D2 : I/O chip disabled (1= disabled, 0= enabled)
            D1 : Unknown
            D0 : Unknown
         */
        
        if self.io.has_pending(0x3e, IoMode::Write) {
            let value = self.io.pop(0x3e, false).unwrap();
            self.bios_enabled = value & 0b0000_1000 == 0;
        }
    }

    pub(crate) fn push_io_data(&mut self, port: u8, value: u8, mode: IoMode, is_answer: bool) {
        self.io.push(port, value, mode, is_answer);
    }

    pub(crate) fn pop_io_data(&mut self, port: u8, expects_answer: bool) -> Option<u8> {
        self.io.pop(port, expects_answer)
    }

    #[allow(unused_comparisons)]
    pub(crate) fn read(&self, mut address: u16) -> Result<u8, GgError> {
        if address == 0xfffc || address == 0xfffd || address == 0xfffe || address == 0xffff {
            address = address - 0xe000;
        }

        if self.bios_enabled && address >= 0x0000 && address < 0x0400 {
            return Ok(self.bios_rom.read(address));
        }

        if address >= 0x0000 && address < 0xc000 {
            return Ok(self.rom.read(address));
        }

        if address >= 0xc000 && address <= 0xffff {
            return Ok(self.ram.read(address));
        }

        Err(GgError::BusRequestOutOfBounds { address })
    }

    #[allow(unused_comparisons)]
    pub(crate) fn read_word(&self, mut address: u16) -> Result<u16, GgError> {
        if address == 0xfffc || address == 0xfffd || address == 0xfffe || address == 0xffff {
            address = address - 0xe000;
        }

        if self.bios_enabled && address >= 0x0000 && address < 0x0400 {
            return Ok(self.bios_rom.read_word(address));
        }

        if address >= 0x0000 && address < 0xc000 {
            return Ok(self.rom.read_word(address));
        }

        if address >= 0xc000 && address <= 0xffff {
            return Ok(self.ram.read_word(address));
        }

        Err(GgError::BusRequestOutOfBounds { address })
    }

    #[allow(unused_comparisons)]
    pub(crate) fn write(&mut self, mut address: u16, value: u8) -> Result<(), GgError> {
        if address == 0xfffc || address == 0xfffd || address == 0xfffe || address == 0xffff {
            address = address - 0xe000;
        }

        if self.bios_enabled && address >= 0x0000 && address < 0x0400 {
            return Ok(self.bios_rom.write(address, value));
        }

        if address >= 0x0000 && address < 0xc000 {
            return Ok(self.rom.write(address, value));
        }

        if address >= 0xc000 && address <= 0xffff {
            return Ok(self.ram.write(address, value));
        }

        Err(GgError::BusRequestOutOfBounds { address })
    }

    #[allow(unused_comparisons)]
    pub(crate) fn write_word(&mut self, mut address: u16, value: u16) -> Result<(), GgError> {
        if address == 0xfffc || address == 0xfffd || address == 0xfffe || address == 0xffff {
            address = address - 0xe000;
        }

        if self.bios_enabled && address >= 0x0000 && address < 0x0400 {
            return Ok(self.bios_rom.write_word(address, value));
        }

        if address >= 0x0000 && address < 0xc000 {
            return Ok(self.rom.write_word(address, value));
        }

        if address >= 0xc000 && address <= 0xffff {
            return Ok(self.ram.write_word(address, value));
        }

        Err(GgError::BusRequestOutOfBounds { address })
    }
}
