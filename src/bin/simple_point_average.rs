use clap::Parser;
use computer_enhance_rust::{args, haversine, haversine::*, json, perf};
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
    let start = perf::read_cpu_timer();

    let mut timings: Vec<(u64, &'static str)> = Vec::with_capacity(10);

    let args = Args::parse();
    computer_enhance_rust::args::evaluate_log(&args.base);

    let end = perf::read_cpu_timer();
    timings.push((end - start, "Startup"));

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
                let start = perf::read_cpu_timer();

                let bytes = std::fs::read(filename)?;
                info!("Input size: {}", bytes.len());

                timings.push((perf::read_cpu_timer() - start, "Read File"));

                let start = perf::read_cpu_timer();

                let json::JsonValue::Array(array) = json::parse(&bytes)? else {
                    return Err(Box::new(Error::new(ErrorKind::InvalidData, "Expected array")));
                };

                timings.push((perf::read_cpu_timer() - start, "Parse JSON"));

                let start = perf::read_cpu_timer();

                info!("Pair count: {}", array.values.len());

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

                timings.push((perf::read_cpu_timer() - start, "Extract Coords from JSON Object"));
            }
        }
    }

    {
        let start = perf::read_cpu_timer();
        let average = haversine_average(&coords, args.haversine.earth_radius);
        timings.push((perf::read_cpu_timer() - start, "Calculate Average"));

        info!("Havensine average: {:?}", average);
    }

    let cycles: u64 = timings.iter().map(|(num, _)| num).sum();
    let freq = perf::estimate_cpu_frequency();

    let seconds = perf::get_seconds_from_cpu(cycles, freq);

    info!("");
    info!("Total Time: {:.4}s - CPU freq. {} (~{})", seconds, freq, perf::print_freq(freq));
    for (section_cycles, name) in timings {
        let seconds = perf::get_seconds_from_cpu(section_cycles, freq);
        info!(
            "    {} - Cycles: {}, Time: {:.4}s ({:.4}%)",
            name,
            section_cycles,
            seconds,
            100.0 * (section_cycles as f64) / (cycles as f64)
        );
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
