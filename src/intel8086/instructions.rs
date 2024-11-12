use super::registers::*;

#[derive(Debug)]
pub enum Instruction {
    RegisterMemoryToFromRegister(Instruction_RegisterMemoryToFromRegister),
}

impl std::fmt::Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Instruction::RegisterMemoryToFromRegister(mov_instruction) => {
                write!(f, "{}", mov_instruction)
            }
        }
    }
}

// RegisterMemoryToFromRegister --------------------------------------------------------------------

#[allow(non_camel_case_types)]
pub struct Instruction_RegisterMemoryToFromRegister {
    pub data: [u8; 6],
}

impl Instruction_RegisterMemoryToFromRegister {
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

    pub fn mnemonic(&self) -> String {
        format!("mov {}, {}", self.get_dst_register(), self.get_src_register(),)
    }
}

impl std::fmt::Display for Instruction_RegisterMemoryToFromRegister {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if self.r#mod() != 0b11 {
            unimplemented!()
        }

        write!(f, "{}", self.mnemonic())
    }
}

impl std::fmt::Debug for Instruction_RegisterMemoryToFromRegister {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "")?;
        writeln!(f, "opcode: {:06b}", self.opcode())?;
        writeln!(f, "D bit: {}", self.d())?;
        writeln!(f, "W bit: {}", self.w())?;
        writeln!(f, "MOD: {0} ({0:02b})", self.r#mod())?;
        writeln!(
            f,
            "REG: {0} ({0:03b}) -> {1}",
            self.reg(),
            interpret_register(self.reg(), self.w())
        )?;
        writeln!(
            f,
            "R/M: {0} ({0:03b}) -> {1}",
            self.rm(),
            interpret_register(self.rm(), self.w())
        )?;
        writeln!(f, "Mnemonic: {}", self.mnemonic())?;

        Ok(())
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
