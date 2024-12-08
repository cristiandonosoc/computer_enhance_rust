use clap::{Parser, ValueEnum};

#[derive(Debug, Clone, ValueEnum)]
pub enum JsonParser {
    Serde,
    Custom,
}

#[derive(Debug, Parser)]
pub struct JsonArgs {
    #[arg(long, value_enum, default_value_t = JsonParser::Serde)]
    pub json_parser: JsonParser,
}
