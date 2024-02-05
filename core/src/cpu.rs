use crate::{bus::{Bus, MEMORY_CONTROL_PORT}, error::GgError, handlers::Handlers, io::Controller, psg::Psg, vdp::{Vdp, CONTROL_PORT, DATA_PORT, V_COUNTER_PORT}};
use bitflags::bitflags;
use log::{debug, error, trace};
use std::fmt;
use z80::{
    disassembler::Disassembler,
    instruction::{Opcode, Reg16, Reg8, Register},
};

#[derive(Debug, Clone, Copy)]
pub struct Registers {
    pub a: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub h: u8,
    pub l: u8,
    pub f: u8,
    pub pc: u16,
    pub sp: u16,
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

pub(crate) enum InterruptMode {
    IM0,
    IM1,
    IM2,
}

impl InterruptMode {
    pub(crate) fn from(value: u8) -> Result<InterruptMode, GgError> {
        match value {
            0 => Ok(InterruptMode::IM0),
            1 => Ok(InterruptMode::IM1),
            2 => Ok(InterruptMode::IM2),
            _ => Err(GgError::InvalidInterruptMode { mode: value }),
        }
    }
}

pub struct Cpu {
    pub registers: Registers,
    pub(crate) flags: Flags,
    pub(crate) interrupts_enabled: bool,
    pub(crate) interrupt_mode: InterruptMode,
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
            interrupts_enabled: true,
            interrupt_mode: InterruptMode::IM0,
        }
    }

    pub(crate) fn tick(&mut self, bus: &mut Bus, vdp: &mut Vdp, psg: &mut Psg) -> Result<(), GgError> {
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
                
                let mut handlers = Handlers::new(self, bus, vdp, psg);
                let result = match instruction.opcode {
                    Opcode::Jump(_, _, _) => handlers.jump(&instruction),
                    Opcode::DisableInterrupts(_) => handlers.disable_interrupts(&instruction),
                    Opcode::Load(_, _, _) => handlers.load(&instruction),
                    Opcode::LoadIndirectRepeat(_) => handlers.load_indirect_repeat(&instruction),
                    Opcode::Out(_, _, _) => handlers.out(&instruction),
                    Opcode::In(_, _, _) => handlers.in_(&instruction),
                    Opcode::Compare(_, _) => handlers.compare(&instruction),
                    Opcode::JumpRelative(_, _, _) => handlers.jump_relative(&instruction),
                    Opcode::CallUnconditional(_, _) => handlers.call_unconditional(&instruction),
                    Opcode::Return(_, _) => handlers.return_(&instruction),
                    Opcode::OutIndirectRepeat(_) => handlers.out_indirect_repeat(&instruction),
                    Opcode::Or(_, _) => handlers.or(&instruction),
                    Opcode::Push(_, _) => handlers.push(&instruction),
                    Opcode::Pop(_, _) => handlers.pop(&instruction),
                    Opcode::Increment(_, _) => handlers.increment(&instruction),
                    Opcode::Decrement(_, _) => handlers.decrement(&instruction),
                    Opcode::ResetBit(_, _, _) => handlers.reset_bit(&instruction),
                    Opcode::DecrementAndJumpRelative(_, _) => handlers.decrement_and_jump_relative(&instruction),
                    Opcode::Xor(_, _) => handlers.xor(&instruction),
                    Opcode::Outi(_) => handlers.outi(&instruction),
                    Opcode::Restart(_, _) => handlers.restart(&instruction),
                    Opcode::SetInterruptMode(_, _) => handlers.set_interrupt_mode(&instruction),
                    Opcode::Subtract(_, _) => handlers.subtract(&instruction),
                    Opcode::Add(_, _, _) => handlers.add(&instruction),
                    Opcode::And(_, _) => handlers.and(&instruction),
                    _ => {
                        error!("Hanlder missing for instruction: {}\n{}", instruction.opcode, self);
                        return Err(GgError::OpcodeNotImplemented {
                            opcode: instruction.opcode,
                        });
                    }
                };

                match result {
                    Err(GgError::BusRequestOutOfBounds { address }) => {
                        error!("Bus request out of bounds: {:04x}\n{}", address, self);
                        return Err(GgError::BusRequestOutOfBounds { address });
                    },
                    _ => ()
                }

                let io_skip = match result {
                    Err(GgError::IoRequestNotFulfilled) => {
                        debug!("I/O request not fulfilled, waiting for next tick");
                        true
                    }
                    _ => false,
                };

                let call_skip = match instruction.opcode {
                    Opcode::CallUnconditional(_, _) => true,
                    // only skip the PC increment if we actually returned somewhere
                    Opcode::Jump(_, _, _) => result.is_ok(),
                    Opcode::Return(_, _) => result.is_ok(),
                    Opcode::Restart(_, _) => result.is_ok(),
                    _ => false,
                };

                if !call_skip && !io_skip {
                    self.registers.pc += instruction.length as u16;
                }

                result
            }
            Err(msg) => Err(GgError::DecoderError { msg }),
        }
    }

    pub(crate) fn write_io(&mut self, port: u8, value: u8, vdp: &mut Vdp, bus: &mut Bus, psg: &mut Psg) -> Result<(), GgError> {
        match port {
            0x00..=0x06 => bus.write_io(port, value)?,
            DATA_PORT | CONTROL_PORT => vdp.write_io(port, value)?,
            MEMORY_CONTROL_PORT => bus.write_io(port, value)?,
            0x7f => psg.write_io(port, value)?,
            _ => {
                error!("Unassigned port (write): {:02x}", port);
                return Err(GgError::IoControllerInvalidPort);
            }
        }

        Ok(())
    }

    pub(crate) fn read_io(&self, port: u8, vdp: &mut Vdp, bus: &mut Bus, _psg: &mut Psg) -> Result<u8, GgError> {
        match port {
            0x00..=0x06 => bus.read_io(port),
            DATA_PORT | CONTROL_PORT | V_COUNTER_PORT => vdp.read_io(port),
            _ => {
                error!("Unassigned port (read): {:02x}", port);
                Err(GgError::IoControllerInvalidPort)
            }
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
            Reg8::F => self.registers.f,
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
            "C: {} N: {} P: {} H: {} Z: {} S: {}",
            self.contains(Flags::CARRY) as u8,
            self.contains(Flags::SUBTRACT) as u8,
            self.contains(Flags::PARITY_OR_OVERFLOW) as u8,
            self.contains(Flags::HALF_CARRY) as u8,
            self.contains(Flags::ZERO) as u8,
            self.contains(Flags::SIGN) as u8
        )
    }
}

impl fmt::Display for Cpu {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "AF: {:04x}  BC: {:04x}  DE: {:04x}  HL: {:04x}  PC: {:04x}  SP: {:04x}\n",
            self.get_register_u16(Reg16::AF),
            self.get_register_u16(Reg16::BC),
            self.get_register_u16(Reg16::DE),
            self.get_register_u16(Reg16::HL),
            self.registers.pc,
            self.registers.sp
        )?;
        write!(f, "{}\n", self.flags)
    }
}
