use env_logger::Builder;
use log::{Level, error};
use pixels::wgpu::PresentMode;
use pixels::{Pixels, PixelsBuilder, SurfaceTexture};
use winit::dpi::LogicalSize;
use winit::event::{Event, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{Window, WindowBuilder};
use winit_input_helper::WinitInputHelper;

use core::system::System;
use core::vdp::{Color, INTERNAL_HEIGHT, INTERNAL_WIDTH};
use std::sync::mpsc::{self, Receiver, Sender};

const RENDER_SCALE: usize = 4;

fn main() {
    initialize_logging();
    let (window, event_loop, mut pixels) = initialize_renderer();
    let mut input = WinitInputHelper::new();

    let (tx, rx): (Sender<(Color, Vec<Color>)>, Receiver<(Color, Vec<Color>)>) = mpsc::channel();

    std::thread::spawn(move || {
        let mut system = initialize_system();
        let mut emulator_paused = false;

        while !emulator_paused {
            match system.tick() {
                Ok(true) => tx.send(system.render()).unwrap(),
                Ok(false) => (),
                Err(error) => {
                    error!("Encountered error: {}", error);
                    emulator_paused = true;
                }
            };
        }
    });

    event_loop.run(move |event, _, control_flow| {
        if let Event::RedrawRequested(_) = event {
            pixels.render().unwrap();
        }

        if input.update(&event) {
            if input.key_pressed(VirtualKeyCode::Escape) || input.close_requested() {
                *control_flow = ControlFlow::Exit;
                return;
            }
        }

        match rx.try_recv() {
            Ok(frame) => {
                draw_frame(&mut pixels.frame_mut(), &frame);
                window.request_redraw();
            },
            _ => (),
        }
    });
}

fn draw_frame(frame_dst: &mut [u8], frame_src: &(Color, Vec<Color>)) {
    let (background_color, frame_src) = frame_src;

    for (i, pixel) in frame_dst.chunks_exact_mut(4).enumerate() {
        let color = frame_src.get(i).unwrap();
        let (r, g, b, a) = if *color == (0, 0, 0, 0) { *background_color } else { *color };
        pixel.copy_from_slice([r, g, b, a].as_ref());
    }
}

fn initialize_system() -> System {
    let bios = include_bytes!("../../external/majbios.gg");
    let sonic2 = include_bytes!("../../external/sonic2.gg");
    let lua_script = String::from(include_str!("../../external/test.lua"));

    let mut system = System::new(Some(lua_script));
    system.load_bios(bios);
    system.load_cartridge(sonic2[..0xc000].as_ref()); // todo: need this cause of mapper

    system
}

fn initialize_renderer() -> (Window, EventLoop<()>, Pixels) {
    let event_loop = EventLoop::new();
    let window = {
        let size = LogicalSize::new((INTERNAL_WIDTH * RENDER_SCALE) as f64, (INTERNAL_HEIGHT * RENDER_SCALE) as f64);
        WindowBuilder::new()
            .with_title("geegee")
            .with_inner_size(size)
            .with_min_inner_size(size)
            .with_resizable(false)
            .build(&event_loop)
            .unwrap()
    };

    let pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        PixelsBuilder::new(INTERNAL_WIDTH as u32, INTERNAL_HEIGHT as u32, surface_texture)
            .present_mode(PresentMode::Immediate)
            .build()
            .unwrap()
    };

    (window, event_loop, pixels)
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
        .filter(Some("core"), default_log_level)
        .format_timestamp(None)
        .init();
}
