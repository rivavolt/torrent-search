use anyhow::Result;
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::provider::Provider;
use crate::torrent::Torrent;

#[derive(Serialize)]
struct SearchRequest<'a> {
    search_type: &'a str,
    search_field: &'a str,
    query: &'a str,
    order_by: &'a str,
    order_direction: &'a str,
    hide_unsafe: bool,
    hide_xxx: bool,
    limit: u32,
}

#[derive(Deserialize)]
struct SearchResponse {
    #[serde(default)]
    hits: Vec<Hit>,
}

#[derive(Deserialize)]
struct Hit {
    title: String,
    hash: Option<String>,
    seeders: u32,
    peers: u32,
    bytes: u64,
    #[serde(rename = "magnetUrl")]
    magnet_url: Option<String>,
}

pub struct Knaben;

#[async_trait]
impl Provider for Knaben {
    async fn search(&self, client: &Client, query: &str) -> Result<Vec<Torrent>> {
        let body = SearchRequest {
            search_type: "100%",
            search_field: "title",
            query,
            order_by: "seeders",
            order_direction: "desc",
            hide_unsafe: true,
            hide_xxx: true,
            limit: 100,
        };

        let resp: SearchResponse = client
            .post("https://api.knaben.org/v1")
            .json(&body)
            .send()
            .await?
            .json()
            .await?;

        Ok(resp
            .hits
            .into_iter()
            .filter_map(|h| {
                let hash = h.hash.filter(|s| !s.is_empty())?;
                let magnet = h
                    .magnet_url
                    .unwrap_or_else(|| crate::magnet::build_magnet(&hash, &h.title));
                Some(Torrent {
                    info_hash: hash.to_lowercase(),
                    name: h.title,
                    magnet,
                    seeders: h.seeders,
                    leechers: h.peers,
                    size_bytes: h.bytes,
                    providers: vec!["knaben"],
                })
            })
            .collect())
    }
}
