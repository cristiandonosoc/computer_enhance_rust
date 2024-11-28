use super::error::*;
use super::instructions::*;
use super::registers::*;
use ::function_name::named;
use log::debug;

pub(super) type IntelResult = Result<Instruction, IntelError>;

#[named]
pub(super) fn decode_op_register_memory_to_from_either(bytes: &[u8]) -> IntelResult {
    debug!(function_name!());

    let mut instruction = Instruction::new();
    instruction.consume(bytes, 2)?;

    instruction.bits.set_d((instruction.data[0] & 0b10) != 0);
    instruction.bits.set_w((instruction.data[0] & 0b01) != 0);
    instruction.bits.set_vmod(instruction.data[1] >> 6);
    instruction.bits.set_reg((instruction.data[1] >> 3) & 0b111);
    instruction.bits.set_rm(instruction.data[1] & 0b111);

    debug!("BYTE 0: 0x{0:02X} 0b{0:08b}", instruction.data[0]);
    debug!("BYTE 1: 0x{0:02X} 0b{0:08b}", instruction.data[1]);
    debug!("{:?}", instruction.bits);

    let reg = Operand::Register(Register::interpret(instruction.bits.reg(), instruction.bits.w()));

    let operand = consume_displacement(bytes, &mut instruction)?;
    let (src, dst) = if instruction.bits.d() {
        (operand, reg)
    } else {
        (reg, operand)
    };

    let operation = decode_op((instruction.data[0] >> 3) & 0b111)?;

    instruction.operation = operation;
    instruction.src = src;
    instruction.dst = dst;

    Ok(instruction)
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

    let mut instruction = Instruction::new();
    instruction.consume(bytes, 2)?;

    instruction.bits.set_w((instruction.data[0] & 0b1) != 0);
    instruction.bits.set_s((instruction.data[0] & 0b10) != 0);
    instruction.bits.set_vmod((instruction.data[1] >> 6) & 0b11);
    instruction.bits.set_rm(instruction.data[1] & 0b111);

    debug!("BYTE 0: 0x{0:02X} 0b{0:08b}", instruction.data[0]);
    debug!("BYTE 1: 0x{0:02X} 0b{0:08b}", instruction.data[1]);
    debug!("{:?}", instruction.bits);

    let dst = consume_displacement(bytes, &mut instruction)?;
    let src = consume_immediate(bytes, &mut instruction)?;

    instruction.operation = operation;
    instruction.src = src;
    instruction.dst = dst;

    Ok(instruction)
}

pub(super) fn decode_mov_immediate_to_register(bytes: &[u8]) -> IntelResult {
    let peek = bytes[0];
    let w: bool = (peek & 0b1000) != 0;
    let reg: u8 = peek & 0b111;
    let register = Register::interpret(reg, w);

    decode_op_immediate_to_register(bytes, Operation::Mov, register, w)
}

pub(super) fn decode_op_immediate_to_accumulator(
    bytes: &[u8],
    operation: Operation,
) -> IntelResult {
    let peek = bytes[0];
    let w: bool = (peek & 0b1) != 0;
    let register = Register::interpret_accumulator(w);

    decode_op_immediate_to_register(bytes, operation, register, w)
}

pub(super) fn decode_op_immediate_to_register(
    bytes: &[u8],
    operation: Operation,
    register: Register,
    w: bool,
) -> IntelResult {
    let mut instruction = Instruction::new();
    instruction.consume(bytes, 1)?;

    instruction.bits.set_w(w);
    instruction.bits.set_s(false);
    instruction.bits.set_reg(register.reg);

    // No sign-extension.
    let src = consume_immediate(bytes, &mut instruction)?;

    instruction.operation = operation;
    instruction.src = src;
    instruction.dst = Operand::Register(register);

    Ok(instruction)
}

// Depending on this "d" bit, determines whether the accumulator is the destination.
pub(super) fn decode_mov_accumulator_to_from_memory(bytes: &[u8], direction: bool) -> IntelResult {
    let mut instruction = Instruction::new();
    instruction.consume(bytes, 3)?;

    instruction.bits.set_w((instruction.data[0] & 0b1) != 0);
    let accum = Operand::Register(Register::interpret_accumulator(instruction.bits.w()));
    let value = instruction.lastu16();
    let eac = Operand::EAC(EAC::DirectAccess(value));

    let (src, dst) = if direction {
        (eac, accum)
    } else {
        (accum, eac)
    };

    instruction.operation = Operation::Mov;
    instruction.src = src;
    instruction.dst = dst;

    Ok(instruction)
}

pub(super) fn decode_jump(bytes: &[u8], jump_op_name: &'static str) -> IntelResult {
    let mut instruction = Instruction::new();
    instruction.consume(bytes, 2)?;

    // We use signed offset.
    let offset = instruction.data[1] as i8;

    instruction.operation = Operation::Jump(jump_op_name);
    instruction.dst = Operand::JumpOffset(offset);

    Ok(instruction)
}

// HELPERS -----------------------------------------------------------------------------------------

fn consume_displacement(
    bytes: &[u8],
    instruction: &mut Instruction,
) -> Result<Operand, IntelError> {
    let vmod = instruction.bits.vmod();
    match vmod {
        0b00 => {
            if instruction.bits.rm() != 0b110 {
                let eac = EAC::new(instruction.bits.rm(), vmod, 0);
                let operand = Operand::EAC(eac);
                return Ok(operand);
            } else {
                // Otherwise it is a DIRECT ACCESS.
                instruction.consume(bytes, 2)?;

                let offset = instruction.lastu16();
                let eac = EAC::DirectAccess(offset);
                let operand = Operand::EAC(eac);
                return Ok(operand);
            }
        }
        0b01 => {
            instruction.consume(bytes, 1)?;

            let offset = instruction.lastu8() as u16;
            let eac = EAC::new(instruction.bits.rm(), vmod, offset);
            let operand = Operand::EAC(eac);
            return Ok(operand);
        }
        0b10 => {
            instruction.consume(bytes, 2)?;

            let offset = instruction.lastu16();
            let eac = EAC::new(instruction.bits.rm(), vmod, offset);
            let operand = Operand::EAC(eac);
            return Ok(operand);
        }
        0b11 => {
            let register = Register::interpret(instruction.bits.rm(), instruction.bits.w());
            let operand = Operand::Register(register);
            return Ok(operand);
        }
        _ => {}
    };

    panic!()
}

fn consume_immediate(bytes: &[u8], instruction: &mut Instruction) -> Result<Operand, IntelError> {
    // Non-wide means just 8 bits.
    if !instruction.bits.w() {
        instruction.consume(bytes, 1)?;
        let value = instruction.lastu8() as u16;
        return Ok(Operand::Immediate(value));
    }

    // Depending on the s bit, it's whether we need to sign extend one byte,
    // or actually get 2 bytes.
    if instruction.bits.s() {
        instruction.consume(bytes, 1)?;

        // If the higher bit is 1, we need to sign extend.
        let mut value = instruction.lastu8() as u16;
        if (value & 0b1000_0000) != 0 {
            value = value | (0xFF << 8);
        }

        return Ok(Operand::Immediate(value));
    }

    // Just return the 16 bits.
    instruction.consume(bytes, 2)?;
    let value = instruction.lastu16();

    return Ok(Operand::Immediate(value));
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
