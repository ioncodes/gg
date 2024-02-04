#![feature(let_chains)]

use eframe::egui::{CentralPanel, Context, TextureOptions, Window};
use eframe::epaint::{ColorImage, ImageDelta, TextureHandle};
use eframe::{egui::Frame, egui::ViewportBuilder, Renderer};
use env_logger::Builder;
use log::{error, Level};

use core::system::System;
use core::vdp::{Color, INTERNAL_HEIGHT, INTERNAL_WIDTH};
use std::sync::mpsc::{sync_channel, Receiver, SyncSender};

const SCALE: usize = 4;

fn main() {
    initialize_logging();

    let options = eframe::NativeOptions {
        viewport: ViewportBuilder::default().with_inner_size([(INTERNAL_WIDTH * SCALE) as f32, (INTERNAL_HEIGHT * SCALE) as f32]),
        renderer: Renderer::Glow,
        ..Default::default()
    };
    let _ = eframe::run_native("geegee", options, Box::new(|cc| Box::new(Debugger::new(cc))));
}

struct DebuggerTexture {
    texture: TextureHandle,
    buffer: Vec<u8>,
}

impl DebuggerTexture {
    fn new(cc: &eframe::CreationContext<'_>, name: &str, width: usize, height: usize) -> Self {
        let buffer = vec![0xff; width * height * 4];
        let texture = cc.egui_ctx.load_texture(
            name,
            ColorImage::from_rgba_unmultiplied([width, height], &buffer),
            TextureOptions::NEAREST,
        );

        Self { texture, buffer }
    }
}

struct Debugger {
    emu_texture: DebuggerTexture,
    //emu_texture_bg: DebuggerTexture,
    render_rx: Receiver<(Color, Vec<Color>)>,
}

impl Debugger {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let emu_texture = DebuggerTexture::new(cc, "emu_frame", INTERNAL_WIDTH, INTERNAL_HEIGHT);
        //let emu_texture_bg = DebuggerTexture::new(cc, "emu_frame_bg", INTERNAL_WIDTH * SCALE, INTERNAL_HEIGHT * SCALE);

        let (render_tx, render_rx): (SyncSender<(Color, Vec<Color>)>, Receiver<(Color, Vec<Color>)>) = sync_channel(1);

        std::thread::spawn(move || {
            let mut system = initialize_system();
            let mut paused = false;

            loop {
                if paused {
                    std::thread::sleep(std::time::Duration::from_micros(500));
                    continue;
                }

                match system.tick() {
                    Ok(redraw) => {
                        if redraw {
                            let (background_color, frame_src) = system.render();
                            render_tx.send((background_color, frame_src)).unwrap();
                        }
                    }
                    Err(e) => {
                        error!("{}", e);
                        paused = true;
                    }
                }

                
            }
        });

        Self { emu_texture, /*emu_texture_bg,*/ render_rx }
    }

    fn update_emulator_texture(&mut self, ctx: &Context, frame: &(Color, Vec<Color>)) {
        let (background_color, frame_src) = frame;

        self.emu_texture.buffer.clear();
        self.emu_texture.buffer.extend(
            frame_src
                .iter()
                .map(|&color| {
                    let (r, g, b, a) = if color == (0, 0, 0, 0) {
                        *background_color
                    } else {
                        (color.0, color.1, color.2, color.3)
                    };
                    [r, g, b, a]
                })
                .flatten(),
        );

        // Upscale the frame for background texture
        // self.emu_texture_bg.buffer.clear();
        // let mut resizer = resize::new(INTERNAL_WIDTH, INTERNAL_HEIGHT, self.emu_texture_bg.texture.size()[0], self.emu_texture_bg.texture.size()[1], Pixel::RGBA8, Type::Point).unwrap();
        // let src = self.emu_texture.buffer.chunks(4).map(|color| RGBA::new(color[0], color[1], color[2], color[3])).collect::<Vec<_>>();
        // let mut dst = vec![RGBA::new(0, 0, 0, 0); INTERNAL_WIDTH * INTERNAL_HEIGHT * SCALE];
        // let _ = resizer.resize(src.as_slice(), &mut dst);
        // let dst: Vec<u8> = dst.iter().map(|color| [color.r, color.g, color.b, color.a]).flatten().collect::<Vec<_>>();
        // self.emu_texture_bg.buffer.extend(dst);

        let frame = ColorImage::from_rgba_unmultiplied([INTERNAL_WIDTH, INTERNAL_HEIGHT], &self.emu_texture.buffer);
        ctx.tex_manager()
            .write()
            .set(self.emu_texture.texture.id(), ImageDelta::full(frame, TextureOptions::NEAREST));

        // let frame = ColorImage::from_rgba_unmultiplied([INTERNAL_WIDTH * (SCALE / 2), INTERNAL_HEIGHT * (SCALE / 2)], &self.emu_texture_bg.buffer);
        // ctx.tex_manager()
        //     .write()
        //     .set(self.emu_texture_bg.texture.id(), ImageDelta::full(frame, TextureOptions::NEAREST));
    }
}

impl eframe::App for Debugger {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        match self.render_rx.try_recv() {
            Ok(frame) => self.update_emulator_texture(ctx, &frame),
            Err(_) => (),
        }

        CentralPanel::default().frame(Frame::none()).show(ctx, |ui| {
            //ui.image(&self.emu_texture_bg.texture);

            // Window::new("Registers").collapsible(false).show(ui.ctx(), |ui| {
            //     if let Some(dbg_state) = &self.dbg_state {
            //         ui.label(format!("A: {:02X} F: {:02X}", dbg_state.registers.a, dbg_state.registers.f));
            //         ui.label(format!("B: {:02X} C: {:02X}", dbg_state.registers.b, dbg_state.registers.c));
            //         ui.label(format!("D: {:02X} E: {:02X}", dbg_state.registers.d, dbg_state.registers.e));
            //         ui.label(format!("H: {:02X} L: {:02X}", dbg_state.registers.h, dbg_state.registers.l));
            //         ui.label(format!("PC: {:04X}", dbg_state.registers.pc));
            //         ui.label(format!("SP: {:04X}", dbg_state.registers.sp));
            //     }
            // });

            Window::new("Framebuffer").collapsible(false).show(ui.ctx(), |ui| {
                ui.image(&self.emu_texture.texture);
            });
        });

        ctx.request_repaint();
    }
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
        .filter(Some("core"), default_log_level)
        .filter(Some("gg"), default_log_level)
        .format_timestamp(None)
        .init();
}
