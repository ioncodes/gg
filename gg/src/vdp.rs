use crate::{bus::Bus, io::IoMode};
use log::{debug, error, trace};
use bitmatch::bitmatch;

// todo: ????
const H_COUNTER_COUNT: u8 = 171;
const NTSC_SCANLINE_COUNT: u16 = 262; // 60 frames
const V_COUNTER_PORT: u8 = 0x7e;
const CONTROL_PORT: u8 = 0xbf;
// const PAL_SCANLINE_COUNT: u8 = 312;

#[derive(Default)]
pub(crate) struct Registers {
    pub(crate) r0: u8,
    pub(crate) r1: u8,
    pub(crate) r2: u8,
    pub(crate) r3: u8,
}

pub(crate) struct Vdp {
    pub(crate) v: u8,
    pub(crate) h: u8,
    control_data: Vec<u8>,
    registers: Registers,
}

impl Vdp {
    pub(crate) fn new() -> Vdp {
        Vdp {
            v: 0,
            h: 0,
            control_data: Vec::new(),
            registers: Registers::default(),
        }
    }

    #[bitmatch]
    pub(crate) fn tick(&mut self, bus: &mut Bus) {
        self.handle_io(bus);
        self.handle_counters();

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

                    let value = self.control_data[0];
                    let register = self.control_data[1] & 0b0000_1111;
                    match register {
                        0b0000_0001 => self.registers.r0 = value,
                        0b0000_0010 => self.registers.r1 = value,
                        0b0000_0100 => self.registers.r2 = value,
                        0b0000_1000 => self.registers.r3 = value,
                        _ => error!("Invalid VDP register: {:08b}", register),
                    }
                }
                "01??_????" => {
                    // Write to VRAM requests are denoted by a 0b01 at bit 15 and 14 of the 2nd byte in the control data.
                    // The rest adds up to a VRAM address.
                    let address = (((self.control_data[1] & 0b0011_1111) as u16) << 8) | (self.control_data[0] as u16);
                    debug!("Writing to VRAM address {:04x}", address);
                }
                _ => error!("Seemingly invalid control data found: {:02x?}", self.control_data),
            }
        }
    }

    fn handle_io(&mut self, bus: &mut Bus) {
        if bus.io.has_pending(V_COUNTER_PORT, IoMode::Read) {
            trace!("I/O request for V counter: {:02x}", self.v);
            bus.io.answer(V_COUNTER_PORT, self.v, IoMode::Read);
        }

        if let Some(data) = bus.io.pop(CONTROL_PORT, false) {
            trace!("Received byte via I/O control port ({:02x}): {:02x}", CONTROL_PORT, data);
            self.control_data.push(data);
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
        Ok(())
    }
}
