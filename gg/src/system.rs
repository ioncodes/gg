use crate::bus::Bus;
use crate::cpu::Cpu;
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

    pub(crate) fn load_bios(&mut self, data: &[u8]) {
        for i in 0..data.len() {
            self.bus
                .write(i as u16, data[i])
                .expect("failed to write to bus while loading BIOS");
        }
    }

    pub(crate) fn run(&mut self) {
        loop {
            self.cpu.tick(&mut self.bus);
            self.vdp.tick(&mut self.bus);

            // execute other components here (e.g. VDP or I/O interaction)
            self.bus.io.process_default();
        }
    }
}
