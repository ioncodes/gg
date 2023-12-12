use crate::bus::Bus;
use crate::cpu::Cpu;
use crate::error::GgError;
use crate::vdp::Vdp;

pub(crate) struct System {
    pub(crate) cpu: Cpu,
    pub(crate) bus: Bus,
    pub(crate) vdp: Vdp,
}

impl System {
    pub(crate) fn new() -> System {
        System {
            cpu: Cpu::new(),
            bus: Bus::new(),
            vdp: Vdp::new(),
        }
    }

    pub(crate) fn load_rom(&mut self, data: &[u8]) {
        for i in 0..data.len() {
            self.bus
                .write(i as u16, data[i])
                .expect("Failed to write to bus while loading into ROM");
        }
    }

    pub(crate) fn run(&mut self) {
        loop {
            let result = self.cpu.tick(&mut self.bus);
            match result {
                Err(GgError::OpcodeNotImplemented { opcode: _ }) => panic!("{}", self),
                Err(GgError::DecoderError { msg }) => panic!("Decoder error: {}\n{}", msg, self),
                _ => {}
            };
            self.vdp.tick(&mut self.bus);

            // execute other components here (e.g. VDP or I/O interaction)
            self.bus.io.process_default();
        }
    }
}

impl std::fmt::Display for System {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "CPU state:\n{}", self.cpu)?;

        write!(f, "\n\nStack:\n")?;
        for i in -8..=8 {
            let address = self.cpu.registers.sp.wrapping_add_signed(i);
            let value = self.bus.read_word(address).unwrap_or(0x6969);
            write!(f, "{:04x}: {:04x}\n", address, value)?;
        }

        write!(f, "\nVDP state:\n{}", self.vdp)?;

        Ok(())
    }
}
