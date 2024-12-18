use thiserror::Error;
use super::instructions::*;

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

    #[error("Supplied program is too big ({0} vs max {1}")]
    ProgramTooBig(usize, usize),

    #[error("Invalid address 0x{0:06X} ({0})")]
    InvalidAddress(usize),

    #[error("Unsupported operation for simulation: {0}")]
    UnsupportedSimulationOperation(String),

    #[error("Invalid instruction: {0:?}")]
    InvalidInstruction(Instruction),

    #[error("Invalid Operand: {0}")]
    InvalidOperand(String),

    #[error("Unknown Register: {0}")]
    UnknownRegister(String),
}
