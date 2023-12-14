use std::collections::VecDeque;

use crate::{bus::Bus, io::IoMode, memory::Memory};
use bitmatch::bitmatch;
use log::{debug, error, trace};

// todo: ????
const H_COUNTER_COUNT: u8 = 171;
const NTSC_SCANLINE_COUNT: u16 = 262; // 60 frames
const V_COUNTER_PORT: u8 = 0x7e;
const CONTROL_PORT: u8 = 0xbf;
const DATA_PORT: u8 = 0xbe;
// const PAL_SCANLINE_COUNT: u8 = 312;

pub(crate) enum Mode {
    VramWrite,
    CramWrite,
    None,
}

#[derive(Default, Debug)]
pub(crate) struct Registers {
    pub(crate) r0: u8,
    pub(crate) r1: u8,
    pub(crate) r2: u8,
    pub(crate) r3: u8,
    pub(crate) r4: u8,
    pub(crate) r5: u8,
    pub(crate) r6: u8,
    pub(crate) r7: u8,
    pub(crate) r8: u8,
    pub(crate) r9: u8,
    pub(crate) r10: u8,
    pub(crate) address: u16,
}

pub(crate) struct Vdp {
    pub(crate) v: u8,
    pub(crate) h: u8,
    control_data: VecDeque<u8>,
    registers: Registers,
    vram: Memory,
    cram: Memory,
    mode: Mode,
}

impl Vdp {
    pub(crate) fn new() -> Vdp {
        Vdp {
            v: 0,
            h: 0,
            control_data: VecDeque::new(),
            registers: Registers::default(),
            vram: Memory::new(16 * 1024, 0x0000),
            cram: Memory::new(64, 0x0000),
            mode: Mode::None,
        }
    }

    pub(crate) fn tick(&mut self, bus: &mut Bus) {
        self.handle_io(bus);
        self.handle_control_data();
        self.handle_counters();
    }

    #[bitmatch]
    fn handle_control_data(&mut self) {
        loop {
            if self.control_data.len() >= 2 {
                trace!("VDP control type: {:08b}", self.control_data[1]);

                #[bitmatch]
                match self.control_data[1] {
                    "1000_????" => {
                        /*
                         * Set VDP register:
                         * To set data in a VDP register, the data is inputted in the first byte. The second byte is used
                         * to indicate the register where the data is to be transferred.
                         * The bottom four bits (R3 to R0) of the second byte designate the data transfer destination
                         * registers (#0 to #10). b7 must be“1”and b6 to b4 must be“0”.
                         */

                        let value = self.control_data.pop_front().unwrap();
                        let register = self.control_data.pop_front().unwrap() & 0b0000_1111;
                        match register {
                            0b0000_0000 => self.registers.r0 = value,
                            0b0000_0001 => self.registers.r1 = value,
                            0b0000_0010 => self.registers.r2 = value,
                            0b0000_0011 => self.registers.r3 = value,
                            0b0000_0100 => self.registers.r4 = value,
                            0b0000_0101 => self.registers.r5 = value,
                            0b0000_0110 => self.registers.r6 = value,
                            0b0000_0111 => self.registers.r7 = value,
                            0b0000_1000 => self.registers.r8 = value,
                            0b0000_1001 => self.registers.r9 = value,
                            0b0000_1010 => self.registers.r10 = value,
                            // registers 11..15 have no effect when written to
                            _ => error!("Invalid VDP register: {:08b}", register),
                        }
                    }
                    "01??_????" => {
                        // Write to VRAM requests are denoted by a 0b01 at bit 15 and 14 of the 2nd byte in the control data.
                        // The rest adds up to a VRAM address.
                        let control_byte1 = self.control_data.pop_front().unwrap();
                        let control_byte2 = self.control_data.pop_front().unwrap();
                        let address = (((control_byte2 & 0b0011_1111) as u16) << 8) | (control_byte1 as u16);
                        self.registers.address = address;
                        debug!("Setting address register to {:04x}", address);
                        self.mode = Mode::VramWrite;
                    }
                    "11??_????" => {
                        // Write to CRAM requests are denoted by a 0b11 at bit 15 and 14 of the 2nd byte in the control data.
                        // The rest adds up to a CRAM address.
                        let control_byte1 = self.control_data.pop_front().unwrap();
                        let control_byte2 = self.control_data.pop_front().unwrap();
                        let address = (((control_byte2 & 0b0011_1111) as u16) << 8) | (control_byte1 as u16);
                        self.registers.address = address;
                        debug!("Setting address register to {:04x}", address);
                        self.mode = Mode::CramWrite;
                    }
                    _ => error!("Seemingly unimplemented control data found: {:02x?}", self.control_data),
                }
            } else {
                break;
            }
        }
    }

    fn handle_io(&mut self, bus: &mut Bus) {
        if let Some(_) = bus.io.pop(V_COUNTER_PORT, false) {
            trace!("I/O request for V counter: {:02x}", self.v);
            bus.push_io_data(V_COUNTER_PORT, self.v, IoMode::Read, true);
        }

        if let Some(buffer) = bus.io.pop_all(CONTROL_PORT) {
            debug!("Received buffer via I/O control port ({:02x}): {:02x?}", CONTROL_PORT, buffer);
            self.control_data.extend(buffer);
        }

        if let Some(buffer) = bus.io.pop_all(DATA_PORT) {
            trace!("Received buffer via I/O data port ({:02x}): {:02x?}", DATA_PORT, buffer);

            match self.mode {
                Mode::VramWrite => {
                    // Write data bytes to VRAM
                    self.vram.copy(self.registers.address, &buffer);
                    debug!("Wrote {} bytes to {:04x} @ VRAM", buffer.len(), self.registers.address);

                    self.registers.address += buffer.len() as u16;
                    if self.registers.address >= 0x4000 {
                        self.registers.address = self.registers.address - 0x4000;
                    }
                }
                Mode::CramWrite => {
                    // Write data bytes to CRAM
                    // "If the address register exceeds the CRAM size (32 or 64 bytes), the high bits are ignored so it will always address CRAM;""
                    // "for example, address $1000 wil read from CRAM address $00.""
                    let address = self.registers.address & 0b0000_0000_0111_1111;
                    self.cram.copy(address, &buffer);
                    debug!("Wrote {} bytes to {:04x} @ CRAM", buffer.len(), address);

                    self.registers.address += buffer.len() as u16;
                    if self.registers.address >= 0x4000 {
                        self.registers.address = self.registers.address - 0x4000;
                    }
                }
                Mode::None => {
                    error!("Received byte on data port ({:02x}) without being in a specific mode", DATA_PORT);
                }
            }
        }
    }

    fn handle_counters(&mut self) {
        if self.h < H_COUNTER_COUNT {
            self.h += 1;
        } else {
            self.h = 0;

            if (self.v as u16) < NTSC_SCANLINE_COUNT {
                self.v += 1;
            } else {
                self.v = 0;
            }
        }
    }
}

impl std::fmt::Display for Vdp {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "V counter: {:02x}\n", self.v)?;
        write!(f, "H counter: {:02x}\n", self.h)?;

        write!(f, "Registers: {:04x?}\n", self.registers)?;

        for addr in -8..=8 {
            let addr = self.registers.address as i16 + addr;
            if addr < 0 {
                continue;
            }

            let addr = addr as u16;
            let value = self.vram.read(addr);
            write!(f, "{:04x}: {:02x}\n", addr, value)?;
        }

        Ok(())
    }
}
