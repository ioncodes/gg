use crate::bus::Bus;
use crate::cpu::InterruptMode;
use crate::error::GgError;

use crate::cpu::{Cpu, Flags};
use crate::psg::Psg;
use crate::vdp::Vdp;
use core::panic;
use z80::instruction::{Condition, Immediate, Instruction, Opcode, Operand, Reg16, Reg8, Register};

pub(crate) struct Handlers<'a> {
    cpu: &'a mut Cpu,
    bus: &'a mut Bus,
    vdp: &'a mut Vdp,
    psg: &'a mut Psg,
}

#[allow(unused_variables)]
impl<'a> Handlers<'a> {
    pub(crate) fn new(cpu: &'a mut Cpu, bus: &'a mut Bus, vdp: &'a mut Vdp, psg: &'a mut Psg) -> Handlers<'a> {
        Handlers { cpu, bus, vdp, psg }
    }

    pub(crate) fn load(&mut self, instruction: &Instruction) -> Result<(), GgError> {
        match instruction.opcode {
            Opcode::Load(
                Operand::Register(Register::Reg16(dst_register), dst_deref),
                Operand::Immediate(Immediate::U16(src_imm), src_deref),
                _,
            ) => {
                let mut imm = src_imm;
                if src_deref {
                    imm = self.bus.read_word(src_imm)?;
                }

                if dst_deref {
                    let reg = self.cpu.get_register_u16(dst_register);
                    let dst = self.bus.read_word(reg)?;
                    self.bus.write_word(dst, imm)?;
                } else {
                    self.cpu.set_register_u16(dst_register, imm);
                }

                Ok(())
            }
            Opcode::Load(
                Operand::Register(Register::Reg16(dst_register), dst_deref),
                Operand::Immediate(Immediate::U8(src_imm), false),
                _,
            ) => {
                if dst_deref {
                    let dst = self.cpu.get_register_u16(dst_register);
                    self.bus.write(dst, src_imm)?;
                } else {
                    // is this even possibel?
                    self.cpu.set_register_u16(dst_register, src_imm as u16);
                }

                Ok(())
            }
            Opcode::Load(
                Operand::Register(Register::Reg16(dst_register), true),
                Operand::Register(Register::Reg8(src_register), false),
                _,
            ) => {
                let dst = self.cpu.get_register_u16(dst_register);
                let src = self.cpu.get_register_u8(src_register);
                self.bus.write(dst, src)?;
                Ok(())
            }
            Opcode::Load(Operand::Register(Register::Reg8(dst_register), false), Operand::Immediate(Immediate::U8(src_imm), false), _) => {
                self.cpu.set_register_u8(dst_register, src_imm);
                Ok(())
            }
            Opcode::Load(Operand::Register(Register::Reg8(dst_register), false), Operand::Immediate(Immediate::U16(src_imm), true), _) => {
                let src = self.bus.read(src_imm)?;
                self.cpu.set_register_u8(dst_register, src);
                Ok(())
            }
            Opcode::Load(
                Operand::Register(Register::Reg8(dst_register), false),
                Operand::Register(Register::Reg16(src_register), true),
                _,
            ) => {
                let src = self.cpu.get_register_u16(src_register);
                let src = self.bus.read(src)?;
                self.cpu.set_register_u8(dst_register, src);
                Ok(())
            }
            Opcode::Load(Operand::Immediate(Immediate::U16(dst_imm), true), Operand::Register(Register::Reg16(src_register), false), _) => {
                self.bus.write_word(dst_imm, self.cpu.get_register_u16(src_register))?;
                Ok(())
            }
            Opcode::Load(Operand::Immediate(Immediate::U16(dst_imm), true), Operand::Register(Register::Reg8(src_register), false), _) => {
                self.bus.write(dst_imm, self.cpu.get_register_u8(src_register))?;
                Ok(())
            }
            Opcode::Load(Operand::Register(Register::Reg8(dst_reg), false), Operand::Register(Register::Reg8(src_reg), false), _) => {
                let src = self.cpu.get_register_u8(src_reg);
                self.cpu.set_register_u8(dst_reg, src);
                Ok(())
            }
            _ => Err(GgError::InvalidOpcodeImplementation {
                instruction: instruction.opcode,
            }),
        }
    }

    pub(crate) fn jump(&mut self, instruction: &Instruction) -> Result<(), GgError> {
        let dst = match instruction.opcode {
            Opcode::Jump(condition, Operand::Immediate(Immediate::U16(imm), deref), _) => {
                if self.check_cpu_flag(condition) {
                    Ok(if deref { self.bus.read_word(imm)? } else { imm })
                } else {
                    Err(GgError::JumpNotTaken)
                }
            }
            Opcode::Jump(condition, Operand::Register(Register::Reg16(reg), deref), _) => {
                let dst = self.cpu.get_register_u16(reg);
                if self.check_cpu_flag(condition) {
                    Ok(if deref { self.bus.read_word(dst)? } else { dst })
                } else {
                    Err(GgError::JumpNotTaken)
                }
            }
            _ => {
                return Err(GgError::InvalidOpcodeImplementation {
                    instruction: instruction.opcode,
                })
            }
        }?;

        self.cpu.set_register_u16(Reg16::PC, dst);
        Ok(())
    }

    pub(crate) fn set_interrupt_state(&mut self, enabled: bool, instruction: &Instruction) -> Result<(), GgError> {
        self.cpu.interrupts_enabled = enabled;
        Ok(())
    }

    pub(crate) fn load_indirect_repeat(&mut self, instruction: &Instruction) -> Result<(), GgError> {
        loop {
            let src = {
                let hl = self.cpu.get_register_u16(Reg16::HL);
                self.bus.read(hl)?
            };
            let dst = self.cpu.get_register_u16(Reg16::DE);

            self.bus.write(dst, src)?;

            let hl = self.cpu.get_register_u16(Reg16::HL);
            let de = self.cpu.get_register_u16(Reg16::DE);
            self.cpu.set_register_u16(Reg16::HL, hl + 1);
            self.cpu.set_register_u16(Reg16::DE, de + 1);

            let bc = self.cpu.get_register_u16(Reg16::BC);
            self.cpu.set_register_u16(Reg16::BC, bc - 1);

            if self.cpu.get_register_u16(Reg16::BC) == 0 {
                break;
            }
        }

        Ok(())
    }

    pub(crate) fn load_repeat(&mut self, instruction: &Instruction) -> Result<(), GgError> {
        loop {
            let src = {
                let hl = self.cpu.get_register_u16(Reg16::HL);
                self.bus.read(hl)?
            };
            let dst = self.cpu.get_register_u16(Reg16::DE);
            self.bus.write(dst, src)?;

            let hl = self.cpu.get_register_u16(Reg16::HL);
            let de = self.cpu.get_register_u16(Reg16::DE);
            let bc = self.cpu.get_register_u16(Reg16::BC);
            self.cpu.set_register_u16(Reg16::HL, hl - 1);
            self.cpu.set_register_u16(Reg16::DE, de - 1);
            self.cpu.set_register_u16(Reg16::BC, bc - 1);

            if self.cpu.get_register_u16(Reg16::BC) == 0 {
                break;
            }
        }

        self.cpu.flags.set(Flags::HALF_CARRY, false);
        self.cpu.flags.set(Flags::SUBTRACT, false);
        self.cpu.flags.set(Flags::PARITY_OR_OVERFLOW, false);

        Ok(())
    }

    pub(crate) fn out(&mut self, instruction: &Instruction) -> Result<(), GgError> {
        let (port, value) = match instruction.opcode {
            Opcode::Out(Operand::Immediate(Immediate::U8(dst_port), true), Operand::Register(Register::Reg8(src_reg), false), _) => {
                (dst_port, self.cpu.get_register_u8(src_reg))
            }
            Opcode::Out(Operand::Register(Register::Reg8(dst_port), true), Operand::Register(Register::Reg8(src_reg), false), _) => {
                let dst = self.cpu.get_register_u8(dst_port);
                let src = self.cpu.get_register_u8(src_reg);
                (dst, src)
            }
            _ => {
                return Err(GgError::InvalidOpcodeImplementation {
                    instruction: instruction.opcode,
                })
            }
        };

        self.cpu.write_io(port, value, self.vdp, self.bus, self.psg)?;

        Ok(())
    }

    pub(crate) fn in_(&mut self, instruction: &Instruction) -> Result<(), GgError> {
        match instruction.opcode {
            Opcode::In(Operand::Register(Register::Reg8(dst_reg), false), Operand::Immediate(Immediate::U8(src_port), true), _) => {
                let imm = self.cpu.read_io(src_port, self.vdp, self.bus, self.psg)?;
                self.cpu.set_register_u8(dst_reg, imm);
                Ok(())
            }
            _ => Err(GgError::InvalidOpcodeImplementation {
                instruction: instruction.opcode,
            }),
        }
    }

    pub(crate) fn compare(&mut self, instruction: &Instruction) -> Result<(), GgError> {
        let (carry, zero) = match instruction.opcode {
            Opcode::Compare(Operand::Immediate(Immediate::U8(imm), false), _) => {
                let a = self.cpu.get_register_u8(Reg8::A);
                let result = a.wrapping_sub(imm);
                (a < imm, result == 0)
            }
            Opcode::Compare(Operand::Register(Register::Reg16(src_reg), true), _) => {
                let src = {
                    let reg = self.cpu.get_register_u16(src_reg);
                    self.bus.read(reg)?
                };
                let a = self.cpu.get_register_u8(Reg8::A);
                let result = a.wrapping_sub(src);
                (a < src, result == 0)
            }
            _ => panic!("Invalid opcode for compare instruction: {}", instruction.opcode),
        };

        self.cpu.flags.set(Flags::SUBTRACT, true);
        self.cpu.flags.set(Flags::CARRY, carry);
        self.cpu.flags.set(Flags::HALF_CARRY, carry);
        self.cpu.flags.set(Flags::ZERO, zero);

        Ok(())
    }

    pub(crate) fn jump_relative(&mut self, instruction: &Instruction) -> Result<(), GgError> {
        match instruction.opcode {
            Opcode::JumpRelative(condition, Immediate::S8(imm), _) => {
                if self.check_cpu_flag(condition) {
                    let pc = self.cpu.get_register_u16(Reg16::PC);
                    let dst = pc.wrapping_add_signed(imm.into());
                    self.cpu.set_register_u16(Reg16::PC, dst);
                    Ok(())
                } else {
                    Err(GgError::JumpNotTaken)
                }
            }
            _ => Err(GgError::InvalidOpcodeImplementation {
                instruction: instruction.opcode,
            }),
        }
    }

    pub(crate) fn call(&mut self, instruction: &Instruction) -> Result<(), GgError> {
        match instruction.opcode {
            Opcode::Call(condition, Operand::Immediate(Immediate::U16(imm), false), instruction_length) => {
                if self.check_cpu_flag(condition) {
                    let next_instruction_addr = self.cpu.get_register_u16(Reg16::PC) + instruction_length as u16;
                    self.cpu.push_stack(self.bus, next_instruction_addr)?;
                    self.cpu.set_register_u16(Reg16::PC, imm);
                    Ok(())
                } else {
                    Err(GgError::JumpNotTaken)
                }
            }
            _ => Err(GgError::InvalidOpcodeImplementation {
                instruction: instruction.opcode,
            }),
        }
    }

    pub(crate) fn return_(&mut self, instruction: &Instruction) -> Result<(), GgError> {
        match instruction.opcode {
            Opcode::Return(condition, _) => {
                if self.check_cpu_flag(condition) {
                    let addr = self.cpu.pop_stack(self.bus)?;
                    self.cpu.set_register_u16(Reg16::PC, addr);
                    return Ok(());
                }
                Err(GgError::JumpNotTaken)
            }
            _ => Err(GgError::InvalidOpcodeImplementation {
                instruction: instruction.opcode,
            }),
        }
    }

    pub(crate) fn out_indirect_repeat(&mut self, instruction: &Instruction) -> Result<(), GgError> {
        loop {
            let b = self.cpu.get_register_u8(Reg8::B);
            let hl = self.cpu.get_register_u16(Reg16::HL);

            let value = self.bus.read(hl)?;
            let port = self.cpu.get_register_u8(Reg8::C);
            self.cpu.write_io(port, value, self.vdp, self.bus, self.psg)?;

            self.cpu.set_register_u16(Reg16::HL, hl + 1);
            self.cpu.set_register_u8(Reg8::B, b.wrapping_sub(1));

            if self.cpu.get_register_u8(Reg8::B) == 0 {
                break;
            }
        }

        Ok(())
    }

    pub(crate) fn or(&mut self, instruction: &Instruction) -> Result<(), GgError> {
        let src = match instruction.opcode {
            Opcode::Or(Operand::Register(Register::Reg16(src_reg), true), _) => self.bus.read(self.cpu.get_register_u16(src_reg))?,
            Opcode::Or(Operand::Register(Register::Reg8(src_reg), false), _) => self.cpu.get_register_u8(src_reg),
            Opcode::Or(Operand::Immediate(Immediate::U8(imm), false), _) => imm,
            _ => {
                return Err(GgError::InvalidOpcodeImplementation {
                    instruction: instruction.opcode,
                })
            }
        };

        let a = self.cpu.get_register_u8(Reg8::A);
        let result = a | src;

        self.cpu.set_register_u8(Reg8::A, result);

        self.cpu.flags.set(Flags::ZERO, result == 0);
        self.cpu.flags.set(Flags::SIGN, result & 0b1000_0000 != 0);
        // FIXME: self.cpu.flags.set(Flags::PARITY_OR_OVERFLOW, )
        self.cpu.flags.set(Flags::SUBTRACT, false);
        self.cpu.flags.set(Flags::HALF_CARRY, false);
        self.cpu.flags.set(Flags::CARRY, false);

        Ok(())
    }

    pub(crate) fn push(&mut self, instruction: &Instruction) -> Result<(), GgError> {
        match instruction.opcode {
            Opcode::Push(Register::Reg16(src_reg), _) => {
                let src = self.cpu.get_register_u16(src_reg);
                self.cpu.push_stack(self.bus, src)?;
                Ok(())
            }
            _ => Err(GgError::InvalidOpcodeImplementation {
                instruction: instruction.opcode,
            }),
        }
    }

    pub(crate) fn pop(&mut self, instruction: &Instruction) -> Result<(), GgError> {
        match instruction.opcode {
            Opcode::Pop(Register::Reg16(dst_reg), _) => {
                let dst = self.cpu.pop_stack(self.bus)?;
                self.cpu.set_register_u16(dst_reg, dst);
                Ok(())
            }
            _ => Err(GgError::InvalidOpcodeImplementation {
                instruction: instruction.opcode,
            }),
        }
    }

    pub(crate) fn increment(&mut self, instruction: &Instruction) -> Result<(), GgError> {
        match instruction.opcode {
            Opcode::Increment(Operand::Register(Register::Reg8(dst_reg), false), _) => {
                let dst = self.cpu.get_register_u8(dst_reg);
                let result = dst.wrapping_add(1);
                self.cpu.set_register_u8(dst_reg, result);

                Ok(())
            }
            Opcode::Increment(Operand::Register(Register::Reg16(dst_reg), false), _) => {
                let dst = self.cpu.get_register_u16(dst_reg);
                let result = dst.wrapping_add(1);
                self.cpu.set_register_u16(dst_reg, result);

                Ok(())
            }
            _ => Err(GgError::InvalidOpcodeImplementation {
                instruction: instruction.opcode,
            }),
        }
    }

    pub(crate) fn decrement(&mut self, instruction: &Instruction) -> Result<(), GgError> {
        match instruction.opcode {
            Opcode::Decrement(Operand::Register(Register::Reg8(dst_reg), false), _) => {
                let dst = self.cpu.get_register_u8(dst_reg);
                let result = dst.wrapping_sub(1);
                self.cpu.set_register_u8(dst_reg, result);

                self.cpu.flags.set(Flags::ZERO, result == 0);
                self.cpu.flags.set(Flags::SIGN, result & 0b1000_0000 != 0);
                self.cpu.flags.set(Flags::PARITY_OR_OVERFLOW, result.count_ones() % 2 == 0);
                self.cpu.flags.set(Flags::SUBTRACT, true);
                self.cpu.flags.set(Flags::HALF_CARRY, result & 0b0000_1111 == 0b0000_1111);

                Ok(())
            }
            Opcode::Decrement(Operand::Register(Register::Reg16(dst_reg), false), _) => {
                let dst = self.cpu.get_register_u16(dst_reg);
                let result = dst.wrapping_sub(1);
                self.cpu.set_register_u16(dst_reg, result);

                self.cpu.flags.set(Flags::ZERO, result == 0);
                self.cpu.flags.set(Flags::SIGN, result & 0b1000_0000 != 0);
                self.cpu.flags.set(Flags::PARITY_OR_OVERFLOW, result.count_ones() % 2 == 0);
                self.cpu.flags.set(Flags::SUBTRACT, true);
                self.cpu.flags.set(Flags::HALF_CARRY, result & 0b0000_1111 == 0b0000_1111);

                Ok(())
            }
            _ => Err(GgError::InvalidOpcodeImplementation {
                instruction: instruction.opcode,
            }),
        }
    }

    pub(crate) fn reset_bit(&mut self, instruction: &Instruction) -> Result<(), GgError> {
        match instruction.opcode {
            Opcode::ResetBit(Immediate::U8(bit), Operand::Register(Register::Reg8(dst_reg), false), _) => {
                let dst = self.cpu.get_register_u8(dst_reg);
                let result = dst & !(1 << bit);
                self.cpu.set_register_u8(dst_reg, result);
                Ok(())
            }
            _ => Err(GgError::InvalidOpcodeImplementation {
                instruction: instruction.opcode,
            }),
        }
    }

    pub(crate) fn decrement_and_jump_relative(&mut self, instruction: &Instruction) -> Result<(), GgError> {
        match instruction.opcode {
            Opcode::DecrementAndJumpRelative(Immediate::S8(imm), _) => {
                let condition = self.cpu.get_register_u8(Reg8::B);
                let result = condition.wrapping_sub(1);
                self.cpu.set_register_u8(Reg8::B, result);

                if result != 0 {
                    let pc = self.cpu.get_register_u16(Reg16::PC);
                    let dst = pc.wrapping_add_signed(imm.into());
                    self.cpu.set_register_u16(Reg16::PC, dst);
                }

                Ok(())
            }
            _ => Err(GgError::InvalidOpcodeImplementation {
                instruction: instruction.opcode,
            }),
        }
    }

    pub(crate) fn xor(&mut self, instruction: &Instruction) -> Result<(), GgError> {
        let value = match instruction.opcode {
            Opcode::Xor(Operand::Register(Register::Reg8(src_reg), deref), _) => self.cpu.get_register_u8(src_reg),
            Opcode::Xor(Operand::Register(Register::Reg16(src_reg), true), _) => {
                let a = self.cpu.get_register_u8(Reg8::A);
                let src = self.cpu.get_register_u16(src_reg);
                self.bus.read(src)?
            }
            Opcode::Xor(Operand::Immediate(Immediate::U8(imm), false), _) => imm,
            _ => {
                return Err(GgError::InvalidOpcodeImplementation {
                    instruction: instruction.opcode,
                })
            }
        };

        let a = self.cpu.get_register_u8(Reg8::A);
        let result = a ^ value;
        self.cpu.set_register_u8(Reg8::A, result);

        self.cpu.flags.set(Flags::ZERO, result == 0);
        self.cpu.flags.set(Flags::SIGN, result & 0b1000_0000 != 0);
        // todo: self.cpu.flags.set(Flags::PARITY_OR_OVERFLOW, )
        self.cpu.flags.set(Flags::SUBTRACT, false);
        self.cpu.flags.set(Flags::HALF_CARRY, false);
        self.cpu.flags.set(Flags::CARRY, false);

        Ok(())
    }

    pub(crate) fn outi(&mut self, instruction: &Instruction) -> Result<(), GgError> {
        let b = self.cpu.get_register_u8(Reg8::B);
        let result = b.wrapping_sub(1);
        self.cpu.set_register_u8(Reg8::B, result);

        let hl = self.cpu.get_register_u16(Reg16::HL);
        let value = self.bus.read(hl)?;

        let port = self.cpu.get_register_u8(Reg8::C);
        self.cpu.write_io(port, value, self.vdp, self.bus, self.psg)?;

        self.cpu.set_register_u16(Reg16::HL, hl.wrapping_add(1));

        self.cpu.flags.set(Flags::ZERO, result == 0);
        self.cpu.flags.set(Flags::SUBTRACT, true);

        Ok(())
    }

    pub(crate) fn restart(&mut self, instruction: &Instruction) -> Result<(), GgError> {
        match instruction.opcode {
            Opcode::Restart(Immediate::U8(imm), _) => {
                let pc = self.cpu.get_register_u16(Reg16::PC);
                self.cpu.push_stack(self.bus, pc + 1)?;
                self.cpu.set_register_u16(Reg16::PC, imm as u16);
                Ok(())
            }
            _ => Err(GgError::InvalidOpcodeImplementation {
                instruction: instruction.opcode,
            }),
        }
    }

    pub(crate) fn set_interrupt_mode(&mut self, instruction: &Instruction) -> Result<(), GgError> {
        match instruction.opcode {
            Opcode::SetInterruptMode(Immediate::U8(mode), _) => {
                self.cpu.interrupt_mode = InterruptMode::from(mode)?;
                Ok(())
            }
            _ => Err(GgError::InvalidOpcodeImplementation {
                instruction: instruction.opcode,
            }),
        }
    }

    pub(crate) fn and(&mut self, instruction: &Instruction) -> Result<(), GgError> {
        let src = match instruction.opcode {
            Opcode::And(Operand::Register(Register::Reg8(src_reg), false), _) => self.cpu.get_register_u8(src_reg),
            Opcode::And(Operand::Immediate(Immediate::U8(imm), false), _) => imm,
            _ => {
                return Err(GgError::InvalidOpcodeImplementation {
                    instruction: instruction.opcode,
                })
            }
        };

        let a = self.cpu.get_register_u8(Reg8::A);
        let result = a & src;

        self.cpu.set_register_u8(Reg8::A, result);

        self.cpu.flags.set(Flags::ZERO, result == 0);
        self.cpu.flags.set(Flags::SIGN, result & 0b1000_0000 != 0);

        Ok(())
    }

    pub(crate) fn subtract(&mut self, instruction: &Instruction) -> Result<(), GgError> {
        match instruction.opcode {
            Opcode::Subtract(Operand::Register(Register::Reg8(src_reg), false), _) => {
                let a = self.cpu.get_register_u8(Reg8::A);
                let src = self.cpu.get_register_u8(src_reg);
                let result = a.wrapping_sub(src);

                self.cpu.set_register_u8(Reg8::A, result);

                self.cpu.flags.set(Flags::ZERO, result == 0);
                self.cpu.flags.set(Flags::SIGN, result & 0b1000_0000 != 0);

                Ok(())
            }
            _ => Err(GgError::InvalidOpcodeImplementation {
                instruction: instruction.opcode,
            }),
        }
    }

    pub(crate) fn add(&mut self, instruction: &Instruction) -> Result<(), GgError> {
        match instruction.opcode {
            Opcode::Add(Operand::Register(Register::Reg8(dst_reg), false), Operand::Register(Register::Reg8(src_reg), false), _) => {
                let src = self.cpu.get_register_u8(src_reg);
                let dst = self.cpu.get_register_u8(dst_reg);
                let result = dst.wrapping_add(src);

                self.cpu.set_register_u8(dst_reg, result);

                self.cpu.flags.set(Flags::ZERO, result == 0);
                self.cpu.flags.set(Flags::SIGN, result & 0b1000_0000 != 0);
            }
            Opcode::Add(Operand::Register(Register::Reg16(dst_reg), false), Operand::Register(Register::Reg16(src_reg), false), _) => {
                let src = self.cpu.get_register_u16(src_reg);
                let dst = self.cpu.get_register_u16(dst_reg);
                let result = dst.wrapping_add(src);

                self.cpu.set_register_u16(dst_reg, result);

                self.cpu.flags.set(Flags::ZERO, result == 0);
                self.cpu.flags.set(Flags::SIGN, result & 0b1000_0000 != 0);
            }
            _ => {
                return Err(GgError::InvalidOpcodeImplementation {
                    instruction: instruction.opcode,
                })
            }
        }

        Ok(())
    }

    pub(crate) fn subtract_with_carry(&mut self, instruction: &Instruction) -> Result<(), GgError> {
        match instruction.opcode {
            Opcode::SubtractWithCarry(Register::Reg16(dst_reg), Register::Reg16(src_reg), _) => {
                let src = self.cpu.get_register_u16(src_reg);
                let dst = self.cpu.get_register_u16(dst_reg);
                let carry = if self.cpu.flags.contains(Flags::CARRY) { 1 } else { 0 };
                let result = dst.wrapping_sub(src).wrapping_sub(carry);
                self.cpu.set_register_u16(dst_reg, result);

                self.cpu.flags.set(Flags::ZERO, result == 0);
                self.cpu.flags.set(Flags::SIGN, result & 0b1000_0000 != 0);

                Ok(())
            }
            _ => Err(GgError::InvalidOpcodeImplementation {
                instruction: instruction.opcode,
            }),
        }
    }

    pub(crate) fn rotate_right_carry(&mut self, instruction: &Instruction) -> Result<(), GgError> {
        match instruction.opcode {
            Opcode::RotateRightCarry(_) => {
                let value = self.cpu.get_register_u8(Reg8::A);
                let carry = value & 0b0000_0001 != 0;
                let result = (value >> 1) | (if self.cpu.flags.contains(Flags::CARRY) { 0b1000_0000 } else { 0 });
                self.cpu.set_register_u8(Reg8::A, result);

                self.cpu.flags.set(Flags::CARRY, carry);
                self.cpu.flags.set(Flags::SUBTRACT, false);
                self.cpu.flags.set(Flags::HALF_CARRY, false);

                Ok(())
            }
            _ => Err(GgError::InvalidOpcodeImplementation {
                instruction: instruction.opcode,
            }),
        }
    }

    pub(crate) fn rotate_right_carry_swap(&mut self, instruction: &Instruction) -> Result<(), GgError> {
        match instruction.opcode {
            Opcode::RotateRightCarrySwap(_) => {
                let previous_carry = self.cpu.flags.contains(Flags::CARRY);
                let value = self.cpu.get_register_u8(Reg8::A);
                let carry = value & 0b0000_0001 != 0;
                let result = (value >> 1) | (if previous_carry { 0b1000_0000 } else { 0 });
                self.cpu.set_register_u8(Reg8::A, result);

                self.cpu.flags.set(Flags::CARRY, carry);
                self.cpu.flags.set(Flags::SUBTRACT, false);
                self.cpu.flags.set(Flags::HALF_CARRY, false);

                Ok(())
            }
            _ => Err(GgError::InvalidOpcodeImplementation {
                instruction: instruction.opcode,
            }),
        }
    }

    pub(crate) fn rotate_left_carry(&mut self, instruction: &Instruction) -> Result<(), GgError> {
        match instruction.opcode {
            Opcode::RotateLeftCarry(_) => {
                let value = self.cpu.get_register_u8(Reg8::A);
                let carry = value & 0b1000_0000 != 0;
                let result = (value << 1) | carry as u8;
                self.cpu.set_register_u8(Reg8::A, result);

                self.cpu.flags.set(Flags::CARRY, carry);
                self.cpu.flags.set(Flags::SUBTRACT, false);
                self.cpu.flags.set(Flags::HALF_CARRY, false);

                Ok(())
            }
            _ => Err(GgError::InvalidOpcodeImplementation {
                instruction: instruction.opcode,
            }),
        }
    }

    pub(crate) fn rotate_left_carry_swap(&mut self, instruction: &Instruction) -> Result<(), GgError> {
        match instruction.opcode {
            Opcode::RotateLeftCarrySwap(_) => {
                let previous_carry = self.cpu.flags.contains(Flags::CARRY);
                let value = self.cpu.get_register_u8(Reg8::A);
                let carry = value & 0b1000_0000 != 0;
                let result = (value << 1) | previous_carry as u8;
                self.cpu.set_register_u8(Reg8::A, result);

                self.cpu.flags.set(Flags::CARRY, carry);
                self.cpu.flags.set(Flags::SUBTRACT, false);
                self.cpu.flags.set(Flags::HALF_CARRY, false);

                Ok(())
            }
            _ => Err(GgError::InvalidOpcodeImplementation {
                instruction: instruction.opcode,
            }),
        }
    }

    pub(crate) fn rotate_right_carry_sideeffect(&mut self, instruction: &Instruction) -> Result<(), GgError> {
        match instruction.opcode {
            Opcode::RotateRightCarrySideeffect(Operand::Register(Register::Reg8(dst_reg), false), _) => {
                let value = self.cpu.get_register_u8(dst_reg);
                let carry = value & 0b0000_0001 != 0;
                let result = (value >> 1) | (if self.cpu.flags.contains(Flags::CARRY) { 0b1000_0000 } else { 0 });
                self.cpu.set_register_u8(dst_reg, result);

                self.cpu.flags.set(Flags::CARRY, carry);
                self.cpu.flags.set(Flags::ZERO, result == 0);
                self.cpu.flags.set(Flags::SIGN, result & 0b1000_0000 != 0);
                self.cpu.flags.set(Flags::SUBTRACT, false);
                self.cpu.flags.set(Flags::HALF_CARRY, false);

                Ok(())
            }
            _ => Err(GgError::InvalidOpcodeImplementation {
                instruction: instruction.opcode,
            }),
        }
    }

    pub(crate) fn rotate_right_carry_swap_sideeffect(&mut self, instruction: &Instruction) -> Result<(), GgError> {
        match instruction.opcode {
            Opcode::RotateRightCarrySwapSideeffect(Operand::Register(Register::Reg8(dst_reg), false), _) => {
                let previous_carry = self.cpu.flags.contains(Flags::CARRY);
                let value = self.cpu.get_register_u8(dst_reg);
                let carry = value & 0b0000_0001 != 0;
                let result = (value >> 1) | (if previous_carry { 0b1000_0000 } else { 0 });
                self.cpu.set_register_u8(dst_reg, result);

                self.cpu.flags.set(Flags::CARRY, carry);
                self.cpu.flags.set(Flags::SUBTRACT, false);
                self.cpu.flags.set(Flags::HALF_CARRY, false);

                Ok(())
            }
            _ => Err(GgError::InvalidOpcodeImplementation {
                instruction: instruction.opcode,
            }),
        }
    }

    pub(crate) fn complement(&mut self, instruction: &Instruction) -> Result<(), GgError> {
        match instruction.opcode {
            Opcode::Complement(_) => {
                let a = self.cpu.get_register_u8(Reg8::A);
                let result = !a;
                self.cpu.set_register_u8(Reg8::A, result);

                self.cpu.flags.set(Flags::SUBTRACT, true);
                self.cpu.flags.set(Flags::HALF_CARRY, true);

                Ok(())
            }
            _ => Err(GgError::InvalidOpcodeImplementation {
                instruction: instruction.opcode,
            }),
        }
    }

    pub(crate) fn set_bit(&mut self, instruction: &Instruction) -> Result<(), GgError> {
        match instruction.opcode {
            Opcode::SetBit(Immediate::U8(bit), Operand::Register(Register::Reg8(dst_reg), false), _) => {
                let dst = self.cpu.get_register_u8(dst_reg);
                let result = dst | (1 << bit);
                self.cpu.set_register_u8(dst_reg, result);
                Ok(())
            }
            Opcode::SetBit(Immediate::U8(bit), Operand::Register(Register::Reg16(dst_reg), true), _) => {
                let dst = self.cpu.get_register_u16(dst_reg);
                let value = self.bus.read(dst)?;
                let result = value | (1 << bit);
                self.bus.write(dst, result)?;
                Ok(())
            }
            _ => Err(GgError::InvalidOpcodeImplementation {
                instruction: instruction.opcode,
            }),
        }
    }

    pub(crate) fn exchange(&mut self, instruction: &Instruction) -> Result<(), GgError> {
        match instruction.opcode {
            Opcode::Exchange(Register::Reg16(lhs_reg), Register::Reg16(rhs_reg), _) => {
                let rhs = self.cpu.get_register_u16(rhs_reg);
                let lhs = self.cpu.get_register_u16(lhs_reg);
                self.cpu.set_register_u16(rhs_reg, lhs);
                self.cpu.set_register_u16(lhs_reg, rhs);
                Ok(())
            }
            _ => Err(GgError::InvalidOpcodeImplementation {
                instruction: instruction.opcode,
            }),
        }
    }

    pub(crate) fn exchange_all(&mut self, instruction: &Instruction) -> Result<(), GgError> {
        let de = self.cpu.get_register_u16(Reg16::DE);
        let hl = self.cpu.get_register_u16(Reg16::HL);
        let bc = self.cpu.get_register_u16(Reg16::BC);
        let af = self.cpu.get_register_u16(Reg16::AF);

        let de_ = self.cpu.get_register_u16(Reg16::DEShadow);
        let hl_ = self.cpu.get_register_u16(Reg16::HLShadow);
        let bc_ = self.cpu.get_register_u16(Reg16::BCShadow);
        let af_ = self.cpu.get_register_u16(Reg16::AFShadow);

        self.cpu.set_register_u16(Reg16::DE, de_);
        self.cpu.set_register_u16(Reg16::HL, hl_);
        self.cpu.set_register_u16(Reg16::BC, bc_);
        self.cpu.set_register_u16(Reg16::AF, af_);

        self.cpu.set_register_u16(Reg16::DEShadow, de);
        self.cpu.set_register_u16(Reg16::HLShadow, hl);
        self.cpu.set_register_u16(Reg16::BCShadow, bc);
        self.cpu.set_register_u16(Reg16::AFShadow, af);

        Ok(())
    }

    pub(crate) fn test_bit(&mut self, instruction: &Instruction) -> Result<(), GgError> {
        let (src, bit) = match instruction.opcode {
            Opcode::TestBit(Immediate::U8(bit), Operand::Register(Register::Reg8(src_reg), false), _) => {
                (self.cpu.get_register_u8(src_reg), bit)
            }
            Opcode::TestBit(Immediate::U8(bit), Operand::Register(Register::Reg16(src_reg), true), _) => {
                let src = self.cpu.get_register_u16(src_reg);
                (self.bus.read(src)?, bit)
            }
            _ => {
                return Err(GgError::InvalidOpcodeImplementation {
                    instruction: instruction.opcode,
                })
            }
        };

        let result = src & (1 << bit);
        self.cpu.flags.set(Flags::ZERO, result == 0);
        self.cpu.flags.set(Flags::HALF_CARRY, true);
        self.cpu.flags.set(Flags::SUBTRACT, false);

        Ok(())
    }

    // Helpers

    fn check_cpu_flag(&self, condition: Condition) -> bool {
        match condition {
            Condition::None => true,
            Condition::Carry => self.cpu.flags.contains(Flags::CARRY),
            Condition::NotCarry => !self.cpu.flags.contains(Flags::CARRY),
            Condition::Zero => self.cpu.flags.contains(Flags::ZERO),
            Condition::NotZero => !self.cpu.flags.contains(Flags::ZERO),
            Condition::Sign => self.cpu.flags.contains(Flags::SIGN),
            Condition::NotSign => !self.cpu.flags.contains(Flags::SIGN),
        }
    }
}
