use crate::error::GgError;
use crate::{
    bus::Bus,
    cpu::{Cpu, Flags},
    io::IoMode,
};
use core::panic;
use log::trace;
use z80::instruction::{Condition, Immediate, Instruction, Opcode, Operand, Reg16, Reg8, Register};

pub(crate) struct Handlers;

impl Handlers {
    pub(crate) fn load(cpu: &mut Cpu, bus: &mut Bus, instruction: &Instruction) -> Result<(), GgError> {
        match instruction.opcode {
            Opcode::Load(
                Operand::Register(Register::Reg16(dst_register), dst_deref),
                Operand::Immediate(Immediate::U16(src_imm), src_deref),
                _,
            ) => {
                let mut imm = src_imm;
                if src_deref {
                    imm = bus.read_word(src_imm)?;
                }

                if dst_deref {
                    let reg = cpu.get_register_u16(dst_register);
                    let dst = bus.read_word(reg)?;
                    bus.write_word(dst, imm)?;
                } else {
                    cpu.set_register_u16(dst_register, imm);
                }

                Ok(())
            }
            Opcode::Load(
                Operand::Register(Register::Reg16(dst_register), true),
                Operand::Register(Register::Reg8(src_register), false),
                _,
            ) => {
                let dst = cpu.get_register_u16(dst_register);
                let src = cpu.get_register_u8(src_register);
                bus.write(dst, src)?;
                Ok(())
            }
            Opcode::Load(Operand::Register(Register::Reg8(dst_register), false), Operand::Immediate(Immediate::U8(src_imm), false), _) => {
                cpu.set_register_u8(dst_register, src_imm);
                Ok(())
            }
            Opcode::Load(Operand::Immediate(Immediate::U16(dst_imm), true), Operand::Register(Register::Reg16(src_register), false), _) => {
                bus.write_word(dst_imm, cpu.get_register_u16(src_register))?;
                Ok(())
            }
            Opcode::Load(Operand::Immediate(Immediate::U16(dst_imm), true), Operand::Register(Register::Reg8(src_register), false), _) => {
                bus.write(dst_imm, cpu.get_register_u8(src_register))?;
                Ok(())
            }
            Opcode::Load(Operand::Register(Register::Reg8(dst_reg), false), Operand::Register(Register::Reg8(src_reg), false), _) => {
                let src = cpu.get_register_u8(src_reg);
                cpu.set_register_u8(dst_reg, src);
                Ok(())
            }
            _ => panic!("Invalid opcode for load instruction: {}", instruction.opcode),
        }
    }

    pub(crate) fn jump(cpu: &mut Cpu, _bus: &mut Bus, instruction: &Instruction) -> Result<(), GgError> {
        match instruction.opcode {
            Opcode::Jump(Condition::None, Immediate::U16(imm), _) => {
                cpu.set_register_u16(Reg16::PC, imm);
                Ok(())
            }
            _ => panic!("Invalid opcode for jump instruction: {}", instruction.opcode),
        }
    }

    pub(crate) fn disable_interrupts(_cpu: &mut Cpu, _bus: &mut Bus, _instruction: &Instruction) -> Result<(), GgError> {
        trace!("Disabling interrupts");
        Ok(())
    }

    pub(crate) fn load_indirect_repeat(cpu: &mut Cpu, bus: &mut Bus, _instruction: &Instruction) -> Result<(), GgError> {
        loop {
            let src = {
                let hl = cpu.get_register_u16(Reg16::HL);
                bus.read(hl)?
            };
            let dst = cpu.get_register_u16(Reg16::DE);

            bus.write(dst, src)?;

            let hl = cpu.get_register_u16(Reg16::HL);
            let de = cpu.get_register_u16(Reg16::DE);
            cpu.set_register_u16(Reg16::HL, hl + 1);
            cpu.set_register_u16(Reg16::DE, de + 1);

            let bc = cpu.get_register_u16(Reg16::BC);
            cpu.set_register_u16(Reg16::BC, bc - 1);

            if cpu.get_register_u16(Reg16::BC) == 0 {
                break;
            }
        }

        Ok(())
    }

