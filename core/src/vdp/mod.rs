mod pattern;
mod sprite;

use std::collections::{HashMap, VecDeque};
use std::rc::Rc;

use crate::error::GgError;
use crate::io::Controller;
use crate::lua_engine::{HookType, LuaEngine};
use crate::memory::Memory;
use crate::vdp::pattern::Pattern;
use log::{debug, error, trace};

use self::sprite::SpriteSize;

// $40-7F = Even locations are V counter/PSG, odd locations are H counter/PSG
// $80-BF = Even locations are data port, odd locations are control port.
pub(crate) const IO_DATA_CONTROL_START: u8 = 0x80;
pub(crate) const IO_DATA_CONTROL_END: u8 = 0xbf;

pub const INTERNAL_WIDTH: usize = 256;
pub const INTERNAL_HEIGHT: usize = 224;
pub const VISIBLE_WIDTH: usize = 160;
pub const VISIBLE_HEIGHT: usize = 144;
pub const OFFSET_X: usize = 48;
pub const OFFSET_Y: usize = 24;

pub type Color = (u8, u8, u8, u8);

enum IoMode {
    VramRead,
    VramWrite,
    CramWrite,
    None,
}

#[derive(Default, Debug)]
pub struct Registers {
    pub r0: u8,
    pub r1: u8,
    pub r2: u8,
    pub r3: u8,
    pub r4: u8,
    pub r5: u8,
    pub r6: u8,
    pub r7: u8,
    pub r8: u8,
    pub r9: u8,
    pub r10: u8,
    pub address: u16,
}

#[derive(PartialEq)]
pub enum Mode {
    SegaMasterSystem,
    GameGear,
}

pub struct Vdp {
    pub v: u8,
    pub h: u8,
    pub registers: Registers,
    pub vram: Memory<u16>,
    pub cram: Memory<u16>,
    pub(crate) data_buffer: u8,
    v_2nd_loop: bool,
    h_2nd_loop: bool,
    control_data: VecDeque<u8>,
    cram_latch: Option<u8>,
    io_mode: IoMode,
    mode: Mode,
    status: u8,
    vram_dirty: bool,
    lua: Rc<LuaEngine>,
    last_frame: Vec<Color>,
    priority_list: Vec<usize>,
    scanline_counter: u8,
    scanline_irq_available: bool,
}

