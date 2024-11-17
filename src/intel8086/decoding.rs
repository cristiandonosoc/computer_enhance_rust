use super::error::*;
use super::instructions::*;
use super::registers::*;
use ::function_name::named;
use log::debug;

pub(super) type IntelResult<'a> = Result<(Instruction, &'a [u8]), IntelError>;

#[named]
pub(super) fn decode_op_register_memory_to_from_either(bytes: &[u8]) -> IntelResult {
    debug!(function_name!());
    let (first_bytes, rest) = consume(bytes, 2)?;

    let val_d: bool = (first_bytes[0] & 0b10) != 0;
    let val_w: bool = (first_bytes[0] & 0b01) != 0;
    let val_mod: u8 = first_bytes[1] >> 6;
    let val_reg: u8 = (first_bytes[1] >> 3) & 0b111;
    let val_rm: u8 = first_bytes[1] & 0b111;

    debug!("BYTE 0: 0x{0:02X} 0b{0:08b}", first_bytes[0]);
    debug!("BYTE 1: 0x{0:02X} 0b{0:08b}", first_bytes[1]);
    debug!(
        "D: {}, W: {}, MOD: {:02b}, REG: {:03b}, RM: {:03b}",
        val_d, val_w, val_mod, val_reg, val_rm
    );

    let mut instruction = Instruction::new();
    instruction.add_bytes(first_bytes)?;

    let reg = interpret_register(val_reg, val_w).to_string();

    let (operand, rest) = consume_displacement(&mut instruction, rest, val_mod, val_rm, val_w)?;
    let (src, dst) = if val_d {
        (operand, reg)
    } else {
        (reg, operand)
    };

    let op = decode_op((first_bytes[0] >> 3) & 0b111)?;

    instruction.mnemonic = format!("{} {}, {}", op, dst, src);
    Ok((instruction, rest))
}

pub(super) fn decode_op_immediate_to_register_memory(bytes: &[u8]) -> IntelResult {
    if bytes.len() < 2 {
        return Err(IntelError::IncompleteByteStream);
    }

    let op = decode_op((bytes[1] >> 3) & 0b111)?;
    decode_immediate_to_register_memory(bytes, op)
}

#[named]
pub(super) fn decode_immediate_to_register_memory<'a>(
    bytes: &'a [u8],
    op: &'static str,
) -> IntelResult<'a> {
    debug!(function_name!());
    let mut instruction = Instruction::new();

    let (first_bytes, rest) = consume(bytes, 2)?;
    instruction.add_bytes(first_bytes)?;

    let val_w: bool = (first_bytes[0] & 0b1) != 0;
    let val_s: bool = (first_bytes[0] & 0b10) != 0;
    let val_mod = (first_bytes[1] >> 6) & 0b11;
    let val_rm = first_bytes[1] & 0b111;

    debug!("BYTE 0: 0x{0:02X} 0b{0:08b}", first_bytes[0]);
    debug!("BYTE 1: 0x{0:02X} 0b{0:08b}", first_bytes[1]);
    debug!("W: {}, S: {}, MOD: {:02b}, RM: {:03b}", val_w, val_s, val_mod, val_rm);

    let (dst, rest) = consume_displacement(&mut instruction, rest, val_mod, val_rm, val_w)?;
    let (src, rest) = consume_immediate(&mut instruction, rest, val_w, val_s)?;

    let mut size_specifier: &'static str = "";
    if is_memory_displacement(&dst) {
        size_specifier = if val_w { "word" } else { "byte" };
    }

    instruction.mnemonic = format!("{} {} {}, {}", op, size_specifier, dst, src);
    Ok((instruction, rest))
}

pub(super) fn decode_mov_immediate_to_register(bytes: &[u8]) -> IntelResult {
    let peek = bytes[0];
    let val_w: bool = (peek & 0b1000) != 0;
    let val_reg: u8 = peek & 0b111;
    let register = interpret_register(val_reg, val_w);

    decode_op_immediate_to_register(bytes, "mov", register, val_w)
}

pub(super) fn decode_op_immediate_to_accumulator<'a>(
    bytes: &'a [u8],
    op: &'static str,
) -> IntelResult<'a> {
    let peek = bytes[0];
    let val_w: bool = (peek & 0b1) != 0;
    let register = interpret_accumulator(val_w);

    decode_op_immediate_to_register(bytes, op, register, val_w)
}

