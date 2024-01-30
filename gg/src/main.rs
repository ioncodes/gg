#![feature(let_chains)]

mod bus;
mod cpu;
mod error;
mod handlers;
mod io;
mod memory;
mod system;
mod vdp;
mod lua_engine;
mod dbg;

use crate::system::System;
use env_logger::Builder;
use log::Level;

fn main() {
    let mut default_log_level = Level::Info.to_level_filter();

    let enable_trace = std::env::args().any(|arg| arg == "--trace" || arg == "-t");
    if enable_trace {
        default_log_level = Level::Trace.to_level_filter();
    }

    let enable_debug = std::env::args().any(|arg| arg == "--debug" || arg == "-d");
    if enable_debug {
        default_log_level = Level::Debug.to_level_filter();
    }

    Builder::new()
        .filter(None, default_log_level)
        .format_timestamp(None)
        .init();

    let bios = include_bytes!("../../external/majbios.gg");
    let sonic2 = include_bytes!("../../external/sonic2.gg");
    let lua_script = String::from(include_str!("../../external/test.lua"));

    let mut system = System::new(Some(lua_script));
    system.load_rom(bios, true);
    system.load_rom(sonic2[..0xc000].as_ref(), false); // todo: need this cause of mapper
    system.run();
}
