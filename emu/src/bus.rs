use crate::error::GgError;
use crate::io::Controller;
use crate::mapper::Mapper;
use crate::memory::Memory;

pub(crate) const MEMORY_CONTROL_PORT: u8 = 0x3e;
//pub(crate) const MEMORY_REGISTER_RAM_MAPPING: u16 = 0xfffc;
pub const MEMORY_REGISTER_CR_BANK_SELECT_0: u16 = 0xfffd;
pub const MEMORY_REGISTER_CR_BANK_SELECT_1: u16 = 0xfffe;
pub const MEMORY_REGISTER_CR_BANK_SELECT_2: u16 = 0xffff;


pub struct Bus {
    pub rom: Box<dyn Mapper>,  // 0x0000 - 0xbfff
    pub ram: Memory<u16>,      // 0xc000 - 0xffff
    pub bios_rom: Memory<u16>, // Only for BIOS. Enabled on startup, disabled by end of BIOS
    pub bios_enabled: bool,    // BIOS is enabled by default
    gear_to_gear_cache: Option<u8> // Cache for Gear to Gear communication (ports 0..6)
}

impl Bus {
    pub(crate) fn new(rom: impl Mapper + 'static) -> Bus {
        Bus {
            rom: Box::new(rom),
            ram: Memory::new(0x1024 * 16, 0x0000), /* changed from 0xc000 */
            bios_rom: Memory::new(0x400, 0x0000),
            bios_enabled: true,
            gear_to_gear_cache: None
        }
    }

    #[allow(unused_comparisons)]
    pub fn read(&self, address: u16) -> Result<u8, GgError> {
        if self.bios_enabled && address >= 0x0000 && address < 0x0400 {
            return Ok(self.bios_rom.read(address));
        }

        if address >= 0x0000 && address < 0x4000 {
            let bank = self.read(MEMORY_REGISTER_CR_BANK_SELECT_0)? as usize;
            return Ok(self.rom.read_from_bank(bank, address));
        }

        if address >= 0x4000 && address < 0x8000 {
            let bank = self.read(MEMORY_REGISTER_CR_BANK_SELECT_1)? as usize;
            return Ok(self.rom.read_from_bank(bank, address - 0x4000));
        }

        if address >= 0x8000 && address < 0xc000 {
            let bank = self.read(MEMORY_REGISTER_CR_BANK_SELECT_2)? as usize;
            return Ok(self.rom.read_from_bank(bank, address - 0x8000));
        }

        if address >= 0xc000 && address <= 0xffff {
            return Ok(self.ram.read(address));
        }

        Err(GgError::BusRequestOutOfBounds { address: address as usize })
    }

    #[allow(unused_comparisons)]
    pub fn read_word(&self, address: u16) -> Result<u16, GgError> {
        if self.bios_enabled && address >= 0x0000 && address < 0x0400 {
            return Ok(self.bios_rom.read_word(address));
        }

        if address >= 0x0000 && address < 0x4000 {
            let bank = self.read(MEMORY_REGISTER_CR_BANK_SELECT_0)? as usize;
            return Ok(self.rom.read_word_from_bank(bank, address));
        }

        if address >= 0x4000 && address < 0x8000 {
            let bank = self.read(MEMORY_REGISTER_CR_BANK_SELECT_1)? as usize;
            return Ok(self.rom.read_word_from_bank(bank, address - 0x4000));
        }

        if address >= 0x8000 && address < 0xc000 {
            let bank = self.read(MEMORY_REGISTER_CR_BANK_SELECT_2)? as usize;
            return Ok(self.rom.read_word_from_bank(bank, address - 0x8000));
        }

        if address >= 0xc000 && address <= 0xffff {
            return Ok(self.ram.read_word(address));
        }

        Err(GgError::BusRequestOutOfBounds { address: address as usize })
    }

