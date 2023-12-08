use std::fmt;

#[derive(PartialEq, Copy, Clone)]
pub enum Register {
    A, B, C, D, E, H, L, F,
    AF, BC, DE, HL,
    IX, IY, SP, PC
}

impl Register {
    pub fn is_16bit(&self) -> bool {
        match self {
            Register::AF | Register::BC | Register::DE | Register::HL | Register::IX | Register::IY | Register::SP | Register::PC => true,
            _ => false
        }
    }
}

#[derive(PartialEq, Copy, Clone)]
pub enum Condition {
    NotZero,
    Zero,
    NotSign,
    Sign,
    NotCarry,
    Carry,
    None
}

#[derive(PartialEq, Copy, Clone)]
pub enum Immediate {
    U8(u8),
    U16(u16),
    S8(i8)
}

#[derive(PartialEq, Copy, Clone)]
pub enum Operand {
    Register(Register, bool),
    Immediate(Immediate, bool),
}

#[derive(PartialEq, Copy, Clone)]
pub enum Opcode {
    DisableInterrupts(usize),
    Load(Operand, Operand, usize),
    LoadIndirectRepeat(usize),
    Out(Operand, Operand, usize),
    Outi(usize),
    In(Operand, Operand, usize),
    SubtractNoUpdate(Operand, usize), // todo: ?
    JumpRelative(Condition, Immediate, usize),
    Jump(Condition, Immediate, usize),
    DecrementAndJumpRelative(Immediate, usize),
    Xor(Operand, usize),
    Or(Operand, usize),
    CallUnconditional(Operand, usize),
    OutIndirectRepeat(usize),
    NoOperation(usize),
    ReturnFromNmi(usize),
    Decrement(Operand, usize),
    Increment(Operand, usize),
    Restart(Immediate, usize),
    Return(Condition, usize),
    Push(Register, usize),
    Pop(Register, usize),
    ResetBit(Immediate, Operand, usize),
    SetBit(Immediate, Operand, usize),
    Unknown(usize)
}

pub struct Instruction {
    pub opcode: Opcode,
    pub length: usize,
    pub(crate) offset: usize,
    pub(crate) prefix: Option<u8>
}

impl fmt::Debug for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{:04x}] {:?} ", self.offset, self.opcode)
    }
}

impl fmt::Debug for Condition {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Condition::NotZero => write!(f, "nz"),
            Condition::Zero => write!(f, "z"),
            Condition::NotSign => write!(f, "p"),
            Condition::Sign => write!(f, "m"),
            Condition::NotCarry => write!(f, "nc"),
            Condition::Carry => write!(f, "c"),
            Condition::None => write!(f, "")
        }
    }
}

impl fmt::Debug for Opcode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Opcode::DisableInterrupts(_) => write!(f, "di"),
            Opcode::Load(op1, op2, _) => write!(f, "ld {:?}, {:?}", op1, op2),
            Opcode::LoadIndirectRepeat(_) => write!(f, "ldir"),
            Opcode::Out(op1, op2, _) => write!(f, "out {:?}, {:?}", op1, op2),
            Opcode::In(op1, op2, _) => write!(f, "in {:?}, {:?}", op1, op2),
            Opcode::SubtractNoUpdate(op1, _) => write!(f, "cp {:?}", op1),
            Opcode::JumpRelative(op1, op2, _) => {
                write!(f, "jr")?;
                if *op1 != Condition::None {
                    write!(f, " {:?},", op1)?;
                }
                write!(f, " {:?}", op2)
            },
            Opcode::Jump(op1, op2, _) => {
                write!(f, "jp")?;
                if *op1 != Condition::None {
                    write!(f, " {:?},", op1)?;
                }
                write!(f, " {:?}", op2)
            },
            Opcode::Xor(op, _) => write!(f, "xor {:?}", op),
            Opcode::Or(op, _) => write!(f, "or {:?}", op),
            Opcode::CallUnconditional(op, _) => write!(f, "call {:?}", op),
            Opcode::OutIndirectRepeat(_) => write!(f, "otir"),
            Opcode::NoOperation(_) => write!(f, "nop"),
            Opcode::ReturnFromNmi(_) => write!(f, "retn"),
            Opcode::Decrement(op, _) => write!(f, "dec {:?}", op),
            Opcode::Increment(op, _) => write!(f, "inc {:?}", op),
            Opcode::DecrementAndJumpRelative(op, _) => write!(f, "djnz {:?}", op),
            Opcode::Restart(op, _) => write!(f, "rst {:?}", op),
            Opcode::Return(op, _) => {
                write!(f, "ret")?;
                if *op != Condition::None {
                    write!(f, " {:?}", op)?;
                }
                Ok(())
            },
            Opcode::Push(op, _) => write!(f, "push {:?}", op),
            Opcode::Pop(op, _) => write!(f, "pop {:?}", op),
            Opcode::ResetBit(op1, op2, _) => write!(f, "res {:?}, {:?}", op1, op2),
            Opcode::SetBit(op1, op2, _) => write!(f, "set {:?}, {:?}", op1, op2),
            Opcode::Outi(_) => write!(f, "outi"),
            Opcode::Unknown(_) => unreachable!("Unknown opcode")
        }
    }
}

impl fmt::Debug for Immediate {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Immediate::U8(value) => write!(f, "#{:02x}", value),
            Immediate::U16(value) => write!(f, "#{:04x}", value),
            Immediate::S8(value) => {
                if *value >= 0 {
                    write!(f, "$+#{:02x}", value)
                } else {
                    write!(f, "$-#{:02x}", value)
                }
            }
        }
    }
}

impl fmt::Debug for Operand {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Operand::Register(register, indirect) => {
                if *indirect {
                    write!(f, "[{:?}]", register)
                } else {
                    write!(f, "{:?}", register)
                }
            },
            Operand::Immediate(immediate, indirect) => {
                if *indirect {
                    write!(f, "[{:?}]", immediate)
                } else {
                    write!(f, "{:?}", immediate)
                }
            }
        }
    }
}

impl fmt::Debug for Register {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Register::A => write!(f, "a"),
            Register::B => write!(f, "b"),
            Register::C => write!(f, "c"),
            Register::D => write!(f, "d"),
            Register::E => write!(f, "e"),
            Register::H => write!(f, "h"),
            Register::L => write!(f, "l"),
            Register::F => write!(f, "f"),
            Register::AF => write!(f, "af"),
            Register::BC => write!(f, "bc"),
            Register::DE => write!(f, "de"),
            Register::HL => write!(f, "hl"),
            Register::IX => write!(f, "ix"),
            Register::IY => write!(f, "iy"),
            Register::SP => write!(f, "sp"),
            Register::PC => write!(f, "pc")
        }
    }
}