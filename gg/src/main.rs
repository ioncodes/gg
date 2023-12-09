mod cpu;
mod memory;
mod system;
mod bus;
mod handlers;

use crate::system::System;

fn main() {
    let data = include_bytes!("../../external/majbios.gg");

    let mut system = System::new();
    system.load_bios(data);
    system.run();
}