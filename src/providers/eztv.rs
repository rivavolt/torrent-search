use anyhow::Result;
use async_trait::async_trait;
use reqwest::Client;

use crate::provider::Provider;
use crate::torrent::Torrent;

pub struct Eztv;

fn parse_size_eztv(s: &str) -> u64 {
    let s = s.trim().to_lowercase();
    let parts: Vec<&str> = s.split_whitespace().collect();
    if parts.len() != 2 {
        return 0;
    }
    let num: f64 = parts[0].parse().unwrap_or(0.0);
    let mult = match parts[1] {
        "tb" => 1_000_000_000_000.0,
        "gb" => 1_000_000_000.0,
        "mb" => 1_000_000.0,
        "kb" => 1_000.0,
        _ => 1.0,
    };
    (num * mult) as u64
}

#[async_trait]
impl Provider for Eztv {
    async fn search(&self, client: &Client, query: &str) -> Result<Vec<Torrent>> {
        let url = format!(
            "https://eztvx.to/search/{}",
            query.replace(' ', "-")
        );
        // POST with layout=def_wlinks to get magnet links in the response
        let html = client
            .post(&url)
            .form(&[("layout", "def_wlinks")])
            .send()
            .await?
            .text()
            .await?;

        let doc = scraper::Html::parse_document(&html);
        let row_sel = scraper::Selector::parse("tbody tr.forum_header_border").unwrap();
        let td_sel = scraper::Selector::parse("td").unwrap();
        let a_sel = scraper::Selector::parse("a").unwrap();

        let mut results = Vec::new();

        for row in doc.select(&row_sel) {
            let tds: Vec<_> = row.select(&td_sel).collect();
            if tds.len() < 6 {
                continue;
            }

            // Column 1: name
            let name = tds[1]
                .select(&a_sel)
                .next()
                .map(|a| a.text().collect::<String>().trim().to_string())
                .unwrap_or_default();

            // Column 2: magnet link
            let magnet = tds[2]
                .select(&a_sel)
                .find_map(|a| {
                    let href = a.value().attr("href")?;
                    href.starts_with("magnet:").then(|| href.to_string())
                })
                .unwrap_or_default();

            let info_hash = crate::magnet::extract_hash(&magnet).unwrap_or_default();

            // Column 3: size
            let size_text = tds[3].text().collect::<String>();
            let size_bytes = parse_size_eztv(&size_text);

            // Column 5: seeders
            let seeders: u32 = tds[5]
                .text()
                .collect::<String>()
                .trim()
                .replace(",", "")
                .parse()
                .unwrap_or(0);

            if name.is_empty() || magnet.is_empty() {
                continue;
            }

            results.push(Torrent {
                name,
                info_hash,
                magnet,
                seeders,
                leechers: 0,
                size_bytes,
                providers: vec!["eztv"],
            });
        }

        Ok(results)
    }
}
