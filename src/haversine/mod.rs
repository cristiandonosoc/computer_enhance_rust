pub mod args;

use rand::Rng;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Coord {
    pub x0: f64,
    pub y0: f64,
    pub x1: f64,
    pub y1: f64,
}

impl Coord {
    pub fn new_random<R: Rng>(rng: &mut R) -> Self {
        Coord {
            x0: rng.gen_range(-180.0..180.0),
            y0: rng.gen_range(-90.0..90.0),
            x1: rng.gen_range(-180.0..180.0),
            y1: rng.gen_range(-90.0..90.0),
        }
    }
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
