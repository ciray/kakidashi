use clap::ValueEnum;
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use serde_json::to_string_pretty;
use std::str::FromStr;

// 作品データ
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Work {
    author: String,
    title: String,
    pub text: String,
    url: Option<String>,
}

pub trait Works {
    fn random(&self, random: bool) -> Vec<Work>;
    fn take(&self, n: usize) -> Vec<Work>;
    fn filter(&self, queries: &[Query]) -> Vec<Work>;
    fn print(&self, format: &Format, template: Option<&String>);
}

impl Works for Vec<Work> {
    fn random(&self, random: bool) -> Vec<Work> {
        if !random {
            return self.clone();
        }

        let mut rng = rand::rng();
        let mut records = self.clone();
        records.shuffle(&mut rng);
        records
    }

    fn take(&self, n: usize) -> Vec<Work> {
        self.iter().take(n).cloned().collect()
    }

    fn filter(&self, queries: &[Query]) -> Vec<Work> {
        self.iter()
            .filter(|record| {
                queries.iter().all(|query| match query.key {
                    QueryKey::Author => record.author.contains(&query.value),
                    QueryKey::Title => record.title.contains(&query.value),
                    QueryKey::Text => record.text.contains(&query.value),
                })
            })
            .cloned()
            .collect()
    }

    fn print(&self, format: &Format, template: Option<&String>) {
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
                let template = template.map_or("{text}｜{author}『{title}』", |v| v);
                for record in self {
                    let output = template
                        .replace("\\n", "\n")
                        .replace("{author}", &record.author)
                        .replace("{title}", &record.title)
                        .replace("{text}", &record.text)
                        .replace("{url}", record.url.as_deref().unwrap_or(""));
                    println!("{output}");
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
#[derive(Clone, Debug)]
pub struct Query {
    key: QueryKey,
    value: String,
}

#[derive(ValueEnum, Clone, Debug, PartialEq)]
enum QueryKey {
    Author,
    Title,
    Text,
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

#[derive(ValueEnum, Clone, Debug)]
pub enum Format {
    Plain,
    Quote,
    Csv,
    Json,
}
