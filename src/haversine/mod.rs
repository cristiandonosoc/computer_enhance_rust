pub mod args;

use clap::ValueEnum;
use log::{debug, info};
use rand::Rng;
use rand::{rngs::StdRng, SeedableRng};
use serde::{Deserialize, Serialize};
use std::ops::Range;
use std::time::Instant;

#[derive(Debug, Clone, ValueEnum)]
pub enum GenerationMethod {
    Uniform,
    Clustered,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Coord {
    pub x0: f64,
    pub y0: f64,
    pub x1: f64,
    pub y1: f64,
}

impl Coord {
    fn new_uniform<R: Rng>(rng: &mut R) -> Self {
        Coord {
            x0: rng.gen_range(-180.0..180.0),
            y0: rng.gen_range(-90.0..90.0),
            x1: rng.gen_range(-180.0..180.0),
            y1: rng.gen_range(-90.0..90.0),
        }
    }

    fn new_clustered<R: Rng>(rng: &mut R, cluster: &Cluster) -> Self {
        Coord {
            x0: rng.gen_range(cluster.range_x.clone()),
            y0: rng.gen_range(cluster.range_y.clone()),
            x1: rng.gen_range(cluster.range_x.clone()),
            y1: rng.gen_range(cluster.range_y.clone()),
        }
    }
}

pub fn generate_points(method: GenerationMethod, count: usize, seed: u64) -> Vec<Coord> {
    match method {
        GenerationMethod::Uniform => generate_uniform_points(count, seed),
        GenerationMethod::Clustered => generate_clustered_points(count, seed),
    }
}

fn generate_uniform_points(count: usize, seed: u64) -> Vec<Coord> {
    let mut rng = generate_rng(seed);
    let mut coords = Vec::with_capacity(count);

    {
        let start = Instant::now();

        for _ in 0..count {
            coords.push(Coord::new_uniform(&mut rng));
        }

        info!("Uniform Point generation took {:?}", start.elapsed());
    }

    coords
}

#[derive(Debug)]
struct Cluster {
    _x: f64,
    _y: f64,
    _distance: f64,
    range_x: Range<f64>,
    range_y: Range<f64>,
}

impl Cluster {
    fn new<R: Rng>(rng: &mut R) -> Self {
        let distance = rng.gen_range(20.0..50.0);

        // We create the ranges so that the generated points will always be in the sphere.
        let x = rng.gen_range((-180.0 + distance)..(180.0 - distance));
        let y = rng.gen_range((-90.0 + distance)..(90.0 - distance));

        Cluster {
            _x: x,
            _y: y,
            _distance: distance,
            range_x: (x - distance..x + distance),
            range_y: (y - distance..y + distance),
        }
    }
}

fn generate_clustered_points(count: usize, seed: u64) -> Vec<Coord> {
    let mut rng = generate_rng(seed);
    let mut coords = Vec::with_capacity(count);

    // Generate random clusters.
    let cluster_count = 3;
    let mut clusters = Vec::with_capacity(cluster_count);

    {
        let start = Instant::now();
        for i in 0..cluster_count {
            let cluster = Cluster::new(&mut rng);
            debug!("Generated cluster {:?} {:?}", i, cluster);
            clusters.push(cluster);
        }

        for _ in 0..count {
            let cluster_index = rng.gen_range(0..cluster_count);
            let cluster = &clusters[cluster_index];

            let coord = Coord::new_clustered(&mut rng, cluster);
            coords.push(coord);
        }
        info!("Clustered point generation took {:?}", start.elapsed());
    }

    coords
}

pub fn haversine_average(coords: &[Coord], radius: f64) -> f64 {
    if coords.is_empty() {
        return 0.0;
    }

    let start = Instant::now();

    let mut sum: f64 = 0.0;
    for coord in coords {
        let hd = reference_haversine(coord, radius);
        sum += hd;
    }

    let average = sum / (coords.len() as f64);

    debug!("Averaging {:?} haversine took {:?}", coords.len(), start.elapsed());

    average
}

// This haversine implementation assumes a |coord| in degrees and |radius| in km.
//
// This is not meant to be a "good" way to calculate the Haversine distance.
// Instead, it attempts to follow, as closely as possible, the formula used in the real-world
// question on which these homework exercises are loosely based.
pub fn reference_haversine(coord: &Coord, radius: f64) -> f64 {
    let dy = (coord.y1 - coord.y0).to_radians();
    let dx = (coord.x1 - coord.x0).to_radians();

    let y0 = coord.y0.to_radians();
    let y1 = coord.y1.to_radians();

    let root_term = (dy / 2.0).sin().powi(2) + y0.cos() * y1.cos() * (dx / 2.).sin().powi(2);
    let c = (root_term.sqrt()).asin();
    (2. * radius) * c
}

pub fn generate_rng(seed: u64) -> StdRng {
    if seed != 0 {
        StdRng::seed_from_u64(seed)
    } else {
        StdRng::from_entropy()
    }
}
