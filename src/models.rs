use clap::ValueEnum;
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use serde_json::to_string;
use std::str::FromStr;

// 作品データ
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Work {
    pub author: String,
    pub title: String,
    pub text: String,
    url: Option<String>,
}

pub trait Works {
    fn random(&self, random: bool) -> Vec<Work>;
    fn take(&self, n: usize) -> Vec<Work>;
    fn filter(&self, queries: &[Query]) -> Vec<Work>;
    fn print(&self, format: &Format, template: Option<&String>);
    fn authors(&self) -> Vec<String>;
    fn titles(&self, author: &str) -> Vec<String>;
}

impl Works for Vec<Work> {
    fn random(&self, random: bool) -> Vec<Work> {
        if !random {
            return self.clone();
        }

        let mut rng = rand::rng();
        let mut works = self.clone();
        works.shuffle(&mut rng);
        works
    }

    fn take(&self, n: usize) -> Vec<Work> {
        self.iter().take(n).cloned().collect()
    }

    fn filter(&self, queries: &[Query]) -> Vec<Work> {
        self.iter()
            .filter(|work| {
                queries.iter().all(|query| match query.key {
                    QueryKey::Author => work.author.contains(&query.value),
                    QueryKey::Title => work.title.contains(&query.value),
                    QueryKey::Text => work.text.contains(&query.value),
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
                for work in self {
                    println!("{}", work.text);
                }
            }
            Format::Quote => {
                let template = template.map_or("{text}｜{author}『{title}』", |v| v);
                for work in self {
                    let output = template
                        .replace("\\n", "\n")
                        .replace("{author}", &work.author)
                        .replace("{title}", &work.title)
                        .replace("{text}", &work.text)
                        .replace("{url}", work.url.as_deref().unwrap_or(""));
                    println!("{output}");
                }
            }
            Format::Csv => {
                let mut writer = csv::Writer::from_writer(std::io::stdout());
                for work in self {
                    writer.serialize(work).expect("Failed to write CSV");
                }
                writer.flush().expect("Failed to flush CSV writer");
            }
            Format::Json => {
                let json = if self.len() == 1 {
                    to_string(&self[0]).expect("Failed to serialize to JSON")
                } else {
                    to_string(&self).expect("Failed to serialize to JSON")
                };
                println!("{json}");
            }
        }
    }

    fn authors(&self) -> Vec<String> {
        let mut authors: Vec<String> = self.iter().map(|work| work.author.clone()).collect();
        authors.sort();
        authors.dedup();
        authors
    }

    fn titles(&self, author: &str) -> Vec<String> {
        let mut titles: Vec<String> = self
            .iter()
            .filter(|work| work.author.contains(author))
            .map(|work| work.title.clone())
            .collect();
        titles.sort();
        titles.dedup();
        titles
    }
}

/// CLIオプション
#[derive(Clone, Debug)]
pub struct Query {
    pub(crate) key: QueryKey,
    pub(crate) value: String,
}

#[derive(ValueEnum, Clone, Debug, PartialEq)]
pub(crate) enum QueryKey {
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
