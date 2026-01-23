use anyhow::{Context, Result};
use aozora_core::zip::read_first_txt_from_zip;
use aozora2::strip::convert;
use scraper::{Html, Selector};
use std::fs;
use std::path::Path;

use crate::models::{Author, Work};

/// 著者一覧を抽出 (person_all.htmlより)
pub fn extract_authors(base_path: &Path) -> Result<Vec<Author>> {
    let html_path = base_path.join("index_pages/person_all.html");
    let document = read_html(&html_path)?;

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
                parse_id_from_href(href, "person", ".html").map(|id| Author { id, name })
            })
        })
        .collect();

    Ok(authors)
}

/// 作品一覧を抽出 (著者ページより)
///
/// 著者ページが存在しない場合は空のVecを返す
pub fn extract_works(base_path: &Path, author_id: &str) -> Result<Vec<Work>> {
    let html_path = base_path.join(format!("index_pages/person{}.html", author_id));

    if !html_path.exists() {
        return Ok(Vec::new());
    }

    let document = read_html(&html_path)?;

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
                parse_id_from_href(href, "card", ".html").map(|id| Work { id, title })
            })
        })
        .collect();

    Ok(works)
}

/// 作品のzipファイルパス(ルビ付きテキスト)を抽出
///
/// 作品カードページが存在しない、またはzipが見つからない場合はNoneを返す
pub fn extract_ruby_zip_path(
    base_path: &Path,
    author_id: &str,
    work_id: &str,
) -> Result<Option<String>> {
    let padded_author_id = format!("{:06}", author_id.parse::<u32>().unwrap_or(0));
    let html_path = base_path.join(format!("cards/{}/card{}.html", padded_author_id, work_id));

    if !html_path.exists() {
        return Ok(None);
    }

    let document = read_html(&html_path)?;

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
                    format!(
                        "aozorabunko/cards/{}/{}",
                        padded_author_id,
                        href.trim_start_matches("./")
                    )
                })
        });

    Ok(zip_path)
}

/// zipファイルからテキストを抽出
///
/// zipファイル内の最初のtxtファイルを読み込み、ルビ記号等を除去したプレーンテキストを返す
pub fn extract_text_from_zip(zip_path: &Path) -> Result<String> {
    let bytes = read_first_txt_from_zip(zip_path)?;
    Ok(convert(&bytes))
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
