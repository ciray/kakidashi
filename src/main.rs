use clap::Parser;
use flate2::read::MultiGzDecoder;
use std::io::Read;

mod models;

use models::WorkRecord;
use models::WorkRecords;
use models::{Format, Query};

#[derive(Parser, Debug)]
#[command(version, about)]
pub struct Args {
    #[arg(short, long, default_value_t = 1, help = "Number of records to output")]
    pub number: usize,

    #[arg(
        short,
        long,
        default_value_t = false,
        conflicts_with = "number",
        help = "Output all records [conflicts with --number]"
    )]
    pub all: bool,

    #[arg(
        long,
        default_value_t = false,
        help = "Disable randomization of records"
    )]
    pub no_random: bool,

    #[arg(short, long, help = "Output format")]
    #[clap(value_enum, default_value_t=Format::Plain)]
    pub format: Format,

    #[arg(
        short,
        long,
        help = "Filter queries [format: key=value] [possible keys: author, title]",
        value_parser
    )]
    pub query: Vec<Query>,
}

fn main() {
    let args = Args::parse();

    let bytes = include_bytes!("resources/data.csv.gz");
    let records = read(bytes);

    let n = if args.all { records.len() } else { args.number };
    records
        .filter(&args.query)
        .random(!args.no_random)
        .take(n)
        .print(&args.format);
}

fn read(bytes: &[u8]) -> Vec<WorkRecord> {
    let mut decompressed = Vec::new();
    MultiGzDecoder::new(bytes)
        .read_to_end(&mut decompressed)
        .expect("Failed to decompress data");

    let mut csv = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_reader(&decompressed[..]);

    csv.deserialize().filter_map(Result::ok).collect()
}
