use clap::Parser;
use computer_enhance_rust::profile_block;
use computer_enhance_rust::{args, haversine, haversine::*, json, perf::profiler::*};
use log::info;
use std::{
    fs::File,
    io::{BufReader, Error, ErrorKind},
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
    init_profiler();

    let args;
    {
        profile_block!("Startup");
        args = Args::parse();
        computer_enhance_rust::args::evaluate_log(&args.base);
    }

    let filename = args.input;
    let mut coords: Vec<Coord> = vec![];

    {
        match args.json.json_parser {
            json::args::JsonParser::Serde => {
                let file = File::open(filename)?;
                let reader = BufReader::new(file);
                coords = serde_json::from_reader(reader)?;
            }
            json::args::JsonParser::Custom => {
                let bytes;
                {
                    let _size = std::fs::metadata(&filename)?.len();
                    profile_block!("Read File", _size);

                    bytes = std::fs::read(filename)?;
                    info!("Input size: {}", bytes.len());
                }

                let json::JsonValue::Array(array) = json::parse(&bytes)? else {
                    return Err(Box::new(Error::new(ErrorKind::InvalidData, "Expected array")));
                };

                {
                    info!("Pair count: {}", array.values.len());

                    profile_block!("Extract Coords from JSON Object");

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
        }
    }

    {
        let average = haversine_average(&coords, args.haversine.earth_radius);
        info!("Havensine average: {:?}", average);
    }

    shutdown_profiler();

    print_timings();

    Ok(())
}

fn extract_number(object: &json::Object, key: &str) -> f64 {
    let value = object.get(key).unwrap();
    let json::JsonValue::Number(n) = value else {
        panic!("expected number");
    };

    *n
}
