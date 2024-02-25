use crate::bus::Bus;
use crate::cpu::InterruptMode;
use crate::error::GgError;

use crate::cpu::{Cpu, Flags};
use crate::psg::Psg;
use crate::vdp::Vdp;
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
            Opcode::Load(Operand::Register(Register::Reg16(dst_reg), false), Operand::Register(Register::Reg16(src_reg), false), _) => {
                let src = self.cpu.get_register_u16(src_reg);
                self.cpu.set_register_u16(dst_reg, src);
                Ok(())
            }
            Opcode::Load(Operand::Register(Register::Reg16(dst_reg), true), Operand::Register(Register::Reg16(src_reg), false), _) => {
                let src = self.cpu.get_register_u16(src_reg);
                self.bus.write_word(self.cpu.get_register_u16(dst_reg), src)?;
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
            Opcode::Jump(condition, Operand::Register(Register::Reg16(reg), true), _) => {
                let dst = self.cpu.get_register_u16(reg);
                if self.check_cpu_flag(condition) {
                    Ok(dst)
                } else {
                    Err(GgError::JumpNotTaken)
                }
            }
            _ => {
                return Err(GgError::InvalidOpcodeImplementation {
                    instruction: instruction.opcode,
                });
            }
        }?;

        self.cpu.set_register_u16(Reg16::PC, dst);
        Ok(())
    }

    pub(crate) fn set_interrupt_state(&mut self, enabled: bool, instruction: &Instruction) -> Result<(), GgError> {
        self.cpu.interrupts_enabled = enabled;
        Ok(())
    }

    pub(crate) fn load_increment_repeat(&mut self, instruction: &Instruction) -> Result<(), GgError> {
        let src = {
            let hl = self.cpu.get_register_u16(Reg16::HL);
            self.bus.read(hl)?
        };
        let dst = self.cpu.get_register_u16(Reg16::DE);

        self.bus.write(dst, src)?;

        let hl = self.cpu.get_register_u16(Reg16::HL);
        let de = self.cpu.get_register_u16(Reg16::DE);
        self.cpu.set_register_u16(Reg16::HL, hl.wrapping_add(1));
        self.cpu.set_register_u16(Reg16::DE, de.wrapping_add(1));

        let bc = self.cpu.get_register_u16(Reg16::BC);
        self.cpu.set_register_u16(Reg16::BC, bc.wrapping_sub(1));

        self.cpu.registers.f.set(Flags::HALF_CARRY, false);
        self.cpu.registers.f.set(Flags::SUBTRACT, false);
        self.cpu.registers.f.set(Flags::PARITY_OR_OVERFLOW, bc.wrapping_sub(1) > 0);

        if self.cpu.get_register_u16(Reg16::BC) == 0 {
            Ok(())
        } else {
            Err(GgError::RepeatNotFulfilled)
        }
    }

    pub(crate) fn load_decrement_repeat(&mut self, instruction: &Instruction) -> Result<(), GgError> {
        let src = {
            let hl = self.cpu.get_register_u16(Reg16::HL);
            self.bus.read(hl)?
        };
        let dst = self.cpu.get_register_u16(Reg16::DE);
        self.bus.write(dst, src)?;

        let hl = self.cpu.get_register_u16(Reg16::HL);
        let de = self.cpu.get_register_u16(Reg16::DE);
        let bc = self.cpu.get_register_u16(Reg16::BC);
        self.cpu.set_register_u16(Reg16::HL, hl.wrapping_sub(1));
        self.cpu.set_register_u16(Reg16::DE, de.wrapping_sub(1));
        self.cpu.set_register_u16(Reg16::BC, bc.wrapping_sub(1));

        self.cpu.registers.f.set(Flags::HALF_CARRY, false);
        self.cpu.registers.f.set(Flags::SUBTRACT, false);
        self.cpu.registers.f.set(Flags::PARITY_OR_OVERFLOW, bc.wrapping_sub(1) > 0);

        if self.cpu.get_register_u16(Reg16::BC) == 0 {
            Ok(())
        } else {
            Err(GgError::RepeatNotFulfilled)
        }
    }

    pub(crate) fn load_decrement(&mut self, instruction: &Instruction) -> Result<(), GgError> {
        let src = {
            let hl = self.cpu.get_register_u16(Reg16::HL);
            self.bus.read(hl)?
        };
        let dst = self.cpu.get_register_u16(Reg16::DE);
        self.bus.write(dst, src)?;

        let hl = self.cpu.get_register_u16(Reg16::HL);
        let de = self.cpu.get_register_u16(Reg16::DE);
        let bc = self.cpu.get_register_u16(Reg16::BC);
        self.cpu.set_register_u16(Reg16::HL, hl.wrapping_sub(1));
        self.cpu.set_register_u16(Reg16::DE, de.wrapping_sub(1));
        self.cpu.set_register_u16(Reg16::BC, bc.wrapping_sub(1));

        self.cpu.registers.f.set(Flags::HALF_CARRY, false);
        self.cpu.registers.f.set(Flags::SUBTRACT, false);
        self.cpu.registers.f.set(Flags::PARITY_OR_OVERFLOW, bc.wrapping_sub(1) > 0);

        Ok(())
    }

    pub(crate) fn load_increment(&mut self, instruction: &Instruction) -> Result<(), GgError> {
        let src = {
            let hl = self.cpu.get_register_u16(Reg16::HL);
            self.bus.read(hl)?
        };
        let dst = self.cpu.get_register_u16(Reg16::DE);
        self.bus.write(dst, src)?;

        let hl = self.cpu.get_register_u16(Reg16::HL);
        let de = self.cpu.get_register_u16(Reg16::DE);
        let bc = self.cpu.get_register_u16(Reg16::BC);
        self.cpu.set_register_u16(Reg16::HL, hl.wrapping_add(1));
        self.cpu.set_register_u16(Reg16::DE, de.wrapping_add(1));
        self.cpu.set_register_u16(Reg16::BC, bc.wrapping_sub(1));

        self.cpu.registers.f.set(Flags::HALF_CARRY, false);
        self.cpu.registers.f.set(Flags::SUBTRACT, false);
        self.cpu.registers.f.set(Flags::PARITY_OR_OVERFLOW, bc.wrapping_sub(1) > 0);

        Ok(())
    }

    pub(crate) fn negate(&mut self, instruction: &Instruction) -> Result<(), GgError> {
        let a = self.cpu.get_register_u8(Reg8::A);
        let result = a.wrapping_neg();
        self.cpu.set_register_u8(Reg8::A, result);

        self.cpu.registers.f.set(Flags::CARRY, result > 0);
        self.cpu.registers.f.set(Flags::ZERO, result == 0);
        self.cpu.registers.f.set(Flags::SIGN, result & 0b1000_0000 != 0);
        self.cpu.registers.f.set(Flags::PARITY_OR_OVERFLOW, result == 128);
        self.cpu.registers.f.set(Flags::SUBTRACT, true);
        self.cpu
            .registers
            .f
            .set(Flags::HALF_CARRY, self.detect_half_carry_u8(0, a, result));

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
                });
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
            Opcode::In(Operand::Register(Register::Reg8(dst_reg), false), Operand::Register(Register::Reg8(src_reg), true), _) => {
                let src = self.cpu.get_register_u8(src_reg);
                let imm = self.cpu.read_io(src, self.vdp, self.bus, self.psg)?;
                self.cpu.set_register_u8(dst_reg, imm);
                Ok(())
            }
            _ => Err(GgError::InvalidOpcodeImplementation {
                instruction: instruction.opcode,
            }),
        }
    }

    pub(crate) fn ini(&mut self, instruction: &Instruction) -> Result<(), GgError> {
        let c = self.cpu.get_register_u8(Reg8::C);
        let hl = self.cpu.get_register_u16(Reg16::HL);
        let imm = self.cpu.read_io(c, self.vdp, self.bus, self.psg)?;
        self.bus.write(hl, imm)?;

        self.cpu.set_register_u16(Reg16::HL, hl.wrapping_add(1));
        let b = self.cpu.get_register_u8(Reg8::B);
        self.cpu.set_register_u8(Reg8::B, b.wrapping_sub(1));

        self.cpu.registers.f.set(Flags::ZERO, b.wrapping_sub(1) == 0);
        self.cpu.registers.f.set(Flags::SUBTRACT, true);

        Ok(())
    }

    pub(crate) fn compare(&mut self, instruction: &Instruction) -> Result<(), GgError> {
        let (lhs, rhs, result) = match instruction.opcode {
            Opcode::Compare(Operand::Immediate(Immediate::U8(imm), false), _) => {
                let a = self.cpu.get_register_u8(Reg8::A);
                let result = a.wrapping_sub(imm);
                (a, imm, result)
            }
            Opcode::Compare(Operand::Register(Register::Reg16(src_reg), true), _) => {
                let src = {
                    let reg = self.cpu.get_register_u16(src_reg);
                    self.bus.read(reg)?
                };
                let a = self.cpu.get_register_u8(Reg8::A);
                let result = a.wrapping_sub(src);
                (a, src, result)
            }
            Opcode::Compare(Operand::Register(Register::Reg8(src_reg), false), _) => {
                let src = self.cpu.get_register_u8(src_reg);
                let a = self.cpu.get_register_u8(Reg8::A);
                let result = a.wrapping_sub(src);
                (a, src, result)
            }
            _ => panic!("Invalid opcode for compare instruction: {}", instruction.opcode),
        };

        self.detect_half_carry_u8(lhs, rhs, result);
        self.cpu.registers.f.set(Flags::SUBTRACT, true);
        self.cpu.registers.f.set(Flags::CARRY, result > lhs);
        self.cpu
            .registers
            .f
            .set(Flags::HALF_CARRY, self.detect_half_carry_u8(lhs, rhs, result));
        self.cpu.registers.f.set(Flags::ZERO, result == 0);
        self.cpu.registers.f.set(Flags::SIGN, result & 0b1000_0000 != 0);
        self.cpu
            .registers
            .f
            .set(Flags::PARITY_OR_OVERFLOW, self.is_underflow(lhs, rhs, result));

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

    pub(crate) fn return_from_irq(&mut self, instruction: &Instruction) -> Result<(), GgError> {
        match instruction.opcode {
            Opcode::ReturnFromIrq(_) => {
                let addr = self.cpu.pop_stack(self.bus)?;
                self.cpu.set_register_u16(Reg16::PC, addr);
                Ok(())
            }
            _ => Err(GgError::InvalidOpcodeImplementation {
                instruction: instruction.opcode,
            }),
        }
    }

    pub(crate) fn out_increment_repeat(&mut self, instruction: &Instruction) -> Result<(), GgError> {
        let b = self.cpu.get_register_u8(Reg8::B);
        let hl = self.cpu.get_register_u16(Reg16::HL);

        let value = self.bus.read(hl)?;
        let port = self.cpu.get_register_u8(Reg8::C);
        self.cpu.write_io(port, value, self.vdp, self.bus, self.psg)?;

        self.cpu.set_register_u16(Reg16::HL, hl + 1);
        self.cpu.set_register_u8(Reg8::B, b.wrapping_sub(1));

        if self.cpu.get_register_u8(Reg8::B) == 0 {
            Ok(())
        } else {
            Err(GgError::RepeatNotFulfilled)
        }
    }

    pub(crate) fn out_decrement_repeat(&mut self, instruction: &Instruction) -> Result<(), GgError> {
        let b = self.cpu.get_register_u8(Reg8::B);
        let hl = self.cpu.get_register_u16(Reg16::HL);

        let value = self.bus.read(hl)?;
        let port = self.cpu.get_register_u8(Reg8::C);
        self.cpu.write_io(port, value, self.vdp, self.bus, self.psg)?;

        self.cpu.set_register_u16(Reg16::HL, hl.wrapping_sub(1));
        self.cpu.set_register_u8(Reg8::B, b.wrapping_sub(1));

        if self.cpu.get_register_u8(Reg8::B) == 0 {
            Ok(())
        } else {
            Err(GgError::RepeatNotFulfilled)
        }
    }

    pub(crate) fn compare_increment_repeat(&mut self, instruction: &Instruction) -> Result<(), GgError> {
        let src = {
            let hl = self.cpu.get_register_u16(Reg16::HL);
            self.bus.read(hl)?
        };
        let a = self.cpu.get_register_u8(Reg8::A);
        let result = a.wrapping_sub(src);

        self.cpu.set_register_u8(Reg8::A, result);

        let hl = self.cpu.get_register_u16(Reg16::HL);
        self.cpu.set_register_u16(Reg16::HL, hl.wrapping_add(1));

        let bc = self.cpu.get_register_u16(Reg16::BC);
        self.cpu.set_register_u16(Reg16::BC, bc.wrapping_sub(1));

        self.cpu
            .registers
            .f
            .set(Flags::HALF_CARRY, self.detect_half_carry_u8(a, src, result));
        self.cpu.registers.f.set(Flags::SUBTRACT, true);
        self.cpu.registers.f.set(Flags::CARRY, result > a);
        self.cpu.registers.f.set(Flags::ZERO, result == 0);
        self.cpu.registers.f.set(Flags::SIGN, result & 0b1000_0000 != 0);
        self.cpu
            .registers
            .f
            .set(Flags::PARITY_OR_OVERFLOW, self.cpu.get_register_u16(Reg16::BC) != 0);

        if self.cpu.get_register_u16(Reg16::BC) == 0 && self.cpu.registers.f.contains(Flags::ZERO) {
            Ok(())
        } else {
            Err(GgError::RepeatNotFulfilled)
        }
    }

    pub(crate) fn or(&mut self, instruction: &Instruction) -> Result<(), GgError> {
        let src = match instruction.opcode {
            Opcode::Or(Operand::Register(Register::Reg16(src_reg), true), _) => self.bus.read(self.cpu.get_register_u16(src_reg))?,
            Opcode::Or(Operand::Register(Register::Reg8(src_reg), false), _) => self.cpu.get_register_u8(src_reg),
            Opcode::Or(Operand::Immediate(Immediate::U8(imm), false), _) => imm,
            _ => {
                return Err(GgError::InvalidOpcodeImplementation {
                    instruction: instruction.opcode,
                });
            }
        };

        let a = self.cpu.get_register_u8(Reg8::A);
        let result = a | src;

        self.cpu.set_register_u8(Reg8::A, result);

        self.cpu.registers.f.set(Flags::ZERO, result == 0);
        self.cpu.registers.f.set(Flags::SIGN, result & 0b1000_0000 != 0);
        self.cpu.registers.f.set(Flags::PARITY_OR_OVERFLOW, self.check_parity(result));
        self.cpu.registers.f.set(Flags::SUBTRACT, false);
        self.cpu.registers.f.set(Flags::HALF_CARRY, false);
        self.cpu.registers.f.set(Flags::CARRY, false);

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

                self.cpu.registers.f.set(Flags::ZERO, result == 0);
                self.cpu.registers.f.set(Flags::SIGN, result & 0b1000_0000 != 0);
                self.cpu.registers.f.set(Flags::PARITY_OR_OVERFLOW, result == 128);
                self.cpu.registers.f.set(Flags::SUBTRACT, false);
                self.cpu.registers.f.set(Flags::HALF_CARRY, result & 0b0000_1111 == 0);

                Ok(())
            }
            Opcode::Increment(Operand::Register(Register::Reg16(dst_reg), false), _) => {
                let dst = self.cpu.get_register_u16(dst_reg);
                let result = dst.wrapping_add(1);
                self.cpu.set_register_u16(dst_reg, result);

                Ok(())
            }
            Opcode::Increment(Operand::Register(Register::Reg16(dst_reg), true), _) => {
                let dst = self.cpu.get_register_u16(dst_reg);
                let value = self.bus.read(dst)?;
                let result = value.wrapping_add(1);
                self.bus.write(dst, result)?;

                self.cpu.registers.f.set(Flags::ZERO, result == 0);
                self.cpu.registers.f.set(Flags::SIGN, result & 0b1000_0000 != 0);
                self.cpu.registers.f.set(Flags::PARITY_OR_OVERFLOW, result == 128); // result overflowed from 127 to -128
                self.cpu.registers.f.set(Flags::SUBTRACT, false);
                self.cpu.registers.f.set(Flags::HALF_CARRY, result & 0b0000_1111 == 0);

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

                self.cpu.registers.f.set(Flags::ZERO, result == 0);
                self.cpu.registers.f.set(Flags::SIGN, result & 0b1000_0000 != 0);
                self.cpu.registers.f.set(Flags::PARITY_OR_OVERFLOW, result == 127);
                self.cpu.registers.f.set(Flags::SUBTRACT, true);
                self.cpu.registers.f.set(Flags::HALF_CARRY, result & 0b0000_1111 == 0b0000_1111);

                Ok(())
            }
            Opcode::Decrement(Operand::Register(Register::Reg16(dst_reg), true), _) => {
                let dst = self.cpu.get_register_u16(dst_reg);
                let value = self.bus.read(dst)?;
                let result = value.wrapping_sub(1);
                self.bus.write(dst, result)?;

                self.cpu.registers.f.set(Flags::ZERO, result == 0);
                self.cpu.registers.f.set(Flags::SIGN, result & 0b1000_0000 != 0);
                self.cpu.registers.f.set(Flags::PARITY_OR_OVERFLOW, result == 127);
                self.cpu.registers.f.set(Flags::SUBTRACT, true);
                self.cpu.registers.f.set(Flags::HALF_CARRY, result & 0b0000_1111 == 0b0000_1111);

                Ok(())
            }
            Opcode::Decrement(Operand::Register(Register::Reg16(dst_reg), false), _) => {
                let dst = self.cpu.get_register_u16(dst_reg);
                let result = dst.wrapping_sub(1);
                self.cpu.set_register_u16(dst_reg, result);

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
            Opcode::ResetBit(Immediate::U8(bit), Operand::Register(Register::Reg16(dst_reg), true), _) => {
                let dst = self.cpu.get_register_u16(dst_reg);
                let value = self.bus.read(dst)?;
                let result = value & !(1 << bit);
                self.bus.write(dst, result)?;
                Ok(())
            }
            _ => Err(GgError::InvalidOpcodeImplementation {
                instruction: instruction.opcode,
            }),
        }
    }

    pub(crate) fn reset_bit_store(&mut self, instruction: &Instruction) -> Result<(), GgError> {
        match instruction.opcode {
            Opcode::ResetBitStore(
                Immediate::U8(bit),
                Operand::Register(Register::Reg16(src_reg), true),
                Operand::Register(Register::Reg8(dst_reg), false),
                _,
            ) => {
                let src = self.cpu.get_register_u16(src_reg);
                let value = self.bus.read(src)?;
                let dst = self.cpu.get_register_u8(dst_reg);
                let result = value & !(1 << bit);
                self.cpu.set_register_u8(dst_reg, result);
                self.bus.write(src, result)?;
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

    pub(crate) fn xor(&mut self, instruction: &Instruction) -> Result<(), GgError> {
        let value = match instruction.opcode {
            Opcode::Xor(Operand::Register(Register::Reg8(src_reg), deref), _) => self.cpu.get_register_u8(src_reg),
            Opcode::Xor(Operand::Register(Register::Reg16(src_reg), true), _) => {
                let src = self.cpu.get_register_u16(src_reg);
                self.bus.read(src)?
            }
            Opcode::Xor(Operand::Immediate(Immediate::U8(imm), false), _) => imm,
            _ => {
                return Err(GgError::InvalidOpcodeImplementation {
                    instruction: instruction.opcode,
                });
            }
        };

        let a = self.cpu.get_register_u8(Reg8::A);
        let result = a ^ value;
        self.cpu.set_register_u8(Reg8::A, result);

        self.cpu.registers.f.set(Flags::ZERO, result == 0);
        self.cpu.registers.f.set(Flags::SIGN, result & 0b1000_0000 != 0);
        self.cpu.registers.f.set(Flags::PARITY_OR_OVERFLOW, self.check_parity(result));
        self.cpu.registers.f.set(Flags::SUBTRACT, false);
        self.cpu.registers.f.set(Flags::HALF_CARRY, false);
        self.cpu.registers.f.set(Flags::CARRY, false);

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

        self.cpu.registers.f.set(Flags::ZERO, result == 0);
        self.cpu.registers.f.set(Flags::SUBTRACT, true);

        Ok(())
    }

    pub(crate) fn outd(&mut self, instruction: &Instruction) -> Result<(), GgError> {
        let b = self.cpu.get_register_u8(Reg8::B);
        let result = b.wrapping_sub(1);
        self.cpu.set_register_u8(Reg8::B, result);

        let hl = self.cpu.get_register_u16(Reg16::HL);
        let value = self.bus.read(hl)?;

        let port = self.cpu.get_register_u8(Reg8::C);
        self.cpu.write_io(port, value, self.vdp, self.bus, self.psg)?;

        self.cpu.set_register_u16(Reg16::HL, hl.wrapping_sub(1));

        self.cpu.registers.f.set(Flags::ZERO, result == 0);
        self.cpu.registers.f.set(Flags::SUBTRACT, true);

        Ok(())
    }

    pub(crate) fn restart(&mut self, instruction: &Instruction) -> Result<(), GgError> {
        match instruction.opcode {
            Opcode::Restart(Immediate::U8(imm), _) => {
                let pc = self.cpu.get_register_u16(Reg16::PC);
                self.cpu.push_stack(self.bus, pc.wrapping_add(1))?;
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
            Opcode::And(Operand::Register(Register::Reg16(src_reg), true), _) => {
                let src = self.cpu.get_register_u16(src_reg);
                self.bus.read(src)?
            }
            _ => {
                return Err(GgError::InvalidOpcodeImplementation {
                    instruction: instruction.opcode,
                });
            }
        };

        let a = self.cpu.get_register_u8(Reg8::A);
        let result = a & src;

        self.cpu.set_register_u8(Reg8::A, result);

        self.cpu.registers.f.set(Flags::ZERO, result == 0);
        self.cpu.registers.f.set(Flags::SIGN, result & 0b1000_0000 != 0);
        self.cpu.registers.f.set(Flags::PARITY_OR_OVERFLOW, self.check_parity(result));
        self.cpu.registers.f.set(Flags::CARRY, false);
        self.cpu.registers.f.set(Flags::HALF_CARRY, true);
        self.cpu.registers.f.set(Flags::SUBTRACT, false);

        Ok(())
    }

    pub(crate) fn subtract(&mut self, instruction: &Instruction) -> Result<(), GgError> {
        let value = match instruction.opcode {
            Opcode::Subtract(Operand::Register(Register::Reg8(src_reg), false), _) => self.cpu.get_register_u8(src_reg),
            Opcode::Subtract(Operand::Immediate(Immediate::U8(imm), false), _) => imm,
            Opcode::Subtract(Operand::Register(Register::Reg16(src_reg), true), _) => {
                let src = self.cpu.get_register_u16(src_reg);
                self.bus.read(src)?
            }
            _ => {
                return Err(GgError::InvalidOpcodeImplementation {
                    instruction: instruction.opcode,
                });
            }
        };

        let a = self.cpu.get_register_u8(Reg8::A);
        let result = a.wrapping_sub(value);

        self.cpu.set_register_u8(Reg8::A, result);

        let hc = self.detect_half_carry_u8(a, value, result);
        self.cpu.registers.f.set(Flags::ZERO, result == 0);
        self.cpu.registers.f.set(Flags::SIGN, result & 0b1000_0000 != 0);
        self.cpu
            .registers
            .f
            .set(Flags::PARITY_OR_OVERFLOW, self.is_underflow(a, value, result));
        self.cpu.registers.f.set(Flags::SUBTRACT, true);
        self.cpu.registers.f.set(Flags::HALF_CARRY, hc);
        self.cpu.registers.f.set(Flags::CARRY, result > a);

        Ok(())
    }

    pub(crate) fn add(&mut self, instruction: &Instruction) -> Result<(), GgError> {
        match instruction.opcode {
            Opcode::Add(Operand::Register(Register::Reg8(dst_reg), false), Operand::Register(Register::Reg8(src_reg), false), _) => {
                let src = self.cpu.get_register_u8(src_reg);
                let dst = self.cpu.get_register_u8(dst_reg);
                let result = dst.wrapping_add(src);

                self.cpu.set_register_u8(dst_reg, result);

                let hc = self.detect_half_carry_u8(dst, src, result);
                self.cpu.registers.f.set(Flags::CARRY, result < dst);
                self.cpu.registers.f.set(Flags::HALF_CARRY, hc);
                self.cpu.registers.f.set(Flags::SUBTRACT, false);
                self.cpu.registers.f.set(Flags::ZERO, result == 0);
                self.cpu.registers.f.set(Flags::SIGN, result & 0b1000_0000 != 0);
                self.cpu
                    .registers
                    .f
                    .set(Flags::PARITY_OR_OVERFLOW, self.is_overflow(dst, src, result));
            }
            Opcode::Add(Operand::Register(Register::Reg16(dst_reg), false), Operand::Register(Register::Reg16(src_reg), false), _) => {
                let src = self.cpu.get_register_u16(src_reg);
                let dst = self.cpu.get_register_u16(dst_reg);
                let result = dst.wrapping_add(src);

                self.cpu.set_register_u16(dst_reg, result);

                let hc = self.detect_half_carry_u16(dst, src, result);
                self.cpu.registers.f.set(Flags::CARRY, result < dst);
                self.cpu.registers.f.set(Flags::HALF_CARRY, hc);
                self.cpu.registers.f.set(Flags::SUBTRACT, false);
            }
            Opcode::Add(Operand::Register(Register::Reg8(dst_reg), false), Operand::Immediate(Immediate::U8(imm), false), _) => {
                let dst = self.cpu.get_register_u8(dst_reg);
                let result = dst.wrapping_add(imm);

                self.cpu.set_register_u8(dst_reg, result);

                let hc = self.detect_half_carry_u8(dst, imm, result);
                self.cpu.registers.f.set(Flags::CARRY, result < dst);
                self.cpu.registers.f.set(Flags::SUBTRACT, false);
                self.cpu.registers.f.set(Flags::HALF_CARRY, hc);
                self.cpu.registers.f.set(Flags::ZERO, result == 0);
                self.cpu.registers.f.set(Flags::SIGN, result & 0b1000_0000 != 0);
                self.cpu
                    .registers
                    .f
                    .set(Flags::PARITY_OR_OVERFLOW, self.is_overflow(dst, imm, result));
            }
            Opcode::Add(Operand::Register(Register::Reg8(dst_reg), false), Operand::Register(Register::Reg16(src_reg), true), _) => {
                let dst = self.cpu.get_register_u8(dst_reg);
                let src = self.cpu.get_register_u16(src_reg);
                let src = self.bus.read(src)?;
                let result = dst.wrapping_add(src);

                self.cpu.set_register_u8(dst_reg, result);

                let hc = self.detect_half_carry_u8(dst, src, result);
                self.cpu.registers.f.set(Flags::CARRY, result < dst);
                self.cpu.registers.f.set(Flags::SUBTRACT, false);
                self.cpu.registers.f.set(Flags::HALF_CARRY, hc);
                self.cpu.registers.f.set(Flags::ZERO, result == 0);
                self.cpu.registers.f.set(Flags::SIGN, result & 0b1000_0000 != 0);
                self.cpu
                    .registers
                    .f
                    .set(Flags::PARITY_OR_OVERFLOW, self.is_overflow(dst, src, result));
            }
            _ => {
                return Err(GgError::InvalidOpcodeImplementation {
                    instruction: instruction.opcode,
                });
            }
        }

        Ok(())
    }

    pub(crate) fn subtract_carry(&mut self, instruction: &Instruction) -> Result<(), GgError> {
        match instruction.opcode {
            Opcode::SubtractCarry(
                Operand::Register(Register::Reg16(dst_reg), false),
                Operand::Register(Register::Reg16(src_reg), false),
                _,
            ) => {
                let src = self.cpu.get_register_u16(src_reg);
                let dst = self.cpu.get_register_u16(dst_reg);
                let carry = if self.cpu.registers.f.contains(Flags::CARRY) { 1 } else { 0 };
                let (result, carry) = self.sub_and_detect_carry(dst, src, carry);
                self.cpu.set_register_u16(dst_reg, result);

                let hc = self.detect_half_carry_u16(src, dst, result);
                self.cpu.registers.f.set(Flags::CARRY, carry);
                self.cpu.registers.f.set(Flags::HALF_CARRY, hc);
                self.cpu.registers.f.set(Flags::SUBTRACT, true);
                self.cpu.registers.f.set(Flags::ZERO, result == 0);
                self.cpu.registers.f.set(Flags::SIGN, result & 0b1000_0000_0000_0000 != 0);
                self.cpu
                    .registers
                    .f
                    .set(Flags::PARITY_OR_OVERFLOW, self.is_underflow_u16(dst, src, result));

                Ok(())
            }
            Opcode::SubtractCarry(
                Operand::Register(Register::Reg8(dst_reg), false),
                Operand::Register(Register::Reg8(src_reg), false),
                _,
            ) => {
                let src = self.cpu.get_register_u8(src_reg);
                let dst = self.cpu.get_register_u8(dst_reg);
                let carry = if self.cpu.registers.f.contains(Flags::CARRY) { 1 } else { 0 };
                let (result, carry) = self.sub_and_detect_carry(dst, src, carry);
                self.cpu.set_register_u8(dst_reg, result);

                let hc = self.detect_half_carry_u8(src, dst, result);
                self.cpu.registers.f.set(Flags::CARRY, carry);
                self.cpu.registers.f.set(Flags::HALF_CARRY, hc);
                self.cpu.registers.f.set(Flags::SUBTRACT, true);
                self.cpu.registers.f.set(Flags::ZERO, result == 0);
                self.cpu.registers.f.set(Flags::SIGN, result & 0b1000_0000 != 0);
                self.cpu
                    .registers
                    .f
                    .set(Flags::PARITY_OR_OVERFLOW, self.is_underflow(dst, src, result));

                Ok(())
            }
            Opcode::SubtractCarry(
                Operand::Register(Register::Reg8(dst_reg), false),
                Operand::Immediate(Immediate::U8(src_imm), false),
                _,
            ) => {
                let dst = self.cpu.get_register_u8(dst_reg);
                let carry = if self.cpu.registers.f.contains(Flags::CARRY) { 1 } else { 0 };
                let (result, carry) = self.sub_and_detect_carry(dst, src_imm, carry);
                self.cpu.set_register_u8(dst_reg, result);

                let hc = self.detect_half_carry_u8(src_imm, dst, result);
                self.cpu.registers.f.set(Flags::CARRY, carry);
                self.cpu.registers.f.set(Flags::HALF_CARRY, hc);
                self.cpu.registers.f.set(Flags::SUBTRACT, true);
                self.cpu.registers.f.set(Flags::ZERO, result == 0);
                self.cpu.registers.f.set(Flags::SIGN, result & 0b1000_0000 != 0);
                self.cpu
                    .registers
                    .f
                    .set(Flags::PARITY_OR_OVERFLOW, self.is_underflow(dst, src_imm, result));

                Ok(())
            }
            Opcode::SubtractCarry(
                Operand::Register(Register::Reg8(dst_reg), false),
                Operand::Register(Register::Reg16(src_reg), true),
                _,
            ) => {
                let src = self.cpu.get_register_u16(src_reg);
                let src = self.bus.read(src)?;
                let dst = self.cpu.get_register_u8(dst_reg);
                let carry = if self.cpu.registers.f.contains(Flags::CARRY) { 1 } else { 0 };
                let (result, carry) = self.sub_and_detect_carry(dst, src, carry);
                self.cpu.set_register_u8(dst_reg, result);

                let hc = self.detect_half_carry_u8(src, dst, result);
                self.cpu.registers.f.set(Flags::CARRY, carry);
                self.cpu.registers.f.set(Flags::HALF_CARRY, hc);
                self.cpu.registers.f.set(Flags::SUBTRACT, true);
                self.cpu.registers.f.set(Flags::ZERO, result == 0);
                self.cpu.registers.f.set(Flags::SIGN, result & 0b1000_0000 != 0);
                self.cpu
                    .registers
                    .f
                    .set(Flags::PARITY_OR_OVERFLOW, self.is_underflow(dst, src, result));

                Ok(())
            }
            _ => Err(GgError::InvalidOpcodeImplementation {
                instruction: instruction.opcode,
            }),
        }
    }

    pub(crate) fn rotate_left_digit(&mut self, instruction: &Instruction) -> Result<(), GgError> {
        match instruction.opcode {
            Opcode::RotateLeftDigit(_) => {
                let a = self.cpu.get_register_u8(Reg8::A);
                let hl = self.cpu.get_register_u16(Reg16::HL);
                let value = self.bus.read(hl)?;
                let result = (a & 0b0000_1111) | (value << 4);
                self.cpu.set_register_u8(Reg8::A, result);
                let result = (value >> 4) | ((a & 0b0000_1111) << 4);
                self.bus.write(hl, result)?;

                self.cpu.registers.f.set(Flags::SUBTRACT, false);
                self.cpu.registers.f.set(Flags::HALF_CARRY, false);
                self.cpu.registers.f.set(Flags::PARITY_OR_OVERFLOW, self.check_parity(result));

                Ok(())
            }
            _ => Err(GgError::InvalidOpcodeImplementation {
                instruction: instruction.opcode,
            }),
        }
    }

    pub(crate) fn rotate_right_digit(&mut self, instruction: &Instruction) -> Result<(), GgError> {
        match instruction.opcode {
            Opcode::RotateRightDigit(_) => {
                let a = self.cpu.get_register_u8(Reg8::A);
                let hl = self.cpu.get_register_u16(Reg16::HL);
                let value = self.bus.read(hl)?;
                let result = (a & 0b1111_0000) | (value & 0b0000_1111);
                self.cpu.set_register_u8(Reg8::A, result);
                let result = (value >> 4) | ((a & 0b1111_0000) << 4);
                self.bus.write(hl, result)?;

                self.cpu.registers.f.set(Flags::SUBTRACT, false);
                self.cpu.registers.f.set(Flags::HALF_CARRY, false);
                self.cpu.registers.f.set(Flags::PARITY_OR_OVERFLOW, self.check_parity(result));

                Ok(())
            }
            _ => Err(GgError::InvalidOpcodeImplementation {
                instruction: instruction.opcode,
            }),
        }
    }

    pub(crate) fn rotate_right_carry_accumulator(&mut self, instruction: &Instruction) -> Result<(), GgError> {
        match instruction.opcode {
            Opcode::RotateRightCarryAccumulator(_) => {
                let value = self.cpu.get_register_u8(Reg8::A);
                let carry = value & 0b0000_0001 > 0;
                let value = if carry { value >> 1 | 0b1000_0000 } else { value >> 1 };

                self.cpu.set_register_u8(Reg8::A, value);

                self.cpu.registers.f.set(Flags::CARRY, carry);
                self.cpu.registers.f.set(Flags::SUBTRACT, false);
                self.cpu.registers.f.set(Flags::HALF_CARRY, false);

                Ok(())
            }
            _ => Err(GgError::InvalidOpcodeImplementation {
                instruction: instruction.opcode,
            }),
        }
    }

    pub(crate) fn rotate_right_accumulator(&mut self, instruction: &Instruction) -> Result<(), GgError> {
        match instruction.opcode {
            Opcode::RotateRightAccumulator(_) => {
                let value = self.cpu.get_register_u8(Reg8::A);
                let previous_carry = self.cpu.registers.f.contains(Flags::CARRY);
                let carry = value & 0b0000_0001 > 0;

                let value = if previous_carry { value >> 1 | 0b1000_0000 } else { value >> 1 };

                self.cpu.set_register_u8(Reg8::A, value);

                self.cpu.registers.f.set(Flags::CARRY, carry);
                self.cpu.registers.f.set(Flags::SUBTRACT, false);
                self.cpu.registers.f.set(Flags::HALF_CARRY, false);

                Ok(())
            }
            _ => Err(GgError::InvalidOpcodeImplementation {
                instruction: instruction.opcode,
            }),
        }
    }

    pub(crate) fn rotate_left_carry_accumulator(&mut self, instruction: &Instruction) -> Result<(), GgError> {
        match instruction.opcode {
            Opcode::RotateLeftCarryAccumulator(_) => {
                let value = self.cpu.get_register_u8(Reg8::A);
                let carry = value & 0b1000_0000 > 0;
                let value = if carry { value << 1 | 0b0000_0001 } else { value << 1 };

                self.cpu.set_register_u8(Reg8::A, value);

                self.cpu.registers.f.set(Flags::CARRY, carry);
                self.cpu.registers.f.set(Flags::SUBTRACT, false);
                self.cpu.registers.f.set(Flags::HALF_CARRY, false);

                Ok(())
            }
            _ => Err(GgError::InvalidOpcodeImplementation {
                instruction: instruction.opcode,
            }),
        }
    }

    pub(crate) fn rotate_left_accumulator(&mut self, instruction: &Instruction) -> Result<(), GgError> {
        match instruction.opcode {
            Opcode::RotateLeftAccumulator(_) => {
                let value = self.cpu.get_register_u8(Reg8::A);
                let previous_carry = self.cpu.registers.f.contains(Flags::CARRY);
                let carry = value & 0b1000_0000 > 0;

                let value = if previous_carry { value << 1 | 0b0000_0001 } else { value << 1 };

                self.cpu.set_register_u8(Reg8::A, value);

                self.cpu.registers.f.set(Flags::CARRY, carry);
                self.cpu.registers.f.set(Flags::SUBTRACT, false);
                self.cpu.registers.f.set(Flags::HALF_CARRY, false);

                Ok(())
            }
            _ => Err(GgError::InvalidOpcodeImplementation {
                instruction: instruction.opcode,
            }),
        }
    }

    pub(crate) fn rotate_right_carry(&mut self, instruction: &Instruction) -> Result<(), GgError> {
        match instruction.opcode {
            Opcode::RotateRightCarry(Operand::Register(Register::Reg8(dst_reg), false), _) => {
                let value = self.cpu.get_register_u8(dst_reg);
                let carry = value & 0b0000_0001 != 0;
                let result = if carry { value >> 1 | 0b1000_0000 } else { value >> 1 };
                self.cpu.set_register_u8(dst_reg, result);

                self.cpu.registers.f.set(Flags::CARRY, carry);
                self.cpu.registers.f.set(Flags::ZERO, result == 0);
                self.cpu.registers.f.set(Flags::SIGN, result & 0b1000_0000 != 0);
                self.cpu.registers.f.set(Flags::SUBTRACT, false);
                self.cpu.registers.f.set(Flags::HALF_CARRY, false);
                self.cpu.registers.f.set(Flags::PARITY_OR_OVERFLOW, self.check_parity(result));

                Ok(())
            }
            Opcode::RotateRightCarry(Operand::Register(Register::Reg16(dst_reg), true), _) => {
                let dst = self.cpu.get_register_u16(dst_reg);
                let value = self.bus.read(dst)?;
                let carry = value & 0b0000_0001 > 0;
                let result = if carry { value >> 1 | 0b1000_0000 } else { value >> 1 };
                self.bus.write(dst, result)?;

                self.cpu.registers.f.set(Flags::CARRY, carry);
                self.cpu.registers.f.set(Flags::ZERO, result == 0);
                self.cpu.registers.f.set(Flags::SIGN, result & 0b1000_0000 != 0);
                self.cpu.registers.f.set(Flags::SUBTRACT, false);
                self.cpu.registers.f.set(Flags::HALF_CARRY, false);
                self.cpu.registers.f.set(Flags::PARITY_OR_OVERFLOW, self.check_parity(result));

                Ok(())
            }
            _ => Err(GgError::InvalidOpcodeImplementation {
                instruction: instruction.opcode,
            }),
        }
    }

    pub(crate) fn rotate_right(&mut self, instruction: &Instruction) -> Result<(), GgError> {
        match instruction.opcode {
            Opcode::RotateRight(Operand::Register(Register::Reg8(dst_reg), false), _) => {
                let previous_carry = self.cpu.registers.f.contains(Flags::CARRY);
                let value = self.cpu.get_register_u8(dst_reg);
                let carry = value & 0b0000_0001 != 0;
                let result = (value >> 1) | (if previous_carry { 0b1000_0000 } else { 0 });
                self.cpu.set_register_u8(dst_reg, result);

                self.cpu.registers.f.set(Flags::CARRY, carry);
                self.cpu.registers.f.set(Flags::ZERO, result == 0);
                self.cpu.registers.f.set(Flags::SIGN, result & 0b1000_0000 != 0);
                self.cpu.registers.f.set(Flags::SUBTRACT, false);
                self.cpu.registers.f.set(Flags::HALF_CARRY, false);
                self.cpu.registers.f.set(Flags::PARITY_OR_OVERFLOW, self.check_parity(result));

                Ok(())
            }
            Opcode::RotateRight(Operand::Register(Register::Reg16(dst_reg), true), _) => {
                let dst = self.cpu.get_register_u16(dst_reg);
                let value = self.bus.read(dst)?;
                let carry = value & 0b0000_0001 != 0;
                let previous_carry = self.cpu.registers.f.contains(Flags::CARRY);
                let result = (value >> 1) | (if previous_carry { 0b1000_0000 } else { 0 });
                self.bus.write(dst, result)?;

                self.cpu.registers.f.set(Flags::CARRY, carry);
                self.cpu.registers.f.set(Flags::ZERO, result == 0);
                self.cpu.registers.f.set(Flags::SIGN, result & 0b1000_0000 != 0);
                self.cpu.registers.f.set(Flags::SUBTRACT, false);
                self.cpu.registers.f.set(Flags::HALF_CARRY, false);
                self.cpu.registers.f.set(Flags::PARITY_OR_OVERFLOW, self.check_parity(result));

                Ok(())
            }
            _ => Err(GgError::InvalidOpcodeImplementation {
                instruction: instruction.opcode,
            }),
        }
    }

    pub(crate) fn rotate_left_carry(&mut self, instruction: &Instruction) -> Result<(), GgError> {
        match instruction.opcode {
            Opcode::RotateLeftCarry(Operand::Register(Register::Reg8(dst_reg), false), _) => {
                let value = self.cpu.get_register_u8(dst_reg);
                let carry = value & 0b1000_0000 > 0;
                let result = if carry { value << 1 | 0b0000_0001 } else { value << 1 };
                self.cpu.set_register_u8(dst_reg, result);

                self.cpu.registers.f.set(Flags::CARRY, carry);
                self.cpu.registers.f.set(Flags::ZERO, result == 0);
                self.cpu.registers.f.set(Flags::SIGN, result & 0b1000_0000 != 0);
                self.cpu.registers.f.set(Flags::SUBTRACT, false);
                self.cpu.registers.f.set(Flags::HALF_CARRY, false);
                self.cpu.registers.f.set(Flags::PARITY_OR_OVERFLOW, self.check_parity(result));

                Ok(())
            }
            Opcode::RotateLeftCarry(Operand::Register(Register::Reg16(dst_reg), true), _) => {
                let dst = self.cpu.get_register_u16(dst_reg);
                let value = self.bus.read(dst)?;
                let carry = value & 0b1000_0000 > 0;
                let result = if carry { value << 1 | 0b0000_0001 } else { value << 1 };
                self.bus.write(dst, result)?;

                self.cpu.registers.f.set(Flags::CARRY, carry);
                self.cpu.registers.f.set(Flags::ZERO, result == 0);
                self.cpu.registers.f.set(Flags::SIGN, result & 0b1000_0000 != 0);
                self.cpu.registers.f.set(Flags::SUBTRACT, false);
                self.cpu.registers.f.set(Flags::HALF_CARRY, false);
                self.cpu.registers.f.set(Flags::PARITY_OR_OVERFLOW, self.check_parity(result));

                Ok(())
            }
            _ => Err(GgError::InvalidOpcodeImplementation {
                instruction: instruction.opcode,
            }),
        }
    }

    pub(crate) fn rotate_left(&mut self, instruction: &Instruction) -> Result<(), GgError> {
        match instruction.opcode {
            Opcode::RotateLeft(Operand::Register(Register::Reg8(dst_reg), false), _) => {
                let previous_carry = self.cpu.registers.f.contains(Flags::CARRY);
                let value = self.cpu.get_register_u8(dst_reg);
                let carry = value & 0b1000_0000 != 0;
                let result = (value << 1) | (if previous_carry { 0b0000_0001 } else { 0 });
                self.cpu.set_register_u8(dst_reg, result);

                self.cpu.registers.f.set(Flags::CARRY, carry);
                self.cpu.registers.f.set(Flags::ZERO, result == 0);
                self.cpu.registers.f.set(Flags::SIGN, result & 0b1000_0000 != 0);
                self.cpu.registers.f.set(Flags::SUBTRACT, false);
                self.cpu.registers.f.set(Flags::HALF_CARRY, false);
                self.cpu.registers.f.set(Flags::PARITY_OR_OVERFLOW, self.check_parity(result));

                Ok(())
            }
            Opcode::RotateLeft(Operand::Register(Register::Reg16(dst_reg), true), _) => {
                let dst = self.cpu.get_register_u16(dst_reg);
                let value = self.bus.read(dst)?;
                let carry = value & 0b1000_0000 != 0;
                let result = (value << 1)
                    | (if self.cpu.registers.f.contains(Flags::CARRY) {
                        0b0000_0001
                    } else {
                        0
                    });
                self.bus.write(dst, result)?;

                self.cpu.registers.f.set(Flags::CARRY, carry);
                self.cpu.registers.f.set(Flags::ZERO, result == 0);
                self.cpu.registers.f.set(Flags::SIGN, result & 0b1000_0000 != 0);
                self.cpu.registers.f.set(Flags::SUBTRACT, false);
                self.cpu.registers.f.set(Flags::HALF_CARRY, false);
                self.cpu.registers.f.set(Flags::PARITY_OR_OVERFLOW, self.check_parity(result));

                Ok(())
            }
            _ => Err(GgError::InvalidOpcodeImplementation {
                instruction: instruction.opcode,
            }),
        }
    }

    pub(crate) fn rotate_right_carry_store(&mut self, instruction: &Instruction) -> Result<(), GgError> {
        match instruction.opcode {
            Opcode::RotateRightCarryStore(
                Operand::Register(Register::Reg16(src_reg), true),
                Operand::Register(Register::Reg8(dst_reg), false),
                _,
            ) => {
                let src = self.cpu.get_register_u16(src_reg);
                let value = self.bus.read(src)?;
                let carry = value & 0b0000_0001 != 0;
                let result = if carry { value >> 1 | 0b1000_0000 } else { value >> 1 };
                self.cpu.set_register_u8(dst_reg, result);
                self.bus.write(src, result)?;

                self.cpu.registers.f.set(Flags::CARRY, carry);
                self.cpu.registers.f.set(Flags::ZERO, result == 0);
                self.cpu.registers.f.set(Flags::SIGN, result & 0b1000_0000 != 0);
                self.cpu.registers.f.set(Flags::SUBTRACT, false);
                self.cpu.registers.f.set(Flags::HALF_CARRY, false);
                self.cpu.registers.f.set(Flags::PARITY_OR_OVERFLOW, self.check_parity(result));

                Ok(())
            }
            _ => Err(GgError::InvalidOpcodeImplementation {
                instruction: instruction.opcode,
            }),
        }
    }

    pub(crate) fn rotate_right_store(&mut self, instruction: &Instruction) -> Result<(), GgError> {
        match instruction.opcode {
            Opcode::RotateRightStore(
                Operand::Register(Register::Reg16(src_reg), true),
                Operand::Register(Register::Reg8(dst_reg), false),
                _,
            ) => {
                let previous_carry = self.cpu.registers.f.contains(Flags::CARRY);
                let src = self.cpu.get_register_u16(src_reg);
                let value = self.bus.read(src)?;
                let carry = value & 0b0000_0001 != 0;
                let result = (value >> 1) | (if previous_carry { 0b1000_0000 } else { 0 });
                self.cpu.set_register_u8(dst_reg, result);
                self.bus.write(src, result)?;

                self.cpu.registers.f.set(Flags::CARRY, carry);
                self.cpu.registers.f.set(Flags::ZERO, result == 0);
                self.cpu.registers.f.set(Flags::SIGN, result & 0b1000_0000 != 0);
                self.cpu.registers.f.set(Flags::SUBTRACT, false);
                self.cpu.registers.f.set(Flags::HALF_CARRY, false);
                self.cpu.registers.f.set(Flags::PARITY_OR_OVERFLOW, self.check_parity(result));

                Ok(())
            }
            _ => Err(GgError::InvalidOpcodeImplementation {
                instruction: instruction.opcode,
            }),
        }
    }

    pub(crate) fn rotate_left_carry_store(&mut self, instruction: &Instruction) -> Result<(), GgError> {
        match instruction.opcode {
            Opcode::RotateLeftCarryStore(
                Operand::Register(Register::Reg16(src_reg), true),
                Operand::Register(Register::Reg8(dst_reg), false),
                _,
            ) => {
                let src = self.cpu.get_register_u16(src_reg);
                let value = self.bus.read(src)?;
                let carry = value & 0b1000_0000 > 0;
                let result = if carry { value << 1 | 0b0000_0001 } else { value << 1 };
                self.cpu.set_register_u8(dst_reg, result);
                self.bus.write(src, result)?;

                self.cpu.registers.f.set(Flags::CARRY, carry);
                self.cpu.registers.f.set(Flags::ZERO, result == 0);
                self.cpu.registers.f.set(Flags::SIGN, result & 0b1000_0000 != 0);
                self.cpu.registers.f.set(Flags::SUBTRACT, false);
                self.cpu.registers.f.set(Flags::HALF_CARRY, false);
                self.cpu.registers.f.set(Flags::PARITY_OR_OVERFLOW, self.check_parity(result));

                Ok(())
            }
            _ => Err(GgError::InvalidOpcodeImplementation {
                instruction: instruction.opcode,
            }),
        }
    }

    pub(crate) fn rotate_left_store(&mut self, instruction: &Instruction) -> Result<(), GgError> {
        match instruction.opcode {
            Opcode::RotateLeftStore(
                Operand::Register(Register::Reg16(src_reg), true),
                Operand::Register(Register::Reg8(dst_reg), false),
                _,
            ) => {
                let previous_carry = self.cpu.registers.f.contains(Flags::CARRY);
                let src = self.cpu.get_register_u16(src_reg);
                let value = self.bus.read(src)?;
                let carry = value & 0b1000_0000 != 0;
                let result = (value << 1) | (if previous_carry { 0b0000_0001 } else { 0 });
                self.cpu.set_register_u8(dst_reg, result);
                self.bus.write(src, result)?;

                self.cpu.registers.f.set(Flags::CARRY, carry);
                self.cpu.registers.f.set(Flags::ZERO, result == 0);
                self.cpu.registers.f.set(Flags::SIGN, result & 0b1000_0000 != 0);
                self.cpu.registers.f.set(Flags::SUBTRACT, false);
                self.cpu.registers.f.set(Flags::HALF_CARRY, false);
                self.cpu.registers.f.set(Flags::PARITY_OR_OVERFLOW, self.check_parity(result));

                Ok(())
            }
            Opcode::RotateLeft(Operand::Register(Register::Reg16(dst_reg), true), _) => {
                let dst = self.cpu.get_register_u16(dst_reg);
                let value = self.bus.read(dst)?;
                let carry = value & 0b1000_0000 != 0;
                let result = (value << 1)
                    | (if self.cpu.registers.f.contains(Flags::CARRY) {
                        0b0000_0001
                    } else {
                        0
                    });
                self.bus.write(dst, result)?;

                self.cpu.registers.f.set(Flags::CARRY, carry);
                self.cpu.registers.f.set(Flags::ZERO, result == 0);
                self.cpu.registers.f.set(Flags::SIGN, result & 0b1000_0000 != 0);
                self.cpu.registers.f.set(Flags::SUBTRACT, false);
                self.cpu.registers.f.set(Flags::HALF_CARRY, false);
                self.cpu.registers.f.set(Flags::PARITY_OR_OVERFLOW, self.check_parity(result));

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

                self.cpu.registers.f.set(Flags::SUBTRACT, true);
                self.cpu.registers.f.set(Flags::HALF_CARRY, true);

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

    pub(crate) fn set_bit_store(&mut self, instruction: &Instruction) -> Result<(), GgError> {
        match instruction.opcode {
            Opcode::SetBitStore(
                Immediate::U8(bit),
                Operand::Register(Register::Reg16(src_reg), true),
                Operand::Register(Register::Reg8(dst_reg), false),
                _,
            ) => {
                let src = self.cpu.get_register_u16(src_reg);
                let value = self.bus.read(src)?;
                let dst = self.cpu.get_register_u8(dst_reg);
                let result = value | (1 << bit);
                self.cpu.set_register_u8(dst_reg, result);
                self.bus.write(src, result)?;
                Ok(())
            }
            _ => Err(GgError::InvalidOpcodeImplementation {
                instruction: instruction.opcode,
            }),
        }
    }

    pub(crate) fn exchange(&mut self, instruction: &Instruction) -> Result<(), GgError> {
        match instruction.opcode {
            Opcode::Exchange(Operand::Register(Register::Reg16(lhs_reg), false), Operand::Register(Register::Reg16(rhs_reg), false), _) => {
                let rhs = self.cpu.get_register_u16(rhs_reg);
                let lhs = self.cpu.get_register_u16(lhs_reg);
                self.cpu.set_register_u16(rhs_reg, lhs);
                self.cpu.set_register_u16(lhs_reg, rhs);
                Ok(())
            }
            Opcode::Exchange(Operand::Register(Register::Reg16(lhs_reg), true), Operand::Register(Register::Reg16(rhs_reg), false), _) => {
                let src = self.cpu.get_register_u16(lhs_reg);
                let data1 = self.bus.read_word(src)?;
                let data2 = self.cpu.get_register_u16(rhs_reg);
                self.bus.write_word(src, data2)?;
                self.cpu.set_register_u16(rhs_reg, data1);
                Ok(())
            }
            Opcode::Exchange(Operand::Register(Register::Reg16(lhs_reg), true), Operand::Register(Register::Reg16(rhs_reg), true), _) => {
                let src = self.cpu.get_register_u16(lhs_reg);
                let data1 = self.bus.read_word(src)?;
                let src = self.cpu.get_register_u16(rhs_reg);
                let data2 = self.bus.read_word(src)?;
                self.bus.write_word(src, data1)?;
                let src = self.cpu.get_register_u16(lhs_reg);
                self.bus.write_word(src, data2)?;
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

        let de_ = self.cpu.get_register_u16(Reg16::DEShadow);
        let hl_ = self.cpu.get_register_u16(Reg16::HLShadow);
        let bc_ = self.cpu.get_register_u16(Reg16::BCShadow);

        self.cpu.set_register_u16(Reg16::DE, de_);
        self.cpu.set_register_u16(Reg16::HL, hl_);
        self.cpu.set_register_u16(Reg16::BC, bc_);

        self.cpu.set_register_u16(Reg16::DEShadow, de);
        self.cpu.set_register_u16(Reg16::HLShadow, hl);
        self.cpu.set_register_u16(Reg16::BCShadow, bc);

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
                });
            }
        };

        let result = src & (1 << bit);
        self.cpu.registers.f.set(Flags::ZERO, result == 0);
        self.cpu.registers.f.set(Flags::HALF_CARRY, true);
        self.cpu.registers.f.set(Flags::SUBTRACT, false);
        self.cpu.registers.f.set(Flags::PARITY_OR_OVERFLOW, self.check_parity(result));
        self.cpu.registers.f.set(Flags::SIGN, result & 0b1000_0000 != 0);

        Ok(())
    }

    pub(crate) fn invert_carry(&mut self, instruction: &Instruction) -> Result<(), GgError> {
        match instruction.opcode {
            Opcode::InvertCarry(_) => {
                let carry = self.cpu.registers.f.contains(Flags::CARRY);
                self.cpu.registers.f.set(Flags::CARRY, !carry);
                self.cpu.registers.f.set(Flags::SUBTRACT, false);
                self.cpu.registers.f.set(Flags::HALF_CARRY, carry);
                Ok(())
            }
            _ => Err(GgError::InvalidOpcodeImplementation {
                instruction: instruction.opcode,
            }),
        }
    }

    pub(crate) fn add_carry(&mut self, instruction: &Instruction) -> Result<(), GgError> {
        let (src, dst, carry, dst_reg) = match instruction.opcode {
            Opcode::AddCarry(Operand::Register(Register::Reg8(dst_reg), false), Operand::Register(Register::Reg8(src_reg), false), _) => {
                let src = self.cpu.get_register_u8(src_reg);
                let dst = self.cpu.get_register_u8(dst_reg);
                let carry = if self.cpu.registers.f.contains(Flags::CARRY) { 1 } else { 0 };
                (src, dst, carry, dst_reg)
            }
            Opcode::AddCarry(Operand::Register(Register::Reg8(dst_reg), false), Operand::Immediate(Immediate::U8(imm), false), _) => {
                let dst = self.cpu.get_register_u8(dst_reg);
                let carry = if self.cpu.registers.f.contains(Flags::CARRY) { 1 } else { 0 };
                (imm, dst, carry, dst_reg)
            }
            Opcode::AddCarry(Operand::Register(Register::Reg8(dst_reg), false), Operand::Register(Register::Reg16(src_reg), true), _) => {
                let src = self.cpu.get_register_u16(src_reg);
                let src = self.bus.read(src)?;
                let dst = self.cpu.get_register_u8(dst_reg);
                let carry = if self.cpu.registers.f.contains(Flags::CARRY) { 1 } else { 0 };
                (src, dst, carry, dst_reg)
            }
            Opcode::AddCarry(Operand::Register(Register::Reg16(dst_reg), false), Operand::Register(Register::Reg16(src_reg), false), _) => {
                let src = self.cpu.get_register_u16(src_reg);
                let dst = self.cpu.get_register_u16(dst_reg);
                let carry = if self.cpu.registers.f.contains(Flags::CARRY) { 1 } else { 0 };
                let (result, carry) = self.add_and_detect_carry(dst, src, carry);
                self.cpu.set_register_u16(dst_reg, result);

                let hc = self.detect_half_carry_u16(dst, src, result);
                self.cpu.registers.f.set(Flags::ZERO, result == 0);
                self.cpu.registers.f.set(Flags::SIGN, result & 0b1000_0000_0000_0000 != 0);
                self.cpu
                    .registers
                    .f
                    .set(Flags::PARITY_OR_OVERFLOW, self.is_overflow_u16(dst, src, result));
                self.cpu.registers.f.set(Flags::SUBTRACT, false);
                self.cpu.registers.f.set(Flags::HALF_CARRY, hc);
                self.cpu.registers.f.set(Flags::CARRY, carry);

                return Ok(());
            }
            _ => {
                return Err(GgError::InvalidOpcodeImplementation {
                    instruction: instruction.opcode,
                })
            }
        };

        let (result, carry) = self.add_and_detect_carry(dst, src, carry);
        self.cpu.set_register_u8(dst_reg, result);

        let hc = self.detect_half_carry_u8(dst, src, result);
        self.cpu.registers.f.set(Flags::ZERO, result == 0);
        self.cpu.registers.f.set(Flags::SIGN, result & 0b1000_0000 != 0);
        self.cpu
            .registers
            .f
            .set(Flags::PARITY_OR_OVERFLOW, self.is_overflow(dst, src, result));
        self.cpu.registers.f.set(Flags::SUBTRACT, false);
        self.cpu.registers.f.set(Flags::HALF_CARRY, hc);
        self.cpu.registers.f.set(Flags::CARRY, carry);

        Ok(())
    }

    pub(crate) fn set_carry_flag(&mut self, instruction: &Instruction) -> Result<(), GgError> {
        self.cpu.registers.f.set(Flags::CARRY, true);
        self.cpu.registers.f.set(Flags::SUBTRACT, false);
        self.cpu.registers.f.set(Flags::HALF_CARRY, false);
        Ok(())
    }

    pub(crate) fn decimal_adjust_accumulator(&mut self, instruction: &Instruction) -> Result<(), GgError> {
        let mut a = self.cpu.get_register_u8(Reg8::A);
        if self.cpu.registers.f.contains(Flags::CARRY) || a > 0x99 {
            let value = a.wrapping_add_signed(if self.cpu.registers.f.contains(Flags::SUBTRACT) {
                -0x60
            } else {
                0x60
            });
            self.cpu.set_register_u8(Reg8::A, value);
            self.cpu.registers.f.set(Flags::CARRY, true);
        }

        a = self.cpu.get_register_u8(Reg8::A);
        if self.cpu.registers.f.contains(Flags::HALF_CARRY) || ((a & 0x0f) > 0x09) {
            let value = a.wrapping_add_signed(if self.cpu.registers.f.contains(Flags::SUBTRACT) { -6 } else { 6 });
            self.cpu.set_register_u8(Reg8::A, value);
        }

        let result = self.cpu.get_register_u8(Reg8::A);
        self.cpu.registers.f.set(Flags::PARITY_OR_OVERFLOW, self.check_parity(result));
        self.cpu.registers.f.set(Flags::ZERO, result == 0);
        self.cpu.registers.f.set(Flags::SIGN, result & 0b1000_0000 != 0);
        self.cpu.registers.f.set(Flags::HALF_CARRY, (result ^ a) & 0x10 > 0);

        Ok(())
    }

    pub(crate) fn shift_right_arithmetic(&mut self, instruction: &Instruction) -> Result<(), GgError> {
        let (result, carry) = match instruction.opcode {
            Opcode::ShiftRightArithmetic(Operand::Register(Register::Reg8(dst_reg), false), _) => {
                let value = self.cpu.get_register_u8(dst_reg);
                let carry = value & 0b0000_0001 > 0;
                let result = (value & 0b1000_0000) | (value >> 1);
                self.cpu.set_register_u8(dst_reg, result);
                (result, carry)
            }
            Opcode::ShiftRightArithmetic(Operand::Register(Register::Reg16(dst_reg), true), _) => {
                let dst = self.cpu.get_register_u16(dst_reg);
                let value = self.bus.read(dst)?;
                let carry = value & 0b0000_0001 > 0;
                let result = (value & 0b1000_0000) | (value >> 1);
                self.bus.write(dst, result)?;
                (result, carry)
            }
            _ => {
                return Err(GgError::InvalidOpcodeImplementation {
                    instruction: instruction.opcode,
                })
            }
        };

        self.cpu.registers.f.set(Flags::CARRY, carry);
        self.cpu.registers.f.set(Flags::ZERO, result == 0);
        self.cpu.registers.f.set(Flags::SIGN, result & 0b1000_0000 != 0);
        self.cpu.registers.f.set(Flags::SUBTRACT, false);
        self.cpu.registers.f.set(Flags::HALF_CARRY, false);
        self.cpu.registers.f.set(Flags::PARITY_OR_OVERFLOW, self.check_parity(result));

        Ok(())
    }

    pub(crate) fn shift_right_logical(&mut self, instruction: &Instruction) -> Result<(), GgError> {
        let (result, carry) = match instruction.opcode {
            Opcode::ShiftRightLogical(Operand::Register(Register::Reg8(dst_reg), false), _) => {
                let value = self.cpu.get_register_u8(dst_reg);
                let carry = value & 0b0000_0001 > 0;
                let result = value >> 1;
                self.cpu.set_register_u8(dst_reg, result);
                (result, carry)
            }
            Opcode::ShiftRightLogical(Operand::Register(Register::Reg16(dst_reg), true), _) => {
                let dst = self.cpu.get_register_u16(dst_reg);
                let value = self.bus.read(dst)?;
                let carry = value & 0b0000_0001 > 0;
                let result = value >> 1;
                self.bus.write(dst, result)?;
                (result, carry)
            }
            _ => {
                return Err(GgError::InvalidOpcodeImplementation {
                    instruction: instruction.opcode,
                })
            }
        };

        self.cpu.registers.f.set(Flags::CARRY, carry);
        self.cpu.registers.f.set(Flags::ZERO, result == 0);
        self.cpu.registers.f.set(Flags::SIGN, result & 0b1000_0000 != 0);
        self.cpu.registers.f.set(Flags::SUBTRACT, false);
        self.cpu.registers.f.set(Flags::HALF_CARRY, false);
        self.cpu.registers.f.set(Flags::PARITY_OR_OVERFLOW, self.check_parity(result));

        Ok(())
    }

    pub(crate) fn shift_right_arithmetic_store(&mut self, instruction: &Instruction) -> Result<(), GgError> {
        let (result, carry) = match instruction.opcode {
            Opcode::ShiftRightArithmeticStore(
                Operand::Register(Register::Reg16(src_reg), true),
                Operand::Register(Register::Reg8(dst_reg), false),
                _,
            ) => {
                let src = self.cpu.get_register_u16(src_reg);
                let value = self.bus.read(src)?;
                let carry = value & 0b0000_0001 > 0;
                let result = (value & 0b1000_0000) | (value >> 1);
                self.cpu.set_register_u8(dst_reg, result);
                self.bus.write(src, result)?;
                (result, carry)
            }
            _ => {
                return Err(GgError::InvalidOpcodeImplementation {
                    instruction: instruction.opcode,
                })
            }
        };

        self.cpu.registers.f.set(Flags::CARRY, carry);
        self.cpu.registers.f.set(Flags::ZERO, result == 0);
        self.cpu.registers.f.set(Flags::SIGN, result & 0b1000_0000 != 0);
        self.cpu.registers.f.set(Flags::SUBTRACT, false);
        self.cpu.registers.f.set(Flags::HALF_CARRY, false);
        self.cpu.registers.f.set(Flags::PARITY_OR_OVERFLOW, self.check_parity(result));

        Ok(())
    }

    pub(crate) fn shift_right_logical_store(&mut self, instruction: &Instruction) -> Result<(), GgError> {
        let (result, carry) = match instruction.opcode {
            Opcode::ShiftRightLogicalStore(
                Operand::Register(Register::Reg16(src_reg), true),
                Operand::Register(Register::Reg8(dst_reg), false),
                _,
            ) => {
                let src = self.cpu.get_register_u16(src_reg);
                let value = self.bus.read(src)?;
                let carry = value & 0b0000_0001 > 0;
                let result = value >> 1;
                self.cpu.set_register_u8(dst_reg, result);
                self.bus.write(src, result)?;
                (result, carry)
            }
            _ => {
                return Err(GgError::InvalidOpcodeImplementation {
                    instruction: instruction.opcode,
                })
            }
        };

        self.cpu.registers.f.set(Flags::CARRY, carry);
        self.cpu.registers.f.set(Flags::ZERO, result == 0);
        self.cpu.registers.f.set(Flags::SIGN, result & 0b1000_0000 != 0);
        self.cpu.registers.f.set(Flags::SUBTRACT, false);
        self.cpu.registers.f.set(Flags::HALF_CARRY, false);
        self.cpu.registers.f.set(Flags::PARITY_OR_OVERFLOW, self.check_parity(result));

        Ok(())
    }

    pub(crate) fn shift_left_arithmetic(&mut self, instruction: &Instruction) -> Result<(), GgError> {
        let (result, carry) = match instruction.opcode {
            Opcode::ShiftLeftArithmetic(Operand::Register(Register::Reg8(dst_reg), false), _) => {
                let value = self.cpu.get_register_u8(dst_reg);
                let carry = value & 0b1000_0000 > 0;
                let result = value << 1;
                self.cpu.set_register_u8(dst_reg, result);
                (result, carry)
            }
            Opcode::ShiftLeftArithmetic(Operand::Register(Register::Reg16(dst_reg), true), _) => {
                let dst = self.cpu.get_register_u16(dst_reg);
                let value = self.bus.read(dst)?;
                let carry = value & 0b1000_0000 > 0;
                let result = value << 1;
                self.bus.write(dst, result)?;
                (result, carry)
            }
            _ => {
                return Err(GgError::InvalidOpcodeImplementation {
                    instruction: instruction.opcode,
                })
            }
        };

        self.cpu.registers.f.set(Flags::CARRY, carry);
        self.cpu.registers.f.set(Flags::ZERO, result == 0);
        self.cpu.registers.f.set(Flags::SIGN, result & 0b1000_0000 != 0);
        self.cpu.registers.f.set(Flags::SUBTRACT, false);
        self.cpu.registers.f.set(Flags::HALF_CARRY, false);
        self.cpu.registers.f.set(Flags::PARITY_OR_OVERFLOW, self.check_parity(result));

        Ok(())
    }

    pub(crate) fn shift_left_logical(&mut self, instruction: &Instruction) -> Result<(), GgError> {
        let (result, carry) = match instruction.opcode {
            Opcode::ShiftLeftLogical(Operand::Register(Register::Reg8(dst_reg), false), _) => {
                let value = self.cpu.get_register_u8(dst_reg);
                let carry = value & 0b1000_0000 > 0;
                let result = value << 1 | 0b0000_0001;
                self.cpu.set_register_u8(dst_reg, result);
                (result, carry)
            }
            Opcode::ShiftLeftLogical(Operand::Register(Register::Reg16(dst_reg), true), _) => {
                let dst = self.cpu.get_register_u16(dst_reg);
                let value = self.bus.read(dst)?;
                let carry = value & 0b1000_0000 > 0;
                let result = value << 1 | 0b0000_0001;
                self.bus.write(dst, result)?;
                (result, carry)
            }
            _ => {
                return Err(GgError::InvalidOpcodeImplementation {
                    instruction: instruction.opcode,
                })
            }
        };

        self.cpu.registers.f.set(Flags::CARRY, carry);
        self.cpu.registers.f.set(Flags::ZERO, result == 0);
        self.cpu.registers.f.set(Flags::SIGN, result & 0b1000_0000 != 0);
        self.cpu.registers.f.set(Flags::SUBTRACT, false);
        self.cpu.registers.f.set(Flags::HALF_CARRY, false);
        self.cpu.registers.f.set(Flags::PARITY_OR_OVERFLOW, self.check_parity(result));

        Ok(())
    }

    pub(crate) fn shift_left_arithmetic_store(&mut self, instruction: &Instruction) -> Result<(), GgError> {
        let (result, carry) = match instruction.opcode {
            Opcode::ShiftLeftArithmeticStore(
                Operand::Register(Register::Reg16(src_reg), true),
                Operand::Register(Register::Reg8(dst_reg), false),
                _,
            ) => {
                let src = self.cpu.get_register_u16(src_reg);
                let value = self.bus.read(src)?;
                let carry = value & 0b1000_0000 > 0;
                let result = value << 1;
                self.cpu.set_register_u8(dst_reg, result);
                self.bus.write(src, result)?;
                (result, carry)
            }
            _ => {
                return Err(GgError::InvalidOpcodeImplementation {
                    instruction: instruction.opcode,
                })
            }
        };

        self.cpu.registers.f.set(Flags::CARRY, carry);
        self.cpu.registers.f.set(Flags::ZERO, result == 0);
        self.cpu.registers.f.set(Flags::SIGN, result & 0b1000_0000 != 0);
        self.cpu.registers.f.set(Flags::SUBTRACT, false);
        self.cpu.registers.f.set(Flags::HALF_CARRY, false);
        self.cpu.registers.f.set(Flags::PARITY_OR_OVERFLOW, self.check_parity(result));

        Ok(())
    }

    pub(crate) fn shift_left_logical_store(&mut self, instruction: &Instruction) -> Result<(), GgError> {
        let (result, carry) = match instruction.opcode {
            Opcode::ShiftLeftLogicalStore(
                Operand::Register(Register::Reg16(src_reg), true),
                Operand::Register(Register::Reg8(dst_reg), false),
                _,
            ) => {
                let src = self.cpu.get_register_u16(src_reg);
                let value = self.bus.read(src)?;
                let carry = value & 0b1000_0000 > 0;
                let result = value << 1 | 0b0000_0001;
                self.cpu.set_register_u8(dst_reg, result);
                self.bus.write(src, result)?;
                (result, carry)
            }
            _ => {
                return Err(GgError::InvalidOpcodeImplementation {
                    instruction: instruction.opcode,
                })
            }
        };

        self.cpu.registers.f.set(Flags::CARRY, carry);
        self.cpu.registers.f.set(Flags::ZERO, result == 0);
        self.cpu.registers.f.set(Flags::SIGN, result & 0b1000_0000 != 0);
        self.cpu.registers.f.set(Flags::SUBTRACT, false);
        self.cpu.registers.f.set(Flags::HALF_CARRY, false);
        self.cpu.registers.f.set(Flags::PARITY_OR_OVERFLOW, self.check_parity(result));

        Ok(())
    }

    // Helpers

    fn check_cpu_flag(&self, condition: Condition) -> bool {
        match condition {
            Condition::None => true,
            Condition::Carry => self.cpu.registers.f.contains(Flags::CARRY),
            Condition::NotCarry => !self.cpu.registers.f.contains(Flags::CARRY),
            Condition::Zero => self.cpu.registers.f.contains(Flags::ZERO),
            Condition::NotZero => !self.cpu.registers.f.contains(Flags::ZERO),
            Condition::Sign => self.cpu.registers.f.contains(Flags::SIGN),
            Condition::NotSign => !self.cpu.registers.f.contains(Flags::SIGN),
            Condition::ParityOrOverflow => self.cpu.registers.f.contains(Flags::PARITY_OR_OVERFLOW),
            Condition::NotParityOrOverflow => !self.cpu.registers.f.contains(Flags::PARITY_OR_OVERFLOW),
        }
    }

    fn detect_half_carry_u8(&self, lhs: u8, rhs: u8, result: u8) -> bool {
        (lhs ^ rhs ^ result) & 0x10 > 0
    }

    fn detect_half_carry_u16(&self, lhs: u16, rhs: u16, result: u16) -> bool {
        (lhs ^ rhs ^ result) & 0x1000 > 0
    }

    fn add_and_detect_carry<T: num_traits::ops::overflowing::OverflowingAdd>(&self, lhs: T, rhs: T, carry: T) -> (T, bool) {
        let (tmp, overflow1) = lhs.overflowing_add(&rhs);
        let (result, overflow2) = tmp.overflowing_add(&carry);
        (result, overflow1 || overflow2)
    }

    fn sub_and_detect_carry<T: num_traits::ops::overflowing::OverflowingSub>(&self, lhs: T, rhs: T, carry: T) -> (T, bool) {
        let (tmp, overflow1) = lhs.overflowing_sub(&rhs);
        let (result, overflow2) = tmp.overflowing_sub(&carry);
        (result, overflow1 || overflow2)
    }

    fn is_overflow(&self, lhs: u8, rhs: u8, result: u8) -> bool {
        ((lhs < 128 && rhs < 128) && result >= 128) || ((lhs > 127 && rhs > 127) && result <= 127)
    }

    fn is_overflow_u16(&self, lhs: u16, rhs: u16, result: u16) -> bool {
        ((lhs < 32768 && rhs < 32768) && result >= 32768) || ((lhs > 32767 && rhs > 32767) && result <= 32767)
    }

    fn is_underflow(&self, lhs: u8, rhs: u8, result: u8) -> bool {
        ((lhs < 128 && rhs > 127) && result >= 128) || ((lhs > 127 && rhs < 128) && result <= 127)
    }

    fn is_underflow_u16(&self, lhs: u16, rhs: u16, result: u16) -> bool {
        ((lhs < 32768 && rhs > 32767) && result >= 32768) || ((lhs > 32767 && rhs < 32768) && result <= 32767)
    }

    fn check_parity(&self, mut n: u8) -> bool {
        let mut count = 0;
        while n != 0 {
            count ^= n & 1;
            n >>= 1;
        }
        count == 0
    }
}
