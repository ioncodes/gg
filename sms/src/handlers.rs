use crate::{bus::Bus, cpu::Cpu};
use z80::instruction::{Condition, Immediate, Instruction, Opcode, Operand, Register};

pub(crate) struct Handlers;

impl Handlers {
    pub(crate) fn load(cpu: &mut Cpu, bus: &mut Bus, instruction: &Instruction) {
        match instruction.opcode {
            Opcode::Load(
                Operand::Register(dst_register, dst_deref),
                Operand::Immediate(Immediate::U16(src_imm), src_deref),
                _,
            ) => {
                let mut imm = src_imm;
                if src_deref {
                    imm = bus
                        .read_word(src_imm)
                        .expect(&format!("CPU crashed with: {:?}", cpu));
                }

                if dst_deref {
                    let reg = cpu.get_register_u16(dst_register);
                    let dst = bus
                        .read_word(reg)
                        .expect(&format!("CPU crashed with: {:?}", cpu));
                    bus.write_word(dst, imm)
                        .expect(&format!("CPU crashed with: {:?}", cpu));
                } else {
                    cpu.set_reg(dst_register, imm);
                }
            }
            Opcode::Load(
                Operand::Register(dst_register, dst_deref),
                Operand::Register(src_register, false),
                _,
            ) => {
                if dst_deref {
                    match src_register.is_16bit() {
                        true => {
                            let reg = cpu.get_register_u16(dst_register);
                            let dst = bus
                                .read_word(reg)
                                .expect(&format!("CPU crashed with: {:?}", cpu));
                            bus.write_word(dst, cpu.get_register_u16(src_register))
                                .expect(&format!("CPU crashed with: {:?}", cpu));
                        }
                        false => {
                            let reg = cpu.get_register_u16(dst_register);
                            let dst = bus
                                .read_word(reg)
                                .expect(&format!("CPU crashed with: {:?}", cpu));
                            bus.write(dst, cpu.get_register(src_register))
                                .expect(&format!("CPU crashed with: {:?}", cpu));
                        }
                    };
                } else {
                    match src_register.is_16bit() {
                        true => cpu.set_reg(dst_register, cpu.get_register_u16(src_register)),
                        false => cpu.set_register(dst_register, cpu.get_register(src_register)),
                    };
                }
            }
            Opcode::Load(
                Operand::Immediate(Immediate::U16(dst_imm), true),
                Operand::Register(src_register, false),
                _,
            ) => {
                bus.write_word(dst_imm, cpu.get_register_u16(src_register))
                    .expect(&format!("CPU crashed with: {:?}", cpu));
            }
            _ => panic!(
                "Invalid opcode for load instruction: {:?}",
                instruction.opcode
            ),
        }
    }

    pub(crate) fn jump(cpu: &mut Cpu, _bus: &mut Bus, instruction: &Instruction) {
        match instruction.opcode {
            Opcode::Jump(Condition::None, Immediate::U16(imm), _) => {
                cpu.set_register_u16(Register::PC, imm);
            }
            _ => panic!(
                "Invalid opcode for jump instruction: {:?}",
                instruction.opcode
            ),
        }
    }

    pub(crate) fn disable_interrupts(_cpu: &mut Cpu, _bus: &mut Bus, instruction: &Instruction) {
        match instruction.opcode {
            Opcode::DisableInterrupts(_) => {
                println!("Disabling interrupts");
            }
            _ => panic!(
                "Invalid opcode for disable interrupts instruction: {:?}",
                instruction.opcode
            ),
        }
    }

    pub(crate) fn load_indirect_repeat(cpu: &mut Cpu, bus: &mut Bus, _instruction: &Instruction) {
        loop {
            let src = {
                let hl = cpu.get_register_u16(Register::HL);
                bus.read(hl).expect(&format!("CPU crashed with: {:?}", cpu))
            };
            let dst = cpu.get_register_u16(Register::DE);

            bus.write(dst, src)
                .expect(&format!("CPU crashed with: {:?}", cpu));

            let hl = cpu.get_register_u16(Register::HL);
            let de = cpu.get_register_u16(Register::DE);
            cpu.set_register_u16(Register::HL, hl + 1);
            cpu.set_register_u16(Register::DE, de + 1);

            let bc = cpu.get_register_u16(Register::BC);
            cpu.set_register_u16(Register::BC, bc - 1);

            if cpu.get_register_u16(Register::BC) == 0 {
                break;
            }
        }
    }
}
