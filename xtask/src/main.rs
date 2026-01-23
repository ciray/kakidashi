use anyhow::Result;
use std::env;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

mod extractor;
mod models;

use extractor::{extract_authors, extract_ruby_zip_path, extract_text_from_zip, extract_works};
use models::WorkRecord;

const INPUT_PATH: &str = "aozorabunko";
const OUTPUT_PATH: &str = "src/resources/data.csv";

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: cargo xtask <command>");
        eprintln!("Commands:");
        eprintln!("  generate-csv  - Generate works CSV from aozorabunko");
        return Ok(());
    }

    match args[1].as_str() {
        "generate-csv" => generate_csv()?,
        _ => eprintln!("Unknown command: {}", args[1]),
    }

    Ok(())
}

fn generate_csv() -> Result<()> {
    let input_path = PathBuf::from(INPUT_PATH);
    let author_list_path = input_path.join("index_pages/person_all.html");
    let authors = extract_authors(&author_list_path)?;
    let author_count = authors.len();

    let records: Vec<WorkRecord> = authors
        .into_iter()
        .enumerate()
        .flat_map(|(idx, author)| {
            println!(
                "Processing author {}/{}: {} (ID: {})",
                idx + 1,
                author_count,
                author.name,
                author.id
            );

            let works = extract_works(Path::new(&author.page_path)).unwrap_or_default();
            println!("  Found {} works", works.len());

            works.into_iter().filter_map(move |work| {
                let zip_path = extract_ruby_zip_path(Path::new(&work.page_path)).ok()??;
                let text = extract_text_from_zip(Path::new(&zip_path)).unwrap_or_default();
                println!("  Text: {}", &text);

                Some(WorkRecord {
                    author_id: author.id.clone(),
                    author_name: author.name.clone(),
                    work_id: work.id,
                    work_title: work.title,
                    zip_file_path: zip_path,
                })
            })
        })
        .collect();

    write_csv(&records, OUTPUT_PATH)?;
    println!("CSV file generated successfully.");

    Ok(())
}

fn write_csv(records: &[WorkRecord], output_path: &str) -> Result<()> {
    if let Some(parent) = Path::new(output_path).parent() {
        fs::create_dir_all(parent)?;
    }
    let mut wtr = csv::Writer::from_path(output_path)?;

    // ヘッダー
    wtr.write_record([
        "author_id",
        "author_name",
        "work_id",
        "work_title",
        "zip_file_path",
    ])?;

    // レコード
    for record in records {
        wtr.write_record([
            &record.author_id,
            &record.author_name,
            &record.work_id,
            &record.work_title,
            &record.zip_file_path,
        ])?;
    }

    wtr.flush()?;
    Ok(())
}
