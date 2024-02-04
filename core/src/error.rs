use snafu::prelude::*;
use z80::instruction::Opcode;

#[derive(Debug, Snafu)]
pub enum GgError {
    #[snafu(display("I/O request not fulfilled"))]
    IoRequestNotFulfilled,
    #[snafu(display("Bus request out of bounds {address}"))]
    BusRequestOutOfBounds { address: u16 },
    #[snafu(display("Opcode not implemented {opcode}"))]
    OpcodeNotImplemented { opcode: Opcode },
    #[snafu(display("Decoder errored with message: {msg}"))]
    DecoderError { msg: String },
    #[snafu(display("Jump not taken"))]
    JumpNotTaken,
    #[snafu(display("Breakpoint hit"))]
    BreakpointHit,
    #[snafu(display("Invalid interrupt mode: {mode}"))]
    InvalidInterruptMode { mode: u8},
    #[snafu(display("Missing operand implementation for instruction: {instruction}"))]
    InvalidOpcodeImplementation { instruction: Opcode },
    #[snafu(display("Invalid port for I/O controller"))]
    IoControllerInvalidPort,
    #[snafu(display("Invalid VDP I/O mode set"))]
    VdpInvalidIoMode,
}
