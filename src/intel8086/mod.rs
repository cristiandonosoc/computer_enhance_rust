pub mod instructions;
pub mod registers;

use instructions::*;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum IntelError {
    #[error("Input byte stream ended unexpectedly")]
    IncompleteByteStream,
    #[error("Unsupported opcode: {0:08b}")]
    UnsupportedOpcode(u8),
    #[error("Invalid opcode: {0:08b}")]
    InvalidOpcode(u8),
}

pub fn disassemble(mut bytes: &[u8]) -> Result<Vec<Instruction>, IntelError> {
    let mut instructions = vec![];

    while !bytes.is_empty() {
        let (instruction, remaining_bytes) = decode_instruction(bytes)?;
        println!("------------------------------------------\n{:?}", instruction);
        instructions.push(instruction);
        bytes = remaining_bytes;
    }

    Ok(instructions)
}

fn decode_instruction(bytes: &[u8]) -> Result<(Instruction, &[u8]), IntelError> {
    if bytes.is_empty() {
        return Err(IntelError::IncompleteByteStream);
    }

    let opcode = opcode_utils::opcode(bytes[0]);
    match opcode {
        0b100010 => decode_mov(bytes),
        _ => Err(IntelError::UnsupportedOpcode(opcode)),
    }
}

fn decode_mov(bytes: &[u8]) -> Result<(Instruction, &[u8]), IntelError> {
    if bytes.len() < 2 {
        return Err(IntelError::IncompleteByteStream);
    }

    // Create a new mov instruction with the first 2 bytes set.
    let mut mi = MovInstruction { data: [0; 6] };
    mi.data[0] = bytes[0];
    mi.data[1] = bytes[1];
    Ok((Instruction::Mov(mi), skip(bytes, 2)))
}

fn skip(bytes: &[u8], skip_amount: usize) -> &[u8] {
    bytes.get(skip_amount..).unwrap_or(&[])
}
