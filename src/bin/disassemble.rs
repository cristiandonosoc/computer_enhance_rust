use computer_enhance_rust::{self, config::*, intel8086};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let filter = if args.debug { "debug" } else { "info" };
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(filter)).init();

    let contents = std::fs::read(&args.input)?;
    let instructions = intel8086::disassemble(&contents)?;

    println!("bits 16\n");
    for instruction in instructions {
        println!("{}", instruction)
    }

    Ok(())
}
