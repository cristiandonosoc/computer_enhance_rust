use super::error::*;
use super::registers::*;

#[derive(Debug)]
pub struct Instruction {
    pub data: Vec<u8>,
    pub mnemonic: String,
}

impl Instruction {
    pub fn new(bytes: &[u8]) -> Result<Self, IntelError> {
        if bytes.is_empty() {
            return Err(IntelError::IncompleteByteStream);
        }

        let opcode_byte = bytes[0];

        if opcode_byte & 0b111111_00 == 0b100010_00 {
            return Self::decode_register_memory_to_from_register(bytes);
        }

        Err(IntelError::UnsupportedOpcode(opcode_byte))
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    fn decode_register_memory_to_from_register(bytes: &[u8]) -> Result<Self, IntelError> {
        if bytes.len() < 2 {
            return Err(IntelError::IncompleteByteStream);
        }

        let val_d: bool = (bytes[0] & 0b10) != 0;
        let val_w: bool = (bytes[0] & 0b01) != 0;
        let val_mod: u8 = bytes[1] >> 6;
        let val_reg: u8 = (bytes[1] >> 3) & 0b111;
        let val_rm: u8 = bytes[1] & 0b111;

        let (src, dst, len) = match val_mod {
            0b00 => {
                unimplemented!()
            }
            0b01 => {
                unimplemented!()
            }
            0b10 => {
                unimplemented!()
            }
            0b11 => {
                let len = 2;

                let src = match val_d {
                    false => interpret_register(val_reg, val_w).to_string(),
                    true => interpret_register(val_rm, val_w).to_string(),
                };

                let dst = match val_d {
                    false => interpret_register(val_rm, val_w).to_string(),
                    true => interpret_register(val_reg, val_w).to_string(),
                };

                (src, dst, len)
            }
            _ => panic!(),
        };

        if bytes.len() < len {
            return Err(IntelError::IncompleteByteStream);
        }

        let mnemonic = format!("mov {}, {}", dst, src);
        Ok(Instruction {
            data: bytes[..len].to_vec(),
            mnemonic,
        })
    }
}

impl std::fmt::Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.mnemonic)
    }
}

