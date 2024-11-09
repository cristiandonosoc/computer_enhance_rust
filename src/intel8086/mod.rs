pub mod registers;

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
    let mut instructions = vec!();

    loop {
        let (instruction, bytes) = decode_instruction(bytes)?;
        instructions.push(instruction);
        if bytes.is_empty() {
            break;
        }
    };

    Ok(instructions)
}

pub struct Instruction {
    pub opcode: OpcodeByte
}

fn decode_instruction(bytes: &[u8]) -> Result<(Instruction, &[u8]), IntelError> {
    if bytes.is_empty() {
        return Err(IntelError::IncompleteByteStream);
    }

    let opcode = OpcodeByte(bytes[0]);
    match opcode.opcode() {
        0b100010 => {
            decode_mov(opcode, skip(bytes, 1))
        }
        _ => {
            Err(IntelError::UnsupportedOpcode(opcode.opcode()))
        }
    }
}

fn decode_mov(_opcode: OpcodeByte, _bytes: &[u8]) -> Result<(Instruction, &[u8]), IntelError> {
    todo!()
}

pub struct OpcodeByte(pub u8);

impl OpcodeByte {
    pub fn opcode(&self) -> u8 {
        self.0 >> 2
    }

    pub fn d(&self) -> bool {
        (self.0 & 0b10) != 0
    }

    pub fn w(&self) -> bool {
        (self.0 & 0b01) != 0
    }
}

fn skip(bytes: &[u8], skip_amount: usize) -> &[u8] {
    bytes.get(skip_amount..).unwrap_or(&[])
}
