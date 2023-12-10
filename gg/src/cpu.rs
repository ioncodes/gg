use crate::{bus::Bus, handlers::Handlers, error::GgError};
use std::fmt;
use z80::{
    disassembler::Disassembler,
    instruction::{Opcode, Register, Reg8, Reg16},
};
use bitflags::bitflags;

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

bitflags! {
    pub(crate) struct Flags: u8 {
        const CARRY = 0b0000_0001;
        const SUBTRACT = 0b0000_0010;
        const PARTIY_OR_OVERFLOW = 0b0000_0100;
        const F3 = 0b0000_1000; // unused
        const HALF_CARRY = 0b0001_0000;
        const F5 = 0b0010_0000; // unused
        const ZERO = 0b0100_0000;
        const SIGN = 0b1000_0000;
    }
}

pub(crate) struct Cpu {
    pub(crate) registers: Registers,
    pub(crate) flags: Flags,
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
            flags: Flags::empty(),
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

                let result = match instruction.opcode {
                    Opcode::Jump(_, _, _) => Handlers::jump(self, bus, &instruction),
                    Opcode::DisableInterrupts(_) => Handlers::disable_interrupts(self, bus, &instruction),
                    Opcode::Load(_, _, _) => Handlers::load(self, bus, &instruction),
                    Opcode::LoadIndirectRepeat(_) => Handlers::load_indirect_repeat(self, bus, &instruction),
                    Opcode::Out(_, _, _) => Handlers::out(self, bus, &instruction),
                    Opcode::In(_, _, _) => Handlers::in_(self, bus, &instruction),
                    Opcode::Compare(_, _) => Handlers::compare(self, bus, &instruction),
                    Opcode::JumpRelative(_, _, _) => Handlers::jump_relative(self, bus, &instruction),
                    _ => panic!("Unhandled opcode: {:?}\nCPU state: {:?}", instruction.opcode, self),
                };

                match result {
                    Ok(_) => self.registers.pc += instruction.length as u16,
                    Err(GgError::IoRequestNotFulfilled) => {
                        println!("[cpu] I/O request not fulfilled, waiting for next tick");
                    },
                    Err(error) => panic!("CPU crashed with: {:?}\nError: {}", self, error)
                }
            }
            Err(msg) => panic!("{} @ {:x} =>\n{:?}", msg, self.registers.pc, self),
        }
    }

    #[allow(dead_code)]
    pub(crate) fn set_reg(&mut self, register: Register, value: u16) {
        match register {
            Register::Reg8(reg) => self.set_register(reg, value as u8),
            Register::Reg16(reg) => self.set_register_u16(reg, value),
        }
    }

    pub(crate) fn get_register_u16(&self, register: Reg16) -> u16 {
        match register {
            Reg16::AF => ((self.registers.a as u16) << 8) | (self.registers.f as u16),
            Reg16::BC => ((self.registers.b as u16) << 8) | (self.registers.c as u16),
            Reg16::DE => ((self.registers.d as u16) << 8) | (self.registers.e as u16),
            Reg16::HL => ((self.registers.h as u16) << 8) | (self.registers.l as u16),
            Reg16::SP => self.registers.sp,
            Reg16::PC => self.registers.pc,
            _ => panic!("Invalid register: {:?}", register),
        }
    }

    pub(crate) fn get_register(&self, register: Reg8) -> u8 {
        match register {
            Reg8::A => self.registers.a,
            Reg8::B => self.registers.b,
            Reg8::C => self.registers.c,
            Reg8::D => self.registers.d,
            Reg8::E => self.registers.e,
            Reg8::H => self.registers.h,
            Reg8::L => self.registers.l,
            _ => panic!("Invalid register: {:?}", register),
        }
    }

    pub(crate) fn set_register_u16(&mut self, register: Reg16, value: u16) {
        match register {
            Reg16::AF => {
                self.registers.a = (value >> 8) as u8;
                self.registers.f = value as u8;
            }
            Reg16::BC => {
                self.registers.b = (value >> 8) as u8;
                self.registers.c = value as u8;
            }
            Reg16::DE => {
                self.registers.d = (value >> 8) as u8;
                self.registers.e = value as u8;
            }
            Reg16::HL => {
                self.registers.h = (value >> 8) as u8;
                self.registers.l = value as u8;
            }
            Reg16::SP => self.registers.sp = value,
            Reg16::PC => self.registers.pc = value,
            _ => panic!("Invalid register: {:?}", register),
        }
    }

    pub(crate) fn set_register(&mut self, register: Reg8, value: u8) {
        match register {
            Reg8::A => self.registers.a = value,
            Reg8::B => self.registers.b = value,
            Reg8::C => self.registers.c = value,
            Reg8::D => self.registers.d = value,
            Reg8::E => self.registers.e = value,
            Reg8::H => self.registers.h = value,
            Reg8::L => self.registers.l = value,
            _ => panic!("Invalid register: {:?}", register),
        }
    }
}

impl fmt::Debug for Flags {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "CARRY: {} SUBTRACT: {} PARTIY_OR_OVERFLOW: {} F3: {} HALF_CARRY: {} F5: {} ZERO: {} SIGN: {}",
            self.contains(Flags::CARRY),
            self.contains(Flags::SUBTRACT),
            self.contains(Flags::PARTIY_OR_OVERFLOW),
            self.contains(Flags::F3),
            self.contains(Flags::HALF_CARRY),
            self.contains(Flags::F5),
            self.contains(Flags::ZERO),
            self.contains(Flags::SIGN)
        )
    }
}

impl fmt::Debug for Cpu {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f, 
            "Registers => a: {:02x} b: {:02x} c: {:02x} d: {:02x} e: {:02x} h: {:02x} l: {:02x} f: {:02x} af: {:04x} bc: {:04x} de: {:04x} hl: {:04x} pc: {:04x} sp: {:04x}\n",
            self.registers.a, self.registers.b, self.registers.c, self.registers.d, self.registers.e, self.registers.h, self.registers.l, self.registers.f,
            self.get_register_u16(Reg16::AF), self.get_register_u16(Reg16::BC), self.get_register_u16(Reg16::DE), self.get_register_u16(Reg16::HL),
            self.registers.pc, self.registers.sp)?;
        write!(f, "Flags => {:?}", self.flags)
    }
}
