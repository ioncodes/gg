use log::{error, warn};

use crate::error::GgError;
use crate::io::Controller;
use crate::joystick::{self, Joystick, JoystickPort};
use crate::mapper::Mapper;
use crate::memory::Memory;
use crate::sdsc::{self, DebugConsole};

pub(crate) const MEMORY_CONTROL_PORT: u8 = 0x3e;
pub(crate) const MEMORY_REGISTER_RAM_MAPPING: u16 = 0xfffc;
pub const MEMORY_REGISTER_CR_BANK_SELECT_0: u16 = 0xfffd;
pub const MEMORY_REGISTER_CR_BANK_SELECT_1: u16 = 0xfffe;
pub const MEMORY_REGISTER_CR_BANK_SELECT_2: u16 = 0xffff;

#[derive(PartialEq)]
pub enum Passthrough {
    Bios,
    Rom,
    Ram,
}

pub enum BankSelect {
    Bank0,
    Bank1,
    Bank2,
}

#[derive(PartialEq)]
pub enum RomWriteProtection {
    Abort,
    Warn,
    Allow,
}

pub struct Bus {
    pub rom: Box<dyn Mapper>,       // 0x0000 - 0xbfff
    pub ram: Memory<u16>,           // 0xc000 - 0xffff
    pub sram: Memory<u16>,          // TODO: Depends on cartridge
    pub bios_rom: Memory<u16>,      // Only for BIOS. Enabled on startup, disabled by end of BIOS
    pub bios_enabled: bool,         // BIOS is enabled by default
    gear_to_gear_cache: Option<u8>, // Cache for Gear to Gear communication (ports 0..6)
    pub joysticks: [Joystick; 2],
    joysticks_enabled: bool,
    pub sdsc_console: DebugConsole,
    rom_write_protection: RomWriteProtection, // Useful for unit tests that are not SMS/GG specific
    disable_bank_behavior: bool,              // Useful for unit tests that are not SMS/GG specific
}

impl Bus {
    pub(crate) fn new(rom: impl Mapper + 'static) -> Bus {
        Bus {
            rom: Box::new(rom),
            ram: Memory::new(0x1024 * 16, 0x0000), /* changed from 0xc000 */
            sram: Memory::new(0xffff, 0x0000),     // todo: lol
            bios_rom: Memory::new(0x400, 0x0000),
            bios_enabled: true,
            gear_to_gear_cache: None,
            joysticks: [Joystick::new(JoystickPort::Player1), Joystick::new(JoystickPort::Player2)],
            joysticks_enabled: true,
            sdsc_console: DebugConsole::new(),
            rom_write_protection: RomWriteProtection::Warn,
            disable_bank_behavior: false,
        }
    }

    #[allow(unused_comparisons)]
    pub fn read(&self, address: u16) -> Result<u8, GgError> {
        if self.bios_enabled && address >= 0x0000 && address < 0x0400 {
            return Ok(self.bios_rom.read(address));
        }

        if address >= 0x0000 && address < 0x4000 {
            let bank = if address < 0x400 { 0 } else { self.fetch_bank(BankSelect::Bank0) };
            return Ok(self.rom.read_from_bank(bank, address));
        }

        if address >= 0x4000 && address < 0x8000 {
            let bank = self.fetch_bank(BankSelect::Bank1);
            return Ok(self.rom.read_from_bank(bank, address - 0x4000));
        }

        if address >= 0x8000 && address < 0xc000 {
            let bank = self.fetch_bank(BankSelect::Bank2);

            if self.is_sram_bank_active() {
                let addr = ((bank * 0x4000) + (address - 0x8000) as usize) as u16;
                return Ok(self.sram.read(addr));
            }

            return Ok(self.rom.read_from_bank(bank, address - 0x8000));
        }

        if address >= 0xc000 && address <= 0xffff {
            return Ok(self.ram.read(address - 0xc000));
        }

        Err(GgError::BusRequestOutOfBounds { address: address as usize })
    }

    #[allow(unused_comparisons)]
    pub fn read_word(&self, address: u16) -> Result<u16, GgError> {
        let low = self.read(address)?;
        let high = self.read(address + 1)?;
        Ok((high as u16) << 8 | low as u16)
    }

