mod byte_stream;
pub mod instructions;
pub mod registers;

use byte_stream::ByteStream;
use instructions::*;
use log::debug;
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

pub fn disassemble(bytes: &[u8]) -> Result<Vec<Instruction>, IntelError> {
    let mut instructions = vec![];

    let mut stream = ByteStream::new(bytes);
    while !stream.is_empty() {
        let instruction = decode_instruction(&mut stream)?;

        debug!("\n{:?}", instruction);
        instructions.push(instruction);
    }

    Ok(instructions)
}

pub fn to_asm(instructions: &Vec<Instruction>) -> String {
    let instruction_strings: Vec<String> =
        instructions.iter().map(|i| i.to_string() + "\n").collect();
    let string: String = instruction_strings.into_iter().collect();
    format!("bits 16\n\n{}", string)
}

fn decode_instruction(stream: &mut ByteStream) -> Result<Instruction, IntelError> {
    if stream.is_empty() {
        return Err(IntelError::IncompleteByteStream);
    }

    let opcode = opcode_utils::opcode(stream.peek().unwrap());
    match opcode {
        0b100010 => decode_mov(stream),
        _ => Err(IntelError::UnsupportedOpcode(opcode)),
    }
}

fn decode_mov(stream: &mut ByteStream) -> Result<Instruction, IntelError> {
    let bytes = stream
        .consume(2)
        .map_err(|_| IntelError::IncompleteByteStream)?;

    // Create a new mov instruction with the first 2 bytes set.
    let mut mi = Instruction_RegisterMemoryToFromRegister { data: [0; 6] };
    mi.data[0] = bytes[0];
    mi.data[1] = bytes[1];
    Ok(Instruction::RegisterMemoryToFromRegister(mi))
}
