use super::error::*;
use super::instructions::*;
use super::registers::*;
use log::debug;

#[derive(Debug, Default, Eq, PartialEq)]
pub struct CPU {
    pub ax: u16,
    pub cx: u16,
    pub dx: u16,
    pub bx: u16,
    pub sp: u16,
    pub bp: u16,
    pub si: u16,
    pub di: u16,
}

impl CPU {
    pub fn new() -> Self {
        CPU {
            ..Default::default()
        }
    }

    pub fn simulate(&mut self, instruction: &Instruction) -> Result<(), IntelError> {
        if !matches!(instruction.operation, Operation::Mov) {
            return Err(IntelError::UnsupportedSimulationOperation(
                instruction.operation.to_string(),
            ));
        }

        let dst: Register;
        if let Operand::Register(r) = instruction.dst {
            dst = r;
        } else {
            return Err(IntelError::UnsupportedSimulationOperation(
                "Non register move".to_string(),
            ));
        };

        let value = match instruction.src {
            Operand::Register(reg) => {
                // For now we support only big registers.
                if reg.len() < 2 {
                    return Err(IntelError::InvalidOperand(reg.to_string()));
                }

                self.get_register_value16(&reg)
            },
            Operand::Immediate(value) => {
                value
            },
            _ => {
                let value_type = std::any::type_name_of_val(&instruction.src);
                let msg = format!("{}: {}", value_type, instruction.src);
                return Err(IntelError::InvalidOperand(msg));
            }
        };

        let before = self.get_register_value16(&dst);
        let after = self.set_register_value16(&dst, value);

        debug!("{0}:0x{1:04X}->0x{2:04X}", dst, before, after);

        Ok(())
    }

    pub fn get_register_value16(&self, reg: &Register) -> u16 {
        unsafe {
            let base_ptr = self as *const CPU as *const u16;
            let index = reg.reg as usize;
            *base_ptr.add(index)
        }
    }

    pub fn set_register_value16(&mut self, reg: &Register, value: u16) -> u16 {
        let value = unsafe {
            let base_ptr = self as *mut CPU as *mut u16;
            let index = reg.reg as usize;
            *base_ptr.add(index) = value;
            *base_ptr.add(index)
        };
        value
    }
}
