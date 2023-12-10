use crate::instruction::{Opcode, Operand, Register, Instruction, Immediate, Condition, Reg16, Reg8};

pub struct Disassembler<'a> {
    pub data: &'a [u8]
}

impl<'a> Disassembler<'a> {
    pub fn new(data: &'a [u8]) -> Disassembler {
        Disassembler { data }
    }

    pub fn decode(&self, offset: usize) -> Result<Instruction, String> {
        let opcode = self.data[offset];
        let (prefix, opcode) = if opcode == 0xed || opcode == 0xcb {
            (Some(opcode), self.data[offset + 1])
        } else {
            (None, opcode)
        };

        let opcode = self.decode_opcode(offset, prefix, opcode);

        if opcode != Opcode::Unknown(0) {
            let length = self.calc_length(opcode);
            Ok(Instruction { opcode, length, prefix, offset })
        } else {
            Err(format!("Unknown instruction {:x}", self.data[offset]))
        }
    }

    pub fn disassemble(&self) -> Vec<Instruction> {
        let mut instructions = Vec::new();
        let mut offset = 0;

        while offset < self.data.len() {
            match self.decode(offset) {
                Ok(instruction) => {
                    offset += instruction.length;
                    instructions.push(instruction);
                }
                Err(msg) => panic!("{}", msg)
            }
        }

        instructions
    }

