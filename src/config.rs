pub use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about)]
pub struct Args {
    pub input: String,

    #[arg(short, long)]
    pub debug: bool,

    #[arg(long)]
    pub dump: bool,
}



