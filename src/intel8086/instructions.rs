use super::error::*;
use super::registers::*;

#[derive(Debug)]
pub struct Instruction {
    pub data: [u8; 6], // Instructions are at most 6 bytes.
    pub mnemonic: String,
    len: u8,
}

impl Instruction {
    pub fn decode(bytes: &[u8]) -> Result<(Self, &[u8]), IntelError> {
        if bytes.is_empty() {
            return Err(IntelError::IncompleteByteStream);
        }

        let peek = bytes[0];

        if peek & 0b111111_00 == 0b100010_00 {
            return Self::decode_register_memory_to_from_register(bytes);
        }

        Err(IntelError::UnsupportedOpcode(peek))
    }

    pub fn len(&self) -> usize {
        self.len as usize
    }

    fn decode_register_memory_to_from_register(bytes: &[u8]) -> Result<(Self, &[u8]), IntelError> {
        let (first_bytes, rest) = consume(bytes, 2)?;

        let val_d: bool = (first_bytes[0] & 0b10) != 0;
        let val_w: bool = (first_bytes[0] & 0b01) != 0;
        let val_mod: u8 = first_bytes[1] >> 6;
        let val_reg: u8 = (first_bytes[1] >> 3) & 0b111;
        let val_rm: u8 = first_bytes[1] & 0b111;

        let mut instruction_data = [0; 6];
        instruction_data[0] = first_bytes[0];
        instruction_data[1] = first_bytes[1];

        let (operand, reg, len, rest) = match val_mod {
            0b00 => {
                let reg = interpret_register(val_reg, val_w).to_string();

                if val_rm != 0b110 {
                    let operand = eac(val_rm);
                    (operand, reg, 2, rest)
                } else {
                    // Otherwise it is a DIRECT ACCESS.
                    let (data, rest) = consume(rest, 2)?;
                    instruction_data[2] = data[0];
                    instruction_data[3] = data[1];

                    let operand = format!("{}", to_intel_u16(data));

                    (operand, reg, 4, rest)
                }
            }
            0b01 => {
                let (data, rest) = consume(rest, 1)?;

                let operand = format!("{} + {}", eac(val_rm), data[0]);
                let reg = interpret_register(val_reg, val_w).to_string();
                (operand, reg, 3, rest)
            }
            0b10 => {
                let (data, rest) = consume(rest, 2)?;

                let value = to_intel_u16(data);
                let operand = format!("{} + {}", eac(val_rm), value);
                let reg = interpret_register(val_reg, val_w).to_string();
                (operand, reg, 4, rest)
            }
            0b11 => {
                let operand = interpret_register(val_rm, val_w).to_string();
                let reg = interpret_register(val_reg, val_w).to_string();

                (operand, reg, 2, rest)
            }
            _ => panic!(),
        };

        let (src, dst) = if val_d {
            (operand, reg)
        } else {
            (reg, operand)
        };

        let mnemonic = format!("mov {}, {}", dst, src);
        Ok((
            Instruction {
                data: instruction_data,
                len,
                mnemonic,
            },
            rest,
        ))
    }
}

impl std::fmt::Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.mnemonic)
    }
}

fn consume(bytes: &[u8], amount: usize) -> Result<(&[u8], &[u8]), IntelError> {
    if bytes.len() < amount {
        return Err(IntelError::IncompleteByteStream);
    }

    let consumed = &bytes[..amount];
    let rest = &bytes[amount..];
    Ok((consumed, rest))
}

fn to_intel_u16(data: &[u8]) -> u16 {
    (data[0] as u16) & ((data[1] as u16) << 8)
}

fn eac(val_rm: u8) -> String {
    EAC_REGISTER[val_rm as usize].to_string()
}
