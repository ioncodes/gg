use std::collections::VecDeque;

use core::bus::{
    BankSelect, RomWriteProtection, MEMORY_REGISTER_CR_BANK_SELECT_0, MEMORY_REGISTER_CR_BANK_SELECT_1, MEMORY_REGISTER_CR_BANK_SELECT_2,
};
use core::system::{System, SystemState};
use core::vdp::{Color, INTERNAL_HEIGHT, INTERNAL_WIDTH, OFFSET_X, OFFSET_Y, VISIBLE_HEIGHT, VISIBLE_WIDTH};
use eframe::egui::scroll_area::ScrollBarVisibility;
use eframe::egui::{
    self, vec2, CentralPanel, Color32, ColorImage, ComboBox, Context, Image, Key, ScrollArea, SidePanel, TextureHandle, TextureOptions,
    Window,
};
use eframe::CreationContext;
use log::error;
use std::time::{Duration, Instant};
use z80::disassembler::Disassembler;
use z80::instruction::{Instruction, Opcode};

use crate::EmulatorSettings;

pub(crate) const SCALE: usize = 8;

#[derive(PartialEq, Debug)]
enum MemoryView {
    Rom,
    Ram,
    Sram,
    Vram,
    Cram,
}

pub(crate) struct Emulator {
    system: System,
    background_color: Color,
    dissasembly_cache: Vec<Instruction>,
    trace: VecDeque<(u16, Opcode)>,
    paused: bool,
    stepping: bool,
    debugger_enabled: bool,
    break_condition_active: bool,
    break_condition: String,
    internal_texture: TextureHandle,
    visible_texture: TextureHandle,
    memory_view: MemoryView,
    frame_time_cap: Duration,
    frame_time: Instant,
}

impl eframe::App for Emulator {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.frame_time.elapsed() > self.frame_time_cap {
            if self.paused && self.stepping {
                if self.run(1) {
                    self.render();
                }
                self.stepping = false;
            } else if !self.paused && !self.stepping {
                if self.run(50000) {
                    self.render();
                }
            }

            self.frame_time = Instant::now();
        }

        CentralPanel::default().show(ctx, |ui| {
            let image = Image::new(&self.visible_texture);
            let image = image.fit_to_exact_size(vec2((VISIBLE_WIDTH * SCALE) as f32, (VISIBLE_HEIGHT * SCALE) as f32));
            image.paint_at(ui, ui.ctx().screen_rect());
        });

        if self.debugger_enabled {
            self.draw_debugger(ctx);
        }

        self.handle_input(ctx);

        ctx.request_repaint();
    }
}

impl Emulator {
    pub(crate) fn new(cc: &CreationContext, emulator_settings: EmulatorSettings) -> Emulator {
        let mut system = System::new(emulator_settings.lua, emulator_settings.emulate_sms);
        system.set_abort_on_io_operation_behavior(false); // Let's only log invalid ports
        system.bus.set_rom_write_protection(RomWriteProtection::Abort);

        if emulator_settings.cpu_test {
            system.load_cartridge(emulator_settings.cartridge.as_ref());
            system.disable_bios();
        } else {
            system.load_bios(&emulator_settings.bios);
            system.load_cartridge(emulator_settings.cartridge.as_ref());
        }

        let internal_texture = cc.egui_ctx.load_texture(
            "internal_frame",
            ColorImage::new([INTERNAL_WIDTH, INTERNAL_HEIGHT], Color32::BLACK),
            TextureOptions::NEAREST,
        );
        let visible_texture = cc.egui_ctx.load_texture(
            "visible_frame",
            ColorImage::new([VISIBLE_WIDTH, VISIBLE_HEIGHT], Color32::BLACK),
            TextureOptions::NEAREST,
        );

        Emulator {
            system,
            dissasembly_cache: Vec::new(),
            break_condition_active: false,
            break_condition: String::new(),
            background_color: (0, 0, 0, 0),
            paused: true,
            debugger_enabled: true,
            trace: VecDeque::with_capacity(1024),
            stepping: false,
            internal_texture,
            visible_texture,
            memory_view: MemoryView::Rom,
            frame_time_cap: Duration::from_micros(16667 / 30),
            frame_time: Instant::now(),
        }
    }

