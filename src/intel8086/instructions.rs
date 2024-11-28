use super::decoding::*;
use super::error::*;
use super::registers::*;
use bitfield_struct::bitfield;
use log::debug;

#[derive(Debug, Default)]
pub struct Instruction {
    pub data: [u8; 6], // Instructions are at most 6 bytes.
    pub len: u8,

    pub operation: Operation,

    pub dst: Operand,
    pub src: Operand,

    pub vmod: u8,
    pub val_rm: u8,
    pub val_reg: u8,

    pub bits: InstructionBits,
}

#[bitfield(u16)]
pub struct InstructionBits {
    #[bits(2)]
    pub vmod: u8,
    #[bits(3)]
    pub rm: u8,
    #[bits(3)]
    pub reg: u8,

    pub s: bool,
    pub w: bool,
    pub d: bool,
    pub v: bool,
    pub z: bool,

    #[bits(3)]
    _padding: u8,
}

impl Instruction {
    pub fn decode(bytes: &[u8]) -> Result<Self, IntelError> {
        if bytes.is_empty() {
            return Err(IntelError::IncompleteByteStream);
        }

        let peek = bytes[0];
        debug!("PEEK: 0x{0:02X} 0b{0:08b}", peek);

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
            return decode_immediate_to_register_memory(bytes, Operation::Mov);
        } else if compare_mask(peek, 0b100000, 6) {
            return decode_op_immediate_to_register_memory(bytes);
        }

        // Accumulator.
        if compare_mask(peek, 0b1010000, 7) {
            return decode_mov_accumulator_to_from_memory(bytes, true);
        } else if compare_mask(peek, 0b1010001, 7) {
            return decode_mov_accumulator_to_from_memory(bytes, false);
        } else if compare_mask(peek, 0b000010, 7) {
            return decode_op_immediate_to_accumulator(bytes, Operation::Add);
        } else if compare_mask(peek, 0b0010110, 7) {
            return decode_op_immediate_to_accumulator(bytes, Operation::Sub);
        } else if compare_mask(peek, 0b0011110, 7) {
            return decode_op_immediate_to_accumulator(bytes, Operation::Cmp);
        }

        // Jumps
        for (jump_opcode, operation) in SHORT_JUMPS {
            if peek == *jump_opcode {
                return decode_jump(bytes, operation);
            }
        }

        // Loops.
        for (loop_opcode, operation) in LOOP_JUMPS {
            if peek == *loop_opcode {
                return decode_jump(bytes, operation);
            }
        }

        Err(IntelError::UnsupportedOpcode(peek))
    }

    pub(super) fn new() -> Self {
        Instruction {
            ..Default::default()
        }
    }

    pub fn len(&self) -> usize {
        self.len as usize
    }

    pub(super) fn consume(&mut self, bytes: &[u8], amount: usize) -> Result<(), IntelError> {
        let start = self.len();
        let end = self.len() + amount;

        if end > bytes.len() {
            return Err(IntelError::IncompleteByteStream);
        }

        if end > self.data.len() {
            return Err(IntelError::InstructionOverflow);
        }

        self.data[start..end].copy_from_slice(&bytes[start..end]);
        self.len += amount as u8;

        Ok(())
    }

    pub(super) fn lastu8(&self) -> u8 {
        self.data[self.len() - 1]
    }

    pub(super) fn lastu16(&self) -> u16 {
        let len = self.len();
        let data: &[u8] = &self.data[(len - 2)..len];
        let b1: u16 = data[0] as u16;
        let b2: u16 = (data[1] as u16) << 8;
        b1 | b2
    }
}

#[derive(Debug)]
pub enum Operation {
    Invalid,
    Mov,
    Add,
    Sub,
    Cmp,
    Jump(&'static str),
}

#[derive(Debug)]
pub enum Operand {
    Invalid,
    Register(Register),
    Immediate(u16),
    EAC(EAC),
    JumpOffset(i8),
}

impl Operand {
    pub fn is_valid(&self) -> bool {
        !matches!(self, Operand::Invalid)
    }

    pub fn has_size(&self) -> bool {
        match self {
            Operand::Invalid => false,
            Operand::Register(_) => true,
            Operand::Immediate(_) => false,
            Operand::EAC(_) => false,
            Operand::JumpOffset(_) => false,
        }
    }
}

impl std::fmt::Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if !self.src.is_valid() {
            return write!(f, "{} {}", self.operation, self.dst);
        }

        if !self.src.has_size() && !self.dst.has_size() {
            let size_specifier = if self.bits.w() { "word" } else { "byte" };

            return write!(f, "{} {} {}, {}", self.operation, size_specifier, self.dst, self.src);
        }

        write!(f, "{} {}, {}", self.operation, self.dst, self.src)
    }
}

impl std::fmt::Display for Operation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string: &'static str = match self {
            Operation::Invalid => "<invalid>",
            Operation::Mov => "mov",
            Operation::Add => "add",
            Operation::Sub => "sub",
            Operation::Cmp => "cmp",
            Operation::Jump(name) => name,
        };

        write!(f, "{}", string)
    }
}

impl std::fmt::Display for Operand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Operand::Invalid => write!(f, "<invalid>"),
            Operand::Register(register) => write!(f, "{}", register),
            Operand::Immediate(i) => write!(f, "{}", i),
            Operand::EAC(eac) => write!(f, "{}", eac),
            Operand::JumpOffset(offset) => {
                // The jump encoding has a 2 implicit offset.
                let offset = offset + 2;
                if offset > 0 {
                    write!(f, "$+{}+0", offset)
                } else if offset == 0 {
                    write!(f, "$+0")
                } else {
                    // offset < 0
                    write!(f, "${}+0", offset)
                }
            }
        }
    }
}

impl Default for Operation {
    fn default() -> Self {
        Operation::Invalid
    }
}

impl Default for Operand {
    fn default() -> Self {
        Operand::Invalid
    }
}

// HELPERS -----------------------------------------------------------------------------------------

fn compare_mask(value: u8, mask: u8, mask_len: u8) -> bool {
    let shifted = value >> (8 - mask_len);
    shifted == mask
}
