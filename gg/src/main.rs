use std::sync::mpsc::{self, Receiver, SyncSender};

use env_logger::Builder;
use log::{error, Level};
use raylib::prelude::*;
use raylib::texture::{Image, Texture2D};

use emu::system::System;
use emu::vdp::{self, INTERNAL_HEIGHT, INTERNAL_WIDTH};

// to allow scaling we'd have to render into Image, then call Image::resize_nn and then update the texture
const SCALE: usize = 1;

fn main() {
    initialize_logging();

    let (mut rl, thread) = raylib::init()
        .size((INTERNAL_WIDTH * SCALE) as i32, (INTERNAL_HEIGHT * SCALE) as i32)
        .title("geegee")
        .build();

    let mut texture = rl
        .load_texture_from_image(
            &thread,
            &Image::gen_image_color(INTERNAL_WIDTH as i32, INTERNAL_HEIGHT as i32, Color::BLACK),
        )
        .unwrap();

    let (tx, rx): (SyncSender<(vdp::Color, Vec<vdp::Color>)>, Receiver<(vdp::Color, Vec<vdp::Color>)>) = mpsc::sync_channel(1);

    std::thread::spawn(move || {
        let mut system = initialize_system();
        let mut paused = false;

        loop {
            if !paused {
                match system.tick() {
                    Ok(redraw) => {
                        if redraw {
                            let image = system.render();
                            tx.send(image).unwrap();
                        }
                    }
                    Err(e) => {
                        error!("{}", e);
                        paused = true;
                    }
                }
            }
        }
    });

    while !rl.window_should_close() {
        match rx.try_recv() {
            Ok(internal_frame) => render(&mut texture, &internal_frame),
            Err(_) => {}
        }

        let mut renderer = rl.begin_drawing(&thread);

        renderer.clear_background(Color::BLACK);
        renderer.draw_texture(&texture, 0, 0, Color::WHITE);
        renderer.draw_text("Executing Cartridge (Sonic 2)...", 4, 4, 9, Color::RAYWHITE);
    }
}

fn render(texture: &mut Texture2D, internal_frame: &(vdp::Color, Vec<vdp::Color>)) {
    let mut buffer: Vec<u8> = Vec::with_capacity(INTERNAL_WIDTH * INTERNAL_HEIGHT * 4);

    let (background_color, frame_src) = internal_frame;
    for y in 0..INTERNAL_HEIGHT {
        for x in 0..INTERNAL_WIDTH {
            let (r, g, b, a) = frame_src[y * INTERNAL_WIDTH + x];
            let color = if (r, g, b, a) == (0, 0, 0, 0) {
                (background_color.0, background_color.1, background_color.2, background_color.3)
            } else {
                (r, g, b, a)
            };
            buffer.extend([color.0, color.1, color.2, color.3]);
        }
    }

    texture.update_texture(&buffer);
}

fn initialize_system() -> System {
    let bios = include_bytes!("../../external/majbios.gg");
    let sonic2 = include_bytes!("../../external/sonic2.gg");
    let lua_script = String::from(include_str!("../../external/test.lua"));

    let mut system = System::new(Some(lua_script));
    system.load_bios(bios);
    system.load_cartridge(sonic2.as_ref()); // todo: need this cause of mapper

    system
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
        .filter(Some("emu"), default_log_level)
        .filter(Some("gg"), default_log_level)
        .format_timestamp(None)
        .init();

    logging::set_trace_log(TraceLogLevel::LOG_ERROR);
}
