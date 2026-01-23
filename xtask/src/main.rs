use anyhow::Result;
use std::env;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

mod extractor;
mod models;

use extractor::{extract_authors, extract_ruby_zip_path, extract_text_from_zip, extract_works};
use models::WorkRecord;

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
    let base_path = PathBuf::from("aozorabunko");
    let authors = extract_authors(&base_path)?;
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

            let works = extract_works(&base_path, &author.id).unwrap_or_default();
            println!("  Found {} works", works.len());

            let base_path = base_path.clone();
            works.into_iter().filter_map(move |work| {
                let zip_path = extract_ruby_zip_path(&base_path, &author.id, &work.id).ok()??;
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

    write_csv(&records)?;
    println!("CSV file generated successfully: data/aozora_works.csv");

    Ok(())
}

fn write_csv(records: &[WorkRecord]) -> Result<()> {
    fs::create_dir_all("data")?;
    let output_path = "data/aozora_works.csv";
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
