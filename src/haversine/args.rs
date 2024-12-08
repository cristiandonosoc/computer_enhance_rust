pub use clap::Parser;

use super::GenerationMethod;

#[derive(Parser, Debug)]
pub struct HaversineArgs {
    #[arg(long, value_enum, default_value_t = GenerationMethod::Uniform)]
    pub generation_method: GenerationMethod,

    #[arg(long, default_value = "0")]
    pub seed: u64,

    #[arg(long, default_value = "6372.8")]
    pub earth_radius: f64,
}
