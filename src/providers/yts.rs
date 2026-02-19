use anyhow::Result;
use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;

use crate::provider::Provider;
use crate::torrent::Torrent;

#[derive(Deserialize)]
struct Response {
    data: Data,
}

#[derive(Deserialize)]
struct Data {
    #[serde(default)]
    movies: Option<Vec<Movie>>,
}

#[derive(Deserialize)]
struct Movie {
    title: String,
    torrents: Vec<YtsTorrent>,
}

#[derive(Deserialize)]
struct YtsTorrent {
    hash: String,
    seeds: u32,
    peers: u32,
    size: String,
    quality: String,
    #[serde(rename = "type")]
    codec_type: String,
}

fn parse_size(s: &str) -> u64 {
    let s = s.to_lowercase();
    let (num, mult) = if let Some(n) = s.strip_suffix("gb") {
        (n.trim().parse::<f64>().unwrap_or(0.0), 1_000_000_000.0)
    } else if let Some(n) = s.strip_suffix("mb") {
        (n.trim().parse::<f64>().unwrap_or(0.0), 1_000_000.0)
    } else {
        (0.0, 1.0)
    };
    (num * mult) as u64
}

pub struct Yts;

#[async_trait]
impl Provider for Yts {
    async fn search(&self, client: &Client, query: &str) -> Result<Vec<Torrent>> {
        let resp: Response = client
            .get("https://yts.do/api/v2/list_movies.json")
            .query(&[("query_term", query), ("sort_by", "seeds"), ("order_by", "desc")])
            .send()
            .await?
            .json()
            .await?;

        let movies = resp.data.movies.unwrap_or_default();

        Ok(movies
            .into_iter()
            .flat_map(|movie| {
                movie.torrents.into_iter().map(move |t| Torrent {
                    name: format!("{} [{}] [{}]", movie.title, t.quality, t.codec_type),
                    info_hash: t.hash.to_lowercase(),
                    magnet: crate::magnet::build_magnet(&t.hash, &movie.title),
                    seeders: t.seeds,
                    leechers: t.peers,
                    size_bytes: parse_size(&t.size),
                    providers: vec!["yts"],
                })
            })
            .collect())
    }
}
