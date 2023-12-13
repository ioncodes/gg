use crate::{bus::Bus, io::IoMode};
use log::trace;

// todo: ????
const H_COUNTER_COUNT: u8 = 171;
const NTSC_SCANLINE_COUNT: u16 = 262; // 60 frames
const V_COUNTER_PORT: u8 = 0x7e;
// const PAL_SCANLINE_COUNT: u8 = 312;

pub(crate) struct Vdp {
    pub(crate) v: u8,
    pub(crate) h: u8,
}

impl Vdp {
    pub(crate) fn new() -> Vdp {
        Vdp { v: 0, h: 0 }
    }

    pub(crate) fn tick(&mut self, bus: &mut Bus) {
        self.handle_io(bus);

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

    fn handle_io(&mut self, bus: &mut Bus) {
        if bus.io.has_pending(V_COUNTER_PORT, IoMode::Read) {
            trace!("I/O request for V counter: {:02x}", self.v);
            bus.io.answer(V_COUNTER_PORT, self.v, IoMode::Read);
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
