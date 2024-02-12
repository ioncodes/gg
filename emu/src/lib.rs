#![feature(let_chains)]
#![feature(custom_test_frameworks)]
#![test_runner(datatest::runner)]

mod tests;

mod error;
mod handlers;
mod io;
mod memory;
mod lua_engine;
mod mapper;
mod sdsc;

pub mod cpu;
pub mod vdp;
pub mod system;
pub mod psg;
pub mod bus;
pub mod joystick;