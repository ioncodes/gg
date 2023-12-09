use crate::bus::Bus;
use crate::cpu::Cpu;

pub(crate) struct System {
    pub(crate) cpu: Cpu,
    pub(crate) bus: Bus,
}

impl System {
    pub(crate) fn new() -> System {
        System {
            cpu: Cpu::new(),
            bus: Bus::new(),
        }
    }

    pub(crate) fn load_bios(&mut self, data: &[u8]) {
        for i in 0..data.len() {
            self.bus.write(i as u16, data[i]);
        }
    }

    pub(crate) fn run(&mut self) {
        loop {
            self.cpu.tick(&mut self.bus);
            
            // execute other components here (e.g. VDP or I/O interaction)
            self.bus.io.process_default();
        }
    }
}
