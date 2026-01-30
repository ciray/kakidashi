use clap::ValueEnum;
use rand::seq::IndexedRandom;
use serde::{Deserialize, Serialize};
use serde_json::to_string_pretty;
use std::str::FromStr;

// 作品データ
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WorkRecord {
    author: String,
    title: String,
    pub text: String,
    url: Option<String>,
}

pub trait WorkRecords {
    fn choose_random(&self, n: usize) -> Vec<WorkRecord>;
    fn filter(&self, queries: &[Query]) -> Vec<WorkRecord>;
    fn print(&self, format: &Format);
}

impl WorkRecords for Vec<WorkRecord> {
    fn choose_random(&self, n: usize) -> Vec<WorkRecord> {
        let mut rng = rand::rng();
        self.choose_multiple(&mut rng, n).take(n).cloned().collect()
    }

    fn filter(&self, queries: &[Query]) -> Vec<WorkRecord> {
        self.iter()
            .filter(|record| {
                queries.iter().all(|query| match query.key {
                    QueryKey::Author => record.author.contains(&query.value),
                    QueryKey::Title => record.title.contains(&query.value),
                })
            })
            .cloned()
            .collect()
    }

    fn print(&self, format: &Format) {
        if self.is_empty() {
            return;
        }

        match format {
            Format::Plain => {
                for record in self {
                    println!("{}", record.text);
                }
            }
            Format::Quote => {
                // 書き出し文｜著者名『書名』
                for record in self {
                    println!("{}｜{}『{}』", record.text, record.author, record.title);
                }
            }
            Format::Csv => {
                let mut writer = csv::Writer::from_writer(std::io::stdout());
                for record in self {
                    writer.serialize(record).expect("Failed to write CSV");
                }
                writer.flush().expect("Failed to flush CSV writer");
            }
            Format::Json => {
                let json = if self.len() == 1 {
                    to_string_pretty(&self[0]).expect("Failed to serialize to JSON")
                } else {
                    to_string_pretty(&self).expect("Failed to serialize to JSON")
                };
                println!("{json}");
            }
        }
    }
}

/// CLIオプション
#[derive(ValueEnum, Clone, Debug)]
pub enum Format {
    Plain,
    Quote,
    Csv,
    Json,
}

#[derive(Clone, Debug)]
pub struct Query {
    key: QueryKey,
    value: String,
}

#[derive(ValueEnum, Clone, Debug, PartialEq)]
enum QueryKey {
    Author,
    Title,
}

impl FromStr for Query {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (key, value) = s
            .split_once('=')
            .ok_or("Invalid filter format. Use key=value.")?;
        let key = QueryKey::from_str(key, true)
            .map_err(|_| "Invalid filter key. Valid keys: author, title.".to_string())?;
        Ok(Query {
            key,
            value: value.to_string(),
        })
    }
}
