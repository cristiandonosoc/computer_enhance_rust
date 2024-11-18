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

    let mut instruction = Instruction::new();
    instruction.add_bytes(first_bytes)?;

    instruction.bits.set_d((first_bytes[0] & 0b10) != 0);
    instruction.bits.set_w((first_bytes[0] & 0b01) != 0);
    instruction.bits.set_vmod(first_bytes[1] >> 6);
    instruction.bits.set_reg((first_bytes[1] >> 3) & 0b111);
    instruction.bits.set_rm(first_bytes[1] & 0b111);

    debug!("BYTE 0: 0x{0:02X} 0b{0:08b}", first_bytes[0]);
    debug!("BYTE 1: 0x{0:02X} 0b{0:08b}", first_bytes[1]);
    debug!("{:?}", instruction.bits);

    let reg = Operand::Register(Register::interpret(instruction.bits.reg(), instruction.bits.w()));

    let (operand, rest) = consume_displacement(&mut instruction, rest)?;
    let (src, dst) = if instruction.bits.d() {
        (operand, reg)
    } else {
        (reg, operand)
    };

    let operation = decode_op((first_bytes[0] >> 3) & 0b111)?;

    instruction.operation = operation;
    instruction.src = src;
    instruction.dst = dst;

    Ok((instruction, rest))
}

pub(super) fn decode_op_immediate_to_register_memory(bytes: &[u8]) -> IntelResult {
    if bytes.len() < 2 {
        return Err(IntelError::IncompleteByteStream);
    }

    let operation = decode_op((bytes[1] >> 3) & 0b111)?;
    decode_immediate_to_register_memory(bytes, operation)
}

#[named]
pub(super) fn decode_immediate_to_register_memory(
    bytes: &[u8],
    operation: Operation,
) -> IntelResult {
    debug!(function_name!());

    let (first_bytes, rest) = consume(bytes, 2)?;

    let mut instruction = Instruction::new();
    instruction.add_bytes(first_bytes)?;

    instruction.bits.set_w((first_bytes[0] & 0b1) != 0);
    instruction.bits.set_s((first_bytes[0] & 0b10) != 0);
    instruction.bits.set_vmod((first_bytes[1] >> 6) & 0b11);
    instruction.bits.set_rm(first_bytes[1] & 0b111);

    debug!("BYTE 0: 0x{0:02X} 0b{0:08b}", first_bytes[0]);
    debug!("BYTE 1: 0x{0:02X} 0b{0:08b}", first_bytes[1]);
    debug!("{:?}", instruction.bits);

    let (dst, rest) = consume_displacement(&mut instruction, rest)?;
    let (src, rest) = consume_immediate(&mut instruction, rest)?;

    instruction.operation = operation;
    instruction.src = src;
    instruction.dst = dst;

    Ok((instruction, rest))
}

pub(super) fn decode_mov_immediate_to_register(bytes: &[u8]) -> IntelResult {
    let peek = bytes[0];
    let w: bool = (peek & 0b1000) != 0;
    let reg: u8 = peek & 0b111;
    let register = Register::interpret(reg, w);

    decode_op_immediate_to_register(bytes, Operation::Mov, register, w)
}

pub(super) fn decode_op_immediate_to_accumulator<'a>(
    bytes: &'a [u8],
    operation: Operation,
) -> IntelResult<'a> {
    let peek = bytes[0];
    let w: bool = (peek & 0b1) != 0;
    let register = Register::interpret_accumulator(w);

    decode_op_immediate_to_register(bytes, operation, register, w)
}

pub(super) fn decode_op_immediate_to_register<'a>(
    bytes: &'a [u8],
    operation: Operation,
    register: Register,
    w: bool,
) -> IntelResult<'a> {
    let (data, rest) = consume(bytes, 1)?;

    let mut instruction = Instruction::new();
    instruction.add_byte(data[0])?;

    instruction.bits.set_w(w);
    instruction.bits.set_s(false);
    instruction.bits.set_reg(register.reg);

    // No sign-extension.
    let (src, rest) = consume_immediate(&mut instruction, rest)?;

    instruction.operation = operation;
    instruction.src = src;
    instruction.dst = Operand::Register(register);

    Ok((instruction, rest))
}

