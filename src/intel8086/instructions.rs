use super::decoding::*;
use super::error::*;
use super::registers::*;

#[derive(Debug)]
pub struct Instruction {
    pub data: [u8; 6], // Instructions are at most 6 bytes.
    pub mnemonic: String,
    pub len: usize,
}

impl Instruction {
    pub fn decode(bytes: &[u8]) -> Result<(Self, &[u8]), IntelError> {
        if bytes.is_empty() {
            return Err(IntelError::IncompleteByteStream);
        }

        let peek = bytes[0];

        // Register/Memory to/from either.
        if compare_mask(peek, 0b100010, 6) {
            return decode_op_register_memory_to_from_either(bytes);
        } else if compare_mask(peek, 0b000000, 6) {
            return decode_op_register_memory_to_from_either(bytes);
        }

        if compare_mask(peek, 0b1011, 4) {
            return decode_mov_immediate_to_register(bytes);
        }

        // Immediate register to/from_memory.
        if compare_mask(peek, 0b1100011, 7) {
            return decode_immediate_to_register_memory(bytes, "mov");
        } else if compare_mask(peek, 0b100000, 6) {
            return decode_op_immediate_to_register_memory(bytes);
        }

        // Accumulator.
        if compare_mask(peek, 0b1010000, 7) {
            return decode_memory_to_accumulator(bytes);
        } else if compare_mask(peek, 0b1010001, 7) {
            return decode_accumulator_to_memory(bytes);
        }

        Err(IntelError::UnsupportedOpcode(peek))
    }

    pub(super) fn new() -> Self {
        Instruction {
            data: [0; 6],
            mnemonic: "".to_owned(),
            len: 0,
        }
    }

    pub(super) fn add_byte(&mut self, byte: u8) -> Result<&mut Self, IntelError> {
        if self.len == 6 {
            return Err(IntelError::InstructionOverflow);
        }

        self.data[self.len] = byte;
        self.len += 1;
        Ok(self)
    }

    pub(super) fn add_bytes(&mut self, bytes: &[u8]) -> Result<&mut Self, IntelError> {
        for byte in bytes {
            self.add_byte(*byte)?;
        }
        Ok(self)
    }
}

fn decode_mov_immediate_to_register(bytes: &[u8]) -> IntelResult {
    let (data, rest) = consume(bytes, 1)?;
    let peek = data[0];
    let val_w: bool = (peek & 0b1000) != 0;
    let val_reg: u8 = peek & 0b111;

    let mut instruction = Instruction::new();
    instruction.add_byte(peek)?;

    let (value, rest) = if !val_w {
        let (data, rest) = consume(rest, 1)?;
        instruction.add_byte(data[0])?;
        ((data[0] as u16), rest)
    } else {
        let (data, rest) = consume(rest, 2)?;
        instruction.add_bytes(data)?;
        (to_intel_u16(data), rest)
    };

    let dst = interpret_register(val_reg, val_w);
    instruction.mnemonic = format!("mov {}, {}", dst, value);

    Ok((instruction, rest))
}

fn decode_memory_to_accumulator(bytes: &[u8]) -> IntelResult {
    decode_accumulator_to_from_memory(bytes, true)
}

fn decode_accumulator_to_memory(bytes: &[u8]) -> IntelResult {
    decode_accumulator_to_from_memory(bytes, false)
}

// Depending on this "d" bit, determines whether the accumulator is the destination.
fn decode_accumulator_to_from_memory(bytes: &[u8], direction: bool) -> IntelResult {
    let (data, rest) = consume(bytes, 3)?;

    let val_w: bool = (data[0] & 0b1) != 0;
    let reg = interpret_accumulator(val_w).to_string();
    let value = format!("[{}]", to_intel_u16(&data[1..]));

    let mut instruction = Instruction::new();
    instruction.add_bytes(data)?;

    let (src, dst) = if direction {
        (value, reg)
    } else {
        (reg, value)
    };

    instruction.mnemonic = format!("mov {}, {}", dst, src);

    Ok((instruction, rest))
}

impl std::fmt::Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.mnemonic)
    }
}

fn compare_mask(value: u8, mask: u8, mask_len: u8) -> bool {
    let shifted = value >> (8 - mask_len);
    shifted == mask
}
