use super::error::*;
use super::instructions::*;
use super::registers::*;
use super::tables::*;
use log::*;

#[derive(Default, Eq)]
pub struct CPU {
    registers: [u16; 9],
    memory: Vec<u8>,
    pub flags: CPUFlags,
}

#[derive(Debug, Default, Eq, PartialEq)]
pub struct CPUFlags {
    pub z: bool,
    pub s: bool,
}

impl CPU {
    pub fn new() -> Self {
        CPU {
            memory: vec![0; 1024 * 1024],
            ..Default::default()
        }
    }

    pub fn ax(&self) -> u16 {
        self.registers[0]
    }

    pub fn cx(&self) -> u16 {
        self.registers[1]
    }

    pub fn dx(&self) -> u16 {
        self.registers[2]
    }

    pub fn bx(&self) -> u16 {
        self.registers[3]
    }

    pub fn sp(&self) -> u16 {
        self.registers[4]
    }

    pub fn bp(&self) -> u16 {
        self.registers[5]
    }

    pub fn si(&self) -> u16 {
        self.registers[6]
    }

    pub fn di(&self) -> u16 {
        self.registers[7]
    }

    pub fn ip(&self) -> u16 {
        self.registers[8]
    }

    pub fn ip_address(&self) -> usize {
        self.ip() as usize
    }

    fn set_ip(&mut self, ip: u16) {
        self.registers[8] = ip
    }

    pub fn get_register(&self, reg: &Register) -> u16 {
        self.registers[reg.reg as usize]
    }

    pub fn set_register(&mut self, reg: &Register, value: u16) {
        self.registers[reg.reg as usize] = value
    }

    pub fn set_program(&mut self, program: &[u8]) -> Result<(), IntelError> {
        if program.len() >= self.memory.len() {
            return Err(IntelError::ProgramTooBig(program.len(), self.memory.len()));
        }

        self.memory[..program.len()].copy_from_slice(program);
        Ok(())
    }

    pub fn get_memory(&self) -> &[u8] {
        &self.memory
    }

    pub fn simulate(&mut self, instruction: &Instruction) -> Result<usize, IntelError> {
        // Update the IP immediatelly.
        self.set_ip(self.ip() + instruction.len as u16);

        match &instruction.operation {
            Operation::Mov => {
                return self.simulate_mov(instruction);
            }
            Operation::Add => {
                return self.simulate_op(instruction);
            }
            Operation::Sub => {
                return self.simulate_op(instruction);
            }
            Operation::Cmp => {
                return self.simulate_op(instruction);
            }
            Operation::Jump(jump_description) => {
                return self.simulate_jump(instruction, &jump_description);
            }
            _ => {
                return Err(IntelError::UnsupportedSimulationOperation(
                    instruction.operation.to_string(),
                ));
            }
        }
    }

    fn simulate_mov(&mut self, instruction: &Instruction) -> Result<usize, IntelError> {
        // We simulate the cycles before changing anything.
        let src = match &instruction.src {
            Operand::Register(reg) => {
                // For now we support only big registers.
                if reg.len() < 2 {
                    return Err(IntelError::InvalidOperand(reg.to_string()));
                }

                self.get_register(&reg)
            }
            Operand::Immediate(value) => *value,
            Operand::EAC(eac) => {
                let (_, value) = self.resolve_eac(&eac);
                value
            }
            _ => {
                let value_type = std::any::type_name_of_val(&instruction.src);
                let msg = format!("{}: {}", value_type, instruction.src);
                return Err(IntelError::InvalidOperand(msg));
            }
        };

        let (before, dst_str, after) = match &instruction.dst {
            Operand::Register(reg) => {
                // For now we support only big registers.
                if reg.len() < 2 {
                    return Err(IntelError::InvalidOperand(reg.to_string()));
                }

                let before = self.get_register(&reg);
                self.set_register(&reg, src);
                (before, reg.name.to_string(), src)
            }
            Operand::EAC(eac) => {
                let (address, before) = self.resolve_eac(&eac);

                self.storeu16(address as usize, src);
                let dst_str = format!("address: {}", printu16(address));
                (before, dst_str, src)
            }
            _ => {
                let value_type = std::any::type_name_of_val(&instruction.src);
                let msg = format!("{}: {}", value_type, instruction.src);
                return Err(IntelError::InvalidOperand(msg));
            }
        };

        let mut cycles = 0xFFFFFFF;
        let mut cycles_explanation: String = "".to_string();
        if let Ok((c, e)) = self.determine_instruction_cycle_cost(instruction) {
            cycles = c;
            cycles_explanation = e;
        }

        info!(
            "\"{0}\" dst: {1}, {2} -> {3} (cycles: {4}, ({5}))",
            right_pad(instruction, ' ', PAD_AMOUNT),
            dst_str,
            printu16(before),
            printu16(after),
            cycles,
            cycles_explanation,
        );

        Ok(cycles)
    }

