use anyhow::Result;
use rayon::prelude::*;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

mod extractor;
mod models;

use extractor::{extract_authors, extract_links, extract_text_from_zip, extract_works};
use models::WorkRecord;

const INPUT_PATH: &str = "aozorabunko";
const OUTPUT_PATH: &str = "src/resources/data.csv";

fn main() -> Result<()> {
    generate_csv()?;

    Ok(())
}

fn generate_csv() -> Result<()> {
    let records: Vec<WorkRecord> =
        extract_authors(&PathBuf::from(INPUT_PATH).join("index_pages/person_all.html"))
            .unwrap_or_default()
            .into_par_iter()
            .flat_map(|author| {
                extract_works(&author)
                    .unwrap_or_default()
                    .into_par_iter()
                    .flat_map(move |work| {
                        let work_link = extract_links(Path::new(&work.page_path))?;
                        let text = extract_text_from_zip(Path::new(&work_link.zip_path))
                            .unwrap_or_default();

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
            .collect::<Vec<WorkRecord>>();

    let mut records = records;
    records.sort_by_key(|r| {
        (
            r.author_name.clone(),
            r.work_title.clone(),
            r.text.clone(),
            r.html_link.clone(),
        )
    });

    write_csv(&records, OUTPUT_PATH)?;

    Ok(())
}

fn write_csv(records: &[WorkRecord], output_path: &str) -> Result<()> {
    if let Some(parent) = Path::new(output_path).parent() {
        fs::create_dir_all(parent)?;
    }
    let mut wtr = csv::Writer::from_path(output_path)?;

    for record in records {
        wtr.write_record([
            &record.author_name,
            &record.work_title,
            &record.text,
            &record.html_link.clone().unwrap_or_default(),
        ])?;
    }

    wtr.flush()?;
    Ok(())
}