pub(super) fn decode_op_immediate_to_register<'a>(
    bytes: &'a [u8],
    op: &'static str,
    register: Register,
    val_w: bool,
) -> IntelResult<'a> {
    let (data, rest) = consume(bytes, 1)?;
    let mut instruction = Instruction::new();
    instruction.add_byte(data[0])?;

    // No sign-extension.
    let (value, rest) = consume_immediate(&mut instruction, rest, val_w, false)?;

    instruction.mnemonic = format!("{} {}, {}", op, register.to_string(), value);

    Ok((instruction, rest))
}

// Depending on this "d" bit, determines whether the accumulator is the destination.
pub(super) fn decode_mov_accumulator_to_from_memory(bytes: &[u8], direction: bool) -> IntelResult {
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

pub(super) fn decode_jump<'a>(bytes: &'a[u8], op: &'static str) -> IntelResult<'a> {
    let (data, rest) = consume(bytes, 2)?;

    let mut instruction = Instruction::new();
    instruction.add_bytes(data)?;

    // We use signed offset.
    let offset = data[1] as i8;
    instruction.mnemonic = format!("{} {}", op, encode_jump_offset(offset));

    Ok((instruction, rest))
}

// HELPERS -----------------------------------------------------------------------------------------

fn consume(bytes: &[u8], amount: usize) -> Result<(&[u8], &[u8]), IntelError> {
    if bytes.len() < amount {
        return Err(IntelError::IncompleteByteStream);
    }

    let consumed = &bytes[..amount];
    let rest = &bytes[amount..];
    Ok((consumed, rest))
}

fn consume_displacement<'i, 'a>(
    instruction: &'i mut Instruction,
    bytes: &'a [u8],
    val_mod: u8,
    val_rm: u8,
    val_w: bool,
) -> Result<(String, &'a [u8]), IntelError> {
    let (displacement, rest) = match val_mod {
        0b00 => {
            if val_rm != 0b110 {
                let operand = format!("[{}]", eac(val_rm));
                (operand, bytes)
            } else {
                // Otherwise it is a DIRECT ACCESS.
                let (data, rest) = consume(bytes, 2)?;
                instruction.add_bytes(data)?;

                let operand = format!("[{}]", to_intel_u16(data));
                (operand, rest)
            }
        }
        0b01 => {
            let (data, rest) = consume(bytes, 1)?;
            instruction.add_byte(data[0])?;

            let operand = format!("[{} + {}]", eac(val_rm), data[0]);
            (operand, rest)
        }
        0b10 => {
            let (data, rest) = consume(bytes, 2)?;
            instruction.add_bytes(data)?;

            let value = to_intel_u16(data);
            let operand = format!("[{} + {}]", eac(val_rm), value);
            (operand, rest)
        }
        0b11 => {
            let operand = interpret_register(val_rm, val_w).to_string();
            (operand, bytes)
        }
        _ => panic!(),
    };

    Ok((displacement, rest))
}

// Quite hacky.
fn is_memory_displacement(displacement: &String) -> bool {
    displacement.starts_with("[")
}

fn consume_immediate<'i, 'a>(
    instruction: &'i mut Instruction,
    bytes: &'a [u8],
    val_w: bool,
    val_s: bool,
) -> Result<(u16, &'a [u8]), IntelError> {
    // Non-wide means just 8 bits.
    if !val_w {
        let (data, rest) = consume(bytes, 1)?;
        instruction.add_byte(data[0])?;
        let value = data[0] as u16;
        return Ok((value, rest));
    }

    // Depending on the s bit, it's whether we need to sign extend one byte,
    // or actually get 2 bytes.
    if val_s {
        let (data, rest) = consume(bytes, 1)?;
        instruction.add_byte(data[0])?;

        // If the higher bit is 1, we need to sign extend.
        let mut value = data[0] as u16;
        if (value & 0b1000_0000) != 0 {
            value = value | (0xFF << 8);
        }

        return Ok((value, rest));
    }

    // Just return the 16 bits.
    let (data, rest) = consume(bytes, 2)?;
    instruction.add_bytes(data)?;
    let value = to_intel_u16(data);

    return Ok((value, rest));
}

fn to_intel_u16(data: &[u8]) -> u16 {
    let b1: u16 = data[0] as u16;
    let b2: u16 = (data[1] as u16) << 8;
    b1 | b2
}

fn eac(val_rm: u8) -> String {
    EAC_REGISTER[val_rm as usize].to_string()
}

fn encode_jump_offset(offset: i8) -> String {
    // The jump encoding has a 2 implicit offset.
    let offset = offset + 2;
    if offset > 0 {
        format!("$+{}+0", offset)
    } else if offset == 0 {
        "$+0".to_string()
    } else { // offset < 0
        format!("${}+0", offset)
    }
}