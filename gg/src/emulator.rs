use eframe::CreationContext;
use egui::{vec2, CentralPanel, Color32, ColorImage, Image, TextureHandle, TextureOptions, Window};
use emu::{
    system::System,
    vdp::{Color, INTERNAL_HEIGHT, INTERNAL_WIDTH},
};
use log::error;

pub(crate) const SCALE: usize = 4;

pub(crate) struct Emulator {
    system: System,
    background_color: Color,
    paused: bool,
    texture: TextureHandle,
}

impl eframe::App for Emulator {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.run() {
            self.render();
        }

        CentralPanel::default().show(ctx, |ui| {
            let image = Image::new(&self.texture);
            let image = image.fit_to_exact_size(vec2((INTERNAL_WIDTH * SCALE) as f32, (INTERNAL_HEIGHT * SCALE) as f32));
            image.paint_at(ui, ui.ctx().screen_rect());
        });

        Window::new("Background")
            .resizable(false)
            .max_width(INTERNAL_WIDTH as f32)
            .max_height(INTERNAL_HEIGHT as f32)
            .show(ctx, |ui| {
                ui.vertical_centered_justified(|ui| {
                    ui.colored_label(
                        Color32::from_rgba_unmultiplied(
                            self.background_color.0,
                            self.background_color.1,
                            self.background_color.2,
                            self.background_color.3,
                        ),
                        format!(
                            "Background Color [r:{:02x} g:{:02x} b:{:02x}]",
                            self.background_color.0, self.background_color.1, self.background_color.2
                        ),
                    );
                    ui.add(Image::new(&self.texture));
                });
            });

        Window::new("Registers")
            .resizable(false)
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    ui.label(format!("A: {:02x} F: {:02x}", self.system.cpu.registers.a, self.system.cpu.registers.f));
                    ui.label(format!("B: {:02x} C: {:02x}", self.system.cpu.registers.b, self.system.cpu.registers.c));
                    ui.label(format!("D: {:02x} E: {:02x}", self.system.cpu.registers.d, self.system.cpu.registers.e));
                    ui.label(format!("H: {:02x} L: {:02x}", self.system.cpu.registers.h, self.system.cpu.registers.l));
                    ui.label(format!("SP: {:04x}", self.system.cpu.registers.sp));
                    ui.label(format!("PC: {:04x}", self.system.cpu.registers.pc));
                });
            });

        ctx.request_repaint();
    }
}

impl Emulator {
    pub(crate) fn new(cc: &CreationContext) -> Emulator {
        let bios = include_bytes!("../../external/majbios.gg");
        let sonic2 = include_bytes!("../../external/sonic2.gg");
        let lua_script = String::from(include_str!("../../external/test.lua"));

        let mut system = System::new(Some(lua_script));
        system.load_bios(bios);
        system.load_cartridge(sonic2.as_ref());

        let texture = cc.egui_ctx.load_texture(
            "frame",
            ColorImage::new([INTERNAL_WIDTH, INTERNAL_HEIGHT], Color32::BLACK),
            TextureOptions::NEAREST,
        );

        Emulator {
            system,
            background_color: (0, 0, 0, 0),
            paused: false,
            texture,
        }
    }

    fn run(&mut self) -> bool {
        if self.paused {
            return false;
        }

        for _ in 0..10000 {
            match self.system.tick() {
                Ok(true) => return true,
                Ok(false) => (),
                Err(e) => {
                    error!("Error: {}", e);
                    self.paused = true;
                    break;
                }
            }
        }

        false
    }

    fn render(&mut self) {
        let mut texture: Vec<Color32> = Vec::new();

        let (background_color, frame_src) = self.system.render();
        for y in 0..INTERNAL_HEIGHT {
            for x in 0..INTERNAL_WIDTH {
                let (r, g, b, a) = frame_src[y * INTERNAL_WIDTH + x];
                let color = if (r, g, b, a) == (0, 0, 0, 0) {
                    (background_color.0, background_color.1, background_color.2, background_color.3)
                } else {
                    (r, g, b, a)
                };
                texture.push(Color32::from_rgba_premultiplied(color.0, color.1, color.2, color.3));
            }
        }

        let image = ColorImage {
            size: [INTERNAL_WIDTH, INTERNAL_HEIGHT],
            pixels: texture,
        };

        self.texture.set(image, TextureOptions::NEAREST);
        self.background_color = background_color;
    }
}
