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
}

#[derive(Debug)]
pub(crate) struct IoBus {
    pub(crate) pipeline: HashMap<u8, Vec<IoData>>,
}

impl IoBus {
    pub(crate) fn new() -> IoBus {
        IoBus { pipeline: HashMap::new() }
    }

    // Contains default values that are returned by the I/O bus in a real system
    // during normal game execution.
    pub(crate) fn process_default(&mut self) {
        for (port, buffer) in self.pipeline.iter_mut() {
            match port {
                0x00..=0x06 => {
                    // Possibly Gear-to-Gear communication
                    const DEFAULT: [u8; 7] = [0xc0, 0x7f, 0xff, 0x00, 0xff, 0x00, 0xff];
                    if let Some(data) = buffer.first_mut() {
                        data.value = DEFAULT[*port as usize];
                    }
                }
                0x7e..=0x7f => {
                    // This is handled by the VDP (read) or PSG (write), ignore.
                }
                _ => warn!("Encountered I/O port with no default setting: {:02x}", port),
            }
        }
    }

    pub(crate) fn push(&mut self, port: u8, value: u8, mode: IoMode) {
        if let Some(buffer) = self.pipeline.get_mut(&port) {
            buffer.push(IoData { value, mode });
        } else {
            self.pipeline.insert(port, vec![IoData { value, mode }]);
        }
    }

    pub(crate) fn pop(&mut self, port: u8, mode: IoMode) -> Option<u8> {
        if let Some(buffer) = self.pipeline.get_mut(&port)
            && let Some(data) = buffer.pop()
            && data.mode == mode
        {
            let value = Some(data.value);
            self.pipeline.remove(&port);
            return value;
        }

        None
    }

    pub(crate) fn has_pending(&self, port: u8, mode: IoMode) -> bool {
        if let Some(buffer) = self.pipeline.get(&port)
            && let Some(data) = buffer.last()
            && data.mode == mode
        {
            true
        } else {
            false
        }
    }
}
