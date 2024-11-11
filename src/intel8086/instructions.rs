use std::fmt::Display;

use super::registers::*;

#[derive(Debug)]
pub enum Instruction {
    Mov(MovInstruction),
}

impl Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Instruction::Mov(mov_instruction) => write!(f, "{}", mov_instruction),
        }
    }
}

// Mov Instruction ---------------------------------------------------------------------------------

#[derive(Debug)]
pub struct MovInstruction {
    pub data: [u8; 6],
}

impl MovInstruction {
    pub fn opcode(&self) -> u8 {
        opcode_utils::opcode(self.data[0])
    }

    pub fn d(&self) -> bool {
        opcode_utils::d(self.data[0])
    }

    pub fn w(&self) -> bool {
        opcode_utils::w(self.data[0])
    }

    pub fn r#mod(&self) -> u8 {
        self.data[1] >> 6
    }

    pub fn reg(&self) -> u8 {
        (self.data[1] >> 3) & 0b111
    }

    pub fn rm(&self) -> u8 {
        self.data[1] & 0b111
    }

    pub fn get_src_register(&self) -> Register {
        match self.d() {
            false => interpret_register(self.reg(), self.w()),
            true => interpret_register(self.rm(), self.w()),
        }
    }

    pub fn get_dst_register(&self) -> Register {
        match self.d() {
            false => interpret_register(self.rm(), self.w()),
            true => interpret_register(self.reg(), self.w()),
        }
    }
}

impl Display for MovInstruction {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if self.r#mod() != 0b11 {
            unimplemented!()
        }

        write!(
            f,
            "mov {}, {}",
            self.get_src_register(),
            self.get_dst_register()
        )
    }
}

pub mod opcode_utils {
    pub fn opcode(byte: u8) -> u8 {
        byte >> 2
    }

    pub fn d(byte: u8) -> bool {
        (byte & 0b10) != 0
    }

    pub fn w(byte: u8) -> bool {
        (byte & 0b01) != 0
    }
}

pub struct SecondByte(pub u8);

impl SecondByte {}
