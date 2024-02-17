use std::fmt;

// todo: Rename this to Reg and create new enum Reg16 and Reg8?
#[derive(PartialEq, Copy, Clone)]
pub enum Register {
    Reg8(Reg8),
    Reg16(Reg16),
}

#[derive(PartialEq, Copy, Clone)]
pub enum Reg8 {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
    F,
    AShadow,
    BShadow,
    CShadow,
    DShadow,
    EShadow,
    HShadow,
    LShadow,
    FShadow,
    IYH,
    IYL,
    IXH,
    IXL,
    I,
    R,
}

#[derive(PartialEq, Copy, Clone)]
pub enum Reg16 {
    AF,
    BC,
    DE,
    HL,
    AFShadow,
    BCShadow,
    DEShadow,
    HLShadow,
    SP,
    PC,
    IX(Option<i8>),
    IY(Option<i8>),
}

#[derive(PartialEq, Copy, Clone)]
pub enum Condition {
    NotZero,
    Zero,
    NotSign,
    Sign,
    NotCarry,
    Carry,
    NotParityOrOverflow,
    ParityOrOverflow,
    None,
}

#[derive(PartialEq, Copy, Clone)]
pub enum Immediate {
    U8(u8),
    U16(u16),
    S8(i8),
}

#[derive(PartialEq, Copy, Clone)]
pub enum Operand {
    Register(Register, bool),
    Immediate(Immediate, bool),
}

#[derive(PartialEq, Copy, Clone)]
pub enum Opcode {
    DisableInterrupts(usize),
    EnableInterrupts(usize),
    Load(Operand, Operand, usize),
    LoadIncrement(usize),
    LoadIncrementRepeat(usize),
    CompareIncrement(usize),
    CompareIncrementRepeat(usize),
    CompareDecrement(usize),
    CompareDecrementRepeat(usize),
    InIncrement(usize),
    InIncrementRepeat(usize),
    InDecrement(usize),
    InDecrementRepeat(usize),
    Out(Operand, Operand, usize),
    OutIncrement(usize),
    OutDecrement(usize),
    OutDecrementRepeat(usize),
    In(Operand, Operand, usize),
    Compare(Operand, usize), // todo: ?
    JumpRelative(Condition, Immediate, usize),
    Jump(Condition, Operand, usize),
    DecrementAndJumpRelative(Immediate, usize),
    Xor(Operand, usize),
    Or(Operand, usize),
    Call(Condition, Operand, usize),
    OutIncrementRepeat(usize),
    NoOperation(usize),
    ReturnFromNmi(usize),
    Decrement(Operand, usize),
    Increment(Operand, usize),
    Restart(Immediate, usize),
    Return(Condition, usize),
    Push(Register, usize),
    Pop(Register, usize),
    Subtract(Operand, usize),
    Add(Operand, Operand, usize),
    ResetBit(Immediate, Operand, usize),
    SetBit(Immediate, Operand, usize),
    SetInterruptMode(Immediate, usize),
    And(Operand, usize),
    SubtractCarry(Operand, Operand, usize),
    RotateRightCarryAccumulator(usize),
    RotateRightAccumulator(usize),
    RotateLeftCarryAccumulator(usize),
    RotateLeftAccumulator(usize),
    RotateRightCarry(Operand, usize),
    RotateRight(Operand, usize),
    Complement(usize),
    Halt(usize),
    Exchange(Operand, Operand, usize),
    ExchangeAll(usize),
    TestBit(Immediate, Operand, usize),
    LoadDecrementRepeat(usize),
    LoadDecrement(usize),
    InvertCarry(usize),
    AddCarry(Operand, Operand, usize),
    SetCarryFlag(usize),
    DecimalAdjustAccumulator(usize),
    RotateLeftCarryStore(Operand, Operand, usize),
    RotateLeftStore(Operand, Operand, usize),
    RotateRightCarryStore(Operand, Operand, usize),
    RotateRightStore(Operand, Operand, usize),
    ShiftRightArithmeticStore(Operand, Operand, usize),
    ShiftRightLogicalStore(Operand, Operand, usize),
    ShiftLeftArithmeticStore(Operand, Operand, usize),
    ShiftLeftLogicalStore(Operand, Operand, usize),
    ResetBitStore(Immediate, Operand, Operand, usize),
    SetBitStore(Immediate, Operand, Operand, usize),
    RotateLeftCarry(Operand, usize),
    RotateLeft(Operand, usize),
    ShiftLeftArithmetic(Operand, usize),
    ShiftRightArithmetic(Operand, usize),
    ShiftLeftLogical(Operand, usize),
    ShiftRightLogical(Operand, usize),
    Negate(usize),
    Unknown(usize),
}

