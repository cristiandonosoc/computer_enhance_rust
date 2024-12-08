use clap::Parser;
use computer_enhance_rust::{self, args, intel8086};

#[derive(Parser)]
struct Args {
    #[command(flatten)]
    base: args::BaseArgs,

    #[command(flatten)]
    intel: intel8086::args::IntelArgs,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    computer_enhance_rust::args::evaluate_log(&args.base);

    let filter = if args.base.debug { "debug" } else { "info" };
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(filter)).init();

    let contents = std::fs::read(&args.intel.input)?;
    let instructions = intel8086::disassemble(&contents)?;

    let asm = intel8086::to_asm(&instructions);
    println!("{}", asm);

    Ok(())
}
