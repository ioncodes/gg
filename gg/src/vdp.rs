use crate::{
    bus::Bus,
    io::{IoMode, IoRequest},
};
use log::{debug, trace};

// todo: ????
const H_COUNTER_COUNT: u8 = 171;
const NTSC_SCANLINE_COUNT: u16 = 262; // 60 frames
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
        match &bus.io.data {
            // I/O VDP v counter read request
            Some(IoRequest {
                port: 0x7e,
                value: _,
                mode: IoMode::Read,
            }) => {
                trace!("I/O request in pipeline for V counter: {:02x}", self.v);
                // todo: Fix this. We shouldn't use Write for answers as read or write is going to be important to select VDP or PSG
                bus.io.push_request(0x7e, self.v, IoMode::Write);
            }
            Some(data) => debug!("Unhandled I/O request {:02x} = {:02x}", data.port, data.value),
            None => {}
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
