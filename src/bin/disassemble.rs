use computer_enhance_rust::{self, intel8086};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() != 1 {
        panic!("Usage: disassemble <FILE>");
    }

    let contents = std::fs::read(&args[0])?;
    let instructions = intel8086::disassemble(&contents)?;

    println!("bits 16\n");
    for instruction in instructions {
        println!("{}", instruction)
    }

    Ok(())
}
