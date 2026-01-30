use clap::{CommandFactory, Parser};
use flate2::read::MultiGzDecoder;
use inquire::{InquireError, Select};
use std::io::Read;

mod models;
use models::{Format, Query, QueryKey, Work, Works};

fn main() {
    let args = Args::parse();
    if let Err(e) = args.validate() {
        e.exit();
    }

    let bytes = include_bytes!("resources/data.csv.gz");
    let works = read(bytes);

    let selected_query: Option<Vec<Query>> = if args.interactive {
        select(&works)
    } else {
        None
    };

    works
        .filter(&selected_query.unwrap_or(args.query))
        .random(!args.no_random)
        .take(if args.all { works.len() } else { args.number })
        .print(&args.format, args.template.as_ref());
}

#[derive(Parser, Debug)]
#[command(version, about)]
pub struct Args {
    #[arg(short, long, default_value_t = 1, help = "Number to output")]
    pub number: usize,

    #[arg(
        short,
        long,
        default_value_t = false,
        conflicts_with = "number",
        help = "Output all [conflicts with --number]"
    )]
    pub all: bool,

    #[arg(long, default_value_t = false, help = "Disable randomization")]
    pub no_random: bool,

    #[arg(
        short,
        long,
        help = "Filter queries [format: key=value] [possible keys: author, title, text]",
        value_parser
    )]
    pub query: Vec<Query>,

    #[arg(
        short,
        long,
        default_value_t = false,
        conflicts_with = "query",
        help = "Interactive selection mode [conflicts with --query]"
    )]
    interactive: bool,

    #[arg(short, long, help = "Output format")]
    #[clap(value_enum, default_value_t=Format::Plain)]
    pub format: Format,

    #[arg(
        short,
        long,
        value_parser = template_validator,
        help = "Template only for 'quote' format [possible laceholders: {author}, {title}, {text}, {url}. example: '{text} - {author} ({title})']"
    )]
    pub template: Option<String>,
}

fn template_validator(s: &str) -> Result<String, String> {
    if s.contains("{author}")
        || s.contains("{title}")
        || s.contains("{text}")
        || s.contains("{url}")
    {
        Ok(s.to_string())
    } else {
        Err(
            "Template must contain at least one of the placeholders: {author}, {title}, {text}, {url}"
                .to_string(),
        )
    }
}

impl Args {
    fn validate(&self) -> Result<(), clap::Error> {
        if self.template.is_some() {
            match self.format {
                Format::Quote => Ok(()),
                _ => Err(Self::command().error(
                    clap::error::ErrorKind::ArgumentConflict,
                    "--template can only be used with --format quote",
                )),
            }
        } else {
            Ok(())
        }
    }
}

fn read(bytes: &[u8]) -> Vec<Work> {
    let mut decompressed = Vec::new();
    MultiGzDecoder::new(bytes)
        .read_to_end(&mut decompressed)
        .expect("Failed to decompress data");

    let mut csv = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_reader(&decompressed[..]);

    csv.deserialize().filter_map(Result::ok).collect()
}

fn select(works: &Vec<Work>) -> Option<Vec<Query>> {
    let selected_author = Select::new("Select author:", works.authors()).prompt();
    let selected_title = if let Ok(author) = &selected_author {
        Select::new("Select title:", works.titles(author)).prompt()
    } else {
        Err(InquireError::OperationCanceled)
    };

    if let (Ok(author), Ok(title)) = (selected_author, selected_title) {
        Some(vec![
            Query {
                key: QueryKey::Author,
                value: author,
            },
            Query {
                key: QueryKey::Title,
                value: title,
            },
        ])
    } else {
        None
    }
}
