use anyhow::{Context, Result};
use aozora_core::zip::read_first_txt_from_zip;
use aozora2::strip::convert;
use scraper::{Html, Selector};
use std::fs;
use std::path::Path;

use crate::models::{Author, Work, WorkLink};

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

/// 作品のzip/htmlファイルパスを抽出
///
/// ダウンロードデータtable内の全行をリスト化し、その中から条件に合致する行を抽出
/// 1. リンク先が存在し"./files"で始まる行
/// 2. 末尾が`.zip`で終わる行をテキストzipファイルリンクとみなす
///   - "ttz_zip"は除外
///   - 複数行となった場合は最初の1行のみ
///   - 存在しない場合は処理を停止しNoneを返す
/// 3. 末尾が`.html`または`.htm`で終わる行をHTMLファイルリンクとみなす
///   - 存在しない場合はNoneとする
pub fn extract_links(work_page_path: &Path) -> Option<WorkLink> {
    if !work_page_path.exists() {
        return None;
    }

    let aozora_url = "https://www.aozora.gr.jp";
    let work_dir = work_page_path.parent().unwrap_or(Path::new("."));
    let document = read_html(work_page_path).ok()?;
    let table_selector = Selector::parse(r#"table.download"#).ok()?;
    let link_selector = Selector::parse("a[href]").ok()?;

    // ダウンロードテーブル内の全リンクを抽出
    let links: Vec<String> = document
        .select(&table_selector)
        .next()?
        .select(&link_selector)
        .filter_map(|link| link.value().attr("href"))
        .filter(|href| href.starts_with("./files"))
        .map(String::from)
        .collect();
    if links.is_empty() {
        return None;
    }

    // zipパスを抽出
    let zip_path = links
        .iter()
        .filter(|link| link.starts_with("./files"))
        .filter(|link| link.ends_with(".zip"))
        .find(|link| !link.contains("_ttz.zip") && !link.ends_with("ttz.zip"))
        .map(|link| {
            work_dir
                .join(link.trim_start_matches("./"))
                .to_string_lossy()
                .to_string()
        })?
        .clone();
    if zip_path.is_empty() {
        return None;
    }

    // htmlパスを抽出
    let html_path = links
        .iter()
        .filter(|link| link.starts_with("./files"))
        .find(|link| link.ends_with(".html") || link.ends_with(".htm"))
        .map(|link| {
            work_dir
                .join(link.trim_start_matches("./"))
                .to_string_lossy()
                .to_string()
                .replace("aozorabunko", aozora_url)
        })
        .clone();

    Some(WorkLink {
        zip_path,
        html_link: html_path,
    })
}

/// zipファイルから書き出しテキストを抽出
pub fn extract_text_from_zip(zip_path: &Path) -> Option<String> {
    let bytes = read_first_txt_from_zip(zip_path).ok()?;
    let text = convert(&bytes);

    // 書き出しとみなす条件 (TODO: 未検証)
    // - 全角スペースで始まる
    // - `。`を含む
    // 最初の`。`までを抽出
    let first_line = text
        .lines()
        .filter(|line| line.starts_with('　'))
        .find(|line| line.contains('。'))
        .map(|line| line.trim_start_matches('　'))
        .map(|line| line.split('。').next().unwrap_or("").to_string() + "。")
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

    #[test]
    // テキストファイルとHTMLファイルがともに1つずつ存在するケース
    fn test_extract_ruby_zip_path() {
        let work_page_path = Path::new("../aozorabunko/cards/000006/card47064.html");
        let work_link = extract_links(work_page_path).unwrap();
        assert_eq!(
            work_link.zip_path,
            "../aozorabunko/cards/000006/files/47064_txt_31250.zip".to_string()
        );
        assert_eq!(
            work_link.html_link,
            Some("../https://www.aozora.gr.jp/cards/000006/files/47064_31847.html".to_string())
        );
    }

    #[test]
    // テキストファイルを含まないケース
    fn test_extract_only_html_zip_path() {
        let work_page_path = Path::new("../aozorabunko/cards/001529/card409.html");
        let work_link = extract_links(work_page_path);
        assert!(work_link.is_none());
    }

    #[test]
    // テキストファイル1つとHTMLファイル2つ存在するケース
    fn test_extract_only_text_zip_path() {
        let work_page_path = Path::new("../aozorabunko/cards/001393/card54926.html");
        let work_link = extract_links(work_page_path).unwrap();
        assert_eq!(
            work_link.zip_path,
            "../aozorabunko/cards/001393/files/54926_txt_47247.zip".to_string()
        );
        assert_eq!(
            work_link.html_link,
            Some("../https://www.aozora.gr.jp/cards/001393/files/54926_53265.html".to_string())
        );
    }

    #[test]
    // テキストファイルとTTZファイル(zip)も含むケース
    fn test_extract_text_and_ttz_zip_path() {
        let work_page_path = Path::new("../aozorabunko/cards/000148/card769.html");
        let work_link = extract_links(work_page_path).unwrap();
        assert_eq!(
            work_link.zip_path,
            "../aozorabunko/cards/000148/files/769_ruby_565.zip".to_string()
        );
        assert_eq!(
            work_link.html_link,
            Some("../https://www.aozora.gr.jp/cards/000148/files/769_14939.html".to_string())
        );
    }

    #[test]
    // HTMLファイル(.files配下ではない)を含むケース
    fn test_extract_html_file_not_in_files() {
        let work_page_path = Path::new("../aozorabunko/cards/001393/card54926.html");
        let work_link = extract_links(work_page_path).unwrap();
        assert_eq!(
            work_link.zip_path,
            "../aozorabunko/cards/001393/files/54926_txt_47247.zip".to_string()
        );
        assert_eq!(
            work_link.html_link,
            Some("../https://www.aozora.gr.jp/cards/001393/files/54926_53265.html".to_string())
        );
    }
}