    #[allow(unused_comparisons)]
    pub fn write(&mut self, address: u16, value: u8) -> Result<(), GgError> {
        if self.bios_enabled && address >= 0x0000 && address < 0x0400 {
            if self.rom_write_protection == RomWriteProtection::Abort {
                return Err(GgError::WriteToReadOnlyMemory { address: address as usize });
            }

            if self.rom_write_protection == RomWriteProtection::Warn {
                warn!("Ignored write to ROM at address {:04x}", address);
            } else {
                self.bios_rom.write(address, value);
            }

            return Ok(());
        }

        if address >= 0x0000 && address < 0x4000 {
            if self.rom_write_protection == RomWriteProtection::Abort {
                return Err(GgError::WriteToReadOnlyMemory { address: address as usize });
            }

            if self.rom_write_protection == RomWriteProtection::Warn {
                warn!("Ignored write to ROM at address {:04x}", address);
            } else {
                let bank = if address < 0x400 { 0 } else { self.fetch_bank(BankSelect::Bank0) };
                self.rom.write_to_bank(bank, address, value);
            }

            return Ok(());
        }

        if address >= 0x4000 && address < 0x8000 {
            if self.rom_write_protection == RomWriteProtection::Abort {
                return Err(GgError::WriteToReadOnlyMemory { address: address as usize });
            }

            if self.rom_write_protection == RomWriteProtection::Warn {
                warn!("Ignored write to ROM at address {:04x}", address);
            } else {
                let bank = self.fetch_bank(BankSelect::Bank1);
                self.rom.write_to_bank(bank, address - 0x4000, value);
            }

            return Ok(());
        }

        if address >= 0x8000 && address < 0xc000 {
            if self.is_sram_bank_active() {
                let bank = self.fetch_bank(BankSelect::Bank2);
                let addr = ((bank * 0x4000) + (address - 0x8000) as usize) as u16;
                return Ok(self.sram.write(addr, value));
            }

            if self.rom_write_protection == RomWriteProtection::Abort {
                return Err(GgError::WriteToReadOnlyMemory { address: address as usize });
            }

            if self.rom_write_protection == RomWriteProtection::Warn {
                warn!("Ignored write to ROM at address {:04x}", address);
            } else {
                let bank = self.fetch_bank(BankSelect::Bank2);
                self.rom.write_to_bank(bank, address - 0x8000, value);
            }

            return Ok(());
        }

        if address >= 0xc000 && address <= 0xffff {
            return Ok(self.ram.write(address - 0xc000, value));
        }

        Err(GgError::BusRequestOutOfBounds { address: address as usize })
    }

    #[allow(unused_comparisons)]
    pub fn write_word(&mut self, address: u16, value: u16) -> Result<(), GgError> {
        let low = (value & 0xff) as u8;
        let high = ((value >> 8) & 0xff) as u8;
        self.write(address, low)?;
        self.write(address + 1, high)?;

        Ok(())
    }

    pub(crate) fn write_passthrough(&mut self, destination: &Passthrough, address: usize, value: u8) {
        match destination {
            Passthrough::Bios => self.bios_rom.write(address as u16, value),
            Passthrough::Rom => self.rom.write(address, value),
            Passthrough::Ram => self.ram.write(address as u16, value),
        }
    }

    /// Translate a 16-bit CPU address to a 32-bit ROM address
    #[allow(unused_comparisons)]
    pub fn translate_address_to_real(&self, address: u16) -> Result<usize, GgError> {
        let address = address as usize;

        if address >= 0x0000 && address < 0x4000 {
            let bank = if address < 0x400 { 0 } else { self.fetch_bank(BankSelect::Bank0) };
            return Ok(bank * 0x4000 + address);
        }

        if address >= 0x4000 && address < 0x8000 {
            let bank = self.fetch_bank(BankSelect::Bank1);
            return Ok(bank * 0x4000 + address - 0x4000);
        }

        if address >= 0x8000 && address < 0xc000 {
            let bank = self.fetch_bank(BankSelect::Bank2);
            return Ok(bank * 0x4000 + address - 0x8000);
        }

        if address >= 0xc000 && address <= 0xffff {
            return Ok(address);
        }

        Err(GgError::BusRequestOutOfBounds { address })
    }

