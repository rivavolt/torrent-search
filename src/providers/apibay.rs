use anyhow::Result;
use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;

use crate::provider::Provider;
use crate::torrent::Torrent;

#[derive(Deserialize)]
struct Entry {
    name: String,
    info_hash: String,
    seeders: String,
    leechers: String,
    size: String,
}

pub struct Apibay;

#[async_trait]
impl Provider for Apibay {
    async fn search(&self, client: &Client, query: &str) -> Result<Vec<Torrent>> {
        let entries: Vec<Entry> = client
            .get("https://apibay.org/q.php")
            .query(&[("q", query)])
            .send()
            .await?
            .json()
            .await?;

        Ok(entries
            .into_iter()
            .filter(|e| e.info_hash != "0000000000000000000000000000000000000000")
            .filter_map(|e| {
                Some(Torrent {
                    magnet: crate::magnet::build_magnet(&e.info_hash, &e.name),
                    info_hash: e.info_hash.to_lowercase(),
                    name: e.name,
                    seeders: e.seeders.parse().ok()?,
                    leechers: e.leechers.parse().ok()?,
                    size_bytes: e.size.parse().ok()?,
                    providers: vec!["apibay"],
                })
            })
            .collect())
    }
}