    fn handle_input(&mut self, ctx: &Context) {
        if ctx.input(|i| i.key_pressed(Key::F1)) {
            self.debugger_enabled = !self.debugger_enabled;
            if !self.debugger_enabled {
                self.paused = false;
            } else {
                self.paused = true;
            }
        }

        ctx.input(|i| {
            if i.key_down(Key::Enter) {
                self.system.bus.joysticks[0].set_start(true);
            } else {
                self.system.bus.joysticks[0].set_start(false);
            }

            if i.key_down(Key::A) {
                self.system.bus.joysticks[0].set_input_button1(true);
            } else {
                self.system.bus.joysticks[0].set_input_button1(false);
            }

            if i.key_down(Key::S) {
                self.system.bus.joysticks[0].set_input_button2(true);
            } else {
                self.system.bus.joysticks[0].set_input_button2(false);
            }

            if i.key_down(Key::ArrowUp) {
                self.system.bus.joysticks[0].set_input_up(true);
            } else {
                self.system.bus.joysticks[0].set_input_up(false);
            }

            if i.key_down(Key::ArrowDown) {
                self.system.bus.joysticks[0].set_input_down(true);
            } else {
                self.system.bus.joysticks[0].set_input_down(false);
            }

            if i.key_down(Key::ArrowLeft) {
                self.system.bus.joysticks[0].set_input_left(true);
            } else {
                self.system.bus.joysticks[0].set_input_left(false);
            }

            if i.key_down(Key::ArrowRight) {
                self.system.bus.joysticks[0].set_input_right(true);
            } else {
                self.system.bus.joysticks[0].set_input_right(false);
            }
        });
    }

