use crate::instruction::{Condition, Immediate, Instruction, Opcode, Operand, Reg16, Reg8, Register};

pub struct Disassembler<'a> {
    pub data: &'a [u8],
}

impl<'a> Disassembler<'a> {
    pub fn new(data: &'a [u8]) -> Disassembler {
        Disassembler { data }
    }

    pub fn decode(&self, offset: usize) -> Result<Instruction, String> {
        let opcode = self.data[offset];
        let (prefix, opcode) = if opcode == 0xdd || opcode == 0xed || opcode == 0xfd || opcode == 0xcb {
            (Some(opcode), self.data[offset + 1])
        } else {
            (None, opcode)
        };

        let opcode = self.decode_opcode(offset, prefix, opcode);

        if opcode != Opcode::Unknown(0) {
            let length = self.calc_length(opcode);
            Ok(Instruction {
                opcode,
                length,
                _prefix: prefix,
                offset,
            })
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
                Err(msg) => panic!("{}", msg),
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
                3,
            ),
            (None, 0x02) => Opcode::Load(
                Operand::Register(Register::Reg16(Reg16::BC), true),
                Operand::Register(Register::Reg8(Reg8::A), false),
                1,
            ),
            (None, 0x03) => Opcode::Increment(Operand::Register(Register::Reg16(Reg16::BC), false), 1),
            (None, 0x04) => Opcode::Increment(Operand::Register(Register::Reg8(Reg8::B), false), 1),
            (None, 0x05) => Opcode::Decrement(Operand::Register(Register::Reg8(Reg8::B), false), 1),
            (None, 0x06) => Opcode::Load(
                Operand::Register(Register::Reg8(Reg8::B), false),
                Operand::Immediate(Immediate::U8(self.data[offset + 1]), false),
                2,
            ),
            (None, 0x09) => Opcode::Add(
                Operand::Register(Register::Reg16(Reg16::HL), false),
                Operand::Register(Register::Reg16(Reg16::BC), false),
                1,
            ),
            (None, 0x0b) => Opcode::Decrement(Operand::Register(Register::Reg16(Reg16::BC), false), 1),
            (None, 0x0c) => Opcode::Increment(Operand::Register(Register::Reg8(Reg8::C), false), 1),
            (None, 0x0d) => Opcode::Decrement(Operand::Register(Register::Reg8(Reg8::C), false), 1),
            (None, 0x0e) => Opcode::Load(
                Operand::Register(Register::Reg8(Reg8::C), false),
                Operand::Immediate(Immediate::U8(self.data[offset + 1]), false),
                2,
            ),
            (None, 0x0f) => Opcode::RotateRightCarry(1),
            (None, 0x10) => Opcode::DecrementAndJumpRelative(Immediate::S8(self.data[offset + 1] as i8), 2),
            (None, 0x11) => Opcode::Load(
                Operand::Register(Register::Reg16(Reg16::DE), false),
                Operand::Immediate(Immediate::U16(self.read_u16(offset + 1)), false),
                3,
            ),
            (None, 0x12) => Opcode::Load(
                Operand::Register(Register::Reg16(Reg16::DE), true),
                Operand::Register(Register::Reg8(Reg8::A), false),
                1,
            ),
            (None, 0x13) => Opcode::Increment(Operand::Register(Register::Reg16(Reg16::DE), false), 1),
            (None, 0x14) => Opcode::Increment(Operand::Register(Register::Reg8(Reg8::D), false), 1),
            (None, 0x15) => Opcode::Decrement(Operand::Register(Register::Reg8(Reg8::D), false), 1),
            (None, 0x16) => Opcode::Load(
                Operand::Register(Register::Reg8(Reg8::D), false),
                Operand::Immediate(Immediate::U8(self.data[offset + 1]), false),
                2,
            ),
            (None, 0x18) => Opcode::JumpRelative(Condition::None, Immediate::S8(self.data[offset + 1] as i8), 2),
            (None, 0x19) => Opcode::Add(
                Operand::Register(Register::Reg16(Reg16::HL), false),
                Operand::Register(Register::Reg16(Reg16::DE), false),
                1,
            ),
            (None, 0x1a) => Opcode::Load(
                Operand::Register(Register::Reg8(Reg8::A), false),
                Operand::Register(Register::Reg16(Reg16::DE), true),
                1,
            ),
            (None, 0x1b) => Opcode::Decrement(Operand::Register(Register::Reg16(Reg16::DE), false), 1),
            (None, 0x1c) => Opcode::Increment(Operand::Register(Register::Reg8(Reg8::E), false), 1),
            (None, 0x1d) => Opcode::Decrement(Operand::Register(Register::Reg8(Reg8::E), false), 1),
            (None, 0x1f) => Opcode::RotateRightCarrySwap(1),
            (None, 0x20) => Opcode::JumpRelative(Condition::NotZero, Immediate::S8(self.data[offset + 1] as i8), 2),
            (None, 0x21) => Opcode::Load(
                Operand::Register(Register::Reg16(Reg16::HL), false),
                Operand::Immediate(Immediate::U16(self.read_u16(offset + 1)), false),
                3,
            ),
            (None, 0x22) => Opcode::Load(
                Operand::Immediate(Immediate::U16(self.read_u16(offset + 1)), true),
                Operand::Register(Register::Reg16(Reg16::HL), false),
                3,
            ),
            (None, 0x23) => Opcode::Increment(Operand::Register(Register::Reg16(Reg16::HL), false), 1),
            (None, 0x24) => Opcode::Increment(Operand::Register(Register::Reg8(Reg8::H), false), 1),
            (None, 0x25) => Opcode::Decrement(Operand::Register(Register::Reg8(Reg8::H), false), 1),
            (None, 0x26) => Opcode::Load(
                Operand::Register(Register::Reg8(Reg8::H), false),
                Operand::Immediate(Immediate::U8(self.data[offset + 1]), false),
                2,
            ),
            (None, 0x28) => Opcode::JumpRelative(Condition::Zero, Immediate::S8(self.data[offset + 1] as i8), 2),
            (None, 0x29) => Opcode::Add(
                Operand::Register(Register::Reg16(Reg16::HL), false),
                Operand::Register(Register::Reg16(Reg16::HL), false),
                1,
            ),
            (None, 0x2a) => Opcode::Load(
                Operand::Register(Register::Reg16(Reg16::HL), false),
                Operand::Immediate(Immediate::U16(self.read_u16(offset + 1)), true),
                3,
            ),
            (None, 0x2b) => Opcode::Decrement(Operand::Register(Register::Reg16(Reg16::HL), false), 1),
            (None, 0x2c) => Opcode::Increment(Operand::Register(Register::Reg8(Reg8::L), false), 1),
            (None, 0x2d) => Opcode::Decrement(Operand::Register(Register::Reg8(Reg8::L), false), 1),
            (None, 0x30) => Opcode::JumpRelative(Condition::NotCarry, Immediate::S8(self.data[offset + 1] as i8), 2),
            (None, 0x31) => Opcode::Load(
                Operand::Register(Register::Reg16(Reg16::SP), false),
                Operand::Immediate(Immediate::U16(self.read_u16(offset + 1)), false),
                3,
            ),
            (None, 0x32) => Opcode::Load(
                Operand::Immediate(Immediate::U16(self.read_u16(offset + 1)), true),
                Operand::Register(Register::Reg8(Reg8::A), false),
                3,
            ),
            (None, 0x33) => Opcode::Increment(Operand::Register(Register::Reg16(Reg16::SP), false), 1),
            (None, 0x34) => Opcode::Increment(Operand::Register(Register::Reg16(Reg16::HL), true), 1),
            (None, 0x35) => Opcode::Decrement(Operand::Register(Register::Reg16(Reg16::HL), true), 1),
            (None, 0x36) => Opcode::Load(
                Operand::Register(Register::Reg16(Reg16::HL), true),
                Operand::Immediate(Immediate::U8(self.data[offset + 1]), false),
                2,
            ),
            (None, 0x38) => Opcode::JumpRelative(Condition::Carry, Immediate::S8(self.data[offset + 1] as i8), 2),
            (None, 0x39) => Opcode::Add(
                Operand::Register(Register::Reg16(Reg16::HL), false),
                Operand::Register(Register::Reg16(Reg16::SP), false),
                1,
            ),
            (None, 0x3a) => Opcode::Load(
                Operand::Register(Register::Reg8(Reg8::A), false),
                Operand::Immediate(Immediate::U16(self.read_u16(offset + 1)), true),
                3,
            ),
            (None, 0x3b) => Opcode::Decrement(Operand::Register(Register::Reg16(Reg16::SP), false), 1),
            (None, 0x3c) => Opcode::Increment(Operand::Register(Register::Reg8(Reg8::A), false), 1),
            (None, 0x3d) => Opcode::Decrement(Operand::Register(Register::Reg8(Reg8::A), false), 1),
            (None, 0x3e) => Opcode::Load(
                Operand::Register(Register::Reg8(Reg8::A), false),
                Operand::Immediate(Immediate::U8(self.data[offset + 1]), false),
                2,
            ),
            (None, 0x40) => Opcode::Load(
                Operand::Register(Register::Reg8(Reg8::B), false),
                Operand::Register(Register::Reg8(Reg8::B), false),
                1,
            ),
            (None, 0x41) => Opcode::Load(
                Operand::Register(Register::Reg8(Reg8::B), false),
                Operand::Register(Register::Reg8(Reg8::C), false),
                1,
            ),
            (None, 0x42) => Opcode::Load(
                Operand::Register(Register::Reg8(Reg8::B), false),
                Operand::Register(Register::Reg8(Reg8::D), false),
                1,
            ),
            (None, 0x43) => Opcode::Load(
                Operand::Register(Register::Reg8(Reg8::B), false),
                Operand::Register(Register::Reg8(Reg8::E), false),
                1,
            ),
            (None, 0x44) => Opcode::Load(
                Operand::Register(Register::Reg8(Reg8::B), false),
                Operand::Register(Register::Reg8(Reg8::H), false),
                1,
            ),
            (None, 0x45) => Opcode::Load(
                Operand::Register(Register::Reg8(Reg8::B), false),
                Operand::Register(Register::Reg8(Reg8::L), false),
                1,
            ),
            (None, 0x46) => Opcode::Load(
                Operand::Register(Register::Reg8(Reg8::B), false),
                Operand::Register(Register::Reg16(Reg16::HL), true),
                1,
            ),
            (None, 0x47) => Opcode::Load(
                Operand::Register(Register::Reg8(Reg8::B), false),
                Operand::Register(Register::Reg8(Reg8::A), false),
                1,
            ),
            (None, 0x48) => Opcode::Load(
                Operand::Register(Register::Reg8(Reg8::C), false),
                Operand::Register(Register::Reg8(Reg8::B), false),
                1,
            ),
            (None, 0x49) => Opcode::Load(
                Operand::Register(Register::Reg8(Reg8::C), false),
                Operand::Register(Register::Reg8(Reg8::C), false),
                1,
            ),
            (None, 0x4a) => Opcode::Load(
                Operand::Register(Register::Reg8(Reg8::C), false),
                Operand::Register(Register::Reg8(Reg8::D), false),
                1,
            ),
            (None, 0x4b) => Opcode::Load(
                Operand::Register(Register::Reg8(Reg8::C), false),
                Operand::Register(Register::Reg8(Reg8::E), false),
                1,
            ),
            (None, 0x4c) => Opcode::Load(
                Operand::Register(Register::Reg8(Reg8::C), false),
                Operand::Register(Register::Reg8(Reg8::H), false),
                1,
            ),
            (None, 0x4d) => Opcode::Load(
                Operand::Register(Register::Reg8(Reg8::C), false),
                Operand::Register(Register::Reg8(Reg8::L), false),
                1,
            ),
            (None, 0x4e) => Opcode::Load(
                Operand::Register(Register::Reg8(Reg8::C), false),
                Operand::Register(Register::Reg16(Reg16::HL), true),
                1,
            ),
            (None, 0x4f) => Opcode::Load(
                Operand::Register(Register::Reg8(Reg8::C), false),
                Operand::Register(Register::Reg8(Reg8::A), false),
                1,
            ),
            (None, 0x50) => Opcode::Load(
                Operand::Register(Register::Reg8(Reg8::D), false),
                Operand::Register(Register::Reg8(Reg8::B), false),
                1,
            ),
            (None, 0x51) => Opcode::Load(
                Operand::Register(Register::Reg8(Reg8::D), false),
                Operand::Register(Register::Reg8(Reg8::C), false),
                1,
            ),
            (None, 0x52) => Opcode::Load(
                Operand::Register(Register::Reg8(Reg8::D), false),
                Operand::Register(Register::Reg8(Reg8::D), false),
                1,
            ),
            (None, 0x53) => Opcode::Load(
                Operand::Register(Register::Reg8(Reg8::D), false),
                Operand::Register(Register::Reg8(Reg8::E), false),
                1,
            ),
            (None, 0x54) => Opcode::Load(
                Operand::Register(Register::Reg8(Reg8::D), false),
                Operand::Register(Register::Reg8(Reg8::H), false),
                1,
            ),
            (None, 0x55) => Opcode::Load(
                Operand::Register(Register::Reg8(Reg8::D), false),
                Operand::Register(Register::Reg8(Reg8::L), false),
                1,
            ),
            (None, 0x56) => Opcode::Load(
                Operand::Register(Register::Reg8(Reg8::D), false),
                Operand::Register(Register::Reg16(Reg16::HL), true),
                1,
            ),
            (None, 0x58) => Opcode::Load(
                Operand::Register(Register::Reg8(Reg8::E), false),
                Operand::Register(Register::Reg8(Reg8::B), false),
                1,
            ),
            (None, 0x5e) => Opcode::Load(
                Operand::Register(Register::Reg8(Reg8::E), false),
                Operand::Register(Register::Reg16(Reg16::HL), true),
                1,
            ),
            (None, 0x5f) => Opcode::Load(
                Operand::Register(Register::Reg8(Reg8::E), false),
                Operand::Register(Register::Reg8(Reg8::A), false),
                1,
            ),
            (None, 0x60) => Opcode::Load(
                Operand::Register(Register::Reg8(Reg8::H), false),
                Operand::Register(Register::Reg8(Reg8::B), false),
                1,
            ),
            (None, 0x61) => Opcode::Load(
                Operand::Register(Register::Reg8(Reg8::H), false),
                Operand::Register(Register::Reg8(Reg8::C), false),
                1,
            ),
            (None, 0x62) => Opcode::Load(
                Operand::Register(Register::Reg8(Reg8::H), false),
                Operand::Register(Register::Reg8(Reg8::D), false),
                1,
            ),
            (None, 0x63) => Opcode::Load(
                Operand::Register(Register::Reg8(Reg8::H), false),
                Operand::Register(Register::Reg8(Reg8::E), false),
                1,
            ),
            (None, 0x64) => Opcode::Load(
                Operand::Register(Register::Reg8(Reg8::H), false),
                Operand::Register(Register::Reg8(Reg8::H), false),
                1,
            ),
            (None, 0x65) => Opcode::Load(
                Operand::Register(Register::Reg8(Reg8::H), false),
                Operand::Register(Register::Reg8(Reg8::L), false),
                1,
            ),
            (None, 0x66) => Opcode::Load(
                Operand::Register(Register::Reg8(Reg8::H), false),
                Operand::Register(Register::Reg16(Reg16::HL), true),
                1,
            ),
            (None, 0x68) => Opcode::Load(
                Operand::Register(Register::Reg8(Reg8::L), false),
                Operand::Register(Register::Reg8(Reg8::B), false),
                1,
            ),
            (None, 0x6f) => Opcode::Load(
                Operand::Register(Register::Reg8(Reg8::L), false),
                Operand::Register(Register::Reg8(Reg8::A), false),
                1,
            ),
            (None, 0x70) => Opcode::Load(
                Operand::Register(Register::Reg16(Reg16::HL), true),
                Operand::Register(Register::Reg8(Reg8::B), false),
                1,
            ),
            (None, 0x71) => Opcode::Load(
                Operand::Register(Register::Reg16(Reg16::HL), true),
                Operand::Register(Register::Reg8(Reg8::C), false),
                1,
            ),
            (None, 0x72) => Opcode::Load(
                Operand::Register(Register::Reg16(Reg16::HL), true),
                Operand::Register(Register::Reg8(Reg8::D), false),
                1,
            ),
            (None, 0x73) => Opcode::Load(
                Operand::Register(Register::Reg16(Reg16::HL), true),
                Operand::Register(Register::Reg8(Reg8::E), false),
                1,
            ),
            (None, 0x74) => Opcode::Load(
                Operand::Register(Register::Reg16(Reg16::HL), true),
                Operand::Register(Register::Reg8(Reg8::H), false),
                1,
            ),
            (None, 0x75) => Opcode::Load(
                Operand::Register(Register::Reg16(Reg16::HL), true),
                Operand::Register(Register::Reg8(Reg8::L), false),
                1,
            ),
            (None, 0x77) => Opcode::Load(
                Operand::Register(Register::Reg16(Reg16::HL), true),
                Operand::Register(Register::Reg8(Reg8::A), false),
                1,
            ),
            (None, 0x78) => Opcode::Load(
                Operand::Register(Register::Reg8(Reg8::A), false),
                Operand::Register(Register::Reg8(Reg8::B), false),
                1,
            ),
            (None, 0x79) => Opcode::Load(
                Operand::Register(Register::Reg8(Reg8::A), false),
                Operand::Register(Register::Reg8(Reg8::C), false),
                1,
            ),
            (None, 0x7a) => Opcode::Load(
                Operand::Register(Register::Reg8(Reg8::A), false),
                Operand::Register(Register::Reg8(Reg8::D), false),
                1,
            ),
            (None, 0x7b) => Opcode::Load(
                Operand::Register(Register::Reg8(Reg8::A), false),
                Operand::Register(Register::Reg8(Reg8::E), false),
                1,
            ),
            (None, 0x7c) => Opcode::Load(
                Operand::Register(Register::Reg8(Reg8::A), false),
                Operand::Register(Register::Reg8(Reg8::H), false),
                1,
            ),
            (None, 0x7d) => Opcode::Load(
                Operand::Register(Register::Reg8(Reg8::A), false),
                Operand::Register(Register::Reg8(Reg8::L), false),
                1,
            ),
            (None, 0x7e) => Opcode::Load(
                Operand::Register(Register::Reg8(Reg8::A), false),
                Operand::Register(Register::Reg16(Reg16::HL), true),
                1,
            ),
            (None, 0x7f) => Opcode::Load(
                Operand::Register(Register::Reg8(Reg8::A), false),
                Operand::Register(Register::Reg8(Reg8::A), false),
                1,
            ),
            (None, 0x80) => Opcode::Add(
                Operand::Register(Register::Reg8(Reg8::A), false),
                Operand::Register(Register::Reg8(Reg8::B), false),
                1,
            ),
            (None, 0x81) => Opcode::Add(
                Operand::Register(Register::Reg8(Reg8::A), false),
                Operand::Register(Register::Reg8(Reg8::C), false),
                1,
            ),
            (None, 0x82) => Opcode::Add(
                Operand::Register(Register::Reg8(Reg8::A), false),
                Operand::Register(Register::Reg8(Reg8::D), false),
                1,
            ),
            (None, 0x83) => Opcode::Add(
                Operand::Register(Register::Reg8(Reg8::A), false),
                Operand::Register(Register::Reg8(Reg8::E), false),
                1,
            ),
            (None, 0x84) => Opcode::Add(
                Operand::Register(Register::Reg8(Reg8::A), false),
                Operand::Register(Register::Reg8(Reg8::H), false),
                1,
            ),
            (None, 0x85) => Opcode::Add(
                Operand::Register(Register::Reg8(Reg8::A), false),
                Operand::Register(Register::Reg8(Reg8::L), false),
                1,
            ),
            (None, 0x86) => Opcode::Add(
                Operand::Register(Register::Reg8(Reg8::A), false),
                Operand::Register(Register::Reg16(Reg16::HL), true),
                1,
            ),
            (None, 0x87) => Opcode::Add(
                Operand::Register(Register::Reg8(Reg8::A), false),
                Operand::Register(Register::Reg8(Reg8::A), false),
                1,
            ),
            (None, 0x90) => Opcode::Subtract(Operand::Register(Register::Reg8(Reg8::B), false), 1),
            (None, 0x91) => Opcode::Subtract(Operand::Register(Register::Reg8(Reg8::C), false), 1),
            (None, 0x92) => Opcode::Subtract(Operand::Register(Register::Reg8(Reg8::D), false), 1),
            (None, 0x93) => Opcode::Subtract(Operand::Register(Register::Reg8(Reg8::E), false), 1),
            (None, 0x94) => Opcode::Subtract(Operand::Register(Register::Reg8(Reg8::H), false), 1),
            (None, 0x95) => Opcode::Subtract(Operand::Register(Register::Reg8(Reg8::L), false), 1),
            (None, 0x96) => Opcode::Subtract(Operand::Register(Register::Reg16(Reg16::HL), true), 1),
            (None, 0x97) => Opcode::Subtract(Operand::Register(Register::Reg8(Reg8::A), false), 1),
            (None, 0xa0) => Opcode::And(Operand::Register(Register::Reg8(Reg8::B), false), 1),
            (None, 0xa1) => Opcode::And(Operand::Register(Register::Reg8(Reg8::C), false), 1),
            (None, 0xa2) => Opcode::And(Operand::Register(Register::Reg8(Reg8::D), false), 1),
            (None, 0xa3) => Opcode::And(Operand::Register(Register::Reg8(Reg8::E), false), 1),
            (None, 0xa4) => Opcode::And(Operand::Register(Register::Reg8(Reg8::H), false), 1),
            (None, 0xa5) => Opcode::And(Operand::Register(Register::Reg8(Reg8::L), false), 1),
            (None, 0xa6) => Opcode::And(Operand::Register(Register::Reg16(Reg16::HL), true), 1),
            (None, 0xa7) => Opcode::And(Operand::Register(Register::Reg8(Reg8::A), false), 1),
            (None, 0xaf) => Opcode::Xor(Operand::Register(Register::Reg8(Reg8::A), false), 1),
            (None, 0xb0) => Opcode::Or(Operand::Register(Register::Reg8(Reg8::B), false), 1),
            (None, 0xb1) => Opcode::Or(Operand::Register(Register::Reg8(Reg8::C), false), 1),
            (None, 0xb2) => Opcode::Or(Operand::Register(Register::Reg8(Reg8::D), false), 1),
            (None, 0xb3) => Opcode::Or(Operand::Register(Register::Reg8(Reg8::E), false), 1),
            (None, 0xb4) => Opcode::Or(Operand::Register(Register::Reg8(Reg8::H), false), 1),
            (None, 0xb5) => Opcode::Or(Operand::Register(Register::Reg8(Reg8::L), false), 1),
            (None, 0xb6) => Opcode::Or(Operand::Register(Register::Reg16(Reg16::HL), true), 1),
            (None, 0xb7) => Opcode::Or(Operand::Register(Register::Reg8(Reg8::A), false), 1),
            (None, 0xbe) => Opcode::Compare(Operand::Register(Register::Reg16(Reg16::HL), true), 1),
            (None, 0xc0) => Opcode::Return(Condition::NotZero, 1),
            (None, 0xc1) => Opcode::Pop(Register::Reg16(Reg16::BC), 1),
            (None, 0xc2) => Opcode::Jump(
                Condition::NotZero,
                Operand::Immediate(Immediate::U16(self.read_u16(offset + 1)), false),
                3,
            ),
            (None, 0xc3) => Opcode::Jump(
                Condition::None,
                Operand::Immediate(Immediate::U16(self.read_u16(offset + 1)), false),
                3,
            ),
            (None, 0xc5) => Opcode::Push(Register::Reg16(Reg16::BC), 1),
            (None, 0xc6) => Opcode::Add(
                Operand::Register(Register::Reg8(Reg8::A), false),
                Operand::Immediate(Immediate::U8(self.data[offset + 1]), false),
                2,
            ),
            (None, 0xc7) => Opcode::Restart(Immediate::U8(0x00), 1),
            (None, 0xc8) => Opcode::Return(Condition::Zero, 1),
            (None, 0xc9) => Opcode::Return(Condition::None, 1),
            (None, 0xcd) => Opcode::CallUnconditional(Operand::Immediate(Immediate::U16(self.read_u16(offset + 1)), false), 3),
            (None, 0xd1) => Opcode::Pop(Register::Reg16(Reg16::DE), 1),
            (None, 0xd2) => Opcode::Jump(
                Condition::NotCarry,
                Operand::Immediate(Immediate::U16(self.read_u16(offset + 1)), false),
                3,
            ),
            (None, 0xd3) => Opcode::Out(
                Operand::Immediate(Immediate::U8(self.data[offset + 1]), true), // todo: is true correct?
                Operand::Register(Register::Reg8(Reg8::A), false),
                2,
            ),
            (None, 0xd5) => Opcode::Push(Register::Reg16(Reg16::DE), 1),
            (None, 0xd6) => Opcode::Subtract(Operand::Immediate(Immediate::U8(self.data[offset + 1]), false), 2),
            (None, 0xd7) => Opcode::Restart(Immediate::U8(0x10), 1),
            (None, 0xdb) => Opcode::In(
                Operand::Register(Register::Reg8(Reg8::A), false),
                Operand::Immediate(Immediate::U8(self.data[offset + 1]), true), // todo: is true correct?
                2,
            ),
            (None, 0xe1) => Opcode::Pop(Register::Reg16(Reg16::HL), 1),
            (None, 0xe5) => Opcode::Push(Register::Reg16(Reg16::HL), 1),
            (None, 0xe6) => Opcode::And(Operand::Immediate(Immediate::U8(self.data[offset + 1]), false), 2),
            (None, 0xe7) => Opcode::Restart(Immediate::U8(0x20), 1),
            (None, 0xe9) => Opcode::Jump(Condition::None, Operand::Register(Register::Reg16(Reg16::HL), true), 1),
            (None, 0xf0) => Opcode::Return(Condition::NotSign, 1),
            (None, 0xf1) => Opcode::Pop(Register::Reg16(Reg16::AF), 1),
            (None, 0xf2) => Opcode::Jump(
                Condition::NotSign,
                Operand::Immediate(Immediate::U16(self.read_u16(offset + 1)), false),
                3,
            ),
            (None, 0xf3) => Opcode::DisableInterrupts(1),
            (None, 0xf5) => Opcode::Push(Register::Reg16(Reg16::AF), 1),
            (None, 0xf6) => Opcode::Or(Operand::Immediate(Immediate::U8(self.data[offset + 1]), false), 2),
            (None, 0xf7) => Opcode::Restart(Immediate::U8(0x30), 1),
            (None, 0xf8) => Opcode::Return(Condition::Sign, 1),
            (None, 0xf9) => Opcode::Load(
                Operand::Register(Register::Reg16(Reg16::SP), false),
                Operand::Register(Register::Reg16(Reg16::HL), false),
                1,
            ),
            (None, 0xfa) => Opcode::Jump(
                Condition::Sign,
                Operand::Immediate(Immediate::U16(self.read_u16(offset + 1)), false),
                3,
            ),
            (None, 0xfb) => Opcode::EnableInterrupts(1),
            (None, 0xfe) => Opcode::Compare(Operand::Immediate(Immediate::U8(self.data[offset + 1]), false), 2),

            // 0xCB PREFIX
            (Some(0xcb), 0x08) => Opcode::RotateRightCarrySideeffect(Operand::Register(Register::Reg8(Reg8::B), false), 2),
            (Some(0xcb), 0x09) => Opcode::RotateRightCarrySideeffect(Operand::Register(Register::Reg8(Reg8::C), false), 2),
            (Some(0xcb), 0x0a) => Opcode::RotateRightCarrySideeffect(Operand::Register(Register::Reg8(Reg8::D), false), 2),
            (Some(0xcb), 0x0b) => Opcode::RotateRightCarrySideeffect(Operand::Register(Register::Reg8(Reg8::E), false), 2),
            (Some(0xcb), 0x0c) => Opcode::RotateRightCarrySideeffect(Operand::Register(Register::Reg8(Reg8::H), false), 2),
            (Some(0xcb), 0x0d) => Opcode::RotateRightCarrySideeffect(Operand::Register(Register::Reg8(Reg8::L), false), 2),
            (Some(0xcb), 0x0e) => Opcode::RotateRightCarrySideeffect(Operand::Register(Register::Reg16(Reg16::HL), true), 2),
            (Some(0xcb), 0x0f) => Opcode::RotateRightCarrySideeffect(Operand::Register(Register::Reg8(Reg8::A), false), 2),
            (Some(0xcb), 0x18) => Opcode::RotateRightCarrySwapSideeffect(Operand::Register(Register::Reg8(Reg8::B), false), 2),
            (Some(0xcb), 0x19) => Opcode::RotateRightCarrySwapSideeffect(Operand::Register(Register::Reg8(Reg8::C), false), 2),
            (Some(0xcb), 0x1a) => Opcode::RotateRightCarrySwapSideeffect(Operand::Register(Register::Reg8(Reg8::D), false), 2),
            (Some(0xcb), 0x1b) => Opcode::RotateRightCarrySwapSideeffect(Operand::Register(Register::Reg8(Reg8::E), false), 2),
            (Some(0xcb), 0x1c) => Opcode::RotateRightCarrySwapSideeffect(Operand::Register(Register::Reg8(Reg8::H), false), 2),
            (Some(0xcb), 0x1d) => Opcode::RotateRightCarrySwapSideeffect(Operand::Register(Register::Reg8(Reg8::L), false), 2),
            (Some(0xcb), 0x1e) => Opcode::RotateRightCarrySwapSideeffect(Operand::Register(Register::Reg16(Reg16::HL), true), 2),
            (Some(0xcb), 0x1f) => Opcode::RotateRightCarrySwapSideeffect(Operand::Register(Register::Reg8(Reg8::A), false), 2),
            (Some(0xcb), 0xbf) => Opcode::ResetBit(Immediate::U8(7), Operand::Register(Register::Reg8(Reg8::A), false), 2),

            // 0xED PREFIX
            (Some(0xed), 0x42) => Opcode::SubtractWithCarry(Register::Reg16(Reg16::HL), Register::Reg16(Reg16::BC), 2),
            (Some(0xed), 0x52) => Opcode::SubtractWithCarry(Register::Reg16(Reg16::HL), Register::Reg16(Reg16::DE), 2),
            (Some(0xed), 0x62) => Opcode::SubtractWithCarry(Register::Reg16(Reg16::HL), Register::Reg16(Reg16::HL), 2),
            (Some(0xed), 0x72) => Opcode::SubtractWithCarry(Register::Reg16(Reg16::HL), Register::Reg16(Reg16::SP), 2),
            (Some(0xed), 0x79) => Opcode::Out(
                Operand::Register(Register::Reg8(Reg8::C), true),
                Operand::Register(Register::Reg8(Reg8::A), false),
                2,
            ),
            (Some(0xed), 0xa3) => Opcode::Outi(2),
            (Some(0xed), 0xb0) => Opcode::LoadIndirectRepeat(2),
            (Some(0xed), 0xb3) => Opcode::OutIndirectRepeat(2),
            (Some(0xed), 0x45) => Opcode::ReturnFromNmi(2),
            (Some(0xed), 0x46) => Opcode::SetInterruptMode(Immediate::U8(0), 2),
            (Some(0xed), 0x56) => Opcode::SetInterruptMode(Immediate::U8(1), 2),
            (Some(0xed), 0x5e) => Opcode::SetInterruptMode(Immediate::U8(2), 2),

            // 0xDD PREFIX
            (Some(0xdd), 0x21) => Opcode::Load(
                Operand::Register(Register::Reg16(Reg16::IX(None)), false),
                Operand::Immediate(Immediate::U16(self.read_u16(offset + 2)), false),
                4,
            ),
            (Some(0xdd), 0x22) => Opcode::Load(
                Operand::Immediate(Immediate::U16(self.read_u16(offset + 2)), true),
                Operand::Register(Register::Reg16(Reg16::IX(None)), false),
                4,
            ),
            (Some(0xdd), 0x23) => Opcode::Increment(Operand::Register(Register::Reg16(Reg16::IX(None)), false), 2),
            (Some(0xdd), 0x36) => Opcode::Load(
                Operand::Register(Register::Reg16(Reg16::IX(Some(self.data[offset + 2] as i8))), true),
                Operand::Immediate(Immediate::U8(self.data[offset + 3]), false),
                4,
            ),
            (Some(0xdd), 0x77) => Opcode::Load(
                Operand::Register(Register::Reg16(Reg16::IX(Some(self.data[offset + 2] as i8))), true),
                Operand::Register(Register::Reg8(Reg8::A), false),
                3,
            ),
            (Some(0xdd), 0x7e) => Opcode::Load(
                Operand::Register(Register::Reg8(Reg8::A), false),
                Operand::Register(Register::Reg16(Reg16::IX(Some(self.data[offset + 2] as i8))), true),
                3,
            ),
            (Some(0xdd), 0xae) => Opcode::Xor(Operand::Register(Register::Reg16(Reg16::IX(Some(self.data[offset + 2] as i8))), true), 3),

            // 0xFD PREFIX
            (Some(0xfd), 0x09) => Opcode::Add(
                Operand::Register(Register::Reg16(Reg16::IY(None)), false),
                Operand::Register(Register::Reg16(Reg16::BC), false),
                2,
            ),
            (Some(0xfd), 0xe1) => Opcode::Pop(Register::Reg16(Reg16::IY(None)), 2),
            (Some(0xfd), 0x66) => Opcode::Load(
                Operand::Register(Register::Reg8(Reg8::H), false),
                Operand::Register(Register::Reg16(Reg16::IY(Some(self.data[offset + 2] as i8))), true),
                3,
            ),
            (Some(0xfd), 0x6e) => Opcode::Load(
                Operand::Register(Register::Reg8(Reg8::L), false),
                Operand::Register(Register::Reg16(Reg16::IY(Some(self.data[offset + 2] as i8))), true),
                3,
            ),
            (Some(0xfd), 0x7e) => Opcode::Load(
                Operand::Register(Register::Reg8(Reg8::A), false),
                Operand::Register(Register::Reg16(Reg16::IY(Some(self.data[offset + 2] as i8))), true),
                3,
            ),

            // Default decode error case
            _ => Opcode::Unknown(0),
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
            Opcode::EnableInterrupts(length) => length,
            Opcode::SubtractWithCarry(_, _, length) => length,
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
            Opcode::SetInterruptMode(_, length) => length,
            Opcode::And(_, length) => length,
            Opcode::Subtract(_, length) => length,
            Opcode::Add(_, _, length) => length,
            Opcode::RotateRightCarry(length) => length,
            Opcode::RotateRightCarrySwap(length) => length,
            Opcode::RotateRightCarrySideeffect(_, length) => length,
            Opcode::RotateRightCarrySwapSideeffect(_, length) => length,
            Opcode::Unknown(length) => length,
        }
    }
}
