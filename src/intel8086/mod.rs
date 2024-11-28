pub mod cpu;
mod decoding;
pub mod error;
pub mod instructions;
pub mod registers;

use cpu::CPU;
use error::IntelError;
use instructions::*;
use log::debug;

pub fn disassemble(mut bytes: &[u8]) -> Result<Vec<Instruction>, IntelError> {
    let mut instructions = vec![];

    while !bytes.is_empty() {
        debug!("Decoding new instruction");
        let instruction = Instruction::decode(bytes)?;
        bytes = &bytes[instruction.len()..];

        debug!("\n{:?}", instruction);
        instructions.push(instruction);
    }

    Ok(instructions)
}

pub struct SimulationResult {
    pub cpu: CPU,
    pub instructions: Vec<Instruction>,
}

pub fn simulate(mut bytes: &[u8]) -> Result<SimulationResult, IntelError> {
    let mut cpu = CPU::new();
    let mut instructions = vec![];

    while !bytes.is_empty() {
        debug!("Decoding new instruction");
        let instruction = Instruction::decode(bytes)?;
        bytes = &bytes[instruction.len()..];

        debug!("\n{:?}", instruction);
        cpu.simulate(&instruction)?;

        instructions.push(instruction);
    }

    let result = SimulationResult { cpu, instructions };

    Ok(result)
}

pub fn to_asm(instructions: &Vec<Instruction>) -> String {
    let instruction_strings: Vec<String> =
        instructions.iter().map(|i| i.to_string() + "\n").collect();
    let string: String = instruction_strings.into_iter().collect();
    format!("bits 16\n\n{}", string)
}
