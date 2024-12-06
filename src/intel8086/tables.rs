use super::instructions::*;
use super::registers::*;

use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Mutex;

// dst, src
pub enum OperandPair {
    RegisterRegister,
    RegisterMemory,
    MemoryRegister,
}

#[derive(Debug)]
pub(super) enum OperandKind {
    Memory,
    Register,
    Accumulator,
    Immediate,
}

#[derive(Debug)]
pub(super) struct InstructionCost {
    pub dst: OperandKind,
    pub src: OperandKind,
    pub base_cost: u8,
    pub transfers: u8,
    pub eac_cost: bool,
}

impl InstructionCost {
    fn new(
        dst: OperandKind,
        src: OperandKind,
        base_cost: u8,
        transfers: u8,
        eac_cost: bool,
    ) -> Self {
        Self {
            dst,
            src,
            base_cost,
            transfers,
            eac_cost,
        }
    }

    pub(super) fn matches(&self, instruction: &Instruction) -> bool {
        if !operand_matches(&instruction.dst, &self.dst) {
            return false;
        }

        if !operand_matches(&instruction.src, &self.src) {
            return false;
        }

        true
    }
}

fn operand_matches(operand: &Operand, operand_kind: &OperandKind) -> bool {
    match operand_kind {
        OperandKind::Memory => matches!(operand, Operand::EAC(_)),
        OperandKind::Register => matches!(operand, Operand::Register(_)),
        OperandKind::Accumulator => {
            if let Operand::Register(register) = operand {
                if *register == REGISTER_AX {
                    return true;
                }
            }
            false
        }
        OperandKind::Immediate => matches!(operand, Operand::Immediate(_)),
    }
}

const MEMORY: OperandKind = OperandKind::Memory;
const REGISTER: OperandKind = OperandKind::Register;
const ACCUMULATOR: OperandKind = OperandKind::Accumulator;
const IMMEDIATE: OperandKind = OperandKind::Immediate;

pub(super) static COST_MAP: Lazy<Mutex<HashMap<Operation, Vec<InstructionCost>>>> =
    Lazy::new(|| {
        let mut map = HashMap::new();
        // MOV.
        map.insert(
            Operation::Mov {},
            vec![
                InstructionCost::new(MEMORY, ACCUMULATOR, 10, 1, false),
                InstructionCost::new(ACCUMULATOR, MEMORY, 10, 1, false),
                InstructionCost::new(REGISTER, REGISTER, 2, 0, false),
                InstructionCost::new(REGISTER, MEMORY, 8, 1, true),
                InstructionCost::new(MEMORY, REGISTER, 9, 1, true),
                InstructionCost::new(REGISTER, IMMEDIATE, 4, 0, false),
                InstructionCost::new(MEMORY, IMMEDIATE, 10, 1, false),
            ],
        );

        // ADD.
        map.insert(
            Operation::Add {},
            vec![
                InstructionCost::new(REGISTER, REGISTER, 3, 0, false),
                InstructionCost::new(REGISTER, MEMORY, 9, 1, true),
                InstructionCost::new(MEMORY, REGISTER, 16, 2, true),
                InstructionCost::new(REGISTER, IMMEDIATE, 4, 0, false),
                InstructionCost::new(MEMORY, IMMEDIATE, 17, 2, true),
                InstructionCost::new(ACCUMULATOR, IMMEDIATE, 4, 0, false),
            ],
        );

        // SUB.
        map.insert(
            Operation::Sub {},
            vec![
                InstructionCost::new(REGISTER, REGISTER, 3, 0, false),
                InstructionCost::new(REGISTER, MEMORY, 9, 1, true),
                InstructionCost::new(MEMORY, REGISTER, 16, 2, true),
                InstructionCost::new(REGISTER, IMMEDIATE, 4, 0, false),
                InstructionCost::new(MEMORY, IMMEDIATE, 17, 2, true),
                InstructionCost::new(ACCUMULATOR, IMMEDIATE, 4, 0, false),
            ],
        );

        // CMP.
        map.insert(
            Operation::Cmp {},
            vec![
                InstructionCost::new(REGISTER, REGISTER, 3, 0, false),
                InstructionCost::new(REGISTER, MEMORY, 9, 1, true),
                InstructionCost::new(MEMORY, REGISTER, 9, 1, true),
                InstructionCost::new(REGISTER, IMMEDIATE, 4, 0, false),
                InstructionCost::new(MEMORY, IMMEDIATE, 10, 1, true),
                InstructionCost::new(ACCUMULATOR, IMMEDIATE, 4, 0, false),
            ],
        );

        Mutex::new(map)
    });

#[rustfmt::skip]
pub(super) fn resolve_eac_cost(eac: &EAC) -> usize {
    match eac {
        EAC::BxSi(offset) => if *offset == 0 { 7 } else { 11 },
        EAC::BxDi(offset) => if *offset == 0 { 8 } else { 12 },
        EAC::BpSi(offset) => if *offset == 0 { 8 } else { 12 },
        EAC::BpDi(offset) => if *offset == 0 { 7 } else { 11 },
        EAC::Si(offset) => if *offset == 0 { 5 } else { 9 },
        EAC::Di(offset) => if *offset == 0 { 5 } else { 9 },
        EAC::Bp(offset) => if *offset == 0 { 5 } else { 9 },
        EAC::Bx(offset) => if *offset == 0 { 5 } else { 9 },
        EAC::DirectAccess(_) => 6,
    }
}
