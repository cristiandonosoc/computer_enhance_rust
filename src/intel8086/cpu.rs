use super::error::*;
use super::instructions::*;
use super::registers::*;
use log::{debug, info};

#[derive(Debug, Default, Eq, PartialEq)]
pub struct CPU {
    registers: [u16; 9],
    pub flags: CPUFlags,
}

#[derive(Debug, Default, Eq, PartialEq)]
pub struct CPUFlags {
    pub z: bool,
    pub s: bool,
}

impl CPU {
    pub fn new() -> Self {
        CPU{
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

    pub fn simulate(&mut self, instruction: &Instruction) -> Result<(), IntelError> {
        // Update the IP immediatelly.
        self.set_ip(self.ip() + instruction.len as u16);

        match &instruction.operation {
            Operation::Mov => {
                return self.simulate_register_op(instruction);
            }
            Operation::Add => {
                return self.simulate_register_op(instruction);
            }
            Operation::Sub => {
                return self.simulate_register_op(instruction);
            }
            Operation::Cmp => {
                return self.simulate_register_op(instruction);
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

    fn simulate_register_op(&mut self, instruction: &Instruction) -> Result<(), IntelError> {
        let src = match instruction.src {
            Operand::Register(reg) => {
                // For now we support only big registers.
                if reg.len() < 2 {
                    return Err(IntelError::InvalidOperand(reg.to_string()));
                }

                self.get_register(&reg)
            }
            Operand::Immediate(value) => value,
            Operand::JumpOffset(offset) => {
                // For jump we calculate where the ip will be.
                let ip = (self.ip() as i32) - (offset as i32);
                ip as u16
            }
            _ => {
                let value_type = std::any::type_name_of_val(&instruction.src);
                let msg = format!("{}: {}", value_type, instruction.src);
                return Err(IntelError::InvalidOperand(msg));
            }
        };

        let dst = get_dst_register(instruction)?;
        let before = self.get_register(&dst);

        match &instruction.operation {
            Operation::Mov => {
                self.set_register(&dst, src);
            }
            Operation::Add => {
                let result: u16 = before + src;
                self.set_register(&dst, result);
                self.process_flags(result as i32);
            }
            Operation::Sub => {
                let result: i32 = (before as i32) - (src as i32);
                self.set_register(&dst, result as u16);
                self.process_flags(result);
            }
            Operation::Cmp => {
                let result: u16 = before - src;
                self.process_flags(result as i32);
            }
            _ => {
                return Err(IntelError::UnsupportedSimulationOperation(
                    instruction.operation.to_string(),
                ));
            }
        }

        let after = self.get_register(&dst);

        info!(
            "{0} {1}:0x{2:04X}->0x{3:04X} ({2} -> {3}) - flags: {4}",
            instruction.operation,
            dst,
            before,
            after,
            self.print_flags()
        );

        Ok(())
    }

    fn simulate_jump(
        &mut self,
        instruction: &Instruction,
        jump_description: &JumpDescription,
    ) -> Result<(), IntelError> {
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

        info!("{0} ip: 0x{1:04X}->0x{2:04X} ({1} -> {2})", instruction.operation, before, after);

        Ok(())
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
}

fn get_dst_register(instruction: &Instruction) -> Result<Register, IntelError> {
    if let Operand::Register(r) = instruction.dst {
        return Ok(r);
    } else {
        return Err(IntelError::UnsupportedSimulationOperation(
            "Non register operation".to_string(),
        ));
    }
}
