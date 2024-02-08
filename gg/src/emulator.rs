use std::collections::VecDeque;

use eframe::CreationContext;
use egui::{vec2, CentralPanel, Color32, ColorImage, Image, ScrollArea, TextureHandle, TextureOptions, Window};
use emu::{
    system::System,
    vdp::{Color, INTERNAL_HEIGHT, INTERNAL_WIDTH},
};
use log::error;
use z80::{
    disassembler::Disassembler,
    instruction::{Instruction, Opcode},
};

pub(crate) const SCALE: usize = 4;

pub(crate) struct Emulator {
    system: System,
    background_color: Color,
    dissasembly_cache: Vec<Instruction>,
    trace: VecDeque<(u16, Opcode)>,
    paused: bool,
    stepping: bool,
    break_condition_active: bool,
    break_condition: String,
    texture: TextureHandle,
}

impl eframe::App for Emulator {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.paused && self.stepping {
            if self.run(1) {
                self.render();
            }
            self.stepping = false;
        } else if !self.paused && !self.stepping {
            if self.run(100000) {
                self.render();
            }
        }

        CentralPanel::default().show(ctx, |ui| {
            let image = Image::new(&self.texture);
            let image = image.fit_to_exact_size(vec2((INTERNAL_WIDTH * SCALE) as f32, (INTERNAL_HEIGHT * SCALE) as f32));
            image.paint_at(ui, ui.ctx().screen_rect());
        });

        Window::new("Background Layer")
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
                            "Background [r:{:02x} g:{:02x} b:{:02x}]",
                            self.background_color.0, self.background_color.1, self.background_color.2
                        ),
                    );
                    ui.add(Image::new(&self.texture));
                });
            });

        Window::new("Debugger").resizable(false).show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("Resume").clicked() {
                    self.paused = false;
                }
                self.stepping = ui.button("Step").clicked();
                ui.label(format!("State: {}", if self.paused { "Paused" } else { "Running" }));
            });

            ui.horizontal(|ui| {
                ui.label(format!(
                    "ROM: {}",
                    if self.system.bus.bios_enabled {
                        String::from("BIOS")
                    } else {
                        format!("Cartridge ({})", self.system.bus.rom.name())
                    }
                ));
            });

            ui.separator();

            ui.horizontal(|ui| {
                ui.text_edit_singleline(&mut self.break_condition);
                if ui.button("Break On...").clicked() {
                    self.break_condition_active = true;
                    self.paused = false;
                }
            });
        });

        Window::new("CPU / VDP").resizable(false).show(ctx, |ui| {
            ui.heading("CPU Registers");

            ui.vertical(|ui| {
                ui.label(format!(
                    "PC: {:04x} [{:08x}]",
                    self.system.cpu.registers.pc,
                    self.system
                        .bus
                        .translate_address_to_real(self.system.cpu.registers.pc)
                        .unwrap_or(self.system.cpu.registers.pc as usize)
                ));
                ui.label(format!("SP: {:04x}", self.system.cpu.registers.sp));
                ui.label(format!(
                    "AF: {:02x}{:02x}",
                    self.system.cpu.registers.a, self.system.cpu.registers.f
                ));
                ui.label(format!(
                    "BC: {:02x}{:02x}",
                    self.system.cpu.registers.b, self.system.cpu.registers.c
                ));
                ui.label(format!(
                    "DE: {:02x}{:02x}",
                    self.system.cpu.registers.d, self.system.cpu.registers.e
                ));
                ui.label(format!(
                    "HL: {:02x}{:02x}",
                    self.system.cpu.registers.h, self.system.cpu.registers.l
                ));
                ui.label(format!("IX: {:04x}", self.system.cpu.registers.ix));
                ui.label(format!("IY: {:04x}", self.system.cpu.registers.iy));
            });

            ui.separator();
        
            ui.heading("CPU Interrupts");
            ui.horizontal(|ui| {
                ui.checkbox(&mut self.system.cpu.interrupts_enabled, "Interrupts Enabled");

                ui.label(format!("Interrupt Mode: {}", self.system.cpu.interrupt_mode.to_u8()));
            });

            ui.separator();

            ui.heading("VDP Registers");

            ui.vertical(|ui| {
                ui.label(format!("V: {:02x}", self.system.vdp.v));
                ui.label(format!("H: {:02x}", self.system.vdp.h));
            });
        });

        Window::new("Disassembly").resizable(false).max_height(100.0).show(ctx, |ui| {
            let mut addr = self.system.cpu.registers.pc;
            for instr in &self.dissasembly_cache {
                ui.label(format!("{:04x}: {}", addr, instr.opcode));
                addr += instr.length as u16;
            }
        });

        Window::new("Trace").resizable(false).max_height(500.0).show(ctx, |ui| {
            ScrollArea::vertical().stick_to_bottom(true).show(ui, |ui| {
                for instr in &self.trace {
                    ui.label(format!("{:04x}: {}", instr.0, instr.1));
                }
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
            dissasembly_cache: Vec::new(),
            break_condition_active: false,
            break_condition: String::new(),
            background_color: (0, 0, 0, 0),
            paused: true,
            trace: VecDeque::with_capacity(1024),
            stepping: false,
            texture,
        }
    }

    fn run(&mut self, steps: usize) -> bool {
        let mut new_frame_available = false;

        for _ in 0..steps {
            if self.break_condition_active {
                let addr = u16::from_str_radix(&self.break_condition, 16);
                if self.system.cpu.registers.pc == addr.unwrap_or(0) {
                    self.paused = true;
                    self.break_condition_active = false;
                    break;
                }
            }

            match self.system.decode_instr_at_pc() {
                Ok(instr) => {
                    if self.trace.len() == 1024 {
                        self.trace.pop_front();
                    }

                    self.trace.push_back((self.system.cpu.registers.pc, instr.opcode))
                }
                Err(e) => error!("{}", e),
            }

            match self.system.tick() {
                Ok(true) => {
                    new_frame_available = true;
                    break;
                }
                Ok(false) => (),
                Err(e) => {
                    error!("{}", e);
                    self.paused = true;
                    break;
                }
            }
        }

        // Update disasaembly cache
        let mut data: Vec<u8> = Vec::new();
        for offset in 0..100 {
            data.push(self.system.bus.read(self.system.cpu.registers.pc + offset).unwrap())
        }

        self.dissasembly_cache.clear();
        let disasm = Disassembler::new(&data);
        let mut current_offset = 0;
        for _ in 0..10 {
            match disasm.decode(current_offset) {
                Ok(instr) => {
                    current_offset += instr.length;
                    self.dissasembly_cache.push(instr);
                }
                Err(e) => {
                    error!("{}", e);
                    break;
                }
            }
        }

        new_frame_available
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
