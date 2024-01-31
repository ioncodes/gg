use log::Level;
use env_logger::Builder;
use minifb::{Window, WindowOptions};

use core::system::System;
use core::vdp::Color;

const WIDTH: usize = 256;
const HEIGHT: usize = 224;

fn main() {
    initialize_logging();
    let mut window = initialize_renderer();

    let bios = include_bytes!("../../external/majbios.gg");
    let sonic2 = include_bytes!("../../external/sonic2.gg");
    let lua_script = String::from(include_str!("../../external/test.lua"));

    let mut system = System::new(Some(lua_script));
    system.load_rom(bios, true);
    system.load_rom(sonic2[..0xc000].as_ref(), false); // todo: need this cause of mapper
    
    loop {
        let redraw = system.tick();
        if redraw {
            draw_frame(&mut window, &system.render());
        }
    }
}

fn draw_frame(window: &mut Window, frame: &(Color, Vec<Vec<Color>>)) {
    let (background_color, frame) = frame;

    let buffer = {
        let mut buffer_: Vec<u32> = Vec::new();
        for x_buffer in frame {
            for color in x_buffer {
                let (r, g, b, a) = *color;
                if *color == (0, 0, 0, 0) {
                    let (r, g, b, a) = *background_color;
                    let color: u32 = ((a as u32) << 24) | ((r as u32) << 16) | ((g as u32) << 8) | b as u32;
                    buffer_.push(color);
                    continue;
                }
                
                let color: u32 = ((a as u32) << 24) | ((r as u32) << 16) | ((g as u32) << 8) | b as u32;
                buffer_.push(color);
            }
        }
        buffer_
    };

    window
        .update_with_buffer(&buffer, WIDTH, HEIGHT)
        .unwrap();
}

fn initialize_renderer() -> Window {
    let mut window = Window::new(
        "gg",
        WIDTH,
        HEIGHT,
        WindowOptions::default(),
    )
    .unwrap_or_else(|e| {
        panic!("{}", e);
    });

    window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

    window
}

fn initialize_logging() {
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
}