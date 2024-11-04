use rand::Rng;
use serde::Serialize;
use std::fs::File;
use std::io::Write;
use std::error::Error;

#[derive(Serialize)]
struct Coord {
    x0: f64,
    y0: f64,
    x1: f64,
    y1: f64,
}

fn generate_coord() -> Coord {
    let mut rng = rand::thread_rng();

    Coord {
        x0: rng.gen_range(-180.0..180.0),
        y0: rng.gen_range(-90.0..90.0),
        x1: rng.gen_range(-180.0..180.0),
        y1: rng.gen_range(-90.0..90.0),
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() != 2 {
        panic!("Usage: point_generator <COUNT> <FILE>");
    }

    let count : usize = args[0].parse()?;
    let filename = &args[1];

    let mut coords = Vec::with_capacity(count);

    for _ in 0..count{
        coords.push(generate_coord());
    }

    // Convert to JSON.
    let json = serde_json::to_string_pretty(&coords)?;

    // Write to file.
    let mut file = File::create(filename)?;
    file.write_all(json.as_bytes())?;

    Ok(())
}
