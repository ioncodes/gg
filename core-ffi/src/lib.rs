use lazy_static::lazy_static;
use std::sync::Mutex;
use core::system::System;

lazy_static! {
    static ref SYSTEM: Mutex<System> = Mutex::new(System::new(None));
}

#[no_mangle]
pub extern "C" fn gg_init() {
    SYSTEM.lock().unwrap().load_bios(include_bytes!("../../external/majbios.gg"));
    SYSTEM.lock().unwrap().load_cartridge(include_bytes!("../../external/sonic2.gg")[..0xc000].as_ref());
}

#[no_mangle]
pub extern "C" fn gg_tick(buffer: *mut std::ffi::c_uchar) -> bool {
    let mut system = SYSTEM.lock().unwrap();
    let redraw = system.tick();

    if redraw {
        let (background_color, frame) = system.render();

        let mut frame_rgb: Vec<u8> = Vec::new();
        for color in frame {
            let (r, g, b, _) = if color == (0, 0, 0, 0) { background_color } else { color };
            frame_rgb.extend([r, g, b]);
        }

        unsafe { frame_rgb.as_ptr().copy_to_nonoverlapping(buffer, frame_rgb.len()); }

        return true;
    }

    false
}