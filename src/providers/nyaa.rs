use anyhow::Result;
use async_trait::async_trait;
use reqwest::Client;

use crate::provider::Provider;
use crate::torrent::Torrent;

pub struct Nyaa;

fn parse_size_nyaa(s: &str) -> u64 {
    let s = s.trim();
    let parts: Vec<&str> = s.split_whitespace().collect();
    if parts.len() != 2 {
        return 0;
    }
    let num: f64 = parts[0].parse().unwrap_or(0.0);
    let mult = match parts[1] {
        "TiB" => 1024.0 * 1024.0 * 1024.0 * 1024.0,
        "GiB" => 1024.0 * 1024.0 * 1024.0,
        "MiB" => 1024.0 * 1024.0,
        "KiB" => 1024.0,
        _ => 1.0,
    };
    (num * mult) as u64
}

#[async_trait]
impl Provider for Nyaa {
    async fn search(&self, client: &Client, query: &str) -> Result<Vec<Torrent>> {
        let html = client
            .get("https://nyaa.si/")
            .query(&[
                ("f", "0"),
                ("c", "0_0"),
                ("q", query),
                ("s", "seeders"),
                ("o", "desc"),
            ])
            .send()
            .await?
            .text()
            .await?;

        let doc = scraper::Html::parse_document(&html);
        let row_sel = scraper::Selector::parse("tbody tr").unwrap();
        let td_sel = scraper::Selector::parse("td").unwrap();
        let a_sel = scraper::Selector::parse("a").unwrap();

        let mut results = Vec::new();

        for row in doc.select(&row_sel) {
            let tds: Vec<_> = row.select(&td_sel).collect();
            if tds.len() < 7 {
                continue;
            }

            // Column 1: name (second <a> or last <a> in the cell)
            let name_links: Vec<_> = tds[1].select(&a_sel).collect();
            let name_el = if name_links.len() >= 2 {
                name_links.last()
            } else {
                name_links.first()
            };
            let name = name_el
                .map(|a| a.text().collect::<String>().trim().to_string())
                .unwrap_or_default();

            // Column 2: links — first <a> is .torrent, second is magnet
            let link_els: Vec<_> = tds[2].select(&a_sel).collect();
            let magnet = link_els
                .iter()
                .find_map(|a| {
                    let href = a.value().attr("href")?;
                    href.starts_with("magnet:").then(|| href.to_string())
                })
                .unwrap_or_default();

            let info_hash = crate::magnet::extract_hash(&magnet).unwrap_or_default();

            // Column 3: size
            let size_text = tds[3].text().collect::<String>();
            let size_bytes = parse_size_nyaa(&size_text);

            // Column 5: seeders, Column 6: leechers
            let seeders: u32 = tds[5]
                .text()
                .collect::<String>()
                .trim()
                .parse()
                .unwrap_or(0);
            let leechers: u32 = tds[6]
                .text()
                .collect::<String>()
                .trim()
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
                leechers,
                size_bytes,
                providers: vec!["nyaa"],
            });
        }

        Ok(results)
    }
}
