use clap::Parser;
use computer_enhance_rust::{args, haversine, haversine::*};
use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::time::Instant;

#[derive(Parser)]
struct Args {
    output: String,

    #[arg(long, required = true)]
    point_count: u64,

    #[command(flatten)]
    base: args::BaseArgs,

    #[command(flatten)]
    haversine: haversine::args::HaversineArgs,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    computer_enhance_rust::args::evaluate_log(&args.base);

    let filename = args.output;

    let coords = generate_points(
        args.haversine.generation_method,
        args.point_count as usize,
        args.haversine.seed,
    );

    let json: String;
    {
        let start = Instant::now();

        // Convert to JSON.
        json = serde_json::to_string_pretty(&coords)?;

        println!("Json generation took {:?}", start.elapsed());
    }

    {
        let start = Instant::now();

        // Write to file.
        let mut file = File::create(&filename)?;
        file.write_all(json.as_bytes())?;

        println!("Writing to {:?} took {:?}", filename, start.elapsed());
    }

    Ok(())
}
