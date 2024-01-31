use std::collections::VecDeque;

use crate::{bus::Bus, io::IoMode, memory::Memory};
use bitmatch::bitmatch;
use log::{debug, error, trace, info};

// todo: ????
const H_COUNTER_COUNT: u8 = 171;
const NTSC_SCANLINE_COUNT: u16 = 262; // 60 frames
const V_COUNTER_PORT: u8 = 0x7e;
const CONTROL_PORT: u8 = 0xbf;
const DATA_PORT: u8 = 0xbe;
// const PAL_SCANLINE_COUNT: u8 = 312;

pub const INTERNAL_WIDTH: usize = 256;
pub const INTERNAL_HEIGHT: usize = 224;

pub type Color = (u8, u8, u8, u8);

#[derive(Debug)]
pub(crate) struct Pattern {
    pub(crate) data: [[Color; 8]; 8],
}

impl Pattern {
    pub(crate) fn new() -> Pattern {
        Pattern { data: [[(0, 0, 0, 0); 8]; 8] }
    }

    pub(crate) fn set_pixel(&mut self, x: u8, y: u8, color: Color) {
        self.data[y as usize][x as usize] = color;
    }

    pub(crate) fn get_pixel(&self, x: u8, y: u8) -> Color {
        self.data[y as usize][x as usize]
    }
}

pub(crate) enum Mode {
    VramWrite,
    CramWrite,
    None,
}

#[derive(Default, Debug)]
pub(crate) struct Registers {
    pub(crate) r0: u8,
    pub(crate) r1: u8,
    pub(crate) r2: u8,
    pub(crate) r3: u8,
    pub(crate) r4: u8,
    pub(crate) r5: u8,
    pub(crate) r6: u8,
    pub(crate) r7: u8,
    pub(crate) r8: u8,
    pub(crate) r9: u8,
    pub(crate) r10: u8,
    pub(crate) address: u16,
}

pub struct Vdp {
    pub(crate) v: u8,
    pub(crate) h: u8,
    v_2nd_loop: bool,
    h_2nd_loop: bool,
    control_data: VecDeque<u8>,
    registers: Registers,
    pub(crate) vram: Memory,
    pub(crate) cram: Memory,
    mode: Mode,
    pub(crate) vram_dirty: bool
}

impl Vdp {
    pub(crate) fn new() -> Vdp {
        Vdp {
            v: 0,
            h: 0,
            v_2nd_loop: false,
            h_2nd_loop: false,
            control_data: VecDeque::new(),
            registers: Registers::default(),
            vram: Memory::new(16 * 1024, 0x0000),
            cram: Memory::new(64, 0x0000),
            mode: Mode::None,
            vram_dirty: false
        }
    }

    pub(crate) fn tick(&mut self, bus: &mut Bus) {
        self.handle_io(bus);
        self.handle_control_data();
        self.handle_counters();
    }

    pub(crate) fn is_vblank(&self) -> bool {
        self.v == 0
    }

    pub(crate) fn is_hblank(&self) -> bool {
        self.h == 0
    }

    pub fn render_background(&mut self) -> (Color, Vec<Color>) {        
        let background_color = self.read_palette_entry(0);

        let mut pixels = vec![(0, 0, 0, 0); INTERNAL_WIDTH * INTERNAL_HEIGHT];

        // if !(background_color.0 == 0 && background_color.1 == 0 && background_color.2 == 0) {
        //     debug!("Background color => r:{:02x} g:{:02x} b:{:02x}", background_color.0, background_color.1, background_color.2);
        // }
        // debug!("{:02x}", self.vram.read(0x3a52));

        debug!("Rendering background");
        
        for row in 0..28 {
            for column in 0..32 {
                let name_table_addr = self.get_name_table_addr(column, row);
                //info!("Name table base address ({},{}): {:04x} => {:04x}", column, row, name_table_addr, self.vram.read_word(name_table_addr));

                // The pattern base address is defined by the pattern generator table (which always starts at 0)
                // Rendering every pattern starting at 0 would yield a classic tile map
                // Source: As per Sega Game Gear Hardware Reference Manual, page 26
                // Source: Chapter 6 "VDP Manual", subchapter 3 "Standard VRAM mapping"
                let pattern_base_addr = self.vram.read_word(name_table_addr);
                let pattern_base_addr = pattern_base_addr & 0b0000_0001_1111_1111;
                let pattern_base_addr = pattern_base_addr * 32;

                // pattern_base_addr = character/tile location in VRAM.
                // Each character/tile is 8x8 pixels, and each pixel consists of 4 bits.
                // So each character/tile is 32 bytes (64 pixels).
                
                let mut pattern = Pattern::new();
                for line in 0..8 {
                    let line_base_addr = pattern_base_addr + (line * 4);
                    let line_data1 = self.vram.read(line_base_addr + 0);
                    let line_data2 = self.vram.read(line_base_addr + 1);
                    let line_data3 = self.vram.read(line_base_addr + 2);
                    let line_data4 = self.vram.read(line_base_addr + 3);

                    for bit in 0..8 {
                        let mut color: u8 = 0;
                        if line_data1 & (1 << bit) != 0 {
                            color |= 0b0000_0001;
                        }
                        if line_data2 & (1 << bit) != 0 {
                            color |= 0b0000_0010;
                        }
                        if line_data3 & (1 << bit) != 0 {
                            color |= 0b0000_0100;
                        }
                        if line_data4 & (1 << bit) != 0 {
                            color |= 0b0000_1000;
                        }
                        let color = if color == 0 {
                            (0, 0, 0, 0) // transparent
                        } else {
                            self.read_palette_entry(color as u16)
                        };
                        pattern.set_pixel(7 - bit, line as u8, color);
                    }
                }

                for y in 0..8 {
                    for x in 0..8 {
                        let color = pattern.get_pixel(x, y);
                        let idx = (row * 8 + y) as usize * INTERNAL_WIDTH + (column * 8 + x) as usize;
                        pixels[idx] = color;
                    }
                }
            }
        }

        self.vram_dirty = false;

        (background_color, pixels)
    }

