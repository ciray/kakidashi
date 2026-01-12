// xtask/src/extractor.rs
use anyhow::{Context, Result};
use scraper::{Html, Selector};
use std::fs;
use std::path::{Path, PathBuf};

use crate::models::{Author, Work};

/// Extract authors from person_all.html
pub fn extract_authors(base_path: &Path) -> Result<Vec<Author>> {
    let html_path = base_path.join("index_pages/person_all.html");
    let content =
        fs::read(&html_path).with_context(|| format!("Failed to read {:?}", html_path))?;
    let document = Html::parse_document(
        String::from_utf8(content)
            .with_context(|| format!("Failed to parse UTF-8 from {:?}", html_path))?
            .as_str(),
    );
    let li_selector = Selector::parse("ol > li").unwrap();
    let a_selector = Selector::parse("a").unwrap();

    let mut authors = Vec::new();
    for li_element in document.select(&li_selector) {
        let li_html = li_element.html();

        // "著作権存続"はスキップ
        if li_html.contains("著作権存続") {
            continue;
        }

        // Extract author link
        if let Some(a_element) = li_element.select(&a_selector).next() {
            let href = a_element.value().attr("href").unwrap_or("");
            let name = a_element.text().collect::<String>().trim().to_string();

            // Extract author ID from href like "person1257.html#sakuhin_list_1"
            if let Some(id) = extract_id_from_href(href, "person", ".html") {
                authors.push(Author { id, name });
            }
        }
    }

    Ok(authors)
}

/// Extract works from author's page
pub fn extract_works(base_path: &Path, author_id: &str, author_name: &str) -> Result<Vec<Work>> {
    let html_path = base_path.join(format!("index_pages/person{}.html", author_id));

    if !html_path.exists() {
        println!(
            "  Warning: Author page not found for {} ({})",
            author_name, author_id
        );
        return Ok(Vec::new());
    }

    let content =
        fs::read(&html_path).with_context(|| format!("Failed to read {:?}", html_path))?;

    let document = Html::parse_document(
        String::from_utf8(content)
            .with_context(|| format!("Failed to parse UTF-8 from {:?}", html_path))?
            .as_str(),
    );

    // Select works under "公開中の作品" section
    let li_selector = Selector::parse("ol > li").unwrap();
    let a_selector = Selector::parse("a").unwrap();

    let mut works = Vec::new();

    for li_element in document.select(&li_selector) {
        let li_html = li_element.html();

        // Check if this is in the published works section
        // Look for pattern: 作品ID：59898
        if !li_html.contains("作品ID") {
            continue;
        }

        // Extract work link
        if let Some(a_element) = li_element.select(&a_selector).next() {
            let href = a_element.value().attr("href").unwrap_or("");
            let title = a_element.text().collect::<String>().trim().to_string();

            // Extract work ID from href like "../cards/001257/card59898.html"
            if let Some(id) = extract_id_from_href(href, "card", ".html") {
                works.push(Work { id, title });
            }
        }
    }

    println!("  Found {} works", works.len());
    Ok(works)
}

/// Extract zip file path from work's card page
pub fn extract_zip_path(
    base_path: &Path,
    author_id: &str,
    work_id: &str,
) -> Result<Option<String>> {
    // Construct path: cards/001257/card59898.html
    let author_id_padded = format!("{:06}", author_id.parse::<u32>().unwrap_or(0));
    let html_path = base_path.join(format!("cards/{}/card{}.html", author_id_padded, work_id));

    if !html_path.exists() {
        println!(
            "    Warning: Work card page not found for work ID {} of author ID {} at {:?}",
            work_id, author_id, html_path
        );
        return Ok(None);
    }

    let content =
        fs::read(&html_path).with_context(|| format!("Failed to read {:?}", html_path))?;

    let document = Html::parse_document(
        String::from_utf8(content)
            .with_context(|| format!("Failed to parse UTF-8 from {:?}", html_path))?
            .as_str(),
    );

    // Look for table with download links
    let tr_selector = Selector::parse("tr[bgcolor='white']").unwrap();
    let a_selector = Selector::parse("a").unwrap();

    for tr_element in document.select(&tr_selector) {
        let tr_html = tr_element.html();

        // Look for ruby text zip file (ルビあり)
        if tr_html.contains("ルビあり") && tr_html.contains(".zip") {
            if let Some(a_element) = tr_element.select(&a_selector).next() {
                if let Some(href) = a_element.value().attr("href") {
                    if href.ends_with(".zip") {
                        // Construct full path: aozorabunko/cards/001257/files/59898_ruby_70679.zip
                        let full_path = format!(
                            "aozorabunko/cards/{}/{}",
                            author_id_padded,
                            href.trim_start_matches("./")
                        );
                        return Ok(Some(full_path));
                    }
                }
            }
        }
    }

    Ok(None)
}

/// Extract ID from href string
fn extract_id_from_href(href: &str, prefix: &str, suffix: &str) -> Option<String> {
    let start = href.rfind(prefix)?;
    let id_start = start + prefix.len();
    let end = href[id_start..].find(suffix)?;
    Some(href[id_start..id_start + end].to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_id_from_href() {
        assert_eq!(
            extract_id_from_href("person1257.html#sakuhin_list_1", "person", ".html"),
            Some("1257".to_string())
        );

        assert_eq!(
            extract_id_from_href("../cards/001257/card59898.html", "card", ".html"),
            Some("59898".to_string())
        );
    }
}
