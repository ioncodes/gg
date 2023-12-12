#![feature(let_chains)]

mod bus;
mod cpu;
mod error;
mod handlers;
mod io;
mod memory;
mod system;
mod vdp;

use crate::system::System;
use env_logger::Builder;
use log::Level;

fn main() {
    let mut default_log_level = Level::Warn.to_level_filter();

    let enable_trace = std::env::args().any(|arg| arg == "--trace" || arg == "-t");
    if enable_trace {
        default_log_level = Level::Trace.to_level_filter();
    }

    Builder::new()
        .filter(None, default_log_level)
        .format_timestamp(None)
        .init();

    let data = include_bytes!("../../external/majbios.gg");

    let mut system = System::new();
    system.load_rom(data);
    system.run();
}
