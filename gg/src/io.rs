use std::collections::HashMap;

use log::{error, warn};

// todo: rewrite entire io bus

#[derive(Debug, PartialEq)]
pub(crate) enum IoMode {
    Read,
    Write,
}

#[derive(Debug)]
pub(crate) struct IoData {
    pub(crate) value: u8,
    pub(crate) mode: IoMode,
    is_answer: bool,
}

#[derive(Debug)]
pub(crate) struct IoBus {
    pub(crate) data: HashMap<u8, IoData>,
}

impl IoBus {
    pub(crate) fn new() -> IoBus {
        IoBus { data: HashMap::new() }
    }

    // Contains default values that are returned by the I/O bus in a real system
    // during normal game execution.
    pub(crate) fn process_default(&mut self) {
        for (port, data) in self.data.iter_mut() {
            match port {
                0x00..=0x06 => {
                    // Possibly Gear-to-Gear communication
                    const DEFAULT: [u8; 7] = [0xc0, 0x7f, 0xff, 0x00, 0xff, 0x00, 0xff];
                    data.value = DEFAULT[*port as usize];
                    data.is_answer = true;
                },
                0x7e..=0x7f => {
                    // This is handled by the VDP (read) or PSG (write), ignore.
                }
                _ => warn!("Encountered I/O data with no default setting: {:02x} = {:02x}", port, data.value),
            }
        }
    }

    pub(crate) fn push(&mut self, port: u8, value: u8, mode: IoMode) {
        if self.data.contains_key(&port) {
            // todo: can this actually exist?
            error!("Writing to I/O data on port {:02x} that has not been answered yet", port);
        }

        self.data.insert(
            port,
            IoData {
                value,
                mode,
                is_answer: false,
            },
        );
    }

    pub(crate) fn pop(&mut self, port: u8, mode: IoMode) -> Option<u8> {
        if let Some(data) = self.data.get(&port)
            && data.mode == mode
        {
            let value = Some(data.value);
            self.data.remove(&port);
            return value;
        }

        None
    }

    pub(crate) fn has_pending(&self, port: u8, mode: IoMode) -> bool {
        if let Some(data) = self.data.get(&port) && data.mode == mode {
            true
        } else {
            false
        }
    }

    pub(crate) fn answer(&mut self, port: u8, value: u8, mode: IoMode) {
        if let Some(data) = self.data.get_mut(&port) {
            data.value = value;
            data.mode = mode;
            data.is_answer = true;
        }
    }
}