    fn simulate_op(&mut self, instruction: &Instruction) -> Result<usize, IntelError> {
        let src = match &instruction.src {
            Operand::Register(reg) => {
                // For now we support only big registers.
                if reg.len() < 2 {
                    return Err(IntelError::InvalidOperand(reg.to_string()));
                }

                self.get_register(&reg)
            }
            Operand::Immediate(value) => *value,
            Operand::EAC(eac) => {
                let (_, value) = self.resolve_eac(&eac);
                value
            }
            _ => {
                let value_type = std::any::type_name_of_val(&instruction.src);
                let msg = format!("{}: {}", value_type, instruction.src);
                return Err(IntelError::InvalidOperand(msg));
            }
        };

        let (before, dst_str, after) = match &instruction.dst {
            Operand::Register(dst) => {
                let before = self.get_register(&dst);

                let result = match &instruction.operation {
                    Operation::Add => {
                        let result: u16 = before + src;
                        self.set_register(&dst, result);
                        self.process_flags(result as i32);
                        result
                    }
                    Operation::Sub => {
                        let result: i32 = (before as i32) - (src as i32);
                        self.set_register(&dst, result as u16);
                        self.process_flags(result);
                        result as u16
                    }
                    Operation::Cmp => {
                        let result: i32 = (before as i32) - (src as i32);
                        self.process_flags(result as i32);
                        0
                    }
                    _ => {
                        return Err(IntelError::UnsupportedSimulationOperation(
                            instruction.operation.to_string(),
                        ));
                    }
                };

                (before, dst.to_string(), result)
            }
            Operand::EAC(eac) => {
                let (address, before) = self.resolve_eac(&eac);
                let dst_str = format!("address: {}", printu16(address));

                let result = match &instruction.operation {
                    Operation::Add => {
                        let result: u16 = before + src;
                        self.storeu16(address as usize, result);
                        self.process_flags(result as i32);
                        result
                    }
                    Operation::Sub => {
                        let result: i32 = (before as i32) - (src as i32);
                        self.storeu16(address as usize, result as u16);
                        self.process_flags(result);
                        result as u16
                    }
                    Operation::Cmp => {
                        let result: i32 = (before as i32) - (src as i32);
                        self.process_flags(result as i32);
                        0
                    }
                    _ => {
                        return Err(IntelError::UnsupportedSimulationOperation(
                            instruction.operation.to_string(),
                        ));
                    }
                };

                (before, dst_str, result)
            }
            _ => {
                let value_type = std::any::type_name_of_val(&instruction.src);
                let msg = format!("{}: {}", value_type, instruction.src);
                return Err(IntelError::InvalidOperand(msg));
            }
        };

        let mut cycles = 0xFFFFFFF;
        let mut cycles_explanation: String = "".to_string();
        if let Ok((c, e)) = self.determine_instruction_cycle_cost(instruction) {
            cycles = c;
            cycles_explanation = e;
        }

        info!(
            "\"{0}\" {1}:0x{2:04X}->0x{3:04X} ({2} -> {3}) - flags: {4} (cycles: {5} ({6}))",
            right_pad(instruction, ' ', PAD_AMOUNT),
            dst_str,
            before,
            after,
            self.print_flags(),
            cycles,
            cycles_explanation,
        );

        Ok(cycles)
    }

    fn simulate_jump(
        &mut self,
        instruction: &Instruction,
        jump_description: &JumpDescription,
    ) -> Result<usize, IntelError> {
        let before = self.ip();

        let src: u16;
        if let Operand::JumpOffset(offset) = instruction.src {
            let ip = (self.ip() as i32) + (offset as i32);
            src = ip as u16;
        } else {
            return Err(IntelError::UnsupportedSimulationOperation(
                "No JumpOffset src operand for jump instruction".to_string(),
            ));
        }

        match jump_description.jump {
            Jump::JO => todo!(),
            Jump::JNO => todo!(),
            Jump::JB => todo!(),
            Jump::JNB => todo!(),
            Jump::JE => {
                if self.flags.z {
                    self.set_ip(src)
                }
            }
            Jump::JNE => {
                if !self.flags.z {
                    self.set_ip(src)
                }
            }
            Jump::JBE => todo!(),
            Jump::JNBE => todo!(),
            Jump::JS => {
                if self.flags.s {
                    self.set_ip(src)
                }
            }
            Jump::JNS => {
                if !self.flags.s {
                    self.set_ip(src)
                }
            }
            Jump::JP => todo!(),
            Jump::JNP => todo!(),
            Jump::JL => todo!(),
            Jump::JNL => todo!(),
            Jump::JLE => todo!(),
            Jump::JNLE => todo!(),
            Jump::JCXZ => todo!(),
            Jump::JMP => todo!(),
            Jump::LOOPNZ => {
                let cx = self.cx() - 1;
                self.set_register(&REGISTER_CX, cx);
                if cx == 0 && !self.flags.z {
                    self.set_ip(src)
                }
            }
            Jump::LOOPZ => {
                let cx = self.cx() - 1;
                self.set_register(&REGISTER_CX, cx);
                if cx == 0 && self.flags.z {
                    self.set_ip(src)
                }
            }
            Jump::LOOP => {
                let cx = self.cx() - 1;
                self.set_register(&REGISTER_CX, cx);
                if cx == 0 && self.flags.z {
                    self.set_ip(src)
                }
            }
        }

        let after = self.ip();

        let mut cycles = 0xFFFFFFF;
        let mut cycles_explanation: String = "".to_string();
        if let Ok((c, e)) = self.determine_instruction_cycle_cost(instruction) {
            cycles = c;
            cycles_explanation = e;
        }

        info!(
            "\"{0}\" ip: 0x{1:04X}->0x{2:04X} ({1} -> {2}) (cycles: {3} ({4}))",
            right_pad(instruction, ' ', PAD_AMOUNT),
            before,
            after,
            cycles,
            cycles_explanation,
        );

        Ok(cycles)
    }

