pub use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about)]
pub struct BaseArgs {
    #[arg(long)]
    pub debug: bool,

    #[arg(long)]
    pub silent: bool,
}

pub fn evaluate_log(args: &BaseArgs) {
    if !args.silent {
        let filter = if args.debug { "debug" } else { "info" };
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(filter)).init();
    }
}
