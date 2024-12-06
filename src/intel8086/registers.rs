#[derive(Copy, Clone, Debug, PartialEq, Eq)]
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

    pub fn find(name: &str) -> Option<Register> {
        for reg in REGISTERS_WORD {
            if reg.name == name {
                return Some(reg);
            }
        }

        for reg in REGISTERS_BYTE {
            if reg.name == name {
                return Some(reg);
            }
        }

        for reg in EXTRA_REGISTERS {
            if reg.name == name {
                return Some(reg);
            }
        }

        None
    }
}

pub const REGISTER_AL: Register = Register::new("al", 1, 0);
pub const REGISTER_BL: Register = Register::new("bl", 1, 1);
pub const REGISTER_CL: Register = Register::new("cl", 1, 2);
pub const REGISTER_DL: Register = Register::new("dl", 1, 3);
pub const REGISTER_AH: Register = Register::new("ah", 1, 4);
pub const REGISTER_BH: Register = Register::new("bh", 1, 5);
pub const REGISTER_CH: Register = Register::new("ch", 1, 6);
pub const REGISTER_DH: Register = Register::new("dh", 1, 7);

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

pub const REGISTER_AX: Register = Register::new("ax", 2, 0);
pub const REGISTER_CX: Register = Register::new("cx", 2, 1);
pub const REGISTER_DX: Register = Register::new("dx", 2, 2);
pub const REGISTER_BX: Register = Register::new("bx", 2, 3);
pub const REGISTER_SP: Register = Register::new("sp", 2, 4);
pub const REGISTER_BP: Register = Register::new("bp", 2, 5);
pub const REGISTER_SI: Register = Register::new("si", 2, 6);
pub const REGISTER_DI: Register = Register::new("di", 2, 7);

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

pub const REGISTER_IP: Register = Register::new("ip", 2, 8);

pub(super) const EXTRA_REGISTERS: [Register; 1] = [REGISTER_IP];

// Represents the Effective Address Calculation plus any optional offset.
#[derive(Debug, Clone)]
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
    pub fn new(rm: u8, vmod: u8, offset: u16) -> Self {
        match rm {
            0b000 => EAC::BxSi(offset),
            0b001 => EAC::BxDi(offset),
            0b010 => EAC::BpSi(offset),
            0b011 => EAC::BpDi(offset),
            0b100 => EAC::Si(offset),
            0b101 => EAC::Di(offset),
            0b110 => {
                if offset == 0 && vmod == 0 {
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
            EAC::Bp(offset) => write!(f, "[bp + {}]", offset),
            EAC::Bx(offset) => write!(f, "[bx + {}]", offset),
            EAC::DirectAccess(value) => write!(f, "[{}]", value),
        }
    }
}

// Operations --------------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Jump {
    JO,
    JNO,
    JB,
    JNB,
    JE,
    JNE,
    JBE,
    JNBE,
    JS,
    JNS,
    JP,
    JNP,
    JL,
    JNL,
    JLE,
    JNLE,
    JCXZ,
    JMP,

    LOOPNZ,
    LOOPZ,
    LOOP,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct JumpDescription {
    pub opcode: u8,
    pub jump: Jump,
    pub name: &'static str,
}

impl JumpDescription {
    const fn new(opcode: u8, jump: Jump, name: &'static str) -> Self {
        JumpDescription { opcode, jump, name }
    }
}

pub const JUMP_JO: JumpDescription = JumpDescription::new(0b0111_0000, Jump::JO, "jo"); // Overflow
pub const JUMP_JNO: JumpDescription = JumpDescription::new(0b0111_0001, Jump::JNO, "jno"); // Not Overflow
pub const JUMP_JB: JumpDescription = JumpDescription::new(0b0111_0010, Jump::JB, "jb"); // Below/Not Above or Equal/Carry
pub const JUMP_JNB: JumpDescription = JumpDescription::new(0b0111_0011, Jump::JNB, "jnb"); // Not Below/Above or Equal/Not Carry
pub const JUMP_JE: JumpDescription = JumpDescription::new(0b0111_0100, Jump::JE, "je"); // Equal/Zero
pub const JUMP_JNE: JumpDescription = JumpDescription::new(0b0111_0101, Jump::JNE, "jne"); // Not Equal/Not Zero
pub const JUMP_JBE: JumpDescription = JumpDescription::new(0b0111_0110, Jump::JBE, "jbe"); // Below or Equal/Not Above
pub const JUMP_JNBE: JumpDescription = JumpDescription::new(0b0111_0111, Jump::JNBE, "jnbe"); // Not Below or Equal/Above
pub const JUMP_JS: JumpDescription = JumpDescription::new(0b0111_1000, Jump::JS, "js"); // Sign
pub const JUMP_JNS: JumpDescription = JumpDescription::new(0b0111_1001, Jump::JNS, "jns"); // Not Sign
pub const JUMP_JP: JumpDescription = JumpDescription::new(0b0111_1010, Jump::JP, "jp"); // Parity/Parity Even
pub const JUMP_JNP: JumpDescription = JumpDescription::new(0b0111_1011, Jump::JNP, "jnp"); // No Parity/Parity Odd
pub const JUMP_JL: JumpDescription = JumpDescription::new(0b0111_1100, Jump::JL, "jl"); // Less/Not Greater or Equal
pub const JUMP_JNL: JumpDescription = JumpDescription::new(0b0111_1101, Jump::JNL, "jnl"); // Not Less/Greater or Equal
pub const JUMP_JLE: JumpDescription = JumpDescription::new(0b0111_1110, Jump::JLE, "jle"); // Less or Equal/Not Greater
pub const JUMP_JNLE: JumpDescription = JumpDescription::new(0b0111_1111, Jump::JNLE, "jnle"); // Not Less or Equal/Greater
pub const JUMP_JCXZ: JumpDescription = JumpDescription::new(0b1110_0011, Jump::JCXZ, "jcxz"); // Jump if CX Zero
pub const JUMP_JMP: JumpDescription = JumpDescription::new(0b1110_1011, Jump::JMP, "jmp"); // Short Jump

#[rustfmt::skip]
pub(super) const SHORT_JUMPS: &[JumpDescription] = &[
    JUMP_JO,
    JUMP_JNO,
    JUMP_JB,
    JUMP_JNB,
    JUMP_JE,
    JUMP_JNE,
    JUMP_JBE,
    JUMP_JNBE,
    JUMP_JS,
    JUMP_JNS,
    JUMP_JP,
    JUMP_JNP,
    JUMP_JL,
    JUMP_JNL,
    JUMP_JLE,
    JUMP_JNLE,
    JUMP_JCXZ,
    JUMP_JMP,
];

pub const JUMP_LOOPNZ: JumpDescription = JumpDescription::new(0b1110_0000, Jump::LOOPNZ, "loopnz"); // Loop while not zero
pub const JUMP_LOOPZ: JumpDescription = JumpDescription::new(0b1110_0001, Jump::LOOPZ, "loopz"); // Loop while zero
pub const JUMP_LOOP: JumpDescription = JumpDescription::new(0b1110_0010, Jump::LOOP, "loop"); // Loop while zero

#[rustfmt::skip]
pub(super) const LOOP_JUMPS: &[JumpDescription] = &[
    JUMP_LOOPNZ,
    JUMP_LOOPZ,
    JUMP_LOOP,
];

impl std::fmt::Display for Register {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}
