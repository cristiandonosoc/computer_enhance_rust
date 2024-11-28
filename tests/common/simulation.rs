use super::error::*;

use super::*;
use computer_enhance_rust::intel8086::cpu::*;
use computer_enhance_rust::intel8086::error::*;
use computer_enhance_rust::intel8086::registers::*;

pub fn run_simulation_test(listing_name: &str) -> Result<(), TestError> {
    // Create a temporary dir for this test.
    let temp_dir = TempDir::new().map_err(|e| TestError::io(String::from("TempDir"), e))?;

    // Run nasm on the input file.
    let listing = find_listing(listing_name)?;

    let want = extract_result(&listing)?;

    let bytes = run_nasm(temp_dir.path(), &listing)?;
    println!("BYTES: {:02X?}", bytes);

    let result = intel8086::simulate(&bytes)?;
    let got = result.cpu;

    if want == got {
        return Ok(());
    }

    println!("Wrong result");
    println!("Want:\n{:?}", want);
    println!(" Got:\n{:?}", got);
    assert!(false);
    Ok(())
}

fn extract_result(filepath: impl AsRef<Path>) -> Result<CPU, TestError> {
    let content = std::fs::read_to_string(&filepath)
        .map_err(|e| TestError::io(filepath.as_ref().display().to_string(), e))?;

    let mut cpu = CPU::new();

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
            if let Some((reg, mut value)) = register_line.split_once(':') {
                // Extract register name
                let reg_name = reg.trim().to_string();

                let reg = Register::find(reg_name.as_str())
                    .ok_or(IntelError::UnknownRegister(reg_name))?;

                value = value.trim();

                // Extract hex value (0xNNNN)
                if let Ok(val) = u16::from_str_radix(&value[2..], 16) {
                    cpu.set_register_value16(&reg, val);
                }
            }
        }
    }

    Ok(cpu)
}