pub struct Instruction {
    pub opcode: Opcode,
    pub length: usize,
    pub offset: usize,
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{:04x}] {} ", self.offset, self.opcode)
    }
}

impl fmt::Display for Condition {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Condition::NotZero => write!(f, "nz"),
            Condition::Zero => write!(f, "z"),
            Condition::NotSign => write!(f, "p"),
            Condition::Sign => write!(f, "m"),
            Condition::NotCarry => write!(f, "nc"),
            Condition::Carry => write!(f, "c"),
            Condition::NotParityOrOverflow => write!(f, "po"),
            Condition::ParityOrOverflow => write!(f, "pe"),
            Condition::None => write!(f, ""),
        }
    }
}

impl fmt::Display for Opcode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Opcode::DisableInterrupts(_) => write!(f, "di"),
            Opcode::EnableInterrupts(_) => write!(f, "ei"),
            Opcode::Load(op1, op2, _) => write!(f, "ld {}, {}", op1, op2),
            Opcode::LoadIncrementRepeat(_) => write!(f, "ldir"),
            Opcode::Out(op1, op2, _) => write!(f, "out {}, {}", op1, op2),
            Opcode::In(op1, op2, _) => write!(f, "in {}, {}", op1, op2),
            Opcode::Compare(op1, _) => write!(f, "cp {}", op1),
            Opcode::SubtractCarry(op1, op2, _) => write!(f, "sbc {}, {}", op1, op2),
            Opcode::LoadDecrementRepeat(_) => write!(f, "lddr"),
            Opcode::LoadDecrement(_) => write!(f, "ldd"),
            Opcode::AddCarry(op1, op2, _) => write!(f, "adc {}, {}", op1, op2),
            Opcode::SetCarryFlag(_) => write!(f, "scf"),
            Opcode::DecimalAdjustAccumulator(_) => write!(f, "daa"),
            Opcode::JumpRelative(op1, op2, _) => {
                write!(f, "jr")?;
                if *op1 != Condition::None {
                    write!(f, " {},", op1)?;
                }
                write!(f, " {}", op2)
            }
            Opcode::Jump(op1, op2, _) => {
                write!(f, "jp")?;
                if *op1 != Condition::None {
                    write!(f, " {},", op1)?;
                }
                write!(f, " {}", op2)
            }
            Opcode::Xor(op, _) => write!(f, "xor {}", op),
            Opcode::InvertCarry(_) => write!(f, "ccf"),
            Opcode::Or(op, _) => write!(f, "or {}", op),
            Opcode::Call(cond, op, _) => {
                write!(f, "call")?;
                if *cond != Condition::None {
                    write!(f, " {},", cond)?;
                }
                write!(f, " {}", op)
            }
            Opcode::OutIncrementRepeat(_) => write!(f, "otir"),
            Opcode::NoOperation(_) => write!(f, "nop"),
            Opcode::ReturnFromNmi(_) => write!(f, "retn"),
            Opcode::Decrement(op, _) => write!(f, "dec {}", op),
            Opcode::Increment(op, _) => write!(f, "inc {}", op),
            Opcode::DecrementAndJumpRelative(op, _) => write!(f, "djnz {}", op),
            Opcode::Restart(op, _) => write!(f, "rst {}", op),
            Opcode::Subtract(op, _) => write!(f, "sub {}", op),
            Opcode::Add(op1, op2, _) => write!(f, "add {}, {}", op1, op2),
            Opcode::Return(op, _) => {
                write!(f, "ret")?;
                if *op != Condition::None {
                    write!(f, " {}", op)?;
                }
                Ok(())
            }
            Opcode::Push(op, _) => write!(f, "push {}", op),
            Opcode::Pop(op, _) => write!(f, "pop {}", op),
            Opcode::ResetBit(op1, op2, _) => write!(f, "res {}, {}", op1, op2),
            Opcode::SetBit(op1, op2, _) => write!(f, "set {}, {}", op1, op2),
            Opcode::OutIncrement(_) => write!(f, "outi"),
            Opcode::SetInterruptMode(op, _) => write!(f, "im {}", op),
            Opcode::And(op, _) => write!(f, "and {}", op),
            Opcode::RotateRightCarryAccumulator(_) => write!(f, "rrca"),
            Opcode::RotateRightAccumulator(_) => write!(f, "rra"),
            Opcode::RotateLeftCarryAccumulator(_) => write!(f, "rlca"),
            Opcode::RotateLeftAccumulator(_) => write!(f, "rla"),
            Opcode::RotateRightCarry(op, _) => write!(f, "rrc {}", op),
            Opcode::RotateRight(op, _) => write!(f, "rr {}", op),
            Opcode::RotateLeftCarry(op, _) => write!(f, "rlc {}", op),
            Opcode::RotateLeft(op, _) => write!(f, "rl {}", op),
            Opcode::Complement(_) => write!(f, "cpl"),
            Opcode::Halt(_) => write!(f, "halt"),
            Opcode::Exchange(op1, op2, _) => write!(f, "ex {}, {}", op1, op2),
            Opcode::ExchangeAll(_) => write!(f, "exx"),
            Opcode::TestBit(op1, op2, _) => write!(f, "bit {}, {}", op1, op2),
            Opcode::OutDecrement(_) => write!(f, "outd"),
            Opcode::ShiftRightArithmetic(op, _) => write!(f, "sra {}", op),
            Opcode::ShiftRightLogical(op, _) => write!(f, "srl {}", op),
            Opcode::ShiftLeftArithmetic(op, _) => write!(f, "sla {}", op),
            Opcode::ShiftLeftLogical(op, _) => write!(f, "sll {}", op),
            Opcode::ResetBitStore(op1, op2, op3, _) => write!(f, "res {}, {}, {}", op1, op2, op3),
            Opcode::SetBitStore(op1, op2, op3, _) => write!(f, "set {}, {}, {}", op1, op2, op3),
            Opcode::RotateLeftCarryStore(op1, op2, _) => write!(f, "rlc {}, {}", op1, op2),
            Opcode::RotateLeftStore(op1, op2, _) => write!(f, "rl {}, {}", op1, op2),
            Opcode::RotateRightCarryStore(op1, op2, _) => write!(f, "rrc {}, {}", op1, op2),
            Opcode::RotateRightStore(op1, op2, _) => write!(f, "rr {}, {}", op1, op2),
            Opcode::ShiftLeftArithmeticStore(op1, op2, _) => write!(f, "sla {}, {}", op1, op2),
            Opcode::ShiftRightArithmeticStore(op1, op2, _) => write!(f, "sra {}, {}", op1, op2),
            Opcode::ShiftLeftLogicalStore(op1, op2, _) => write!(f, "sll {}, {}", op1, op2),
            Opcode::ShiftRightLogicalStore(op1, op2, _) => write!(f, "srl {}, {}", op1, op2),
            Opcode::Negate(_) => write!(f, "neg"),
            Opcode::Unknown(_) => unreachable!("Unknown opcode"),
            Opcode::CompareIncrementRepeat(_) => write!(f, "cpir"),
            Opcode::CompareDecrement(_) => write!(f, "cpd"),
            Opcode::CompareDecrementRepeat(_) => write!(f, "cpdr"),
            Opcode::LoadIncrement(_) => write!(f, "ldi"),
            Opcode::CompareIncrement(_) => write!(f, "cpi"),
            Opcode::InIncrement(_) => write!(f, "ini"),
            Opcode::InDecrementRepeat(_) => write!(f, "indr"),
            Opcode::OutDecrementRepeat(_) => write!(f, "otdr"),
            Opcode::InIncrementRepeat(_) => write!(f, "inir"),
            Opcode::InDecrement(_) => write!(f, "ind"),
        }
    }
}

