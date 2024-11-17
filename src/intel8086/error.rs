use thiserror::Error;

#[derive(Error, Debug)]
pub enum IntelError {
    #[error("Input byte stream ended unexpectedly")]
    IncompleteByteStream,
    #[error("Unsupported opcode: 0x{0:02X} 0b{0:08b}")]
    UnsupportedOpcode(u8),
    #[error("Invalid opcode: 0x{0:02X} 0b{0:08b}")]
    InvalidOpcode(u8),
    #[error("Instruction overflow")]
    InstructionOverflow,
    #[error("Unsupported operation: {0:03b}")]
    UnsupportedOperation(u8),
}


