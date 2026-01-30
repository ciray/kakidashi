use clap::ValueEnum;
use rand::seq::IndexedRandom;
use serde::Deserialize;
use std::str::FromStr;

// 作品データ
#[derive(Debug, Clone, Deserialize)]
pub struct WorkRecord {
    author: String,
    title: String,
    pub text: String,
    html_link: Option<String>,
}

pub trait WorkRecords {
    fn choose_random(&self, n: usize) -> Vec<WorkRecord>;
    fn filter(&self, queries: &[Query]) -> Vec<WorkRecord>;
    fn print_text(&self);
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

    fn print_text(&self) {
        for record in self {
            println!("{}", record.text);
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
