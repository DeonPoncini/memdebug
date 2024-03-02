
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Invalid return byte, {0} expected {1}")]
    InvalidReturnByte(u8, u8),
    #[error("Invalid byte size {0}, only 1, 2, 4 supported")]
    InvalidByteSize(u8),
}
