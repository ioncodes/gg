mod bus;
mod cpu;
mod error;
mod handlers;
mod io;
mod memory;
mod system;
mod vdp;

use crate::system::System;

fn main() {
    let data = include_bytes!("../../external/majbios.gg");

    let mut system = System::new();
    system.load_bios(data);
    system.run();
}