    #[allow(unused_comparisons)]
    pub fn write(&mut self, address: u16, value: u8) -> Result<(), GgError> {
        if self.bios_enabled && address >= 0x0000 && address < 0x0400 {
            return Ok(self.bios_rom.write(address, value));
        }

        if address >= 0x0000 && address < 0x4000 {
            let bank = self.read(MEMORY_REGISTER_CR_BANK_SELECT_0)? as usize;
            return Ok(self.rom.write_to_bank(bank, address, value));
        }

        if address >= 0x4000 && address < 0x8000 {
            let bank = self.read(MEMORY_REGISTER_CR_BANK_SELECT_1)? as usize;
            return Ok(self.rom.write_to_bank(bank, address - 0x4000, value));
        }

        if address >= 0x8000 && address < 0xc000 {
            let bank = self.read(MEMORY_REGISTER_CR_BANK_SELECT_2)? as usize;
            return Ok(self.rom.write_to_bank(bank, address - 0x8000, value));
        }

        if address >= 0xc000 && address <= 0xffff {
            return Ok(self.ram.write(address, value));
        }

        Err(GgError::BusRequestOutOfBounds { address: address as usize })
    }

    #[allow(unused_comparisons)]
    pub fn write_word(&mut self, address: u16, value: u16) -> Result<(), GgError> {
        if self.bios_enabled && address >= 0x0000 && address < 0x0400 {
            return Ok(self.bios_rom.write_word(address, value));
        }

        if address >= 0x0000 && address < 0x4000 {
            let bank = self.read(MEMORY_REGISTER_CR_BANK_SELECT_0)? as usize;
            return Ok(self.rom.write_word_to_bank(bank, address, value));
        }

        if address >= 0x4000 && address < 0x8000 {
            let bank = self.read(MEMORY_REGISTER_CR_BANK_SELECT_1)? as usize;
            return Ok(self.rom.write_word_to_bank(bank, address - 0x4000, value));
        }

        if address >= 0x8000 && address < 0xc000 {
            let bank = self.read(MEMORY_REGISTER_CR_BANK_SELECT_2)? as usize;
            return Ok(self.rom.write_word_to_bank(bank, address - 0x8000, value));
        }

        if address >= 0xc000 && address <= 0xffff {
            return Ok(self.ram.write_word(address, value));
        }

        Err(GgError::BusRequestOutOfBounds { address: address as usize })
    }

    pub(crate) fn write_passthrough(&mut self, address: usize, value: u8) -> Result<(), GgError> {
        if self.bios_enabled {
            self.bios_rom.write(address as u16, value)
        } else {
            self.rom.write(address, value)
        }

        Ok(())
    }

    /// Translate a 16-bit CPU address to a 32-bit ROM address 
    #[allow(unused_comparisons)]
    pub fn translate_address_to_real(&self, address: u16) -> Result<usize, GgError> {
        let address = address as usize;

        if address >= 0x0000 && address < 0x4000 {
            let bank = self.read(MEMORY_REGISTER_CR_BANK_SELECT_0)? as usize;
            return Ok(bank * 0x4000 + address);
        }

        if address >= 0x4000 && address < 0x8000 {
            let bank = self.read(MEMORY_REGISTER_CR_BANK_SELECT_1)? as usize;
            return Ok(bank * 0x4000 + address - 0x4000);
        }

        if address >= 0x8000 && address < 0xc000 {
            let bank = self.read(MEMORY_REGISTER_CR_BANK_SELECT_2)? as usize;
            return Ok(bank * 0x4000 + address - 0x8000);
        }

        Err(GgError::BusRequestOutOfBounds { address })
    }
}

impl Controller for Bus {
    fn read_io(&self, port: u8) -> Result<u8, GgError> {
        match port {
            0x00..=0x06 => {
                if let Some(value) = self.gear_to_gear_cache {
                    return Ok(value);
                } else {
                    return Err(GgError::IoRequestNotFulfilled);
                }
            },
            _ => return Err(GgError::IoControllerInvalidPort)
        }
    }

    fn write_io(&mut self, port: u8, value: u8) -> Result<(), GgError> {
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
        
        match port {
            0x00..=0x06 => self.gear_to_gear_cache = Some(value),
            MEMORY_CONTROL_PORT => self.bios_enabled = value & 0b0000_1000 == 0,
            _ => return Err(GgError::IoControllerInvalidPort)
        }

        Ok(())
    }
}