    pub fn is_sram_bank_active(&self) -> bool {
        if self.disable_bank_behavior {
            return false;
        }

        let ram_mapping = self.read(MEMORY_REGISTER_RAM_MAPPING).unwrap();
        ram_mapping & 0b0000_1000 > 0
    }

    pub fn fetch_bank(&self, bank: BankSelect) -> usize {
        if self.disable_bank_behavior {
            return match bank {
                BankSelect::Bank0 => 0,
                BankSelect::Bank1 => 1,
                BankSelect::Bank2 => 2,
            };
        }

        // Depending on the mapper revision, the number of significant bits in the bank selection register may vary -
        // for example, some revisions have 3 significant bits (supporting 8 banks, 128KB total size), others have 5 (512KB),
        // with the largest known supporting 6 bits (1MB). Some software may also set higher-order bits than those that are
        // relevant to its ROM size. Provided the ROM is a power-of-two size, these issues do not cause problems,
        // because ROM mirroring will nullify the effect.

        let bank = (match bank {
            BankSelect::Bank0 => self.read(MEMORY_REGISTER_CR_BANK_SELECT_0).unwrap(),
            BankSelect::Bank1 => self.read(MEMORY_REGISTER_CR_BANK_SELECT_1).unwrap(),
            BankSelect::Bank2 => {
                if self.is_sram_bank_active() {
                    let ram_mapping = self.read(MEMORY_REGISTER_RAM_MAPPING).unwrap();
                    if ram_mapping & 0b0000_0100 == 0 {
                        0
                    } else {
                        1
                    }
                } else {
                    self.read(MEMORY_REGISTER_CR_BANK_SELECT_2).unwrap()
                }
            }
        }) as usize;

        let rom_size = self.rom.memory().buffer.len();
        let bank = match rom_size {
            0x20000 => bank & 0b0000_0111,
            0x40000 => bank & 0b0000_1111,
            0x80000 => bank & 0b0001_1111,
            0x100000 => bank & 0b0011_1111,
            _ => {
                error!("Unsupported ROM size: {}", rom_size);
                bank
            }
        };

        bank % (rom_size / 0x4000)
    }

    pub(crate) fn powerup_reset_banks(&mut self) -> Result<(), GgError> {
        self.write(MEMORY_REGISTER_CR_BANK_SELECT_0, 0)?;
        self.write(MEMORY_REGISTER_CR_BANK_SELECT_1, 1)?;
        self.write(MEMORY_REGISTER_CR_BANK_SELECT_2, 2)?;
        Ok(())
    }

    pub fn set_rom_write_protection(&mut self, value: RomWriteProtection) {
        self.rom_write_protection = value;
    }

    pub fn disable_bank_behavior(&mut self, value: bool) {
        self.disable_bank_behavior = value;
    }
}

impl Controller for Bus {
    fn read_io(&mut self, port: u8) -> Result<u8, GgError> {
        match port {
            0x01..=0x06 => {
                if let Some(value) = self.gear_to_gear_cache {
                    Ok(value)
                } else {
                    Err(GgError::IoRequestNotFulfilled)
                }
            }
            joystick::JOYSTICK_AB_PORT => {
                if self.joysticks_enabled {
                    self.joysticks[0].read_io(port)
                } else {
                    Err(GgError::JoystickDisabled)
                }
            }
            joystick::JOYSTICK_B_MISC_PORT => {
                if self.joysticks_enabled {
                    self.joysticks[1].read_io(port)
                } else {
                    Err(GgError::JoystickDisabled)
                }
            }
            joystick::JOYSTICK_START_PORT => {
                if self.joysticks_enabled {
                    self.joysticks[0].read_io(port)
                } else {
                    Err(GgError::JoystickDisabled)
                }
            }
            _ => Err(GgError::IoControllerInvalidPort),
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
            MEMORY_CONTROL_PORT => {
                self.bios_enabled = (value & 0b0000_1000) == 0;
                self.joysticks_enabled = (value & 0b0000_0100) == 0;
            }
            sdsc::CONTROL_PORT | sdsc::DATA_PORT => {
                if !self.joysticks_enabled {
                    self.sdsc_console.write_io(port, value)?;
                }
            }
            _ => return Err(GgError::IoControllerInvalidPort),
        }

        Ok(())
    }
}
