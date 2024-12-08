pub mod args;
pub mod cpu;
mod decoding;
pub mod error;
pub mod instructions;
pub mod registers;
pub mod tables;

use cpu::*;
use error::IntelError;
use instructions::*;
use log::*;

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
    pub executed_instructions: Vec<Instruction>,
    pub cycles: usize,
}

pub fn simulate(program: &[u8]) -> Result<SimulationResult, IntelError> {
    let mut cpu = CPU::new();

    // We copy the input bytes into the cpu memory.
    cpu.set_program(program)?;

    let mut executed_instructions = vec![];

    let mut cycles: usize = 0;
    loop {
        let address = cpu.ip_address();

        // For now we simulate until the IP is out of the original program bounds.
        if address >= program.len() {
            break;
        }

        debug!("Decoding at {}", printu16(address as u16));

        // Get the current bytestream for the instruction to decode.
        let bytestream = &cpu.get_memory()[address..];
        if bytestream.is_empty() {
            panic!();
        }

        // Decode the instruction.
        let instruction = Instruction::decode(bytestream)?;
        debug!("\n{:?}", instruction);

        // Simulate the instruction into the cpu.
        cycles += cpu.simulate(&instruction)?;

        executed_instructions.push(instruction);
    }

    info!("Total cycles: {}", cycles);

    let result = SimulationResult {
        cpu,
        executed_instructions,
        cycles,
    };

    Ok(result)
}

pub fn to_asm(instructions: &Vec<Instruction>) -> String {
    let instruction_strings: Vec<String> =
        instructions.iter().map(|i| i.to_string() + "\n").collect();
    let string: String = instruction_strings.into_iter().collect();
    format!("bits 16\n\n{}", string)
}