// Depending on this "d" bit, determines whether the accumulator is the destination.
pub(super) fn decode_mov_accumulator_to_from_memory(bytes: &[u8], direction: bool) -> IntelResult {
    let (data, rest) = consume(bytes, 3)?;

    let mut instruction = Instruction::new();
    instruction.add_bytes(data)?;

    instruction.bits.set_w((data[0] & 0b1) != 0);
    let accum = Operand::Register(Register::interpret_accumulator(instruction.bits.w()));
    let value = to_intel_u16(&data[1..]);
    let eac = Operand::EAC(EAC::DirectAccess(value));

    let (src, dst) = if direction {
        (eac, accum)
    } else {
        (accum, eac)
    };

    instruction.operation = Operation::Mov;
    instruction.src = src;
    instruction.dst = dst;

    Ok((instruction, rest))
}

pub(super) fn decode_jump<'a>(bytes: &'a [u8], jump_op_name: &'static str) -> IntelResult<'a> {
    let (data, rest) = consume(bytes, 2)?;

    let mut instruction = Instruction::new();
    instruction.add_bytes(data)?;

    // We use signed offset.
    let offset = data[1] as i8;

    instruction.operation = Operation::Jump(jump_op_name);
    instruction.dst = Operand::JumpOffset(offset);

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
) -> Result<(Operand, &'a [u8]), IntelError> {
    let vmod = instruction.bits.vmod();
    let (displacement, rest) = match vmod {
        0b00 => {
            if instruction.bits.rm() != 0b110 {
                let eac = EAC::new(instruction.bits.rm(), vmod, 0);
                let operand = Operand::EAC(eac);
                (operand, bytes)
            } else {
                // Otherwise it is a DIRECT ACCESS.
                let (data, rest) = consume(bytes, 2)?;
                instruction.add_bytes(data)?;

                let offset = to_intel_u16(data);
                let eac = EAC::DirectAccess(offset);
                let operand = Operand::EAC(eac);
                (operand, rest)
            }
        }
        0b01 => {
            let (data, rest) = consume(bytes, 1)?;
            instruction.add_byte(data[0])?;

            let offset = data[0] as u16;
            let eac = EAC::new(instruction.bits.rm(), vmod, offset);
            let operand = Operand::EAC(eac);
            (operand, rest)
        }
        0b10 => {
            let (data, rest) = consume(bytes, 2)?;
            instruction.add_bytes(data)?;

            let offset = to_intel_u16(data);
            let eac = EAC::new(instruction.bits.rm(), vmod, offset);
            let operand = Operand::EAC(eac);
            (operand, rest)
        }
        0b11 => {
            let register = Register::interpret(instruction.bits.rm(), instruction.bits.w());
            let operand = Operand::Register(register);
            (operand, bytes)
        }
        _ => panic!(),
    };

    Ok((displacement, rest))
}

fn consume_immediate<'i, 'a>(
    instruction: &'i mut Instruction,
    bytes: &'a [u8],
) -> Result<(Operand, &'a [u8]), IntelError> {
    // Non-wide means just 8 bits.
    if !instruction.bits.w() {
        let (data, rest) = consume(bytes, 1)?;
        instruction.add_byte(data[0])?;
        let value = data[0] as u16;
        return Ok((Operand::Immediate(value), rest));
    }

    // Depending on the s bit, it's whether we need to sign extend one byte,
    // or actually get 2 bytes.
    if instruction.bits.s() {
        let (data, rest) = consume(bytes, 1)?;
        instruction.add_byte(data[0])?;

        // If the higher bit is 1, we need to sign extend.
        let mut value = data[0] as u16;
        if (value & 0b1000_0000) != 0 {
            value = value | (0xFF << 8);
        }

        return Ok((Operand::Immediate(value), rest));
    }

    // Just return the 16 bits.
    let (data, rest) = consume(bytes, 2)?;
    instruction.add_bytes(data)?;
    let value = to_intel_u16(data);

    return Ok((Operand::Immediate(value), rest));
}

fn to_intel_u16(data: &[u8]) -> u16 {
    let b1: u16 = data[0] as u16;
    let b2: u16 = (data[1] as u16) << 8;
    b1 | b2
}

fn decode_op(op: u8) -> Result<Operation, IntelError> {
    match op {
        0b000 => Ok(Operation::Add),
        0b001 => Ok(Operation::Mov),
        0b101 => Ok(Operation::Sub),
        0b111 => Ok(Operation::Cmp),
        _ => Err(IntelError::UnsupportedOperation(op)),
    }
}
