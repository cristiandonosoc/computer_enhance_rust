use super::error::*;

use super::*;
use computer_enhance_rust::intel8086::cpu::*;
use computer_enhance_rust::intel8086::registers::*;
use log::debug;

pub fn run_simulation_test(listing_name: &str) -> Result<(), TestError> {
    // Create a temporary dir for this test.
    let temp_dir = TempDir::new().map_err(|e| TestError::io(String::from("TempDir"), e))?;

    // Run nasm on the input file.
    let listing = find_listing(listing_name)?;

    let (want_cpu, want_cycles) = extract_result(&listing)?;

    let bytes = run_nasm(temp_dir.path(), &listing)
        .map_err(|e| TestError::io(listing.display().to_string(), e))?;
    println!("BYTES: {:02X?}", bytes);

    let result = intel8086::simulate(&bytes)?;
    let got_cpu = result.cpu;

    if want_cpu != got_cpu {
        println!("Wrong CPU result");
        println!("Want:\n{:?}", want_cpu);
        println!(" Got:\n{:?}", got_cpu);
        return Err(TestError::WrongResult {});
    }

    if want_cycles != 0 {
        let got_cycles = result.cycles;
        if want_cycles != got_cycles {
            println!("Wrong cycles result");
            println!("Want:\n{:?}", want_cpu);
            println!(" Got:\n{:?}", got_cpu);
            return Err(TestError::WrongResult {});
        }
    }

    Ok(())
}

fn extract_result(filepath: impl AsRef<Path>) -> Result<(CPU, usize), TestError> {
    let content = std::fs::read_to_string(&filepath)
        .map_err(|e| TestError::io(filepath.as_ref().display().to_string(), e))?;

    let mut cpu = CPU::new();
    let mut cycles: usize = 0;

    let mut answer_mode = false;
    for line in content.lines() {
        let line = line.trim();

        if line == "; ANSWER" {
            answer_mode = true;
            continue;
        }

        if !answer_mode {
            continue;
        }

        if let Some(register_line) = line.strip_prefix("; ") {
            // Check if line matches pattern "reg: 0xNNNN (N)"
            if let Some((pattern, mut value)) = register_line.split_once(':') {
                // Extract register name
                let pattern = pattern.trim();
                value = value.trim();

                if let Some(reg) = Register::find(pattern) {
                    // Extract hex value (0xNNNN)
                    let regval = u16::from_str_radix(&value[2..], 16)
                        .map_err(|e| TestError::custom(e.to_string()))?;
                    debug!("setting test register {0} to 0x{1:04X} ({1})", reg, regval);
                    cpu.set_register(&reg, regval);
                    continue;
                }

                if pattern == "flags" {
                    parse_flags(&mut cpu, value)?;
                    continue;
                }

                if pattern == "cycles" {
                    cycles = usize::from_str_radix(&value[2..], 10)
                        .map_err(|e| TestError::custom(e.to_string()))?;
                    continue;
                }

                return Err(TestError::custom(format!("Unknown pattern {}", pattern)));
            }
        }
    }

    debug!("Parsed CPU: {:?}", cpu);
    if cycles != 0 {
        debug!("Parsed cycles: {:?}", cycles);
    }

    Ok((cpu, cycles))
}

fn parse_flags(cpu: &mut CPU, pattern: &str) -> Result<(), TestError> {
    for c in pattern.chars() {
        match c {
            'Z' => cpu.flags.z = true,
            'S' => cpu.flags.s = true,
            _ => {}
        }
    }

    Ok(())
}
