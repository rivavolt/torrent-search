mod magnet;
mod provider;
mod providers;
mod torrent;

use std::collections::HashMap;
use std::io::{self, Write};

use clap::Parser;
use futures::future::join_all;
use torrent::Torrent;

#[derive(Parser)]
#[command(name = "torrent-search", about = "Search torrents across multiple providers")]
struct Cli {
    /// Search query
    query: String,

    /// Get magnet link for result N (default: 1)
    #[arg(short, long, num_args = 0..=1, default_missing_value = "1")]
    get: Option<usize>,

    /// Output full results as JSON
    #[arg(long)]
    json: bool,

    /// Comma-separated list of providers to use (default: all)
    #[arg(short, long, value_delimiter = ',')]
    provider: Option<Vec<String>>,

    /// List available providers
    #[arg(long)]
    list_providers: bool,
}

fn dedup(mut all: Vec<Torrent>) -> Vec<Torrent> {
    let mut map: HashMap<String, Torrent> = HashMap::new();

    for t in all.drain(..) {
        if t.info_hash.is_empty() {
            continue;
        }
        map.entry(t.info_hash.clone())
            .and_modify(|existing| existing.merge(t.clone()))
            .or_insert(t);
    }

    let mut results: Vec<Torrent> = map.into_values().collect();
    results.sort_by(|a, b| b.seeders.cmp(&a.seeders));
    results
}

fn print_table(results: &[Torrent]) {
    if results.is_empty() {
        eprintln!("No results found.");
        return;
    }

    let term_width = terminal_size::terminal_size()
        .map(|(w, _)| w.0 as usize)
        .unwrap_or(120);

    let src_width = results
        .iter()
        .map(|t| t.providers.join(",").len())
        .max()
        .unwrap_or(6)
        .max(3);

    // fixed overhead: "### | " (6) + " | " (3) + "SSSSSSSS" (8) + " | " (3) + "SSSS" (4) + " | " (3) + "LLLL" (4) + " | " (3) + src
    let overhead = 6 + 8 + 4 + 4 + src_width + 4 * 3;
    let name_width = term_width.saturating_sub(overhead).max(20);

    eprintln!(
        "{:>3} | {:<name_w$} | {:>8} | {:>4} | {:>4} | {}",
        "#", "Name", "Size", "S", "L", "Src",
        name_w = name_width,
    );
    eprintln!("{}", "─".repeat(term_width.min(200)));

    for (i, t) in results.iter().enumerate() {
        let name = if t.name.chars().count() > name_width {
            let truncated: String = t.name.chars().take(name_width - 1).collect();
            format!("{truncated}…")
        } else {
            t.name.clone()
        };
        let src = t.providers.join(",");
        eprintln!(
            "{:>3} | {:<name_w$} | {:>8} | {:>4} | {:>4} | {}",
            i + 1,
            name,
            t.format_size(),
            t.seeders,
            t.leechers,
            src,
            name_w = name_width,
        );
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let all_provs = providers::all_providers();

    if cli.list_providers {
        for (name, _) in &all_provs {
            println!("{name}");
        }
        return Ok(());
    }

    let selected: Vec<_> = if let Some(ref names) = cli.provider {
        all_provs
            .into_iter()
            .filter(|(name, _)| names.iter().any(|n| n == name))
            .collect()
    } else {
        all_provs
    };

    if selected.is_empty() {
        anyhow::bail!("No matching providers found");
    }

    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (X11; Linux x86_64; rv:128.0) Gecko/20100101 Firefox/128.0")
        .timeout(std::time::Duration::from_secs(10))
        .build()?;

    let futures: Vec<_> = selected
        .iter()
        .map(|(name, prov)| {
            let client = client.clone();
            let query = cli.query.clone();
            let name = *name;
            async move {
                match prov.search(&client, &query).await {
                    Ok(results) => results,
                    Err(e) => {
                        eprintln!("warning: {name}: {e}");
                        vec![]
                    }
                }
            }
        })
        .collect();

    let all_results: Vec<Torrent> = join_all(futures).await.into_iter().flatten().collect();
    let results = dedup(all_results);

    if cli.json {
        let json = serde_json::to_string_pretty(&results)?;
        println!("{json}");
        return Ok(());
    }

    if let Some(n) = cli.get {
        let idx = if n == 0 { 0 } else { n - 1 };
        match results.get(idx) {
            Some(t) => {
                io::stdout().write_all(t.magnet.as_bytes())?;
                io::stdout().write_all(b"\n")?;
            }
            None => {
                anyhow::bail!("Result #{n} not found (only {} results)", results.len());
            }
        }
        return Ok(());
    }

    print_table(&results);
    Ok(())
}