    fn decode_opcode(&self, offset: usize, prefix: Option<u8>, opcode: u8) -> Opcode {
        match (prefix, opcode) {
            // NO PREFIX
            (None, 0x00) => Opcode::NoOperation(1),
            (None, 0x01) => Opcode::Load(
                Operand::Register(Register::Reg16(Reg16::BC), false), 
                Operand::Immediate(Immediate::U16(self.read_u16(offset + 1)), false),
                3),
            (None, 0x02) => Opcode::Load(
                Operand::Register(Register::Reg16(Reg16::BC), true),
                Operand::Register(Register::Reg8(Reg8::A), false),
                1),
            (None, 0x06) => Opcode::Load(
                Operand::Register(Register::Reg8(Reg8::B), false),
                Operand::Immediate(Immediate::U8(self.data[offset + 1]), false),
                2),
            (None, 0x0e) => Opcode::Load(
                Operand::Register(Register::Reg8(Reg8::C), false),
                Operand::Immediate(Immediate::U8(self.data[offset + 1]), false),
                2),
            (None, 0x10) => Opcode::DecrementAndJumpRelative(
                Immediate::S8(self.data[offset + 1] as i8),
                2),
            (None, 0x11) => Opcode::Load(
                Operand::Register(Register::Reg16(Reg16::DE), false),
                Operand::Immediate(Immediate::U16(self.read_u16(offset + 1)), false),
                3),
            (None, 0x13) => Opcode::Increment(
                Operand::Register(Register::Reg16(Reg16::DE), false),
                1),
            (None, 0x14) => Opcode::Increment(
                Operand::Register(Register::Reg8(Reg8::D), false),
                1),
            (None, 0x15) => Opcode::Decrement(
                Operand::Register(Register::Reg8(Reg8::D), false),
                1),
            (None, 0x18) => Opcode::JumpRelative(
                Condition::None,
                Immediate::S8(self.data[offset + 1] as i8),
                2),
            (None, 0x1a) => Opcode::Load(
                Operand::Register(Register::Reg8(Reg8::A), false),
                Operand::Register(Register::Reg16(Reg16::DE), true),
                1),
            (None, 0x20) => Opcode::JumpRelative(
                Condition::NotZero,
                Immediate::S8(self.data[offset + 1] as i8),
                2),
            (None, 0x21) => Opcode::Load(
                Operand::Register(Register::Reg16(Reg16::HL), false),
                Operand::Immediate(Immediate::U16(self.read_u16(offset + 1)), false),
                3),
            (None, 0x22) => Opcode::Load(
                Operand::Immediate(Immediate::U16(self.read_u16(offset + 1)), true),
                Operand::Register(Register::Reg16(Reg16::HL), false),
                3),
            (None, 0x23) => Opcode::Increment(
                Operand::Register(Register::Reg16(Reg16::HL), false),
                1),
            (None, 0x24) => Opcode::Increment(
                Operand::Register(Register::Reg8(Reg8::H), false),
                1),
            (None, 0x25) => Opcode::Decrement(
                Operand::Register(Register::Reg8(Reg8::H), false),
                1),
            (None, 0x28) => Opcode::JumpRelative(
                Condition::Zero,
                Immediate::S8(self.data[offset + 1] as i8),
                2),
            (None, 0x2a) => Opcode::Load(
                Operand::Register(Register::Reg16(Reg16::HL), false),
                Operand::Immediate(Immediate::U16(self.read_u16(offset + 1)), true),
                3),
            (None, 0x2b) => Opcode::Decrement(
                Operand::Register(Register::Reg16(Reg16::HL), false),
                1),
            (None, 0x31) => Opcode::Load(
                Operand::Register(Register::Reg16(Reg16::SP), false),
                Operand::Immediate(Immediate::U16(self.read_u16(offset + 1)), false),
                3),
            (None, 0x32) => Opcode::Load(
                Operand::Immediate(Immediate::U16(self.read_u16(offset + 1)), true),
                Operand::Register(Register::Reg8(Reg8::A), false),
                3),
            (None, 0x3a) => Opcode::Load(
                Operand::Register(Register::Reg8(Reg8::A), false),
                Operand::Immediate(Immediate::U16(self.read_u16(offset + 1)), true),
                3),
            (None, 0x3e) => Opcode::Load(
                Operand::Register(Register::Reg8(Reg8::A), false),
                Operand::Immediate(Immediate::U8(self.data[offset + 1]), false),
                2),
            (None, 0x40) => Opcode::Load(
                Operand::Register(Register::Reg8(Reg8::B), false),
                Operand::Register(Register::Reg8(Reg8::B), false),
                1),
            (None, 0x41) => Opcode::Load(
                Operand::Register(Register::Reg8(Reg8::B), false),
                Operand::Register(Register::Reg8(Reg8::C), false),
                1),
            (None, 0x42) => Opcode::Load(
                Operand::Register(Register::Reg8(Reg8::B), false),
                Operand::Register(Register::Reg8(Reg8::D), false),
                1),
            (None, 0x43) => Opcode::Load(
                Operand::Register(Register::Reg8(Reg8::B), false),
                Operand::Register(Register::Reg8(Reg8::E), false),
                1),
            (None, 0x44) => Opcode::Load(
                Operand::Register(Register::Reg8(Reg8::B), false),
                Operand::Register(Register::Reg8(Reg8::H), false),
                1),
            (None, 0x45) => Opcode::Load(
                Operand::Register(Register::Reg8(Reg8::B), false),
                Operand::Register(Register::Reg8(Reg8::L), false),
                1),
            (None, 0x46) => Opcode::Load(
                Operand::Register(Register::Reg8(Reg8::B), false),
                Operand::Register(Register::Reg16(Reg16::HL), true),
                1),
            (None, 0x47) => Opcode::Load(
                Operand::Register(Register::Reg8(Reg8::B), false),
                Operand::Register(Register::Reg8(Reg8::A), false),
                1),
            (None, 0x48) => Opcode::Load(
                Operand::Register(Register::Reg8(Reg8::C), false),
                Operand::Register(Register::Reg8(Reg8::B), false),
                1),
            (None, 0x49) => Opcode::Load(
                Operand::Register(Register::Reg8(Reg8::C), false),
                Operand::Register(Register::Reg8(Reg8::C), false),
                1),
            (None, 0x4a) => Opcode::Load(
                Operand::Register(Register::Reg8(Reg8::C), false),
                Operand::Register(Register::Reg8(Reg8::D), false),
                1),
            (None, 0x4b) => Opcode::Load(
                Operand::Register(Register::Reg8(Reg8::C), false),
                Operand::Register(Register::Reg8(Reg8::E), false),
                1),
            (None, 0x4c) => Opcode::Load(
                Operand::Register(Register::Reg8(Reg8::C), false),
                Operand::Register(Register::Reg8(Reg8::H), false),
                1),
            (None, 0x4d) => Opcode::Load(
                Operand::Register(Register::Reg8(Reg8::C), false),
                Operand::Register(Register::Reg8(Reg8::L), false),
                1),
            (None, 0x4e) => Opcode::Load(
                Operand::Register(Register::Reg8(Reg8::C), false),
                Operand::Register(Register::Reg16(Reg16::HL), true),
                1),
            (None, 0x4f) => Opcode::Load(
                Operand::Register(Register::Reg8(Reg8::C), false),
                Operand::Register(Register::Reg8(Reg8::A), false),
                1),
            (None, 0x75) => Opcode::Load(
                Operand::Register(Register::Reg16(Reg16::HL), true),
                Operand::Register(Register::Reg8(Reg8::L), false),
                1),
            (None, 0x7a) => Opcode::Load(
                Operand::Register(Register::Reg8(Reg8::A), false),
                Operand::Register(Register::Reg8(Reg8::D), false),
                1),
            (None, 0x7b) => Opcode::Load(
                Operand::Register(Register::Reg8(Reg8::A), false),
                Operand::Register(Register::Reg8(Reg8::E), false),
                1),
            (None, 0x7c) => Opcode::Load(
                Operand::Register(Register::Reg8(Reg8::A), false),
                Operand::Register(Register::Reg8(Reg8::H), false),
                1),
            (None, 0x7d) => Opcode::Load(
                Operand::Register(Register::Reg8(Reg8::A), false),
                Operand::Register(Register::Reg8(Reg8::L), false),
                1),
            (None, 0x7e) => Opcode::Load(
                Operand::Register(Register::Reg8(Reg8::A), false),
                Operand::Register(Register::Reg16(Reg16::HL), true),
                1),
            (None, 0x7f) => Opcode::Load(
                Operand::Register(Register::Reg8(Reg8::A), false),
                Operand::Register(Register::Reg8(Reg8::A), false),
                1),
            (None, 0xaf) => Opcode::Xor(
                Operand::Register(Register::Reg8(Reg8::A), false),
                1),
            (None, 0xb0) => Opcode::Or(
                Operand::Register(Register::Reg8(Reg8::B), false),
                1),
            (None, 0xb1) => Opcode::Or(
                Operand::Register(Register::Reg8(Reg8::C), false),
                1),
            (None, 0xb2) => Opcode::Or(
                Operand::Register(Register::Reg8(Reg8::D), false),
                1),
            (None, 0xb3) => Opcode::Or(
                Operand::Register(Register::Reg8(Reg8::E), false),
                1),
            (None, 0xb4) => Opcode::Or(
                Operand::Register(Register::Reg8(Reg8::H), false),
                1),
            (None, 0xb5) => Opcode::Or(
                Operand::Register(Register::Reg8(Reg8::L), false),
                1),
            (None, 0xb6) => Opcode::Or(
                Operand::Register(Register::Reg16(Reg16::HL), true),
                1),
            (None, 0xb7) => Opcode::Or(
                Operand::Register(Register::Reg8(Reg8::A), false),
                1),
            (None, 0xbe) => Opcode::Compare(
                Operand::Register(Register::Reg16(Reg16::HL), true),
                1),
            (None, 0xc0) => Opcode::Return(
                Condition::NotZero,
                1),
            (None, 0xc1) => Opcode::Pop(
                Register::Reg16(Reg16::BC),
                1),
            (None, 0xc2) => Opcode::Jump(
                Condition::NotZero,
                Immediate::U16(self.read_u16(offset + 1)),
                3),
            (None, 0xc3) => Opcode::Jump(
                Condition::None,
                Immediate::U16(self.read_u16(offset + 1)),
                3),
            (None, 0xc5) => Opcode::Push(
                Register::Reg16(Reg16::BC),
                1),
            (None, 0xc7) => Opcode::Restart(
                Immediate::U8(0x00),
                1),
            (None, 0xc9) => Opcode::Return(
                Condition::None,
                1),
            (None, 0xcd) => Opcode::CallUnconditional(
                Operand::Immediate(Immediate::U16(self.read_u16(offset + 1)), false),
                3),
            (None, 0xd1) => Opcode::Pop(
                Register::Reg16(Reg16::DE),
                1),
            (None, 0xd2) => Opcode::Jump(
                Condition::NotCarry,
                Immediate::U16(self.read_u16(offset + 1)),
                3),
            (None, 0xd3) => Opcode::Out(
                Operand::Immediate(Immediate::U8(self.data[offset + 1]), true), // todo: is true correct?
                Operand::Register(Register::Reg8(Reg8::A), false),
                2),
            (None, 0xd5) => Opcode::Push(
                Register::Reg16(Reg16::DE),
                1),
            (None, 0xd7) => Opcode::Restart(
                Immediate::U8(0x10),
                1),
            (None, 0xdb) => Opcode::In(
                Operand::Register(Register::Reg8(Reg8::A), false),
                Operand::Immediate(Immediate::U8(self.data[offset + 1]), true), // todo: is true correct?
                2),
            (None, 0xe5) => Opcode::Push(
                Register::Reg16(Reg16::HL),
                1),
            (None, 0xe7) => Opcode::Restart(
                Immediate::U8(0x20),
                1),
            (None, 0xf1) => Opcode::Pop(
                Register::Reg16(Reg16::AF),
                1),
            (None, 0xf2) => Opcode::Jump(
                Condition::NotSign,
                Immediate::U16(self.read_u16(offset + 1)),
                3),
            (None, 0xf3) => Opcode::DisableInterrupts(1),
            (None, 0xf5) => Opcode::Push(
                Register::Reg16(Reg16::AF),
                1),
            (None, 0xf7) => Opcode::Restart(
                Immediate::U8(0x30),
                1),
            (None, 0xfa) => Opcode::Jump(
                Condition::Sign,
                Immediate::U16(self.read_u16(offset + 1)),
                3),
            (None, 0xfe) => Opcode::Compare(
                Operand::Immediate(Immediate::U8(self.data[offset + 1]), false),
                2),

            // 0xCB PREFIX
            (Some(0xcb), 0xbf) => Opcode::ResetBit(
                Immediate::U8(7),
                Operand::Register(Register::Reg8(Reg8::A), false),
                2),

            // 0xED PREFIX
            (Some(0xed), 0x79) => Opcode::Out(
                Operand::Register(Register::Reg8(Reg8::C), true),
                Operand::Register(Register::Reg8(Reg8::A), false),
                2),
            (Some(0xed), 0xa3) => Opcode::Outi(2),
            (Some(0xed), 0xb0) => Opcode::LoadIndirectRepeat(2),
            (Some(0xed), 0xb3) => Opcode::OutIndirectRepeat(2),
            (Some(0xed), 0x45) => Opcode::ReturnFromNmi(2),

            // Default decode error case
            _ => Opcode::Unknown(0)
        }
    }

