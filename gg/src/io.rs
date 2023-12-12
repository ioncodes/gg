#[derive(Debug)]
pub(crate) enum IoMode {
    Read,  // request
    Write, // answer
}

#[derive(Debug)]
pub(crate) struct IoRequest {
    pub(crate) port: u8,
    pub(crate) value: u8,
    pub(crate) mode: IoMode,
}

#[derive(Debug)]
pub(crate) struct IoBus {
    pub(crate) data: Option<IoRequest>,
}

impl IoBus {
    pub(crate) fn new() -> IoBus {
        IoBus { data: None }
    }

    // Contains default values that are returned by the I/O bus in a real system
    // during normal game execution.
    pub(crate) fn process_default(&mut self) {
        if let Some(request) = self.data.as_ref() {
            match request.port {
                0x00..=0x06 => {
                    // Possibly Gear-to-Gear communication
                    const DEFAULT: [u8; 7] = [0xc0, 0x7f, 0xff, 0x00, 0xff, 0x00, 0xff];
                    let default_value = DEFAULT[request.port as usize];
                    self.push_request(request.port, default_value, IoMode::Write);
                }
                _ => println!(
                    "[io] Encountered I/O request with no default setting: {:02x} = {:02x}",
                    request.port, request.value
                ),
            }
        }
    }

    pub(crate) fn push_request(&mut self, port: u8, value: u8, mode: IoMode) {
        self.data = Some(IoRequest { port, value, mode });
    }

    pub(crate) fn pop_request(&mut self, port: u8) -> Option<u8> {
        let data = self.data.as_ref();

        if let Some(request) = data {
            if request.port == port {
                let value = request.value;
                self.data = None;
                return Some(value);
            }
        }

        None
    }
}
