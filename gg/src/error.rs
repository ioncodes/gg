use snafu::prelude::*;

#[derive(Debug, Snafu)]
pub(crate) enum GgError {
    #[snafu(display("I/O request not fulfilled"))]
    IoRequestNotFulfilled,
    #[snafu(display("Bus request out of bounds"))]
    BusRequestOutOfBounds,
}