    #[bitmatch]
    fn handle_control_data(&mut self) {
        loop {
            if self.control_data.len() >= 2 {
                trace!("VDP control type: {:08b}", self.control_data[1]);

                #[bitmatch]
                match self.control_data[1] {
                    "1000_????" => {
                        /*
                         * Set VDP register:
                         * To set data in a VDP register, the data is inputted in  the first byte. The second byte is used
                         * to indicate the register where the data is to be transferred.
                         * The bottom four bits (R3 to R0) of the second byte designate the data transfer destination
                         * registers (#0 to #10). b7 must be“1”and b6 to b4 must be“0”.
                         */

                        let value = self.control_data.pop_front().unwrap();
                        let register = self.control_data.pop_front().unwrap() & 0b0000_1111;
                        match register {
                            0b0000_0000 => self.registers.r0 = value,
                            0b0000_0001 => self.registers.r1 = value,
                            0b0000_0010 => self.registers.r2 = value,
                            0b0000_0011 => self.registers.r3 = value,
                            0b0000_0100 => self.registers.r4 = value,
                            0b0000_0101 => self.registers.r5 = value,
                            0b0000_0110 => self.registers.r6 = value,
                            0b0000_0111 => self.registers.r7 = value,
                            0b0000_1000 => self.registers.r8 = value,
                            0b0000_1001 => self.registers.r9 = value,
                            0b0000_1010 => self.registers.r10 = value,
                            // registers 11..15 have no effect when written to
                            _ => error!("Invalid VDP register: {:08b}", register),
                        }
                    }
                    "01??_????" => {
                        // Write to VRAM requests are denoted by a 0b01 at bit 15 and 14 of the 2nd byte in the control data.
                        // The rest adds up to a VRAM address.
                        let control_byte1 = self.control_data.pop_front().unwrap();
                        let control_byte2 = self.control_data.pop_front().unwrap();
                        let address = (((control_byte2 & 0b0011_1111) as u16) << 8) | (control_byte1 as u16);
                        self.registers.address = address;
                        debug!("Setting address register to {:04x}", address);
                        self.mode = Mode::VramWrite;
                    }
                    "11??_????" => {
                        // Write to CRAM requests are denoted by a 0b11 at bit 15 and 14 of the 2nd byte in the control data.
                        // The rest adds up to a CRAM address.
                        let control_byte1 = self.control_data.pop_front().unwrap();
                        let control_byte2 = self.control_data.pop_front().unwrap();
                        let address = (((control_byte2 & 0b0011_1111) as u16) << 8) | (control_byte1 as u16);
                        self.registers.address = address;
                        debug!("Setting address register to {:04x}", address);
                        self.mode = Mode::CramWrite;
                    }
                    _ => error!("Seemingly unimplemented control data found: {:02x?}", self.control_data),
                }
            } else {
                break;
            }
        }
    }