    fn process_flags(&mut self, value: i32) {
        self.flags.z = value == 0;
        self.flags.s = value < 0;
    }

    fn print_flags(&self) -> String {
        let mut result = String::new();
        if self.flags.z {
            result.push('Z')
        }
        if self.flags.s {
            result.push('S')
        }
        result
    }

    // Returns the resolved address and value.
    // In the case of DirectAccess, the address is 0.
    fn resolve_eac(&self, eac: &EAC) -> (u16, u16) {
        let address = match eac {
            EAC::BxSi(offset) => self.bx() + self.si() + offset,
            EAC::BxDi(offset) => self.bx() + self.di() + offset,
            EAC::BpSi(offset) => self.bp() + self.si() + offset,
            EAC::BpDi(offset) => self.bp() + self.di() + offset,
            EAC::Si(offset) => self.si() + offset,
            EAC::Di(offset) => self.di() + offset,
            EAC::Bp(offset) => self.bp() + offset,
            EAC::Bx(offset) => self.bx() + offset,
            EAC::DirectAccess(address) => *address,
        };

        (address, self.loadu16(address as usize))
    }

    fn determine_instruction_cycle_cost(
        &self,
        instruction: &Instruction,
    ) -> Result<(usize, String), IntelError> {
        let map = COST_MAP.lock().unwrap();
        let instruction_costs = map
            .get(&instruction.operation)
            .ok_or(IntelError::UnsupportedSimulationOperation(instruction.operation.to_string()))?;

        let mut total_cost: usize = 0;

        let mut explanation: String = "".to_string();

        for instruction_cost in instruction_costs {
            if !instruction_cost.matches(instruction) {
                continue;
            }

            total_cost += instruction_cost.base_cost as usize;
            explanation.push_str(&format!("BASE({})", instruction_cost.base_cost));

            if instruction_cost.transfers > 0 {
                // Get the EAC.
                let eac: &EAC;
                if let Operand::EAC(e) = &instruction.dst {
                    eac = e;
                } else if let Operand::EAC(e) = &instruction.src {
                    eac = e;
                } else {
                    return Err(IntelError::InvalidInstruction(instruction.clone()));
                }

                // Check if the address is odd.
                // If so, we have to add 4 cycles per transfer.
                let (address, _) = self.resolve_eac(eac);
                if is_odd(address) {
                    let transfer_cost = 4 * instruction_cost.transfers as usize;
                    total_cost += transfer_cost;
                    explanation.push_str(&format!(" + TRANSFER({})", transfer_cost));
                }

                // Check if this instruction has a cost associated with EAC.
                if instruction_cost.eac_cost {
                    let eac_cost = resolve_eac_cost(eac);
                    total_cost += eac_cost;
                    explanation.push_str(&format!(" + EA({})", eac_cost));
                }
            }
        }

        Ok((total_cost, explanation))
    }

    fn loadu16(&self, address: usize) -> u16 {
        let b1 = self.memory[address] as u16;
        let b2 = (self.memory[address + 1] as u16) << 8;
        b1 | b2
    }

    fn storeu16(&mut self, address: usize, value: u16) {
        let b1: u8 = value as u8;
        let b2: u8 = (value >> 8) as u8;
        self.memory[address] = b1;
        self.memory[address + 1] = b2;
    }
}

impl PartialEq for CPU {
    fn eq(&self, other: &Self) -> bool {
        if self.registers != other.registers {
            return false;
        }

        if self.flags != other.flags {
            return false;
        }

        // We don't compare memory.
        return true;
    }
}

impl std::fmt::Debug for CPU {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CPU")
            .field("ax", &printu16(self.ax()))
            .field("cx", &printu16(self.cx()))
            .field("dx", &printu16(self.dx()))
            .field("bx", &printu16(self.bx()))
            .field("sp", &printu16(self.sp()))
            .field("bp", &printu16(self.bp()))
            .field("si", &printu16(self.si()))
            .field("di", &printu16(self.di()))
            .field("ip", &printu16(self.ip()))
            .field("flags", &self.print_flags())
            .finish()
    }
}

pub fn printu16(value: u16) -> String {
    format!("0x{0:04X} ({0})", value)
}

fn right_pad(value: impl std::fmt::Display, pad: char, width: usize) -> String {
    value
        .to_string()
        .chars()
        .chain(std::iter::repeat(pad))
        .take(width)
        .collect::<String>()
}

fn is_odd(value: u16) -> bool {
    value & 1 == 1
}

const PAD_AMOUNT: usize = 30;
