use anyhow::anyhow;
use clap::Parser;
use computer_enhance_rust::{self, args, intel8086, nasm::run_nasm};
use std::path::Path;

#[derive(Parser)]
struct Args {
    pub input: String,

    #[command(flatten)]
    base: args::BaseArgs,

    #[command(flatten)]
    intel: intel8086::args::IntelArgs,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    computer_enhance_rust::args::evaluate_log(&args.base);

    let bytes = run_nasm(Path::new("."), &args.input)?;
    let result = intel8086::simulate(&bytes)?;

    if args.intel.dump_memory {
        let filename = Path::new(&args.input)
            .file_stem()
            .ok_or(anyhow!("invalid path"))?;
        let out = format!("{}.data", filename.to_string_lossy());

        let memory = result.cpu.get_memory();
        std::fs::write(&out, &memory)?;

        println!("Wrote result dump to {}", out);
    }

    Ok(())
}
