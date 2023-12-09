use crate::{bus::Bus, cpu::Cpu};
use z80::instruction::{Condition, Immediate, Instruction, Opcode, Operand, Register, Reg16};

pub(crate) struct Handlers;

// TODO: Implement U16 for Register type in Disassembler.
// This should in theory help with a lot of cases for pattern matching.
// Naturally it will also flatten the code a bit as we don't have to check for
// 16 bit wideness in every opcode case.

impl Handlers {
    pub(crate) fn load(
        cpu: &mut Cpu,
        bus: &mut Bus,
        instruction: &Instruction,
    ) -> Result<(), String> {
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
            },
            Opcode::Load(
                Operand::Register(Register::Reg16(dst_register), true),
                Operand::Register(Register::Reg8(src_register), false),
                _, 
            ) => {
                let dst = cpu.get_register_u16(dst_register);
                let src = cpu.get_register(src_register);
                bus.write(dst, src)?;
                Ok(())
            },
            Opcode::Load(
                Operand::Register(Register::Reg8(dst_register), false),
                Operand::Immediate(Immediate::U8(src_imm), false),
                _,
            ) => {
                cpu.set_register(dst_register, src_imm);
                Ok(())
            },
            Opcode::Load(
                Operand::Immediate(Immediate::U16(dst_imm), true),
                Operand::Register(Register::Reg16(src_register), false),
                _,
            ) => {
                bus.write_word(dst_imm, cpu.get_register_u16(src_register))?;
                Ok(())
            }
            _ => panic!(
                "Invalid opcode for load instruction: {:?}",
                instruction.opcode
            ),
        }
    }

    pub(crate) fn jump(cpu: &mut Cpu, _bus: &mut Bus, instruction: &Instruction) -> Result<(), String> {
        match instruction.opcode {
            Opcode::Jump(Condition::None, Immediate::U16(imm), _) => {
                cpu.set_register_u16(Reg16::PC, imm);
                Ok(())
            }
            _ => panic!(
                "Invalid opcode for jump instruction: {:?}",
                instruction.opcode
            ),
        }
    }

    pub(crate) fn disable_interrupts(_cpu: &mut Cpu, _bus: &mut Bus, _instruction: &Instruction) -> Result<(), String> {
        println!("Disabling interrupts");
        Ok(())
    }

    pub(crate) fn load_indirect_repeat(cpu: &mut Cpu, bus: &mut Bus, _instruction: &Instruction) -> Result<(), String> {
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

    pub(crate) fn out(cpu: &mut Cpu, bus: &mut Bus, instruction: &Instruction) -> Result<(), String> {
        match instruction.opcode {
            Opcode::Out(
                Operand::Immediate(Immediate::U8(dst_port), true),
                Operand::Register(Register::Reg8(src_reg), false),
                _
            ) => {
                let imm = cpu.get_register(src_reg);
                bus.push_io_request(dst_port, imm);
                Ok(())
            }
            _ => panic!("Invalid opcode for out instruction: {:?}", instruction.opcode),
        }
    }

    pub(crate) fn in_(cpu: &mut Cpu, bus: &mut Bus, instruction: &Instruction) -> Result<(), String> {
        match instruction.opcode {
            Opcode::In(
                Operand::Register(Register::Reg8(dst_reg), false),
                Operand::Immediate(Immediate::U8(src_port), true),
                _
            ) => {
                let imm = bus.pop_io_request(src_port).unwrap();
                cpu.set_register(dst_reg, imm);
                Ok(())
            }
            _ => panic!("Invalid opcode for out instruction: {:?}", instruction.opcode),
        }
    }
}
