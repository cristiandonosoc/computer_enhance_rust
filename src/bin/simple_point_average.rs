use clap::Parser;
use computer_enhance_rust::{args, haversine, haversine::*, json};
use log::info;
use std::{
    fs::File,
    io::{BufReader, Error, ErrorKind},
    time::Instant,
};

#[derive(Parser)]
struct Args {
    pub input: String,

    #[command(flatten)]
    base: args::BaseArgs,

    #[command(flatten)]
    haversine: haversine::args::HaversineArgs,

    #[command(flatten)]
    json: json::args::JsonArgs,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    computer_enhance_rust::args::evaluate_log(&args.base);

    let filename = args.input;

    let mut coords: Vec<Coord> = vec![];

    {
        let start = Instant::now();

        match args.json.json_parser {
            json::args::JsonParser::Serde => {
                let file = File::open(filename)?;
                let reader = BufReader::new(file);
                coords = serde_json::from_reader(reader)?;
            }
            json::args::JsonParser::Custom => {
                let bytes = std::fs::read(filename)?;
                let json::JsonValue::Array(array) = json::parse(&bytes)? else {
                    return Err(Box::new(Error::new(ErrorKind::InvalidData, "Expected array")));
                };

                coords.reserve(array.values.len());

                for (i, value) in array.values.iter().enumerate() {
                    let json::JsonValue::Object(object) = value else {
                        return Err(Box::new(Error::new(
                            ErrorKind::InvalidData,
                            format!("entry {}: Expected object", i),
                        )));
                    };

                    let coord = Coord {
                        x0: extract_number(object, "x0"),
                        y0: extract_number(object, "y0"),
                        x1: extract_number(object, "x1"),
                        y1: extract_number(object, "y1"),
                    };
                    coords.push(coord);
                }
            }
        }

        info!("Reading json using parser {:?} took {:?}", args.json.json_parser, start.elapsed());
    }

    {
        let start = Instant::now();

        let average = haversine_average(&coords, args.haversine.earth_radius);

        info!("Havensine average: {:?}", average);
        info!("Calculating average took: {:?}", start.elapsed());
    }

    Ok(())
}

fn extract_number(object: &json::Object, key: &str) -> f64 {
    let value = object.get(key).unwrap();
    let json::JsonValue::Number(n) = value else {
        panic!("expected number");
    };

    *n
}
