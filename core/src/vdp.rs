use std::collections::VecDeque;

use crate::{bus::Bus, error::GgError, io::Controller, memory::Memory};
use log::{debug, error, trace};

// todo: ????
//const H_COUNTER_COUNT: u8 = 171;
//const NTSC_SCANLINE_COUNT: u16 = 262; // 60 frames
pub(crate) const V_COUNTER_PORT: u8 = 0x7e;
pub(crate) const CONTROL_PORT: u8 = 0xbf;
pub(crate) const DATA_PORT: u8 = 0xbe;
// const PAL_SCANLINE_COUNT: u8 = 312;

pub const INTERNAL_WIDTH: usize = 256;
pub const INTERNAL_HEIGHT: usize = 224;

pub type Color = (u8, u8, u8, u8);

enum IoMode {
    VramRead,
    VramWrite,
    CramWrite,
    None
}

#[derive(Debug)]
pub(crate) struct Pattern {
    pub(crate) data: [[Color; 8]; 8],
}

impl Pattern {
    pub(crate) fn new() -> Pattern {
        Pattern {
            data: [[(0, 0, 0, 0); 8]; 8],
        }
    }

    pub(crate) fn set_pixel(&mut self, x: u8, y: u8, color: Color) {
        self.data[y as usize][x as usize] = color;
    }

    pub(crate) fn get_pixel(&self, x: u8, y: u8) -> Color {
        self.data[y as usize][x as usize]
    }
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
    cram_latch: Option<u8>,
    pub(crate) vram_dirty: bool,
    pub(crate) buffer: Vec<u8>,
    io_mode: IoMode
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
            cram_latch: None,
            vram_dirty: false,
            buffer: Vec::new(),
            io_mode: IoMode::None
        }
    }

    pub(crate) fn tick(&mut self, _bus: &mut Bus) {
        self.handle_counters();
    }

    pub(crate) fn is_vblank(&self) -> bool {
        self.v == 0
    }

    pub fn render_background(&mut self) -> (Color, Vec<Color>) {
        let background_color = self.read_palette_entry(0);

        let mut pixels = vec![(0, 0, 0, 0); INTERNAL_WIDTH * INTERNAL_HEIGHT];

        debug!("Rendering background");

        for row in 0..28 {
            for column in 0..32 {
                let name_table_addr = self.get_name_table_addr(column, row);

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

    fn process_control_data(&mut self) {
        match self.control_data[1] & 0b1100_0000 {
            0b1000_0000 => {
                /*
                 * Set VDP register:
                 * To set data in a VDP register, the data is inputted in the first byte. The second byte is used
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
            0b0000_0000 => {
                // Read from VRAM requests are denoted by a 0b00 at bit 15 and 14 of the 2nd byte in the control data.
                // The rest adds up to a VRAM address.
                let control_byte1 = self.control_data.pop_front().unwrap();
                let control_byte2 = self.control_data.pop_front().unwrap();
                let address = (((control_byte2 & 0b0011_1111) as u16) << 8) | (control_byte1 as u16);
                let value = self.vram.read(address);

                self.registers.address += 1;
                self.registers.address %= 0x3fff; // ensure we wrap around
                
                debug!("Setting address register to {:04x}", address);
                self.buffer.push(value);

                self.io_mode = IoMode::VramRead;
            }
            0b0100_0000 => {
                // Write to VRAM requests are denoted by a 0b01 at bit 15 and 14 of the 2nd byte in the control data.
                // The rest adds up to a VRAM address.
                let control_byte1 = self.control_data.pop_front().unwrap();
                let control_byte2 = self.control_data.pop_front().unwrap();
                let address = (((control_byte2 & 0b0011_1111) as u16) << 8) | (control_byte1 as u16);
                self.registers.address = address;
                debug!("Setting address register to {:04x}", address);

                self.io_mode = IoMode::VramWrite;
            }
            0b1100_0000 => {
                // Write to CRAM requests are denoted by a 0b11 at bit 15 and 14 of the 2nd byte in the control data.
                // The rest adds up to a CRAM address.
                let control_byte1 = self.control_data.pop_front().unwrap();
                let control_byte2 = self.control_data.pop_front().unwrap();
                let address = (((control_byte2 & 0b0011_1111) as u16) << 8) | (control_byte1 as u16);
                self.registers.address = address;
                debug!("Setting address register to {:04x}", address);

                self.io_mode = IoMode::CramWrite;
            }
            _ => error!("Seemingly unimplemented control data found: {:08b}", self.control_data[1]),
        }
    }

    fn cram_write(&mut self, value: u8) {
        let address = self.registers.address & 0b0000_0000_0111_1111;
        if address % 2 == 0 {
            self.cram_latch = Some(value);
        } else {
            let latched_value = self.cram_latch.unwrap();
            self.cram.write(self.registers.address, latched_value);
            self.cram.write(self.registers.address - 1, value);
            self.cram_latch = None;

            // Force a rerender
            self.vram_dirty = true; // todo: lol
        }

        self.registers.address += 1;
        self.registers.address %= 0x3f;
    }

    fn vram_write(&mut self, value: u8) {
        self.vram.write(self.registers.address, value);

        self.registers.address += 1;
        self.registers.address %= 0x3fff; // ensure we wrap around

        // Force a rerender
        self.vram_dirty = true;
    }

    fn status(&self) -> u8 {
        // The VBlank flag is set when a VBlank interrupt has just occurred. 
        //   An interrupt handler can use this to tell the difference between VBlank interrupts and line interrupts.
        // The sprite overflow flag is set when sprite overflow has occurred.
        // The sprite collision flag will be set if any sprites overlap - see collision detection.
        // The "fifth sprite" field contains undefined data unless the VDP is in a TMS9918a mode, and sprite overflow has occurred,
        //   in which case it contains the number of the first sprite that could not be displayed due to overflow. 
        
        let mut status = 0;
        if self.is_vblank() {
            status |= 0b1000_0000;
        }
        status
    }
}

impl Controller for Vdp {
    fn read_io(&self, port: u8) -> Result<u8, GgError> {
        match port {
            V_COUNTER_PORT => Ok(self.v),
            CONTROL_PORT => Ok(self.status()),
            _ => {
                error!("Invalid port for VDP I/O controller (read): {:02x}", port);
                Err(GgError::IoControllerInvalidPort)
            }
        }
    }

    fn write_io(&mut self, port: u8, value: u8) -> Result<(), GgError> {
        match port {
            CONTROL_PORT => {
                self.control_data.push_back(value);
                if self.control_data.len() >= 2 {
                    trace!("VDP control type: {:08b}", self.control_data[1]);
                    self.process_control_data();
                }

                Ok(())
            }
            DATA_PORT => {
                match self.io_mode {
                    IoMode::VramWrite => self.vram_write(value),
                    IoMode::CramWrite => self.cram_write(value),
                    _ => {
                        error!("Received byte on data port ({:02x}) without being in a specific mode: {:02x}", DATA_PORT, value);
                        return Err(GgError::VdpInvalidIoMode);
                    }
                }

                Ok(())
            }
            _ => {
                error!("Invalid port for VDP I/O controller (write): {:02x}", port);
                Err(GgError::IoControllerInvalidPort)
            }
        }
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
