#[derive(Copy, Clone, Debug)]
pub struct Register {
    pub name: &'static str,
    pub size: u8,
    pub reg: u8,
}

impl Register {
    const fn new(name: &'static str, size: u8, reg: u8) -> Self {
        Register { name, size, reg }
    }

    pub fn len(&self) -> usize {
        self.size as usize
    }

    pub fn interpret(reg: u8, w: bool) -> Self {
        match w {
            false => REGISTERS_BYTE[reg as usize],
            true => REGISTERS_WORD[reg as usize],
        }
    }

    pub fn interpret_accumulator(w: bool) -> Self {
        if w {
            REGISTER_AX
        } else {
            REGISTER_AL
        }
    }
}

pub const REGISTER_AL: Register = Register::new("al", 1, 0b000);
pub const REGISTER_BL: Register = Register::new("bl", 1, 0b001);
pub const REGISTER_CL: Register = Register::new("cl", 1, 0b010);
pub const REGISTER_DL: Register = Register::new("dl", 1, 0b011);
pub const REGISTER_AH: Register = Register::new("ah", 1, 0b100);
pub const REGISTER_BH: Register = Register::new("bh", 1, 0b101);
pub const REGISTER_CH: Register = Register::new("ch", 1, 0b110);
pub const REGISTER_DH: Register = Register::new("dh", 1, 0b111);

#[rustfmt::skip]
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

pub const REGISTER_AX: Register = Register::new("ax", 2, 0b000);
pub const REGISTER_BX: Register = Register::new("bx", 2, 0b001);
pub const REGISTER_CX: Register = Register::new("cx", 2, 0b010);
pub const REGISTER_DX: Register = Register::new("dx", 2, 0b011);
pub const REGISTER_SP: Register = Register::new("sp", 2, 0b100);
pub const REGISTER_BP: Register = Register::new("bp", 2, 0b101);
pub const REGISTER_SI: Register = Register::new("si", 2, 0b110);
pub const REGISTER_DI: Register = Register::new("di", 2, 0b111);

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

// Represents the Effective Address Calculation plus any optional offset.
#[derive(Debug)]
pub enum EAC {
    BxSi(u16),
    BxDi(u16),
    BpSi(u16),
    BpDi(u16),
    Si(u16),
    Di(u16),
    Bp(u16),
    Bx(u16),
    DirectAccess(u16),
}

impl EAC {
    pub fn new(rm: u8, offset: u16) -> Self {
        match rm {
            0b000 => EAC::BxSi(offset),
            0b001 => EAC::BxDi(offset),
            0b010 => EAC::BpSi(offset),
            0b011 => EAC::BpDi(offset),
            0b100 => EAC::Si(offset),
            0b101 => EAC::Di(offset),
            0b110 => {
                if offset == 0 {
                    panic!("RM 0b110 should have an offset. Otherwise should be DirectAccess");
                }
                EAC::Bp(offset)
            }
            0b111 => EAC::Bx(offset),
            _ => panic!("unexpected rm {}", rm),
        }
    }
}

impl std::fmt::Display for EAC {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EAC::BxSi(offset) => write!(f, "[bx + si + {}]", offset),
            EAC::BxDi(offset) => write!(f, "[bx + di + {}]", offset),
            EAC::BpSi(offset) => write!(f, "[bp + si + {}]", offset),
            EAC::BpDi(offset) => write!(f, "[bp + di + {}]", offset),
            EAC::Si(offset) => write!(f, "[si + {}]", offset),
            EAC::Di(offset) => write!(f, "[di + {}]", offset),
            EAC::Bp(offset) => write!(f, "[si + {}]", offset),
            EAC::Bx(offset) => write!(f, "[di + {}]", offset),
            EAC::DirectAccess(value) => write!(f, "[{}]", value),
        }
    }
}

// Operations --------------------------------------------------------------------------------------

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

impl std::fmt::Display for Register {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}
