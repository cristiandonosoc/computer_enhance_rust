use clap::Parser;
use computer_enhance_rust::{args, haversine};
use rand::{rngs::StdRng, SeedableRng};
use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::time::Instant;

#[derive(Parser)]
struct Args {
    pub output: String,

    #[command(flatten)]
    base: args::BaseArgs,

    #[command(flatten)]
    haversine: haversine::args::HaversineArgs,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    computer_enhance_rust::args::evaluate_log(&args.base);

    let count: usize = args.haversine.point_count as usize;
    let filename = args.output;

    let mut coords = Vec::with_capacity(count);

    {
        let start = Instant::now();

        let mut rng: StdRng;
        if args.haversine.seed != 0 {
            rng = StdRng::seed_from_u64(args.haversine.seed);
        } else {
            rng = StdRng::from_entropy();
        }

        for _ in 0..count {
            coords.push(haversine::Coord::new_random(&mut rng));
        }

        println!("Point generation took {:?}", start.elapsed());
    }

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
