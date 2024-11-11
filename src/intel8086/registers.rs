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
