const TRACKERS: &[&str] = &[
    "udp://tracker.opentrackr.org:1337/announce",
    "udp://tracker.openbittorrent.com:6969/announce",
    "udp://open.stealth.si:80/announce",
    "udp://tracker.torrent.eu.org:451/announce",
    "udp://tracker.bittor.pw:1337/announce",
    "udp://tracker.cyberia.is:6969/announce",
];

pub fn build_magnet(hash: &str, name: &str) -> String {
    use std::fmt::Write;

    let mut magnet = format!(
        "magnet:?xt=urn:btih:{}&dn={}",
        hash,
        urlencoding::encode(name)
    );
    for tracker in TRACKERS {
        write!(magnet, "&tr={}", urlencoding::encode(tracker)).unwrap();
    }
    magnet
}

pub fn extract_hash(magnet: &str) -> Option<String> {
    let after = magnet.strip_prefix("magnet:?xt=urn:btih:")?;
    let hash = after.split('&').next()?;
    Some(hash.to_lowercase())
}
