use serde::Deserialize;

use std::{fs::File, io::BufReader, time::Instant};

#[derive(Deserialize)]
struct Coord {
    x0: f32,
    y0: f32,
    x1: f32,
    y1: f32,
}

fn calculate_haversine_degrees(coord: &Coord, radius: f32) -> f32 {
    let dy = (coord.y1 - coord.y0).to_radians();
    let dx = (coord.x1 - coord.x0).to_radians();
    let y0 = coord.y0.to_radians();
    let y1 = coord.y1.to_radians();
    let root_term = (dy / 2.).sin().powi(2) + y0.cos() * y1.cos() * (dx / 2.).sin().powi(2);
    (2. * radius) * (root_term.sqrt()).asin()
}

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
            let hd = calculate_haversine_degrees(coord, radius);
            sum += hd;
        }

        let end = Instant::now();

        let average = sum / (coords.len() as f32);
        println!("Havensine average: {:?}", average);
        println!("TIMING: Calculating average: {:?}", end - start);
    }

    Ok(())
}
