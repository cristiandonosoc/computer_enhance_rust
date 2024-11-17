use super::error::*;

#[derive(Copy, Clone, Debug)]
pub struct Register(pub &'static str);

impl std::fmt::Display for Register {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub fn interpret_register(index: u8, w: bool) -> Register {
    match w {
        false => REGISTERS_BYTE[index as usize],
        true => REGISTERS_WORD[index as usize],
    }
}

pub fn interpret_accumulator(w: bool) -> Register {
    if w {
        REGISTER_AX
    } else {
        REGISTER_AL
    }
}

pub const REGISTER_AL: Register = Register("al");
pub const REGISTER_CL: Register = Register("cl");
pub const REGISTER_DL: Register = Register("dl");
pub const REGISTER_BL: Register = Register("bl");
pub const REGISTER_AH: Register = Register("ah");
pub const REGISTER_CH: Register = Register("ch");
pub const REGISTER_DH: Register = Register("dh");
pub const REGISTER_BH: Register = Register("bh");

pub(super) const REGISTERS_BYTE: [Register; 8] = [
    REGISTER_AL,
    REGISTER_CL,
    REGISTER_DL,
    REGISTER_BL,
    REGISTER_AH,
    REGISTER_CH,
    REGISTER_DH,
    REGISTER_BH,
];

pub const REGISTER_AX: Register = Register("ax");
pub const REGISTER_CX: Register = Register("cx");
pub const REGISTER_DX: Register = Register("dx");
pub const REGISTER_BX: Register = Register("bx");
pub const REGISTER_SP: Register = Register("sp");
pub const REGISTER_BP: Register = Register("bp");
pub const REGISTER_SI: Register = Register("si");
pub const REGISTER_DI: Register = Register("di");

pub(super) const REGISTERS_WORD: [Register; 8] = [
    REGISTER_AX,
    REGISTER_CX,
    REGISTER_DX,
    REGISTER_BX,
    REGISTER_SP,
    REGISTER_BP,
    REGISTER_SI,
    REGISTER_DI,
];

#[rustfmt::skip]
pub(super) const EAC_REGISTER: [&str; 8] = [
    "bx + si",
    "bx + di",
    "bp + si",
    "bp + di",
    "si",
    "di",
    "bp",
    "bx",
];

pub(super) fn decode_op(op: u8) -> Result<&'static str, IntelError> {
    OP_MAPPING[op as usize].ok_or(IntelError::UnsupportedOperation(op))
}

pub(super) const OP_MAPPING: [Option<&'static str>; 8] = [
    Some("add"),
    Some("mov"),
    None,
    None,
    None,
    Some("sub"),
    None,
    Some("cmp"),
];

pub(super) const SHORT_JUMPS: &[(u8, &str)] = &[
    (0b0111_0000, "jo"),   // Overflow
    (0b0111_0001, "jno"),  // Not Overflow
    (0b0111_0010, "jb"),   // Below/Not Above or Equal/Carry
    (0b0111_0011, "jnb"),  // Not Below/Above or Equal/Not Carry
    (0b0111_0100, "je"),   // Equal/Zero
    (0b0111_0101, "jne"),  // Not Equal/Not Zero
    (0b0111_0110, "jbe"),  // Below or Equal/Not Above
    (0b0111_0111, "jnbe"), // Not Below or Equal/Above
    (0b0111_1000, "js"),   // Sign
    (0b0111_1001, "jns"),  // Not Sign
    (0b0111_1010, "jp"),   // Parity/Parity Even
    (0b0111_1011, "jnp"),  // No Parity/Parity Odd
    (0b0111_1100, "jl"),   // Less/Not Greater or Equal
    (0b0111_1101, "jnl"),  // Not Less/Greater or Equal
    (0b0111_1110, "jle"),  // Less or Equal/Not Greater
    (0b0111_1111, "jnle"), // Not Less or Equal/Greater
    (0b1110_0011, "jcxz"), // Jump if CX Zero
    (0b1110_1011, "jmp"),  // Short Jump
];

pub(super) const LOOP_JUMPS: &[(u8, &str)] = &[
    (0b1110_0000, "loopnz"), // Loop while not zero
    (0b1110_0001, "loopz"),  // Loop while zero
    (0b1110_0010, "loop"),   // Loop unconditional
];
