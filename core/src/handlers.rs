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
            Opcode::Load(
                Operand::Register(Register::Reg8(dst_register), false),
                Operand::Immediate(Immediate::U8(src_imm), false),
                _
            ) => {
                cpu.set_register_u8(dst_register, src_imm);
                Ok(())
            },
            Opcode::Load(
                Operand::Register(Register::Reg8(dst_register), false),
                Operand::Immediate(Immediate::U16(src_imm), true),
                _
            ) => {
                let src = bus.read(src_imm)?;
                cpu.set_register_u8(dst_register, src);
                Ok(())
            },
            Opcode::Load(
                Operand::Register(Register::Reg8(dst_register), false),
                Operand::Register(Register::Reg16(src_register), true),
                _
            ) => {
                let src = cpu.get_register_u16(src_register);
                let src = bus.read(src)?;
                cpu.set_register_u8(dst_register, src);
                Ok(())
            }
            Opcode::Load(
                Operand::Immediate(Immediate::U16(dst_imm), true),
                Operand::Register(Register::Reg16(src_register), false),
                _
            ) => {
                bus.write_word(dst_imm, cpu.get_register_u16(src_register))?;
                Ok(())
            }
            Opcode::Load(
                Operand::Immediate(Immediate::U16(dst_imm), true),
                Operand::Register(Register::Reg8(src_register), false), 
                _
            ) => {
                bus.write(dst_imm, cpu.get_register_u8(src_register))?;
                Ok(())
            }
            Opcode::Load(
                Operand::Register(Register::Reg8(dst_reg), false),
                Operand::Register(Register::Reg8(src_reg), false), 
                _
            ) => {
                let src = cpu.get_register_u8(src_reg);
                cpu.set_register_u8(dst_reg, src);
                Ok(())
            }
            _ => panic!("Invalid opcode for load instruction: {}", instruction.opcode),
        }
    }

    pub(crate) fn jump(cpu: &mut Cpu, _bus: &mut Bus, instruction: &Instruction) -> Result<(), GgError> {
        match instruction.opcode {
            Opcode::Jump(condition, Immediate::U16(imm), _) => {
                if Handlers::check_cpu_flag(cpu, condition) {
                    cpu.set_register_u16(Reg16::PC, imm);
                    return Ok(());
                }
                Err(GgError::JumpNotTaken)
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
        let (port, value) = match instruction.opcode {
            Opcode::Out(Operand::Immediate(Immediate::U8(dst_port), true), Operand::Register(Register::Reg8(src_reg), false), _) => {
                (dst_port, cpu.get_register_u8(src_reg))
            },
            Opcode::Out(Operand::Register(Register::Reg8(dst_port), true), Operand::Register(Register::Reg8(src_reg), false), _) => {
                let dst = cpu.get_register_u8(dst_port);
                let src = cpu.get_register_u8(src_reg);
                (dst, src)
            },
            _ => panic!("Invalid opcode for out instruction: {}", instruction.opcode),
        };

        bus.push_io_data(port, value, IoMode::Write, false);
        Ok(())
    }

    pub(crate) fn in_(cpu: &mut Cpu, bus: &mut Bus, instruction: &Instruction) -> Result<(), GgError> {
        match instruction.opcode {
            Opcode::In(
                Operand::Register(Register::Reg8(dst_reg), false),
                Operand::Immediate(Immediate::U8(src_port), true), 
                _
            ) => {
                if let Some(imm) = bus.pop_io_data(src_port, true) {
                    cpu.set_register_u8(dst_reg, imm);
                    return Ok(());
                } else if !bus.io.has_pending(src_port, IoMode::Read) {
                    bus.push_io_data(src_port, 0x00, IoMode::Read, false);
                }

                Err(GgError::IoRequestNotFulfilled)
            }
            _ => panic!("Invalid opcode for out instruction: {}", instruction.opcode),
        }
    }

    pub(crate) fn compare(cpu: &mut Cpu, bus: &mut Bus, instruction: &Instruction) -> Result<(), GgError> {
        let (carry, zero) = match instruction.opcode {
            Opcode::Compare(Operand::Immediate(Immediate::U8(imm), false), _) => {
                let a = cpu.get_register_u8(Reg8::A);
                let result = a.wrapping_sub(imm);
                (a < imm, result == 0)
            },
            Opcode::Compare(Operand::Register(Register::Reg16(src_reg), true), _) => {
                let src = {
                    let reg = cpu.get_register_u16(src_reg);
                    bus.read(reg)?
                };
                let a = cpu.get_register_u8(Reg8::A);
                let result = a.wrapping_sub(src);
                (a < src, result == 0)
            },
            _ => panic!("Invalid opcode for compare instruction: {}", instruction.opcode),
        };

        cpu.flags.set(Flags::SUBTRACT, true);
        cpu.flags.set(Flags::CARRY, carry);
        cpu.flags.set(Flags::HALF_CARRY, carry);
        cpu.flags.set(Flags::ZERO, zero);

        Ok(())
    }

    pub(crate) fn jump_relative(cpu: &mut Cpu, _bus: &mut Bus, instruction: &Instruction) -> Result<(), GgError> {
        match instruction.opcode {
            Opcode::JumpRelative(condition, Immediate::S8(imm), _) => {
                if Handlers::check_cpu_flag(cpu, condition) {
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
            Opcode::Return(condition, _) => {
                if Handlers::check_cpu_flag(cpu, condition) {
                    let addr = cpu.pop_stack(bus)?;
                    cpu.set_register_u16(Reg16::PC, addr);
                    return Ok(());
                }
                Err(GgError::JumpNotTaken)
            }
            _ => panic!("Invalid opcode for return instruction: {}", instruction.opcode),
        }
    }

    pub(crate) fn out_indirect_repeat(cpu: &mut Cpu, bus: &mut Bus, _instruction: &Instruction) -> Result<(), GgError> {
        loop {
            let b = cpu.get_register_u8(Reg8::B);
            let hl = cpu.get_register_u16(Reg16::HL);

            let value = bus.read(hl)?;
            let port = cpu.get_register_u8(Reg8::C);
            bus.push_io_data(port, value, IoMode::Write, false);

            cpu.set_register_u16(Reg16::HL, hl + 1);
            cpu.set_register_u8(Reg8::B, b.wrapping_sub(1));

            if cpu.get_register_u8(Reg8::B) == 0 {
                break;
            }
        }

        Ok(())
    }

    pub(crate) fn or(cpu: &mut Cpu, _bus: &mut Bus, instruction: &Instruction) -> Result<(), GgError> {
        match instruction.opcode {
            Opcode::Or(Operand::Register(Register::Reg8(src_reg), false), _) => {
                let a = cpu.get_register_u8(Reg8::A);
                let src = cpu.get_register_u8(src_reg);
                let result = a | src;

                cpu.set_register_u8(Reg8::A, result);

                cpu.flags.set(Flags::ZERO, result == 0);
                cpu.flags.set(Flags::SIGN, result & 0b1000_0000 != 0);
                // FIXME: cpu.flags.set(Flags::PARITY_OR_OVERFLOW, )
                cpu.flags.set(Flags::SUBTRACT, false);
                cpu.flags.set(Flags::HALF_CARRY, false);
                cpu.flags.set(Flags::CARRY, false);

                Ok(())
            }
            _ => panic!("Invalid opcode for or instruction: {}", instruction.opcode),
        }
    }

    pub(crate) fn push(cpu: &mut Cpu, bus: &mut Bus, instruction: &Instruction) -> Result<(), GgError> {
        match instruction.opcode {
            Opcode::Push(Register::Reg16(src_reg), _) => {
                let src = cpu.get_register_u16(src_reg);
                cpu.push_stack(bus, src)?;
                Ok(())
            }
            _ => panic!("Invalid opcode for push instruction: {}", instruction.opcode),
        }
    }

    pub(crate) fn pop(cpu: &mut Cpu, bus: &mut Bus, instruction: &Instruction) -> Result<(), GgError> {
        match instruction.opcode {
            Opcode::Pop(Register::Reg16(dst_reg), _) => {
                let dst = cpu.pop_stack(bus)?;
                cpu.set_register_u16(dst_reg, dst);
                Ok(())
            }
            _ => panic!("Invalid opcode for pop instruction: {}", instruction.opcode),
        }
    }

    pub(crate) fn increment(cpu: &mut Cpu, _bus: &mut Bus, instruction: &Instruction) -> Result<(), GgError> {
        match instruction.opcode {
            Opcode::Increment(Operand::Register(Register::Reg8(dst_reg), false), _) => {
                let dst = cpu.get_register_u8(dst_reg);
                let result = dst.wrapping_add(1);
                cpu.set_register_u8(dst_reg, result);

                Ok(())
            }
            Opcode::Increment(Operand::Register(Register::Reg16(dst_reg), false), _) => {
                let dst = cpu.get_register_u16(dst_reg);
                let result = dst.wrapping_add(1);
                cpu.set_register_u16(dst_reg, result);

                Ok(())
            }
            _ => panic!("Invalid opcode for inc instruction: {}", instruction.opcode),
        }
    }

    pub(crate) fn decrement(cpu: &mut Cpu, _bus: &mut Bus, instruction: &Instruction) -> Result<(), GgError> {
        match instruction.opcode {
            Opcode::Decrement(Operand::Register(Register::Reg8(dst_reg), false), _) => {
                let dst = cpu.get_register_u8(dst_reg);
                let result = dst.wrapping_sub(1);
                cpu.set_register_u8(dst_reg, result);

                Ok(())
            }
            Opcode::Decrement(Operand::Register(Register::Reg16(dst_reg), false), _) => {
                let dst = cpu.get_register_u16(dst_reg);
                let result = dst.wrapping_sub(1);
                cpu.set_register_u16(dst_reg, result);

                Ok(())
            }
            _ => panic!("Invalid opcode for dec instruction: {}", instruction.opcode),
        }
    }

    pub(crate) fn reset_bit(cpu: &mut Cpu, _bus: &mut Bus, instruction: &Instruction) -> Result<(), GgError> {
        match instruction.opcode {
            Opcode::ResetBit(
                Immediate::U8(bit), 
                Operand::Register(Register::Reg8(dst_reg), false), 
                _
            ) => {
                let dst = cpu.get_register_u8(dst_reg);
                let result = dst & !(1 << bit);
                cpu.set_register_u8(dst_reg, result);
                Ok(())
            }
            _ => panic!("Invalid opcode for reset bit instruction: {}", instruction.opcode),
        }
    }

    pub(crate) fn decrement_and_jump_relative(cpu: &mut Cpu, _bus: &mut Bus, instruction: &Instruction) -> Result<(), GgError> {
        match instruction.opcode {
            Opcode::DecrementAndJumpRelative(Immediate::S8(imm), _) => {
                let condition = cpu.get_register_u8(Reg8::B);
                let result = condition.wrapping_sub(1);
                cpu.set_register_u8(Reg8::B, result);

                if result != 0 {
                    let pc = cpu.get_register_u16(Reg16::PC);
                    let dst = pc.wrapping_add_signed(imm.into());
                    cpu.set_register_u16(Reg16::PC, dst);
                }

                Ok(())
            }
            _ => panic!("Invalid opcode for decrement and jump relative instruction: {}", instruction.opcode),
        }
    }

    pub(crate) fn xor(cpu: &mut Cpu, _bus: &mut Bus, instruction: &Instruction) -> Result<(), GgError> {
        match instruction.opcode {
            Opcode::Xor(Operand::Register(Register::Reg8(src_reg), false), _) => {
                let a = cpu.get_register_u8(Reg8::A);
                let src = cpu.get_register_u8(src_reg);
                let result = a ^ src;

                cpu.set_register_u8(Reg8::A, result);

                cpu.flags.set(Flags::ZERO, result == 0);
                cpu.flags.set(Flags::SIGN, result & 0b1000_0000 != 0);
                // todo: cpu.flags.set(Flags::PARITY_OR_OVERFLOW, )
                cpu.flags.set(Flags::SUBTRACT, false);
                cpu.flags.set(Flags::HALF_CARRY, false);
                cpu.flags.set(Flags::CARRY, false);

                Ok(())
            }
            _ => panic!("Invalid opcode for xor instruction: {}", instruction.opcode),
        }
    }

    pub(crate) fn outi(cpu: &mut Cpu, bus: &mut Bus, _instruction: &Instruction) -> Result<(), GgError> {
        let b = cpu.get_register_u8(Reg8::B);
        let result = b.wrapping_sub(1);
        cpu.set_register_u8(Reg8::B, result);

        let hl = cpu.get_register_u16(Reg16::HL);
        let value = bus.read(hl)?;

        let port = cpu.get_register_u8(Reg8::C);
        bus.push_io_data(port, value, IoMode::Write, false);

        cpu.set_register_u16(Reg16::HL, hl.wrapping_add(1));

        cpu.flags.set(Flags::ZERO, result == 0);
        cpu.flags.set(Flags::SUBTRACT, true);

        Ok(())
    }

    // Helpers

    fn check_cpu_flag(cpu: &Cpu, condition: Condition) -> bool {
        match condition {
            Condition::None => true,
            Condition::Carry => cpu.flags.contains(Flags::CARRY),
            Condition::NotCarry => !cpu.flags.contains(Flags::CARRY),
            Condition::Zero => cpu.flags.contains(Flags::ZERO),
            Condition::NotZero => !cpu.flags.contains(Flags::ZERO),
            Condition::Sign => cpu.flags.contains(Flags::SIGN),
            Condition::NotSign => !cpu.flags.contains(Flags::SIGN),
        }
    }
}
