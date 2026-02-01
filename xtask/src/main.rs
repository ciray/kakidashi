use anyhow::{Ok, Result};
use flate2::{Compression, write::GzEncoder};
use rayon::prelude::*;
use std::fs::{File, create_dir_all};
use std::io::Write;
use std::path::{Path, PathBuf};

mod extractor;
mod models;

use extractor::{extract_authors, extract_links, extract_text_from_zip, extract_works};
use models::WorkRecord;

const INPUT_PATH: &str = "aozorabunko";
const OUTPUT_CSV_PATH: &str = "src/resources/data.csv";
const OUTPUT_GZIP_PATH: &str = "src/resources/data.csv.gz";

fn main() -> Result<()> {
    let mut records = extract(INPUT_PATH);
    records.sort_by_key(|r| {
        (
            r.author.clone(),
            r.title.clone(),
            r.text.clone(),
            r.url.clone(),
        )
    });
    println!("{}", records.len());

    write_csv(&records, OUTPUT_CSV_PATH)?;

    records.retain(|r| !r.text.is_empty());
    println!("{}", records.len());
    compress_csv(&records, OUTPUT_GZIP_PATH)?;

    Ok(())
}

fn extract(aozorabunko: &str) -> Vec<WorkRecord> {
    extract_authors(&PathBuf::from(aozorabunko).join("index_pages/person_all.html"))
        .unwrap_or_default()
        .into_par_iter()
        .flat_map(|author| {
            extract_works(&author)
                .unwrap_or_default()
                .into_par_iter()
                .flat_map(move |work| {
                    let work_link = extract_links(Path::new(&work.page_path))?;
                    let text =
                        extract_text_from_zip(Path::new(&work_link.zip_path)).unwrap_or_default();

                    Some(WorkRecord {
                        author: author.name.clone(),
                        title: work.title,
                        text,
                        url: work_link.url,
                    })
                })
        })
        .collect::<Vec<WorkRecord>>()
}

fn write_csv(records: &Vec<WorkRecord>, output_path: &str) -> Result<()> {
    if let Some(parent) = Path::new(output_path).parent() {
        create_dir_all(parent)?;
    }
    let mut writer = csv::Writer::from_path(output_path)?;

    for record in records {
        writer.write_record([
            &record.author,
            &record.title,
            &record.text,
            &record.url.clone().unwrap_or_default(),
        ])?;
    }

    writer.flush()?;

    Ok(())
}

fn compress_csv(records: &Vec<WorkRecord>, output_path: &str) -> Result<()> {
    let output = File::create(output_path)?;
    let mut encoder = GzEncoder::new(output, Compression::default());

    for record in records {
        let line = format!(
            "{},{},{},{}\n",
            record.author,
            record.title,
            record.text,
            record.url.clone().unwrap_or_default()
        );
        encoder.write_all(line.as_bytes())?;
    }

    encoder.finish()?;

    Ok(())
}
