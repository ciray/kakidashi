use anyhow::{Context, Result};
use aozora_core::zip::read_first_txt_from_zip;
use aozora2::strip::convert;
use scraper::{Html, Selector};
use std::fs;
use std::path::Path;

use crate::models::{Author, Work};

/// 著者一覧を抽出 (person_all.htmlより)
pub fn extract_authors(author_list_path: &Path) -> Option<Vec<Author>> {
    let document = read_html(author_list_path).ok()?;
    let index_pages_dir = author_list_path.parent().unwrap_or(Path::new("."));

    let list_item_selector = Selector::parse("ol > li").unwrap();
    let anchor_selector = Selector::parse("a").unwrap();

    let is_public_domain = |li: &scraper::ElementRef| !li.html().contains("著作権存続");

    let authors = document
        .select(&list_item_selector)
        .filter(is_public_domain)
        .filter_map(|li| {
            li.select(&anchor_selector).next().and_then(|anchor| {
                let href = anchor.value().attr("href").unwrap_or("");
                let name = anchor.text().collect::<String>().trim().to_string();
                parse_id_from_href(href, "person", ".html").map(|id| {
                    let page_path = index_pages_dir.join(format!("person{}.html", id));
                    Author {
                        id,
                        name,
                        page_path: page_path.to_string_lossy().to_string(),
                    }
                })
            })
        })
        .collect();

    Some(authors)
}

/// 作品一覧を抽出 (著者ページより)
///
/// 著者ページが存在しない場合は空のVecを返す
pub fn extract_works(author: &Author) -> Option<Vec<Work>> {
    if !Path::new(&author.page_path).exists() {
        return Some(Vec::new());
    }

    let author_page_path = Path::new(&author.page_path);
    let document = read_html(author_page_path).ok()?;
    let cards_dir = author_page_path
        .parent()
        .and_then(|p| p.parent())
        .map(|p| p.join("cards"))
        .unwrap_or_default();

    let list_item_selector = Selector::parse("ol > li").unwrap();
    let anchor_selector = Selector::parse("a").unwrap();

    // 翻訳書除外
    // TODO: 編者など他パターンは未対応)
    let is_original_work = |html: &str| html.contains("作品ID") && !html.contains("著者");

    let works = document
        .select(&list_item_selector)
        .map(|li| (li.html(), li))
        .filter(|(html, _)| is_original_work(html))
        .filter_map(|(_, li)| {
            li.select(&anchor_selector).next().and_then(|anchor| {
                let href = anchor.value().attr("href").unwrap_or("");
                let title = anchor.text().collect::<String>().trim().to_string();
                parse_id_from_href(href, "card", ".html").and_then(|id| {
                    // href: ../cards/001257/card59898.html から author_id を抽出
                    let author_id = href
                        .strip_prefix("../cards/")
                        .and_then(|s| s.split('/').next())?;
                    let page_path = cards_dir.join(author_id).join(format!("card{}.html", id));
                    Some(Work {
                        id,
                        title,
                        page_path: page_path.to_string_lossy().to_string(),
                    })
                })
            })
        })
        .collect();

    Some(works)
}

/// 作品のzipファイルパス(ルビ付きテキスト)を抽出
///
/// 作品カードページが存在しない、またはzipが見つからない場合はNoneを返す
pub fn extract_ruby_zip_path(work_page_path: &Path) -> Result<Option<String>> {
    if !work_page_path.exists() {
        return Ok(None);
    }

    let document = read_html(work_page_path)?;
    let work_dir = work_page_path.parent().unwrap_or(Path::new("."));

    let table_row_selector = Selector::parse("tr[bgcolor='white']").unwrap();
    let anchor_selector = Selector::parse("a").unwrap();

    let has_ruby_zip = |html: &str| html.contains("ルビあり") && html.contains(".zip");

    let zip_path = document
        .select(&table_row_selector)
        .map(|row| (row.html(), row))
        .filter(|(html, _)| has_ruby_zip(html))
        .find_map(|(_, row)| {
            row.select(&anchor_selector)
                .next()
                .and_then(|anchor| anchor.value().attr("href"))
                .filter(|href| href.ends_with(".zip"))
                .map(|href| {
                    work_dir
                        .join(href.trim_start_matches("./"))
                        .to_string_lossy()
                        .to_string()
                })
        });

    Ok(zip_path)
}

/// zipファイルから書き出しテキストを抽出
pub fn extract_text_from_zip(zip_path: &Path) -> Option<String> {
    let bytes = read_first_txt_from_zip(zip_path).ok()?;
    let text = convert(&bytes);

    // TODO: 最初の一行とみなす条件
    // - 全角スペースで始まる最初の行
    let first_line = text
        .lines()
        .find(|line| line.starts_with('　'))
        .map(|line| line.trim_start_matches('　'))
        .unwrap_or_default()
        .to_string();

    Some(first_line)
}

/// HTMLファイルを読み込みパースする
fn read_html(path: &Path) -> Result<Html> {
    let content = fs::read(path).with_context(|| format!("Failed to read {:?}", path))?;
    let html_str = String::from_utf8(content)
        .with_context(|| format!("Failed to parse UTF-8 from {:?}", path))?;
    Ok(Html::parse_document(&html_str))
}

/// href文字列からID部分をパースして抽出
///
/// 例: parse_id_from_href("person1257.html", "person", ".html") -> Some("1257")
fn parse_id_from_href(href: &str, prefix: &str, suffix: &str) -> Option<String> {
    let prefix_pos = href.rfind(prefix)?;
    let id_start = prefix_pos + prefix.len();
    let id_len = href[id_start..].find(suffix)?;
    Some(href[id_start..id_start + id_len].to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_id_from_href() {
        assert_eq!(
            parse_id_from_href("person1257.html#sakuhin_list_1", "person", ".html"),
            Some("1257".to_string())
        );

        assert_eq!(
            parse_id_from_href("../cards/001257/card59898.html", "card", ".html"),
            Some("59898".to_string())
        );
    }
}