    fn draw_debugger(&mut self, ctx: &Context) {
        Window::new("Internal Frame")
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
                    ui.add(Image::new(&self.internal_texture));
                });
            });

        Window::new("Debugger").resizable(false).show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("Resume").clicked() {
                    self.paused = false;
                }
                if ui.button("Break").clicked() {
                    self.paused = true;
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
                ui.label("Flags: SZ-H-PNC");
                ui.label(format!("       {:08b}", self.system.cpu.registers.f.bits()));
            });

            ui.separator();

            ui.heading("VDP Registers");

            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label(format!("V: {:02x}", self.system.vdp.v));
                    ui.label(format!("H: {:02x}", self.system.vdp.h));
                });
                ui.label(format!("R00: {:08b}", self.system.vdp.registers.r0));
                ui.label(format!("R01: {:08b}", self.system.vdp.registers.r1));
                ui.label(format!("R02: {:08b}", self.system.vdp.registers.r2));
                ui.label(format!("R03: {:08b}", self.system.vdp.registers.r3));
                ui.label(format!("R04: {:08b}", self.system.vdp.registers.r4));
                ui.label(format!("R05: {:08b}", self.system.vdp.registers.r5));
                ui.label(format!("R06: {:08b}", self.system.vdp.registers.r6));
                ui.label(format!("R07: {:08b}", self.system.vdp.registers.r7));
                ui.label(format!("R08: {:08b}", self.system.vdp.registers.r8));
                ui.label(format!("R09: {:08b}", self.system.vdp.registers.r9));
                ui.label(format!("R10: {:08b}", self.system.vdp.registers.r10));
                ui.label(format!("Address: {:04x}", self.system.vdp.registers.address));
            });

            ui.separator();

            ui.heading("CPU Interrupts");
            ui.vertical(|ui| {
                ui.label(format!("IFF1: {}", self.system.cpu.registers.iff1));
                ui.label(format!("IFF2: {}", self.system.cpu.registers.iff2));
                ui.label(format!("Ignoring IRQs: {}", self.system.cpu.ignore_next_irq));
                ui.label(format!("IM: {}", self.system.cpu.interrupt_mode.to_u8()));
            });

            ui.separator();

            ui.heading("VDP Interrupts");
            ui.vertical(|ui| {
                ui.label(format!("VBlank: {}", self.system.vdp.vblank_irq_pending()));
            });
        });

        Window::new("CPU Mappings").resizable(false).show(ctx, |ui| {
            let rom0_bank = self.system.bus.read(MEMORY_REGISTER_CR_BANK_SELECT_0);
            let rom1_bank = self.system.bus.read(MEMORY_REGISTER_CR_BANK_SELECT_1);
            let rom2_bank = self.system.bus.read(MEMORY_REGISTER_CR_BANK_SELECT_2);
            let sram_active = self.system.bus.is_sram_bank_active();
            let sram_bank = self.system.bus.fetch_bank(BankSelect::Bank2);

            ui.label(format!(
                "SRAM Bank #{:02x}: {:08x} [{}]",
                sram_bank,
                self.system.bus.translate_address_to_real(0x8000).unwrap_or(0x69),
                if sram_active { "Active" } else { "Inactive" }
            ));
            ui.label(format!(
                "ROM Bank #{:02x}: {:08x}",
                rom0_bank.unwrap_or(0),
                self.system.bus.translate_address_to_real(0x0000).unwrap_or(0x69)
            ));
            ui.label(format!(
                "ROM Bank #{:02x}: {:08x}",
                rom1_bank.unwrap_or(0),
                self.system.bus.translate_address_to_real(0x4000).unwrap_or(0x69)
            ));
            ui.label(format!(
                "ROM Bank #{:02x}: {:08x}",
                rom2_bank.unwrap_or(0),
                self.system.bus.translate_address_to_real(0x8000).unwrap_or(0x69)
            ));
        });

        Window::new("Memory").resizable(false).min_width(500.0).show(ctx, |ui| {
            ComboBox::from_label("Source")
                .selected_text(format!("{:?}", self.memory_view))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.memory_view, MemoryView::Rom, "ROM");
                    ui.selectable_value(&mut self.memory_view, MemoryView::Ram, "RAM");
                    ui.selectable_value(&mut self.memory_view, MemoryView::Sram, "SRAM");
                    ui.selectable_value(&mut self.memory_view, MemoryView::Vram, "VRAM");
                    ui.selectable_value(&mut self.memory_view, MemoryView::Cram, "CRAM");
                });

            ui.add_space(3.0);
            ui.label("         00 01 02 03 04 05 06 07 08 09 0a 0b 0c 0d 0e 0f");

            let range = match self.memory_view {
                MemoryView::Rom => (0x0000..0xc000).into_iter(),
                MemoryView::Ram => (0xc000..0xffff).into_iter(),
                MemoryView::Sram => (0x0000..0x8000).into_iter(),
                MemoryView::Vram => (0x0000..0x4000).into_iter(),
                MemoryView::Cram => (0x0000..0x40).into_iter(),
            };

            ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
                ui.vertical(|ui| {
                    for base_addr in range.step_by(16) {
                        let mut line = format!("0x{:04x} |", base_addr);

                        for offset in 0..16 {
                            let addr = base_addr + offset;

                            let value = match self.memory_view {
                                MemoryView::Rom | MemoryView::Ram => self.system.bus.read(addr as u16).unwrap_or(0x69),
                                MemoryView::Sram => self.system.bus.sram.read(addr as u16),
                                MemoryView::Vram => self.system.vdp.vram.read(addr as u16),
                                MemoryView::Cram => self.system.vdp.cram.read(addr as u16),
                            };

                            line += &format!(" {:02x}", value);
                        }

                        ui.label(line);
                    }
                });
            });
        });

        Window::new("SDSC Debug Console")
            .resizable(false)
            .default_open(false)
            .show(ctx, |ui| {
                ScrollArea::vertical().stick_to_bottom(true).max_height(200.0).show(ui, |ui| {
                    ui.label(&self.system.bus.sdsc_console.buffer);
                });
            });

        SidePanel::right("Right Panel").show(ctx, |ui| {
            ui.heading("Disassembly");
            let mut addr = self.system.cpu.registers.pc;
            for instr in &self.dissasembly_cache {
                ui.label(format!("{:04x}: {}", addr, instr.opcode));
                addr += instr.length as u16;
            }

            ui.separator();
            ui.heading("Trace");

            ScrollArea::vertical()
                .stick_to_bottom(true)
                .scroll_bar_visibility(ScrollBarVisibility::AlwaysHidden)
                .show(ui, |ui| {
                    for instr in &self.trace {
                        ui.label(format!("{:04x}: {}", instr.0, instr.1));
                    }
                });
        });
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
                Ok(SystemState { frame_ready: true, .. }) => {
                    new_frame_available = true;
                    break;
                }
                Ok(SystemState { frame_ready: false, .. }) => (),
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
                    self.paused = true;
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
                texture.push(Color32::from_rgba_premultiplied(r, g, b, a));
            }
        }

        let image = ColorImage {
            size: [INTERNAL_WIDTH, INTERNAL_HEIGHT],
            pixels: texture,
        };

        self.internal_texture.set(image, TextureOptions::NEAREST);

        let mut texture: Vec<Color32> = Vec::new();

        for y in 0..VISIBLE_HEIGHT {
            for x in 0..VISIBLE_WIDTH {
                let (r, g, b, a) = frame_src[(y + OFFSET_Y) * INTERNAL_WIDTH + (x + OFFSET_X)];
                texture.push(Color32::from_rgba_premultiplied(r, g, b, a));
            }
        }

        let image = ColorImage {
            size: [VISIBLE_WIDTH, VISIBLE_HEIGHT],
            pixels: texture,
        };

        self.visible_texture.set(image, TextureOptions::NEAREST);

        self.background_color = background_color;
    }
}
