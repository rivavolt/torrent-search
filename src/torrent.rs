use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Torrent {
    pub name: String,
    pub info_hash: String,
    pub magnet: String,
    pub seeders: u32,
    pub leechers: u32,
    pub size_bytes: u64,
    pub providers: Vec<&'static str>,
}

impl Torrent {
    pub fn format_size(&self) -> String {
        const KIB: u64 = 1024;
        const MIB: u64 = 1024 * KIB;
        const GIB: u64 = 1024 * MIB;
        const TIB: u64 = 1024 * GIB;

        match self.size_bytes {
            b if b >= TIB => format!("{:.1} TiB", b as f64 / TIB as f64),
            b if b >= GIB => format!("{:.1} GiB", b as f64 / GIB as f64),
            b if b >= MIB => format!("{:.1} MiB", b as f64 / MIB as f64),
            b if b >= KIB => format!("{:.1} KiB", b as f64 / KIB as f64),
            b => format!("{b} B"),
        }
    }

    pub fn merge(&mut self, other: Torrent) {
        self.seeders = self.seeders.max(other.seeders);
        self.leechers = self.leechers.max(other.leechers);
        for p in other.providers {
            if !self.providers.contains(&p) {
                self.providers.push(p);
            }
        }
    }
}
