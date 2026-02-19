use anyhow::Result;
use async_trait::async_trait;
use reqwest::Client;

use crate::torrent::Torrent;

#[async_trait]
pub trait Provider: Send + Sync {
    async fn search(&self, client: &Client, query: &str) -> Result<Vec<Torrent>>;
}
