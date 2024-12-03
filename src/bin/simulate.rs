use anyhow::anyhow;
use computer_enhance_rust::{self, config::*, intel8086, nasm::run_nasm};
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    if !args.silent {
        let filter = if args.debug { "debug" } else { "info" };
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(filter)).init();
    }

    let bytes = run_nasm(Path::new("."), &args.input)?;
    let result = intel8086::simulate(&bytes)?;

    if args.dump {
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
