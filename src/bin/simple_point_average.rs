use clap::Parser;
use computer_enhance_rust::{args, haversine, haversine::*};
use log::info;
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

        info!("Reading json took {:?}", start.elapsed());
    }

    {
        let start = Instant::now();

        let average = haversine_average(&coords, args.haversine.earth_radius);

        info!("Havensine average: {:?}", average);
        info!("Calculating average took: {:?}", start.elapsed());
    }

    Ok(())
}