    fn read_u16(&self, offset: usize) -> u16 {
        let low = self.data[offset] as u16;
        let high = self.data[offset + 1] as u16;

        (high << 8) | low
    }

    fn calc_length(&self, opcode: Opcode) -> usize {
        match opcode {
            Opcode::DisableInterrupts(length) => length,
            Opcode::Load(_, _, length) => length,
            Opcode::LoadIndirectRepeat(length) => length,
            Opcode::Out(_, _, length) => length,
            Opcode::In(_, _, length) => length,
            Opcode::Compare(_, length) => length,
            Opcode::JumpRelative(_, _, length) => length,
            Opcode::Jump(_, _, length) => length,
            Opcode::CallUnconditional(_, length) => length,
            Opcode::Xor(_, length) => length,
            Opcode::OutIndirectRepeat(length) => length,
            Opcode::NoOperation(length) => length,
            Opcode::ReturnFromNmi(length) => length,
            Opcode::Or(_, length) => length,
            Opcode::Decrement(_, length) => length,
            Opcode::Increment(_, length) => length,
            Opcode::DecrementAndJumpRelative(_, length) => length,
            Opcode::Restart(_, length) => length,
            Opcode::Return(_, length) => length,
            Opcode::Push(_, length) => length,
            Opcode::Pop(_, length) => length,
            Opcode::ResetBit(_, _, length) => length,
            Opcode::SetBit(_, _, length) => length,
            Opcode::Outi(length) => length,
            Opcode::Unknown(length) => length
        }
    }
}