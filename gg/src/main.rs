mod emulator;

use core::vdp::{VISIBLE_HEIGHT, VISIBLE_WIDTH};
use eframe::egui::{FontFamily, FontId, Style, TextStyle, ViewportBuilder, Visuals};
use eframe::NativeOptions;
use emulator::{Emulator, SCALE};

fn main() {
    let native_options = NativeOptions {
        viewport: ViewportBuilder::default()
            .with_inner_size([(VISIBLE_WIDTH * SCALE) as f32, (VISIBLE_HEIGHT * SCALE) as f32])
            .with_resizable(false),
        vsync: false,
        ..Default::default()
    };

    let _ = eframe::run_native(
        "geegee",
        native_options,
        Box::new(|cc| {
            let style = Style {
                visuals: Visuals::dark(),
                text_styles: [
                    (TextStyle::Body, FontId::new(14.0, FontFamily::Monospace)),
                    (TextStyle::Button, FontId::new(14.0, FontFamily::Monospace)),
                    (TextStyle::Heading, FontId::new(16.0, FontFamily::Monospace)),
                    (TextStyle::Monospace, FontId::new(14.0, FontFamily::Monospace)),
                ]
                .into(),
                ..Style::default()
            };
            cc.egui_ctx.set_style(style);
            Box::new(Emulator::new(cc))
        }),
    );
}