    fn handle_io(&mut self, bus: &mut Bus) {
        if let Some(_) = bus.io.pop(V_COUNTER_PORT, false) {
            trace!("I/O request for V counter: {:02x}", self.v);
            bus.push_io_data(V_COUNTER_PORT, self.v, IoMode::Read, true);
        }

        if let Some(buffer) = bus.io.pop_all(CONTROL_PORT) {
            debug!("Received buffer via I/O control port ({:02x}): {:02x?}", CONTROL_PORT, buffer);
            self.control_data.extend(buffer);
        }

        if let Some(mut buffer) = bus.io.pop_all(DATA_PORT) {
            debug!("Received buffer via I/O data port ({:02x}): {:02x?}", DATA_PORT, buffer);

            match self.mode {
                Mode::VramWrite => {
                    // Write data bytes to VRAM

                    let bytes_to_write = buffer.len();

                    loop {
                        if buffer.is_empty() {
                            break;
                        }

                        let value = buffer.pop_front().unwrap();
                        self.vram.write(self.registers.address, value);

                        self.registers.address += 1;
                        if self.registers.address >= 0x4000 {
                            self.registers.address = self.registers.address - 0x4000;
                        }
                    }

                    debug!("Wrote {} bytes to {:04x} @ VRAM", bytes_to_write, self.registers.address);

                    // Force a rerender
                    self.vram_dirty = true;
                }
                Mode::CramWrite => {
                    // Write data bytes to CRAM
                    // "If the address register exceeds the CRAM size (32 or 64 bytes), the high bits are ignored so it will always address CRAM;""
                    // "for example, address $1000 wil read from CRAM address $00.""

                    // Even writes to CRAM get cached in a latch, whereas odd writes write to the CRAM along with the latched byte
                    // todo: isn't this just a copy routine basically? aka, what we did before the rewrite?

                    let mut latch: Option<u8> = None;
                    loop {
                        if buffer.is_empty() {
                            break;
                        }

                        let address = self.registers.address & 0b0000_0000_0111_1111;

                        if address % 2 == 0 {
                            latch = buffer.pop_front();
                        } else {
                            let value = buffer.pop_front().unwrap();
                            if let Some(latched_value) = latch {
                                self.cram.write(address, latched_value);
                            }
                            self.cram.write(address - 1, value);
                            debug!(
                                "Wrote 1-2 bytes [current: {:02x}, latched: {:02x?}] to {:04x} @ CRAM",
                                value,
                                latch,
                                address - 1
                            );
                        }

                        self.registers.address += 1;
                        if self.registers.address >= 0x40 {
                            self.registers.address = self.registers.address - 0x40;
                        }
                    }

                    // Force a rerender
                    self.vram_dirty = true;
                }
                Mode::None => {
                    error!("Received byte on data port ({:02x}) without being in a specific mode", DATA_PORT);
                }
            }
        }
    }

    fn handle_counters(&mut self) {
        /*
        The V counter counts up from 00h to EAh, then it jumps back to E5h and
        continues counting up to FFh. This allows it to cover the entire 262 line
        display.

        The H counter counts up from 00h to E9h, then it jumps back to 93h and
        continues counting up to FFh. This allows it to cover an entire 342 pixel
        line.
        */

        // todo: Generate vblank interrupt

        if self.h == 0xe9 && !self.h_2nd_loop {
            self.h = 0x93;
            self.h_2nd_loop = true;
        } else if self.h == 0xff && self.h_2nd_loop {
            self.h = 0x00;
            self.h_2nd_loop = false;

            if self.v == 0xea && !self.v_2nd_loop {
                self.v = 0xe5;
                self.v_2nd_loop = true;
            } else if self.v == 0xff && self.v_2nd_loop {
                self.v = 0x00;
                self.v_2nd_loop = false;
            } else {
                self.v += 1;
            }
        } else {
            self.h += 1;
        }
    }

    fn get_name_table_addr(&self, x: u8, y: u8) -> u16 {
        /*
         *  VRAM address bus layout for name table fetch
         *  MSB             LSB
         *  --bb byyy yyxx xxxw : b= Table base address, y= Row, x= Column
         *  ---- -x-- ---- ---- : x= Mask bit (bit 0 of register $02)
         */

        let mut address: u16 = ((self.registers.r2 & 0b0000_1110) as u16) << 10;
        address |= ((y & 0b0001_1111) as u16) << 6;
        address |= ((x & 0b0001_1111) as u16) << 1;
        address
    }

    fn read_palette_entry(&self, index: u16) -> (u8, u8, u8, u8) {
        trace!("Reading palette entry at {:04x}", index);

        // Convert palette index to absolute address in CRAM
        // Each palette entry is 2 bytes
        let address = index * 2;

        let p1 = self.cram.read(address);
        let p2 = self.cram.read(address + 1);

        // Shifting these values by 4 bits to the left gives us the actual color value in 8bit color space
        let palette = ((p1 as u16) << 8) | (p2 as u16);
        let r = ((palette & 0b0000_0000_0000_1111) << 4) as u8;
        let g = (((palette & 0b0000_0000_1111_0000) << 4) >> 4) as u8;
        let b = (((palette & 0b0000_1111_0000_0000) << 4) >> 8) as u8;

        (r, g, b, 0xff)
    }
}

impl std::fmt::Display for Vdp {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "V: {:02x}  H: {:02x}\n", self.v, self.h)?;
        write!(f, "r0: {:02x}  r1: {:02x}  r2: {:02x}  r3: {:02x}  r4: {:02x}  r5: {:02x}  r6: {:02x}  r7: {:02x}  r8: {:02x}  r9: {:02x}  r10: {:02x}  address: {:04x}\n",
            self.registers.r0,
            self.registers.r1,
            self.registers.r2,
            self.registers.r3,
            self.registers.r4,
            self.registers.r5,
            self.registers.r6,
            self.registers.r7,
            self.registers.r8,
            self.registers.r9,
            self.registers.r10,
            self.registers.address)?;

        let value = self.vram.read(self.registers.address);
        write!(f, "VRAM @ {:04x}: {:02x}", self.registers.address, value)?;

        Ok(())
    }
}