impl Vdp {
    pub(crate) fn new(mode: Mode, lua: Rc<LuaEngine>) -> Vdp {
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
            data_buffer: 0,
            io_mode: IoMode::None,
            mode,
            status: 0,
            vram_dirty: false,
            lua,
            last_frame: vec![(0, 0, 0, 0); INTERNAL_WIDTH * INTERNAL_HEIGHT],
            priority_list: vec![],
            scanline_counter: 0,
            scanline_irq_available: false,
        }
    }

    pub(crate) fn tick(&mut self) -> bool {
        self.handle_counters();

        // Line IRQ
        if self.v <= 192 {
            self.scanline_counter = self.scanline_counter.wrapping_sub(1);
            if self.scanline_counter == 0 {
                self.scanline_irq_available = true;
                self.scanline_counter = self.registers.r10;
            }
        }

        // Frame IRQ (VBlank)
        if self.is_vblank() && self.is_hblank() {
            self.status |= 0b1000_0000;
        }

        self.v_2nd_loop && self.v > INTERNAL_HEIGHT as u8 && self.vram_dirty
    }

    pub fn vblank_irq_pending(&self) -> bool {
        if self.registers.r1 & 0b0010_0000 > 0 {
            return self.status & 0b1000_0000 > 0;
        }

        false
    }

    pub fn scanline_irq_pending(&self) -> bool {
        if self.registers.r0 & 0b0001_0000 > 0 {
            return self.scanline_irq_available;
        }

        false
    }

    pub(crate) fn is_vblank(&self) -> bool {
        self.v == 0
    }

    pub(crate) fn is_hblank(&self) -> bool {
        self.h == 0
    }

    pub fn render(&mut self) -> (Color, &Vec<Color>) {
        let background_color = self.read_palette_entry(0, 0);

        // Using binary search makes it roughly 6x faster
        self.priority_list.clear();
        self.render_background();
        self.priority_list.sort_unstable();
        self.render_sprites();

        self.vram_dirty = false;

        (background_color, &self.last_frame)
    }

    pub fn render_sprites(&mut self) {
        let write_pattern_to_internal = |pattern: &Pattern, pixels: &mut Vec<Color>, x: u8, y: u8| {
            for p_y in 0..8 {
                for p_x in 0..8 {
                    let color = pattern.get_pixel(p_x, p_y);
                    if color == (0, 0, 0, 0) {
                        // do not render transparent pixels to the internal frame
                        // todo: do we rlly not want to render these pixels? this might lead to weird edge cases
                        continue;
                    }

                    let y = if let Some(result) = y.checked_add(p_y) {
                        result
                    } else {
                        continue;
                    };

                    if y as usize >= INTERNAL_HEIGHT {
                        continue;
                    }

                    let x = if let Some(result) = x.checked_add(p_x) {
                        result
                    } else {
                        continue;
                    };

                    if x as usize >= INTERNAL_WIDTH {
                        continue;
                    }

                    let idx = (y as usize * INTERNAL_WIDTH) + x as usize;
                    if self.priority_list.binary_search(&idx).is_err() {
                        pixels[idx] = color;
                    }
                }
            }
        };

        let sprite_size = self.sprite_size();
        // let mut overflow_lookup_table: HashMap<u8, usize> = HashMap::new();
        // let mut collision_lookup_table: Vec<(u8, u8)> = Vec::new();

        for idx in 0..64 {
            let sprite_attr_base_addr = self.get_sprite_attribute_table_addr();
            let y = self.vram.read(sprite_attr_base_addr + idx) + 1;
            //println!("y: {:02x} v: {:02x}", y, self.v);
            // if OFFSET_Y as u8 + y != self.v {
            //     continue;
            // }

            let x = self.vram.read(sprite_attr_base_addr + 0x80 + 2 * idx);
            let n = self.vram.read(sprite_attr_base_addr + 0x80 + 2 * idx + 1);

            if y == 0xd0 {
                break;
            }

            if y == 0xe0 {
                continue;
            }

            // // Process OVR
            // if !overflow_lookup_table.contains_key(&y) {
            //     overflow_lookup_table.insert(y, 0);
            // } else {
            //     let count = overflow_lookup_table.get_mut(&y).unwrap();
            //     *count += 1;
            //     if *count > 8 {
            //         self.status |= 0b0100_0000;
            //     }
            // }

            // // Process COL
            // if collision_lookup_table.contains(&(x, y)) {
            //     self.status |= 0b0010_0000;
            // } else {
            //     collision_lookup_table.push((x, y));
            // }

            // Render sprites for 8x8 and 8x16
            if sprite_size == SpriteSize::Size8x8 {
                let sprite_table_entry = self.get_sprite_generator_entry(n as u16);
                let pattern_addr = sprite_table_entry * 32;
                let pattern = self.fetch_pattern(pattern_addr, false, 1);

                write_pattern_to_internal(&pattern, &mut self.last_frame, x, y);
            } else {
                let sprite_table_entry = self.get_sprite_generator_entry(n as u16 & 0b1111_1110);
                let sprite1_addr = sprite_table_entry * 32;
                let sprite2_addr = (sprite_table_entry + 1) * 32;

                let pattern = self.fetch_pattern(sprite1_addr, false, 1);
                write_pattern_to_internal(&pattern, &mut self.last_frame, x, y);

                let pattern = self.fetch_pattern(sprite2_addr, false, 1);
                write_pattern_to_internal(&pattern, &mut self.last_frame, x, y + 8);
            }
        }
    }

    pub fn render_background(&mut self) {
        let h_scroll = self.registers.r8 as usize;
        let v_scroll = self.registers.r9 as usize;

        let background_color = self.read_palette_entry(0, 0);

        for row in 0..28 {
            for column in 0..32 {
                let name_table_addr = self.get_name_table_addr(column, row);

                // The pattern base address is defined by the pattern generator table (which always starts at 0)
                // Rendering every pattern starting at 0 would yield a classic tile map
                // Source: As per Sega Game Gear Hardware Reference Manual, page 26
                // Source: Chapter 6 "VDP Manual", subchapter 3 "Standard VRAM mapping"
                let pattern_information = self.vram.read_word(name_table_addr);
                let v_flip = (pattern_information & 0b0000_0100_0000_0000) > 0;
                let h_flip = (pattern_information & 0b0000_0010_0000_0000) > 0;
                let palette_row = if (pattern_information & 0b0000_1000_0000_0000) > 0 { 1 } else { 0 };
                let priority = (pattern_information & 0b0001_0000_0000_0000) > 0;

                let pattern_base_addr = pattern_information & 0b0000_0001_1111_1111;
                let pattern_addr = pattern_base_addr * 32;

                // pattern_base_addr = character/tile location in VRAM.
                // Each character/tile is 8x8 pixels, and each pixel consists of 4 bits.
                // So each character/tile is 32 bytes (64 pixels).

                let mut pattern = self.fetch_pattern(pattern_addr, true, palette_row);

                if v_flip {
                    pattern.flip_vertical();
                }

                if h_flip {
                    pattern.flip_horizontal();
                }

                let row = row as usize;
                let column = column as usize;

                // hscroll defines pixel spacing starting at the left of internal screen
                let screen_x = (h_scroll + (column * 8)) % INTERNAL_WIDTH;
                // vscroll defines pixel spacing starting at the bottom of internal screen
                let screen_y = ((INTERNAL_HEIGHT - v_scroll) + (row * 8)) % INTERNAL_HEIGHT;

                for y in 0..8 {
                    for x in 0..8 {
                        let color = pattern.get_pixel(x, y);
                        let idx = (screen_y + y as usize) * INTERNAL_WIDTH + (screen_x + x as usize);
                        if idx < self.last_frame.len() {
                            self.last_frame[idx] = color;

                            if priority && color != background_color {
                                self.priority_list.push(idx);
                            }
                        }
                    }
                }
            }
        }
    }

    fn fetch_pattern(&self, pattern_addr: u16, use_background: bool, palette_row: u8) -> Pattern {
        let mut pattern = Pattern::new();

        for line in 0..8 {
            let line_base_addr = pattern_addr + (line * 4);
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
                let color = if !use_background && color == 0 {
                    (0, 0, 0, 0) // transparent
                } else if use_background && color == 0 {
                    self.read_palette_entry(0, palette_row) // background color
                } else {
                    self.read_palette_entry(color as u16, palette_row)
                };
                pattern.set_pixel(7 - bit, line as u8, color);
            }
        }

        pattern
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

    fn get_sprite_generator_entry(&self, idx: u16) -> u16 {
        /*
         *  Register $06 - Sprite Pattern Generator Base Address
         *
         *  D7 - No effect
         *  D6 - No effect
         *  D5 - No effect
         *  D4 - No effect
         *  D3 - No effect
         *  D2 - Bit 13 of the table base address
         *  D1 - No effect
         *  D0 - No effect
         */

        if self.registers.r6 & 0b0000_0100 > 0 {
            (1 << 13) + idx
        } else {
            idx
        }
    }

    fn get_sprite_attribute_table_addr(&self) -> u16 {
        /*
         *  Register $05 - Sprite Attribute Table Base Address
         *
         *  D7 - No effect
         *  D6 - Bit 13 of the table base address
         *  D5 - Bit 12 of the table base address
         *  D4 - Bit 11 of the table base address
         *  D3 - Bit 10 of the table base address
         *  D2 - Bit  9 of the table base address
         *  D1 - Bit  8 of the table base address
         *  D0 - No effect
         */

        let mut addr = self.registers.r5 as u16;
        addr &= 0b0111_1110;
        addr <<= 7;

        addr
    }

    fn sprite_size(&self) -> SpriteSize {
        /*
         *  Register $01 - Mode Control No. 2
         *
         *  D7 - No effect
         *  D6 - (BLK) 1= Display visible, 0= display blanked.
         *  D5 - (IE0) 1= Frame interrupt enable.
         *  D4 - (M1) Selects 224-line screen for Mode 4 if M2=1, else has no effect.
         *  D3 - (M3) Selects 240-line screen for Mode 4 if M2=1, else has no effect.
         *  D2 - No effect
         *  D1 - Sprites are 1=16x16,0=8x8 (TMS9918), Sprites are 1=8x16,0=8x8 (Mode 4)
         *  D0 - Sprite pixels are doubled in size.
         */

        if self.registers.r1 & 0b0000_0010 > 0 {
            SpriteSize::Size8x16
        } else {
            SpriteSize::Size8x8
        }
    }

    fn read_palette_entry(&self, mut index: u16, row: u8) -> (u8, u8, u8, u8) {
        // row 0 is the background color
        // row 1 is the sprite color?
        // todo: verify

        // 64 bytes CRAM if gamegear mode
        // 32 bytes CRAM if master system mode
        // SMS:  --BBGGRR
        // GG:   --------BBBBGGGGRRRR

        let (r, g, b) = if self.mode == Mode::GameGear {
            index = (index * 2) + (row as u16 * 32);

            let high = self.cram.read(index);
            let low = self.cram.read(index + 1);

            let r = (low & 0b0000_1111) << 4;
            let g = low & 0b1111_0000;
            let b = (high & 0b0000_1111) << 4;

            (r, g, b)
        } else {
            // todo: do we need row select here too?
            let data = self.cram.read(index);

            let r = (data & 0b0000_0011) << 6;
            let g = ((data >> 2) & 0b0000_0011) << 6;
            let b = (data & 0b0000_0011) << 6;

            (r, g, b)
        };

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
                self.increment_address_register(0x4000);

                debug!("Setting address register to {:04x}", address);
                self.data_buffer = value;

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

        if self.mode == Mode::SegaMasterSystem {
            self.cram.write(address, value);
        } else {
            if address % 2 == 0 {
                self.cram_latch = Some(value);
            } else {
                let latched_value = self.cram_latch.unwrap();
                self.cram.write(address, latched_value);
                self.cram.write(address - 1, value);
                self.cram_latch = None;
            }
        }

        self.increment_address_register(0x40);

        // Force a rerender
        self.vram_dirty = true;
    }

    fn vram_write(&mut self, value: u8) {
        if self.lua.hook_exists(self.registers.address, HookType::VramWrite) {
            self.lua.execute_hook(self.registers.address, HookType::VramWrite);
        }

        self.vram.write(self.registers.address, value);

        self.increment_address_register(0x4000);

        // Force a rerender
        self.vram_dirty = true;
    }

    fn increment_address_register(&mut self, boundary: u16) {
        self.registers.address += 1;
        self.registers.address %= boundary; // ensure we wrap around
    }

    fn status(&mut self) -> u8 {
        // The VBlank flag is set when a VBlank interrupt has just occurred.
        //   An interrupt handler can use this to tell the difference between VBlank interrupts and line interrupts.
        // The sprite overflow flag is set when sprite overflow has occurred.
        // The sprite collision flag will be set if any sprites overlap - see collision detection.
        // The "fifth sprite" field contains undefined data unless the VDP is in a TMS9918a mode, and sprite overflow has occurred,
        //   in which case it contains the number of the first sprite that could not be displayed due to overflow.

        let status = self.status;
        self.status &= 0b0001_1111; // clear VBlank, Sprite Overflow and Sprite Collision flags
        self.scanline_irq_available = false; // clear scanline IRQ flag
        status
    }
}

