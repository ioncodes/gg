use crate::{bus::Bus, handlers::Handlers, error::GgError};
use std::fmt;
use z80::{
    disassembler::Disassembler,
    instruction::{Opcode, Register, Reg8, Reg16},
};
use bitflags::bitflags;
use log::{trace, debug, error};

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
        const PARITY_OR_OVERFLOW = 0b0000_0100;
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

    pub(crate) fn tick(&mut self, bus: &mut Bus) -> Result<(), GgError> {
        let data = vec![
            bus.read(self.registers.pc).unwrap(),
            bus.read(self.registers.pc + 1).unwrap(),
            bus.read(self.registers.pc + 2).unwrap(),
            bus.read(self.registers.pc + 3).unwrap(),
        ];
        let disasm = Disassembler::new(&data);

        match disasm.decode(0) {
            Ok(instruction) => {
                trace!("[{:04x}] {}", self.registers.pc, instruction.opcode);

                let result = match instruction.opcode {
                    Opcode::Jump(_, _, _) => Handlers::jump(self, bus, &instruction),
                    Opcode::DisableInterrupts(_) => Handlers::disable_interrupts(self, bus, &instruction),
                    Opcode::Load(_, _, _) => Handlers::load(self, bus, &instruction),
                    Opcode::LoadIndirectRepeat(_) => Handlers::load_indirect_repeat(self, bus, &instruction),
                    Opcode::Out(_, _, _) => Handlers::out(self, bus, &instruction),
                    Opcode::In(_, _, _) => Handlers::in_(self, bus, &instruction),
                    Opcode::Compare(_, _) => Handlers::compare(self, bus, &instruction),
                    Opcode::JumpRelative(_, _, _) => Handlers::jump_relative(self, bus, &instruction),
                    Opcode::CallUnconditional(_, _) => Handlers::call_unconditional(self, bus, &instruction),
                    Opcode::Return(_, _) => Handlers::return_(self, bus, &instruction),
                    Opcode::OutIndirectRepeat(_) => Handlers::out_indirect_repeat(self, bus, &instruction),
                    Opcode::Or(_, _) => Handlers::or(self, bus, &instruction),
                    Opcode::Push(_, _) => Handlers::push(self, bus, &instruction),
                    Opcode::Pop(_, _) => Handlers::pop(self, bus, &instruction),
                    Opcode::Increment(_, _) => Handlers::increment(self, bus, &instruction),
                    Opcode::Decrement(_, _) => Handlers::decrement(self, bus, &instruction),
                    Opcode::ResetBit(_, _, _) => Handlers::reset_bit(self, bus, &instruction),
                    Opcode::DecrementAndJumpRelative(_, _) => Handlers::decrement_and_jump_relative(self, bus, &instruction),
                    _ => {
                        error!("Invalid opcode: {}", instruction.opcode);
                        Err(GgError::OpcodeNotImplemented { opcode: instruction.opcode })
                    },
                };

                let io_skip = match result {
                    Err(GgError::IoRequestNotFulfilled) => {
                        debug!("I/O request not fulfilled, waiting for next tick");
                        true
                    },
                    _ => false,
                };

                let call_skip = match instruction.opcode {
                    Opcode::CallUnconditional(_, _) => true,
                    Opcode::Return(_, _) => true,
                    _ => false
                };

                if !call_skip && !io_skip {
                    self.registers.pc += instruction.length as u16;
                }

                result
            }
            Err(msg) => Err(GgError::DecoderError { msg }),
        }
    }

    #[allow(dead_code)]
    pub(crate) fn set_reg(&mut self, register: Register, value: u16) {
        match register {
            Register::Reg8(reg) => self.set_register_u8(reg, value as u8),
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
            _ => panic!("Invalid register: {}", register),
        }
    }

    pub(crate) fn get_register_u8(&self, register: Reg8) -> u8 {
        match register {
            Reg8::A => self.registers.a,
            Reg8::B => self.registers.b,
            Reg8::C => self.registers.c,
            Reg8::D => self.registers.d,
            Reg8::E => self.registers.e,
            Reg8::H => self.registers.h,
            Reg8::L => self.registers.l,
            _ => panic!("Invalid register: {}", register),
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
            _ => panic!("Invalid register: {}", register),
        }
    }

    pub(crate) fn set_register_u8(&mut self, register: Reg8, value: u8) {
        match register {
            Reg8::A => self.registers.a = value,
            Reg8::B => self.registers.b = value,
            Reg8::C => self.registers.c = value,
            Reg8::D => self.registers.d = value,
            Reg8::E => self.registers.e = value,
            Reg8::H => self.registers.h = value,
            Reg8::L => self.registers.l = value,
            _ => panic!("Invalid register: {}", register),
        }
    }

    pub(crate) fn push_stack(&mut self, bus: &mut Bus, value: u16) -> Result<(), GgError> {
        self.registers.sp -= 2;
        bus.write_word(self.registers.sp, value)?;
        log::trace!("Pushed {:04x} to stack at {:04x}", value, self.registers.sp);
        Ok(())
    }

    pub(crate) fn pop_stack(&mut self, bus: &mut Bus) -> Result<u16, GgError> {
        let value = bus.read_word(self.registers.sp)?;
        log::trace!("Popped {:04x} from stack at {:04x}", value, self.registers.sp);
        self.registers.sp += 2;
        Ok(value)
    }
}

impl fmt::Display for Flags {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "CNPxHxZS\n{}{}{}{}{}{}{}{}",
            self.contains(Flags::CARRY) as u8,
            self.contains(Flags::SUBTRACT) as u8,
            self.contains(Flags::PARITY_OR_OVERFLOW) as u8,
            self.contains(Flags::F3) as u8,
            self.contains(Flags::HALF_CARRY) as u8,
            self.contains(Flags::F5) as u8,
            self.contains(Flags::ZERO) as u8,
            self.contains(Flags::SIGN) as u8
        )
    }
}

impl fmt::Display for Cpu {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f, 
            "REGISTERS\nA: {:02x}\nB: {:02x}\nC: {:02x}\nD: {:02x}\nE: {:02x}\nH: {:02x}\nL: {:02x}\nF: {:02x}\nAF: {:04x}\nBC: {:04x}\nDE: {:04x}\nHL: {:04x}\nPC: {:04x}\nSP: {:04x}\n",
            self.registers.a, self.registers.b, self.registers.c, self.registers.d, self.registers.e, self.registers.h, self.registers.l, self.registers.f,
            self.get_register_u16(Reg16::AF), self.get_register_u16(Reg16::BC), self.get_register_u16(Reg16::DE), self.get_register_u16(Reg16::HL),
            self.registers.pc, self.registers.sp)?;
        write!(f, "\nFLAGS\n{}", self.flags)
    }
}
