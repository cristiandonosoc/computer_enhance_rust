use clap::Parser;
use computer_enhance_rust::{args, haversine, haversine::*};

use std::{fs::File, io::BufReader, time::Instant};

#[derive(Parser)]
struct Args {
    pub input: String,

    #[command(flatten)]
    base: args::BaseArgs,

    #[command(flatten)]
    haversine: haversine::args::HaversineArgs,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    computer_enhance_rust::args::evaluate_log(&args.base);

    let filename = args.input;

    let coords: Vec<Coord>;

    {
        let start = Instant::now();

        let file = File::open(filename)?;
        let reader = BufReader::new(file);

        coords = serde_json::from_reader(reader)?;

        let end = Instant::now();
        println!("TIMING: Reading json {:?}", end - start);
    }

    {
        let start = Instant::now();

        let average = haversine_average(&coords, args.haversine.earth_radius);

        println!("Havensine average: {:?}", average);
        println!("TIMING: Calculating average: {:?}", start.elapsed());
    }

    Ok(())
}