    pub(crate) fn out(cpu: &mut Cpu, bus: &mut Bus, instruction: &Instruction) -> Result<(), GgError> {
        match instruction.opcode {
            Opcode::Out(Operand::Immediate(Immediate::U8(dst_port), true), Operand::Register(Register::Reg8(src_reg), false), _) => {
                let imm = cpu.get_register_u8(src_reg);
                bus.push_io_request(dst_port, imm, IoMode::Write);
                Ok(())
            }
            _ => panic!("Invalid opcode for out instruction: {}", instruction.opcode),
        }
    }

    // todo: change formatting for function signatures
    pub(crate) fn in_(cpu: &mut Cpu, bus: &mut Bus, instruction: &Instruction) -> Result<(), GgError> {
        match instruction.opcode {
            Opcode::In(Operand::Register(Register::Reg8(dst_reg), false), Operand::Immediate(Immediate::U8(src_port), true), _) => {
                if let Some(imm) = bus.pop_io_request(src_port) {
                    cpu.set_register_u8(dst_reg, imm);
                    return Ok(());
                } else {
                    bus.push_io_request(src_port, 0x00, IoMode::Read);
                }

                Err(GgError::IoRequestNotFulfilled)
            }
            _ => panic!("Invalid opcode for out instruction: {}", instruction.opcode),
        }
    }

    pub(crate) fn compare(cpu: &mut Cpu, _bus: &mut Bus, instruction: &Instruction) -> Result<(), GgError> {
        match instruction.opcode {
            Opcode::Compare(Operand::Immediate(Immediate::U8(imm), false), _) => {
                let a = cpu.get_register_u8(Reg8::A);
                let result = a.wrapping_sub(imm);

                // todo: ???
                cpu.flags.set(Flags::SUBTRACT, true);
                cpu.flags.set(Flags::CARRY, a < imm);
                cpu.flags.set(Flags::HALF_CARRY, a < imm);
                cpu.flags.set(Flags::ZERO, result == 0);

                Ok(())
            }
            _ => panic!("Invalid opcode for subtract instruction: {}", instruction.opcode),
        }
    }

    pub(crate) fn jump_relative(cpu: &mut Cpu, _bus: &mut Bus, instruction: &Instruction) -> Result<(), GgError> {
        match instruction.opcode {
            Opcode::JumpRelative(Condition::NotZero, Immediate::S8(imm), _) => {
                if !cpu.flags.contains(Flags::ZERO) {
                    let pc = cpu.get_register_u16(Reg16::PC);
                    let dst = pc.wrapping_add_signed(imm.into());
                    cpu.set_register_u16(Reg16::PC, dst);
                }
                Ok(())
            }
            _ => panic!("Invalid opcode for jump relative instruction: {}", instruction.opcode),
        }
    }

    pub(crate) fn call_unconditional(cpu: &mut Cpu, bus: &mut Bus, instruction: &Instruction) -> Result<(), GgError> {
        match instruction.opcode {
            Opcode::CallUnconditional(Operand::Immediate(Immediate::U16(imm), false), instruction_length) => {
                let next_instruction_addr = cpu.get_register_u16(Reg16::PC) + instruction_length as u16;
                cpu.push_stack(bus, next_instruction_addr)?;
                cpu.set_register_u16(Reg16::PC, imm);
                Ok(())
            }
            _ => panic!("Invalid opcode for call instruction: {}", instruction.opcode),
        }
    }

    pub(crate) fn return_(cpu: &mut Cpu, bus: &mut Bus, instruction: &Instruction) -> Result<(), GgError> {
        match instruction.opcode {
            Opcode::Return(Condition::None, _) => {
                let addr = cpu.pop_stack(bus)?;
                cpu.set_register_u16(Reg16::PC, addr);
                Ok(())
            }
            _ => panic!("Invalid opcode for return instruction: {}", instruction.opcode),
        }
    }
}
