mod emulator;

use eframe::NativeOptions;
use eframe::egui::{FontFamily, FontId, Style, TextStyle, ViewportBuilder, Visuals};
use emu::vdp::{INTERNAL_HEIGHT, INTERNAL_WIDTH};
use emulator::{Emulator, SCALE};
use env_logger::Builder;
use log::Level;

fn main() {
    initialize_logging();
    let native_options = NativeOptions {
        viewport: ViewportBuilder::default()
            .with_inner_size([(INTERNAL_WIDTH * SCALE) as f32, (INTERNAL_HEIGHT * SCALE) as f32])
            .with_resizable(false),
        vsync: true,
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
}
