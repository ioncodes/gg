mod emulator;

use clap::Parser;
use core::vdp::{VISIBLE_HEIGHT, VISIBLE_WIDTH};
use eframe::egui::{FontFamily, FontId, Style, TextStyle, ViewportBuilder, Visuals};
use eframe::NativeOptions;
use emulator::{Emulator, SCALE};
use env_logger::{Builder, Target};
use log::{info, Level};
use std::fs::File;
use std::io;
use std::io::Read;
use zip::ZipArchive;

#[derive(Parser, Debug)]
struct Args {
    #[arg(long)]
    bios: String,

    #[arg(long)]
    rom: String,

    #[arg(long)]
    lua: Option<String>,

    #[arg(long, default_value_t = false)]
    cpu_test: bool,

    #[arg(long, default_value_t = String::from("info"))]
    log_level: String,

    #[arg(long, default_value_t = false)]
    log_to_file: bool,
}

struct EmulatorSettings {
    bios: Vec<u8>,
    cartridge: Vec<u8>,
    cartridge_name: String,
    lua: Option<String>,
    emulate_sms: bool,
    cpu_test: bool,
}

fn main() {
    let args = Args::parse();

    match &args.log_level {
        level if level == "trace" => setup_logging(true, false, args.log_to_file),
        level if level == "debug" => setup_logging(false, true, args.log_to_file),
        _ => setup_logging(false, false, false),
    }

    let settings = get_emulator_settings(&args);

    let native_options = NativeOptions {
        viewport: ViewportBuilder::default()
            .with_inner_size([(VISIBLE_WIDTH * SCALE) as f32, (VISIBLE_HEIGHT * SCALE) as f32])
            .with_resizable(true),
        vsync: false,
        ..Default::default()
    };

    let _ = eframe::run_native(
        format!("geegee - {}", settings.cartridge_name).as_str(),
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
            Box::new(Emulator::new(cc, settings))
        }),
    );
}

fn get_emulator_settings(args: &Args) -> EmulatorSettings {
    let (is_sms, cartridge, filename) = if args.cpu_test {
        (
            true,
            Vec::from(include_bytes!("../../external/test_roms/zexdoc.sms")),
            "zexdoc.sms".to_string(),
        )
    } else {
        let (mut file, path) = if args.rom.ends_with(".zip") {
            let file = File::open(&args.rom).unwrap();
            let filename = unzip_rom(file);
            info!("Unzipped {} to {}", &args.rom, filename);
            (File::open(&filename).unwrap(), filename)
        } else {
            (File::open(&args.rom).unwrap(), args.rom.clone())
        };

        let mut buffer: Vec<u8> = Vec::new();
        let _ = file.read_to_end(&mut buffer).unwrap();
        let emulate_sms = path.ends_with(".sms") || args.rom.contains("[S]");
        (emulate_sms, buffer, path)
    };

    let lua = if let Some(lua) = &args.lua {
        let mut file = File::open(lua).unwrap();
        let mut script = String::new();
        let _ = file.read_to_string(&mut script).unwrap();
        Some(script)
    } else {
        None
    };
    let mut file = File::open(&args.bios).unwrap();
    let mut bios: Vec<u8> = Vec::new();
    let _ = file.read_to_end(&mut bios).unwrap();

    EmulatorSettings {
        bios,
        cartridge,
        lua,
        emulate_sms: is_sms,
        cpu_test: args.cpu_test,
        cartridge_name: filename,
    }
}

fn unzip_rom(file: File) -> String {
    let mut archive = ZipArchive::new(file).unwrap();
    let mut rom = archive.by_index(0).unwrap();

    let filepath = match rom.enclosed_name() {
        Some(name) => name.to_owned(),
        None => panic!("No file found in zip archive"),
    };
    let tempfolder = std::env::temp_dir();
    let filepath = tempfolder.join(&filepath);

    let filename = filepath.to_str().unwrap().to_owned();

    let mut unpacked_file = File::create(&filepath).unwrap();
    io::copy(&mut rom, &mut unpacked_file).unwrap();

    filename
}

fn setup_logging(enable_trace: bool, enable_debug: bool, enable_log_to_file: bool) {
    let mut default_log_level = Level::Info.to_level_filter();

    let mut target = Target::Stderr;

    if enable_trace {
        default_log_level = Level::Trace.to_level_filter();
    }

    if enable_debug {
        default_log_level = Level::Debug.to_level_filter();
    }

    if enable_log_to_file {
        target = Target::Pipe(Box::new(File::create("trace.log").expect("Can't create file")));
    }

    Builder::new()
        .filter(Some("core"), default_log_level)
        .filter(Some("gg"), default_log_level)
        .target(target)
        .format_timestamp(None)
        .init();
}
