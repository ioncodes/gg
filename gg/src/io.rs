pub(crate) struct IoRequest {
    pub(crate) port: u8,
    pub(crate) value: u8
}

pub(crate) struct IoBus {
    pub(crate) request: Option<IoRequest>
}

impl IoBus {
    pub(crate) fn new() -> IoBus {
        IoBus { request: None }
    }

    // Contains default values that are returned by the I/O bus in a real system
    // during normal game execution.
    pub(crate) fn process_default(&mut self) {
        if let Some(request) = self.request.take() {
            match request.port {
                0x00..=0x06 => {
                    // Possibly Gear-to-Gear communication
                    const DEFAULT: [u8; 7] = [0xc0, 0x7f, 0xff, 0x00, 0xff, 0x00, 0xff];
                    let default_value = DEFAULT[request.port as usize];
                    self.request = Some(IoRequest { port: request.port, value: default_value });
                },
                _ => println!("Encountered I/O request with no default setting: {:02x} = {:02x}", request.port, request.value)
            }
        }
    }

    pub(crate) fn push_request(&mut self, port: u8, value: u8) {
        self.request = Some(IoRequest { port, value });
    }

    pub(crate) fn pop_request(&mut self, port: u8) -> Option<u8> {
        let answer = self.request.take();
        if let Some(ref request) = answer {
            if request.port == port {
                return Some(request.value);
            }
        }
        None
    }
}