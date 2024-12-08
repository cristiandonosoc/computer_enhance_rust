pub use clap::Parser;

#[derive(Parser, Debug)]
pub struct HaversineArgs {
    #[arg(long, default_value = "0")]
    pub seed: u64,

    #[arg(long, required = true)]
    pub point_count: u64,

    #[arg(long, default_value = "6372.8")]
    pub earth_radius: f64,
}
