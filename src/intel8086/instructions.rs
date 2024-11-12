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
    pub data: [u8; 4],
}

impl Instruction_RegisterMemoryToFromRegister {
    pub fn opcode(&self) -> u8 {
        self.data[0] >> 2
    }

    pub fn d(&self) -> bool {
        (self.data[0] & 0b10) != 0
    }

    pub fn w(&self) -> bool {
        (self.data[0] & 0b01) != 0
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

    pub fn get_src(&self) -> String {
        match self.r#mod() {
            0b00 => {
                unimplemented!()
            }
            0b01 => {
                unimplemented!()
            }
            0b10 => {
                unimplemented!()
            }
            0b11 => match self.d() {
                false => interpret_register(self.reg(), self.w()).to_string(),
                true => interpret_register(self.rm(), self.w()).to_string(),
            },
            _ => panic!(),
        }
    }

    pub fn get_dst(&self) -> String {
        match self.r#mod() {
            0b00 => {
                unimplemented!()
            }
            0b01 => {
                unimplemented!()
            }
            0b10 => {
                unimplemented!()
            }
            0b11 => match self.d() {
                false => interpret_register(self.rm(), self.w()).to_string(),
                true => interpret_register(self.reg(), self.w()).to_string(),
            },
            _ => panic!(),
        }
    }

    pub fn mnemonic(&self) -> String {
        format!("mov {}, {}", self.get_dst(), self.get_src())
    }
}

impl std::fmt::Display for Instruction_RegisterMemoryToFromRegister {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
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
        writeln!(f, "Mnemonic: {}", self.mnemonic())?;

        Ok(())
    }
}
