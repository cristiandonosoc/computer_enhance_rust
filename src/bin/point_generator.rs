use serde::Serialize;
use std::fs::File;
use std::io::Write;
use rand::Rng;

#[derive(Serialize)]
struct Coord
{
    x: f64,
    y: f64,
}

#[derive(Serialize)]
struct CoordPair(Coord, Coord);

fn generate_coord() -> Coord {
    let mut rng = rand::thread_rng();

    Coord {
        x: rng.gen_range(-180.0..180.0),
        y: rng.gen_range(-90.0..90.0),
    }
}

fn main() {
    let args : Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() {
        panic!("Usage: point_generator <FILE>");
    }

    let num_pairs = 10;
    let mut coords = Vec::with_capacity(num_pairs);

    for _ in 0..num_pairs {
        coords.push(CoordPair(generate_coord(), generate_coord()));
    }

    // Convert to JSON.
    let json = serde_json::to_string_pretty(&coords).unwrap();

    // Write to file.
    let mut file = File::create(&args[0]).unwrap();
    file.write_all(json.as_bytes()).unwrap();
}


