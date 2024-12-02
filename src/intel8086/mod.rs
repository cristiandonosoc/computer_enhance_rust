pub mod cpu;
mod decoding;
pub mod error;
pub mod instructions;
pub mod registers;

use cpu::CPU;
use error::IntelError;
use instructions::*;
use log::{debug, info};

pub fn disassemble(mut bytes: &[u8]) -> Result<Vec<Instruction>, IntelError> {
    let mut instructions = vec![];

    while !bytes.is_empty() {
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

pub fn simulate(bytes: &[u8]) -> Result<SimulationResult, IntelError> {
    let mut cpu = CPU::new();
    let mut instructions = vec![];

    let mut stream = bytes;

    while !stream.is_empty() {
        let mut instruction = Instruction::decode(stream)?;
        instruction.address = cpu.ip_address();

        info!("\n{:?}", instruction);
        cpu.simulate(&instruction)?;

        instructions.push(instruction);

        // Determine where we should continue decoding from.
        // If we overflow the stream, then we consider the program done.
        let address = cpu.ip_address();
        if address >= bytes.len() {
            break;
        }
        stream = &bytes[address..];
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
