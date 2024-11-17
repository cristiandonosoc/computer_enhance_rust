use super::decoding::*;
use super::error::*;

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
            return decode_op_register_memory_to_from_either(bytes); // mov.
        } else if compare_mask(peek, 0b000000, 6) {
            return decode_op_register_memory_to_from_either(bytes); // add.
        } else if compare_mask(peek, 0b001010, 6) {
            return decode_op_register_memory_to_from_either(bytes); // sub.
        } else if compare_mask(peek, 0b001110, 6) {
            return decode_op_register_memory_to_from_either(bytes); // cmp.
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
            return decode_mov_accumulator_to_from_memory(bytes, true);
        } else if compare_mask(peek, 0b1010001, 7) {
            return decode_mov_accumulator_to_from_memory(bytes, false);
        } else if compare_mask(peek, 0b000010, 7) {
            return decode_op_immediate_to_accumulator(bytes, "add");
        } else if compare_mask(peek, 0b0010110, 7) {
            return decode_op_immediate_to_accumulator(bytes, "sub");
        } else if compare_mask(peek, 0b0011110, 7) {
            return decode_op_immediate_to_accumulator(bytes, "cmp");
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

impl std::fmt::Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.mnemonic)
    }
}

fn compare_mask(value: u8, mask: u8, mask_len: u8) -> bool {
    let shifted = value >> (8 - mask_len);
    shifted == mask
}
