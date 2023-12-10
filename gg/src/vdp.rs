use crate::{bus::Bus, io::{IoMode, IoRequest}};

pub(crate) struct Vdp {
    pub(crate) v: u8,
    pub(crate) h: u8,
}

impl Vdp {
    pub(crate) fn new() -> Vdp {
        Vdp {
            v: 0,
            h: 0,
        }
    }

    pub(crate) fn tick(&mut self, bus: &mut Bus) {
        match &bus.io.data {
            // I/O VDP v counter read request
            Some(IoRequest { port: 0x7e, value: _, mode: IoMode::Read }) => {
                println!("[vdp] Found v counter read request");
                bus.io.push_request(0x7e, self.v, IoMode::Write);
            },
            Some(data) => println!("[vdp] Unhandled I/O request {:02x} = {:02x}", data.port, data.value),
            None => {},
        }
    }
}