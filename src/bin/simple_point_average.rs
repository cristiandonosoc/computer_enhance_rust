use computer_enhance_rust::haversine::*;

use std::{fs::File, io::BufReader, time::Instant};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() != 1 {
        panic!("Usage: point_generator <FILE>");
    }

    let filename = &args[0];

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

        let radius = 6371.0;
        let mut sum: f32 = 0.0;
        for coord in &coords {
            let hd = reference_haversine(coord, radius);
            sum += hd;
        }

        let end = Instant::now();

        let average = sum / (coords.len() as f32);
        println!("Havensine average: {:?}", average);
        println!("TIMING: Calculating average: {:?}", end - start);
    }

    Ok(())
}
