use crate::{
    bus::{Bus, MEMORY_CONTROL_PORT},
    error::GgError,
    handlers::Handlers,
    io::Controller as _,
    psg::Psg,
    vdp::{Vdp, CONTROL_PORT, DATA_PORT, V_COUNTER_PORT},
};
use bitflags::bitflags;
use log::{error, trace};
use std::fmt;
use z80::{
    disassembler::Disassembler,
    instruction::{Instruction, Opcode, Reg16, Reg8, Register},
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
    pub f: Flags,
    pub a_shadow: u8,
    pub b_shadow: u8,
    pub c_shadow: u8,
    pub d_shadow: u8,
    pub e_shadow: u8,
    pub h_shadow: u8,
    pub l_shadow: u8,
    pub f_shadow: Flags,
    pub ix: u16,
    pub iy: u16,
    pub pc: u16,
    pub sp: u16,
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct Flags: u8 {
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

pub enum InterruptMode {
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

    pub fn to_u8(&self) -> u8 {
        match self {
            InterruptMode::IM0 => 0,
            InterruptMode::IM1 => 1,
            InterruptMode::IM2 => 2,
        }
    }
}

pub struct Cpu {
    pub registers: Registers,
    pub interrupts_enabled: bool,
    pub interrupt_mode: InterruptMode,
    irq_available: bool,
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
                f: Flags::empty(),
                a_shadow: 0,
                b_shadow: 0,
                c_shadow: 0,
                d_shadow: 0,
                e_shadow: 0,
                h_shadow: 0,
                l_shadow: 0,
                f_shadow: Flags::empty(),
                ix: 0,
                iy: 0,
                pc: 0,
                sp: 0,
            },
            interrupts_enabled: true,
            interrupt_mode: InterruptMode::IM0,
            irq_available: false,
        }
    }

    pub(crate) fn decode_at_pc(&self, bus: &mut Bus) -> Result<Instruction, String> {
        let data = vec![
            bus.read(self.registers.pc).unwrap(),
            bus.read(self.registers.pc + 1).unwrap(),
            bus.read(self.registers.pc + 2).unwrap(),
            bus.read(self.registers.pc + 3).unwrap(),
        ];
        let disasm = Disassembler::new(&data);
        disasm.decode(0)
    }

    pub(crate) fn tick(&mut self, bus: &mut Bus, vdp: &mut Vdp, psg: &mut Psg) -> Result<Instruction, GgError> {
        if self.irq_available {
            self.trigger_irq(bus)?;
            self.irq_available = false;
        }
        
        let instr = self.decode_at_pc(bus);

        match instr {
            Ok(instruction) => {
                let prefix = if self.registers.pc < 0xc000 { "rom" } else { "ram" };
                let real_pc_addr = match bus.translate_address_to_real(self.registers.pc) {
                    Ok(rom_addr) => rom_addr,
                    Err(_) => self.registers.pc as usize, // This can happen if we execute code in RAM (example: end of BIOS)
                };
                trace!(
                    "[{}:{:04x}->{:08x}] {:<20} [{:?}]",
                    prefix,
                    self.registers.pc,
                    real_pc_addr,
                    format!("{}", instruction.opcode),
                    self
                );

                let mut handlers = Handlers::new(self, bus, vdp, psg);
                let result = match instruction.opcode {
                    Opcode::Jump(_, _, _) => handlers.jump(&instruction),
                    Opcode::DisableInterrupts(_) => handlers.set_interrupt_state(false, &instruction),
                    Opcode::EnableInterrupts(_) => handlers.set_interrupt_state(true, &instruction),
                    Opcode::Load(_, _, _) => handlers.load(&instruction),
                    Opcode::LoadIndirectRepeat(_) => handlers.load_indirect_repeat(&instruction),
                    Opcode::Out(_, _, _) => handlers.out(&instruction),
                    Opcode::In(_, _, _) => handlers.in_(&instruction),
                    Opcode::Compare(_, _) => handlers.compare(&instruction),
                    Opcode::JumpRelative(_, _, _) => handlers.jump_relative(&instruction),
                    Opcode::Call(_, _, _) => handlers.call(&instruction),
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
                    Opcode::OutIncrement(_) => handlers.outi(&instruction),
                    Opcode::OutDecrement(_) => handlers.outd(&instruction),
                    Opcode::Restart(_, _) => handlers.restart(&instruction),
                    Opcode::SetInterruptMode(_, _) => handlers.set_interrupt_mode(&instruction),
                    Opcode::Subtract(_, _) => handlers.subtract(&instruction),
                    Opcode::Add(_, _, _) => handlers.add(&instruction),
                    Opcode::And(_, _) => handlers.and(&instruction),
                    Opcode::SubtractWithCarry(_, _, _) => handlers.subtract_with_carry(&instruction),
                    Opcode::RotateRightCarry(_) => handlers.rotate_right_carry(&instruction),
                    Opcode::RotateRightCarrySwap(_) => handlers.rotate_right_carry_swap(&instruction),
                    Opcode::RotateLeftCarry(_) => handlers.rotate_left_carry(&instruction),
                    Opcode::RotateLeftCarrySwap(_) => handlers.rotate_left_carry_swap(&instruction),
                    Opcode::RotateRightCarrySideeffect(_, _) => handlers.rotate_right_carry_sideeffect(&instruction),
                    Opcode::RotateRightCarrySwapSideeffect(_, _) => handlers.rotate_right_carry_swap_sideeffect(&instruction),
                    Opcode::Complement(_) => handlers.complement(&instruction),
                    Opcode::SetBit(_, _, _) => handlers.set_bit(&instruction),
                    Opcode::Halt(_) => return Err(GgError::CpuHalted),
                    Opcode::Exchange(_, _, _) => handlers.exchange(&instruction),
                    Opcode::ExchangeAll(_) => handlers.exchange_all(&instruction),
                    Opcode::TestBit(_, _, _) => handlers.test_bit(&instruction),
                    Opcode::LoadRepeat(_) => handlers.load_repeat(&instruction),
                    Opcode::InvertCarry(_) => handlers.invert_carry(&instruction),
                    Opcode::AddCarry(_, _, _) => handlers.add_carry(&instruction),
                    Opcode::SubtractCarry(_, _, _) => handlers.subtract_carry(&instruction),
                    Opcode::NoOperation(_) => Ok(()),
                    _ => {
                        error!("Handler missing for instruction: {}\n{}", instruction.opcode, self);
                        return Err(GgError::OpcodeNotImplemented {
                            opcode: instruction.opcode,
                        });
                    }
                };

                match result {
                    Err(GgError::BusRequestOutOfBounds { address }) => {
                        error!("Bus request out of bounds: {:04x}\n{}", address, self);
                        return Err(GgError::BusRequestOutOfBounds { address });
                    }
                    _ => (),
                }

                let skip = match instruction.opcode {
                    Opcode::Call(_, _, _) => result.is_ok(),
                    Opcode::Jump(_, _, _) => result.is_ok(),
                    Opcode::Return(_, _) => result.is_ok(),
                    Opcode::Restart(_, _) => result.is_ok(),
                    // Do NOT increase PC if the repeat instruction's condition is not met
                    Opcode::LoadRepeat(_) => result.is_err(),
                    Opcode::LoadIndirectRepeat(_) => result.is_err(),
                    Opcode::OutIndirectRepeat(_) => result.is_err(),
                    _ => false,
                };

                if !skip {
                    self.registers.pc += instruction.length as u16;
                }

                if result.is_ok() {
                    Ok(instruction)
                } else {
                    Err(result.unwrap_err())
                }
            }
            Err(msg) => Err(GgError::DecoderError { msg }),
        }
    }

    pub(crate) fn queue_irq(&mut self) {
        // todo: i think we might even just execute the handler right away...
        // perhaps just execute trigger_irq directly instead of waiting for a new tick?
        self.irq_available = true;
    }

    pub(crate) fn trigger_irq(&mut self, bus: &mut Bus) -> Result<(), GgError> {
        if self.interrupts_enabled {
            let instr_length = self.decode_at_pc(bus).unwrap().length as u16;
            let vector = match self.interrupt_mode {
                InterruptMode::IM0 => 0x0038,
                InterruptMode::IM1 => 0x0038,
                InterruptMode::IM2 => 0x0038,
            };
            self.push_stack(bus, self.registers.pc + instr_length)?;
            self.registers.pc = vector;
        }
        
        Ok(())
    }

    pub(crate) fn write_io(&mut self, port: u8, value: u8, vdp: &mut Vdp, bus: &mut Bus, psg: &mut Psg) -> Result<(), GgError> {
        match port {
            0x00..=0x06 => bus.write_io(port, value)?,
            DATA_PORT | CONTROL_PORT => vdp.write_io(port, value)?,
            MEMORY_CONTROL_PORT => bus.write_io(port, value)?,
            0x7f => psg.write_io(port, value)?,
            0xfc => (),
            0xfd => (),
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
            0xdc | 0xdd => bus.read_io(port),
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
        let get_offset = |offset: Option<i8>| match offset {
            Some(offset) => offset as i16,
            None => 0,
        };

        match register {
            Reg16::AF => ((self.registers.a as u16) << 8) | (self.registers.f.bits() as u16),
            Reg16::BC => ((self.registers.b as u16) << 8) | (self.registers.c as u16),
            Reg16::DE => ((self.registers.d as u16) << 8) | (self.registers.e as u16),
            Reg16::HL => ((self.registers.h as u16) << 8) | (self.registers.l as u16),
            Reg16::AFShadow => ((self.registers.a_shadow as u16) << 8) | (self.registers.f_shadow.bits() as u16),
            Reg16::BCShadow => ((self.registers.b_shadow as u16) << 8) | (self.registers.c_shadow as u16),
            Reg16::DEShadow => ((self.registers.d_shadow as u16) << 8) | (self.registers.e_shadow as u16),
            Reg16::HLShadow => ((self.registers.h_shadow as u16) << 8) | (self.registers.l_shadow as u16),
            Reg16::SP => self.registers.sp,
            Reg16::PC => self.registers.pc,
            Reg16::IX(offset) => self.registers.ix.wrapping_add_signed(get_offset(offset)),
            Reg16::IY(offset) => self.registers.iy.wrapping_add_signed(get_offset(offset)),
        }
    }

    pub(crate) fn get_register_u8(&self, register: Reg8) -> u8 {
        let high = |value: u16| (value >> 8) as u8;
        let low = |value: u16| value as u8;

        match register {
            Reg8::A => self.registers.a,
            Reg8::B => self.registers.b,
            Reg8::C => self.registers.c,
            Reg8::D => self.registers.d,
            Reg8::E => self.registers.e,
            Reg8::H => self.registers.h,
            Reg8::L => self.registers.l,
            Reg8::F => self.registers.f.bits(),
            Reg8::AShadow => self.registers.a_shadow,
            Reg8::BShadow => self.registers.b_shadow,
            Reg8::CShadow => self.registers.c_shadow,
            Reg8::DShadow => self.registers.d_shadow,
            Reg8::EShadow => self.registers.e_shadow,
            Reg8::HShadow => self.registers.h_shadow,
            Reg8::LShadow => self.registers.l_shadow,
            Reg8::FShadow => self.registers.f_shadow.bits(),
            Reg8::IYH => high(self.registers.iy),
            Reg8::IYL => low(self.registers.iy),
            Reg8::IXH => high(self.registers.ix),
            Reg8::IXL => low(self.registers.ix),
        }
    }

    pub(crate) fn set_register_u16(&mut self, register: Reg16, value: u16) {
        match register {
            Reg16::AF => {
                self.registers.a = (value >> 8) as u8;
                self.registers.f = Flags::from_bits(value as u8).unwrap();
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
            Reg16::AFShadow => {
                self.registers.a_shadow = (value >> 8) as u8;
                self.registers.f_shadow = Flags::from_bits(value as u8).unwrap();
            }
            Reg16::BCShadow => {
                self.registers.b_shadow = (value >> 8) as u8;
                self.registers.c_shadow = value as u8;
            }
            Reg16::DEShadow => {
                self.registers.d_shadow = (value >> 8) as u8;
                self.registers.e_shadow = value as u8;
            }
            Reg16::HLShadow => {
                self.registers.h_shadow = (value >> 8) as u8;
                self.registers.l_shadow = value as u8;
            }
            Reg16::SP => self.registers.sp = value,
            Reg16::PC => self.registers.pc = value,
            Reg16::IX(_) => self.registers.ix = value,
            Reg16::IY(_) => self.registers.iy = value,
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
            Reg8::F => self.registers.f = Flags::from_bits(value).unwrap(),
            Reg8::AShadow => self.registers.a_shadow = value,
            Reg8::BShadow => self.registers.b_shadow = value,
            Reg8::CShadow => self.registers.c_shadow = value,
            Reg8::DShadow => self.registers.d_shadow = value,
            Reg8::EShadow => self.registers.e_shadow = value,
            Reg8::HShadow => self.registers.h_shadow = value,
            Reg8::LShadow => self.registers.l_shadow = value,
            Reg8::FShadow => self.registers.f_shadow = Flags::from_bits(value).unwrap(),
            Reg8::IYH => self.registers.iy = (self.registers.iy & 0x00ff) | ((value as u16) << 8),
            Reg8::IYL => self.registers.iy = (self.registers.iy & 0xff00) | (value as u16),
            Reg8::IXH => self.registers.ix = (self.registers.ix & 0x00ff) | ((value as u16) << 8),
            Reg8::IXL => self.registers.ix = (self.registers.ix & 0xff00) | (value as u16),
        }
    }

    pub(crate) fn push_stack(&mut self, bus: &mut Bus, value: u16) -> Result<(), GgError> {
        self.registers.sp -= 2;
        bus.write_word(self.registers.sp, value)?;
        Ok(())
    }

    pub(crate) fn pop_stack(&mut self, bus: &mut Bus) -> Result<u16, GgError> {
        let value = bus.read_word(self.registers.sp)?;
        trace!("Popped {:04x} from stack at {:04x}", value, self.registers.sp);
        println!("Popped {:04x} from stack at {:04x}", value, self.registers.sp);
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
        write!(f, "{}\n", self.registers.f)
    }
}

impl fmt::Debug for Cpu {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "AF: {:04x}  BC: {:04x}  DE: {:04x}  HL: {:04x}  AF': {:04x}  BC': {:04x}  DE': {:04x}  HL': {:04x}  PC: {:04x}  SP: {:04x}  Flags: {:08b} ({})",
            self.get_register_u16(Reg16::AF),
            self.get_register_u16(Reg16::BC),
            self.get_register_u16(Reg16::DE),
            self.get_register_u16(Reg16::HL),
            self.get_register_u16(Reg16::AFShadow),
            self.get_register_u16(Reg16::BCShadow),
            self.get_register_u16(Reg16::DEShadow),
            self.get_register_u16(Reg16::HLShadow),
            self.registers.pc,
            self.registers.sp,
            self.registers.f.bits(),
            self.registers.f
        )
    }
}