mod byte_stream;
pub mod error;
pub mod instructions;
pub mod registers;

use error::IntelError;
use instructions::*;
use log::debug;

pub fn disassemble(mut bytes: &[u8]) -> Result<Vec<Instruction>, IntelError> {
    let mut instructions = vec![];

    while !bytes.is_empty() {
        let instruction = Instruction::new(bytes)?;
        debug!("\n{:?}", instruction);

        bytes = bytes
            .get(instruction.len()..)
            .ok_or(IntelError::IncompleteByteStream)?;
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
