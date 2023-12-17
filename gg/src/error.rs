use snafu::prelude::*;
use z80::instruction::Opcode;

#[derive(Debug, Snafu)]
pub(crate) enum GgError {
    #[snafu(display("I/O request not fulfilled"))]
    IoRequestNotFulfilled,
    #[snafu(display("Bus request out of bounds {address}"))]
    BusRequestOutOfBounds { address: u16 },
    #[snafu(display("Opcode not implemented {opcode}"))]
    OpcodeNotImplemented { opcode: Opcode },
    #[snafu(display("Decoder errored with message: {msg}"))]
    DecoderError { msg: String },
    #[snafu(display("Jump not taken"))]
    JumpNotTaken
}
