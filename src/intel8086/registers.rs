pub struct Register(pub &'static str);

pub const REGISTER_AL: Register = Register("al");
pub const REGISTER_CL: Register = Register("cl");
pub const REGISTER_DL: Register = Register("dl");
pub const REGISTER_BL: Register = Register("bl");
pub const REGISTER_AH: Register = Register("ah");
pub const REGISTER_CH: Register = Register("ch");
pub const REGISTER_DH: Register = Register("dh");
pub const REGISTER_BH: Register = Register("bh");

pub const REGISTER_AX: Register = Register("ax");
pub const REGISTER_CX: Register = Register("cx");
pub const REGISTER_DX: Register = Register("dx");
pub const REGISTER_BX: Register = Register("bx");
pub const REGISTER_SP: Register = Register("sp");
pub const REGISTER_BP: Register = Register("bp");
pub const REGISTER_SI: Register = Register("si");
pub const REGISTER_DI: Register = Register("di");
