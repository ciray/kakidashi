use anyhow::Result;
use std::env;
use std::fs;
use std::path::PathBuf;

mod extractor;
mod models;

use extractor::{extract_authors, extract_works, extract_zip_path};
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

    if !base_path.exists() {
        anyhow::bail!("aozorabunko directory not found. Please initialize the submodule.");
    }

    println!("Extracting authors from person_all.html...");
    let authors = extract_authors(&base_path)?;
    println!("Found {} authors", authors.len());

    let mut records = Vec::new();
    for (idx, author) in authors.iter().enumerate() {
        println!(
            "Processing author {}/{}: {} (ID: {})",
            idx + 1,
            authors.len(),
            author.name,
            author.id
        );

        let works = extract_works(&base_path, &author.id, &author.name)?;
        for work in works {
            let zip_path = extract_zip_path(&base_path, &author.id, &work.id)?;

            if let Some(zip) = zip_path {
                records.push(WorkRecord {
                    author_id: author.id.clone(),
                    author_name: author.name.clone(),
                    work_id: work.id,
                    work_title: work.title,
                    zip_file_path: zip,
                });
            }
        }
    }

    println!("\nWriting CSV with {} records...", records.len());
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