impl Controller for Vdp {
    fn read_io(&mut self, port: u8) -> Result<u8, GgError> {
        match port {
            0x40..=0x7f => {
                if port % 2 == 0 {
                    Ok(self.v)
                } else {
                    Ok(self.h)
                }
            }
            IO_DATA_CONTROL_START..=IO_DATA_CONTROL_END => {
                if port % 2 == 0 {
                    // data port
                    // todo: reset control port flag
                    let data = self.vram.read(self.registers.address);
                    self.increment_address_register(0x4000);

                    let current_data = self.data_buffer;
                    self.data_buffer = data;
                    Ok(current_data)
                } else {
                    // control port
                    Ok(self.status())
                }
            }
            _ => {
                error!("Invalid port for VDP I/O controller (read): {:02x}", port);
                Err(GgError::IoControllerInvalidPort)
            }
        }
    }

    fn write_io(&mut self, port: u8, value: u8) -> Result<(), GgError> {
        match port {
            IO_DATA_CONTROL_START..=IO_DATA_CONTROL_END => {
                if port % 2 == 0 {
                    // data port
                    match self.io_mode {
                        IoMode::VramWrite => self.vram_write(value),
                        IoMode::CramWrite => self.cram_write(value),
                        _ => {
                            error!(
                                "Received byte on data port ({:02x}) without being in a specific mode: {:02x}",
                                port, value
                            );
                            return Err(GgError::VdpInvalidIoMode);
                        }
                    }
                } else {
                    // control port
                    self.control_data.push_back(value);
                    if self.control_data.len() >= 2 {
                        trace!("VDP control type: {:08b}", self.control_data[1]);
                        self.process_control_data();
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
