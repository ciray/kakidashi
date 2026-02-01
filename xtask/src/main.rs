use anyhow::{Ok, Result};
use flate2::{Compression, write::GzEncoder};
use rayon::prelude::*;
use std::fs::{File, create_dir_all};
use std::io::copy;
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
            r.author_name.clone(),
            r.work_title.clone(),
            r.text.clone(),
            r.html_link.clone(),
        )
    });

    write_csv(&records, OUTPUT_CSV_PATH)?;
    compress_csv(OUTPUT_CSV_PATH, OUTPUT_GZIP_PATH)?;

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
                        author_id: author.id.clone(),
                        author_name: author.name.clone(),
                        work_id: work.id,
                        work_title: work.title,
                        html_link: work_link.html_link,
                        text,
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
            &record.author_name,
            &record.work_title,
            &record.text,
            &record.html_link.clone().unwrap_or_default(),
        ])?;
    }

    writer.flush()?;

    Ok(())
}

fn compress_csv(input_path: &str, output_path: &str) -> Result<()> {
    let mut input = File::open(input_path)?;
    let output = File::create(output_path)?;
    let mut encoder = GzEncoder::new(output, Compression::default());

    copy(&mut input, &mut encoder)?;
    encoder.finish()?;

    Ok(())
}
