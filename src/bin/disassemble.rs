use clap::Parser;
use computer_enhance_rust::{self, args, intel8086};

#[derive(Parser)]
struct Args {
    input: String,

    #[command(flatten)]
    base: args::BaseArgs,

    #[command(flatten)]
    intel: intel8086::args::IntelArgs,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    computer_enhance_rust::args::evaluate_log(&args.base);

    let contents = std::fs::read(&args.input)?;
    let instructions = intel8086::disassemble(&contents)?;

    let asm = intel8086::to_asm(&instructions);
    println!("{}", asm);

    Ok(())
}
