#![allow(dead_code, unused)]

use clap::{Parser, arg, command};
use params::{DataSize, Params};
use parse_size::parse_size;

mod generate;
mod params;

// --------------------------------------------------------------------------

fn main() {
    let params: Params = Cli::parse().into();

    println!("Params {:#?}", params.data_size());
}

// --------------------------------------------------------------------------

#[derive(Parser)]
#[command(
    about = "Generate data files for testing and benchmarking jl-pretty",
    version
)]
struct Cli {
    /// Generate this many lines
    #[arg(short, long, conflicts_with = "byte_size", value_name = "COUNT")]
    line_count: Option<u64>,

    /// Generate at least this many bytes (given in human-readable form - 1M, 1.2Gb etc)
    #[arg(short = 's', long, value_name = "SIZE", value_parser = |s: &str| parse_size(s))]
    byte_size: Option<u64>,

    /// Line template to use when generating lines
    #[arg(short, long)]
    template: Option<String>,
}
