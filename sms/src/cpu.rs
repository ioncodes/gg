use std::fmt;

use crate::{bus::Bus, handlers::Handlers};
use z80::{
    disassembler::Disassembler,
    instruction::{Opcode, Register},
};

pub(crate) struct Registers {
    pub(crate) a: u8,
    pub(crate) b: u8,
    pub(crate) c: u8,
    pub(crate) d: u8,
    pub(crate) e: u8,
    pub(crate) h: u8,
    pub(crate) l: u8,
    pub(crate) f: u8,
    pub(crate) pc: u16,
    pub(crate) sp: u16,
}

pub(crate) struct Cpu {
    pub(crate) registers: Registers,
}

impl Cpu {
    pub(crate) fn new() -> Cpu {
        Cpu {
            registers: Registers {
                a: 0,
                b: 0,
                c: 0,
                d: 0,
                e: 0,
                h: 0,
                l: 0,
                f: 0,
                pc: 0,
                sp: 0,
            },
        }
    }

    pub(crate) fn tick(&mut self, bus: &mut Bus) {
        let data = vec![
            bus.read(self.registers.pc).unwrap(),
            bus.read(self.registers.pc + 1).unwrap(),
            bus.read(self.registers.pc + 2).unwrap(),
            bus.read(self.registers.pc + 3).unwrap(),
        ];
        let disasm = Disassembler::new(&data);

        match disasm.decode(0) {
            Ok(instruction) => {
                println!("[{:04x}] {:?}", self.registers.pc, instruction.opcode);

                match instruction.opcode {
                    Opcode::Jump(_, _, _) => Handlers::jump(self, bus, &instruction),
                    Opcode::DisableInterrupts(_) => {
                        Handlers::disable_interrupts(self, bus, &instruction)
                    }
                    Opcode::Load(_, _, _) => Handlers::load(self, bus, &instruction),
                    Opcode::LoadIndirectRepeat(_) => Handlers::load_indirect_repeat(self, bus, &instruction),
                    _ => panic!("Unhandled opcode: {:?}", instruction.opcode),
                }

                self.registers.pc += instruction.length as u16;
            }
            Err(msg) => panic!("{} @ {:x} =>\n{:?}", msg, self.registers.pc, self),
        }
    }

    pub(crate) fn abort(&self, msg: &str) {
        panic!("{} @ {:x} =>\n{:?}", msg, self.registers.pc, self);
    }

    pub(crate) fn set_reg(&mut self, register: Register, value: u16) {
        if register.is_16bit() {
            self.set_register_u16(register, value);
        } else {
            self.set_register(register, value as u8);
        }
    }

    pub(crate) fn get_register_u16(&self, register: Register) -> u16 {
        match register {
            Register::AF => ((self.registers.a as u16) << 8) | (self.registers.f as u16),
            Register::BC => ((self.registers.b as u16) << 8) | (self.registers.c as u16),
            Register::DE => ((self.registers.d as u16) << 8) | (self.registers.e as u16),
            Register::HL => ((self.registers.h as u16) << 8) | (self.registers.l as u16),
            Register::SP => self.registers.sp,
            Register::PC => self.registers.pc,
            _ => panic!("Invalid register: {:?}", register),
        }
    }

    pub(crate) fn get_register(&self, register: Register) -> u8 {
        match register {
            Register::A => self.registers.a,
            Register::B => self.registers.b,
            Register::C => self.registers.c,
            Register::D => self.registers.d,
            Register::E => self.registers.e,
            Register::H => self.registers.h,
            Register::L => self.registers.l,
            _ => panic!("Invalid register: {:?}", register),
        }
    }

    pub(crate) fn set_register_u16(&mut self, register: Register, value: u16) {
        match register {
            Register::AF => {
                self.registers.a = (value >> 8) as u8;
                self.registers.f = value as u8;
            }
            Register::BC => {
                self.registers.b = (value >> 8) as u8;
                self.registers.c = value as u8;
            }
            Register::DE => {
                self.registers.d = (value >> 8) as u8;
                self.registers.e = value as u8;
            }
            Register::HL => {
                self.registers.h = (value >> 8) as u8;
                self.registers.l = value as u8;
            }
            Register::SP => self.registers.sp = value,
            Register::PC => self.registers.pc = value,
            _ => panic!("Invalid register: {:?}", register),
        }
    }

    pub(crate) fn set_register(&mut self, register: Register, value: u8) {
        match register {
            Register::A => self.registers.a = value,
            Register::B => self.registers.b = value,
            Register::C => self.registers.c = value,
            Register::D => self.registers.d = value,
            Register::E => self.registers.e = value,
            Register::H => self.registers.h = value,
            Register::L => self.registers.l = value,
            _ => panic!("Invalid register: {:?}", register),
        }
    }
}

impl fmt::Debug for Cpu {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f, 
            "a: {:02x} b: {:02x} c: {:02x} d: {:02x} e: {:02x} h: {:02x} l: {:02x} f: {:02x} af: {:04x} bc: {:04x} de: {:04x} hl: {:04x} pc: {:04x} sp: {:04x}",
            self.registers.a, self.registers.b, self.registers.c, self.registers.d, self.registers.e, self.registers.h, self.registers.l, self.registers.f,
            self.get_register_u16(Register::AF), self.get_register_u16(Register::BC), self.get_register_u16(Register::DE), self.get_register_u16(Register::HL),
            self.registers.pc, self.registers.sp)
    }
}
