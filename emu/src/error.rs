use snafu::prelude::*;
use z80::instruction::Opcode;

#[derive(Debug, Snafu, PartialEq, Clone)]
pub enum GgError {
    #[snafu(display("I/O request not fulfilled"))]
    IoRequestNotFulfilled,
    #[snafu(display("Bus request out of bounds: {:08x}", address))]
    BusRequestOutOfBounds { address: usize },
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
    #[snafu(display("CPU halted"))]
    CpuHalted,
    #[snafu(display("Joystick disabled"))]
    JoystickDisabled,
    #[snafu(display("Repeat not fulfilled"))]
    RepeatNotFulfilled,
    #[snafu(display("Write to ROM at address: {:08x}", address))]
    WriteToReadOnlyMemory { address: usize },
}
