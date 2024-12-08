pub use clap::Parser;

#[derive(Parser, Debug)]
pub struct IntelArgs {
    #[arg(long)]
    pub dump_memory: bool,
}