impl fmt::Debug for Opcode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

impl fmt::Display for Immediate {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Immediate::U8(value) => write!(f, "#{:02x}", value),
            Immediate::U16(value) => write!(f, "#{:04x}", value),
            Immediate::S8(value) => write!(f, "{}", value),
        }
    }
}

impl fmt::Display for Operand {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Operand::Register(register, indirect) => {
                if *indirect {
                    write!(f, "[{}]", register)
                } else {
                    write!(f, "{}", register)
                }
            }
            Operand::Immediate(immediate, indirect) => {
                if *indirect {
                    write!(f, "[{}]", immediate)
                } else {
                    write!(f, "{}", immediate)
                }
            }
        }
    }
}

impl fmt::Display for Reg8 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Reg8::A => write!(f, "a"),
            Reg8::B => write!(f, "b"),
            Reg8::C => write!(f, "c"),
            Reg8::D => write!(f, "d"),
            Reg8::E => write!(f, "e"),
            Reg8::H => write!(f, "h"),
            Reg8::L => write!(f, "l"),
            Reg8::F => write!(f, "f"),
            Reg8::AShadow => write!(f, "a'"),
            Reg8::BShadow => write!(f, "b'"),
            Reg8::CShadow => write!(f, "c'"),
            Reg8::DShadow => write!(f, "d'"),
            Reg8::EShadow => write!(f, "e'"),
            Reg8::HShadow => write!(f, "h'"),
            Reg8::LShadow => write!(f, "l'"),
            Reg8::FShadow => write!(f, "f'"),
            Reg8::IYH => write!(f, "iyh"),
            Reg8::IYL => write!(f, "iyl"),
            Reg8::IXH => write!(f, "ixh"),
            Reg8::IXL => write!(f, "ixl"),
            Reg8::I => write!(f, "i"),
            Reg8::R => write!(f, "r"),
        }
    }
}

impl fmt::Display for Reg16 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Reg16::AF => write!(f, "af"),
            Reg16::BC => write!(f, "bc"),
            Reg16::DE => write!(f, "de"),
            Reg16::HL => write!(f, "hl"),
            Reg16::AFShadow => write!(f, "af'"),
            Reg16::BCShadow => write!(f, "bc'"),
            Reg16::DEShadow => write!(f, "de'"),
            Reg16::HLShadow => write!(f, "hl'"),
            Reg16::SP => write!(f, "sp"),
            Reg16::PC => write!(f, "pc"),
            Reg16::IX(offset) => {
                if let Some(offset) = offset {
                    write!(f, "ix+#{:01x}", offset)
                } else {
                    write!(f, "ix")
                }
            }
            Reg16::IY(offset) => {
                if let Some(offset) = offset {
                    write!(f, "iy+#{:01x}", offset)
                } else {
                    write!(f, "iy")
                }
            }
        }
    }
}

impl fmt::Display for Register {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Register::Reg8(reg) => write!(f, "{}", reg),
            Register::Reg16(reg) => write!(f, "{}", reg),
        }
    }
}
