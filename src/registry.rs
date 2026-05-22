//! The writeup registry.
//!
//! A central record of every writeup lyceum has rendered, wherever its
//! source file lives on disk. This is what lets writeups stay colocated
//! with the projects they document while still being indexable from one
//! place.
//!
//! The registry is a derived cache: delete `registry.json` and it rebuilds
//! itself as writeups are re-rendered. The home page (`index.html`) is
//! generated from it.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// One rendered writeup.
#[derive(Serialize, Deserialize, Clone)]
pub struct Entry {
    pub title: String,
    #[serde(default)]
    pub subtitle: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub created: Option<String>,
    #[serde(default)]
    pub updated: Option<String>,
    /// The collection (family) this writeup belongs to on the home page.
    #[serde(default)]
    pub collection: Option<String>,
    /// Stable URL slug. The writeup renders to `w/<slug>.html` inside the
    /// bundle; the slug is assigned once and kept, so links stay durable.
    #[serde(default)]
    pub slug: String,
    /// When true, the writeup stays in the local bundle but is excluded
    /// from `lyceum sync` — it never leaves the machine.
    #[serde(default)]
    pub local_only: bool,
    /// Absolute path to the source `.md`.
    pub source: String,
}

impl Entry {
    /// The rendered page's path inside a bundle rooted at `dir`.
    pub fn page_in(&self, dir: &std::path::Path) -> PathBuf {
        dir.join("w").join(format!("{}.html", self.slug))
    }
}

/// The lyceum home directory: `~/lyceum`.
pub fn home_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    PathBuf::from(home).join("lyceum")
}

pub fn registry_path() -> PathBuf {
    home_dir().join("registry.json")
}

pub fn index_path() -> PathBuf {
    home_dir().join("index.html")
}

/// Load the registry, or an empty one if it does not exist yet.
pub fn load() -> Result<Vec<Entry>, String> {
    let path = registry_path();
    match fs::read_to_string(&path) {
        Ok(s) => {
            serde_json::from_str(&s).map_err(|e| format!("parsing {}: {e}", path.display()))
        }
        Err(_) => Ok(Vec::new()),
    }
}

/// Insert or replace the entry for a given source path, then save. Keying
/// on the source path means re-rendering a writeup updates its entry in
/// place rather than duplicating it.
pub fn upsert(entry: Entry) -> Result<(), String> {
    let mut entries = load()?;
    match entries.iter_mut().find(|e| e.source == entry.source) {
        Some(slot) => *slot = entry,
        None => entries.push(entry),
    }
    save(&entries)
}

/// Write the registry back to disk, pretty-printed so it stays
/// git-diff-friendly.
pub fn save(entries: &[Entry]) -> Result<(), String> {
    let dir = home_dir();
    fs::create_dir_all(&dir).map_err(|e| format!("creating {}: {e}", dir.display()))?;
    let json =
        serde_json::to_string_pretty(entries).map_err(|e| format!("serialising registry: {e}"))?;
    fs::write(registry_path(), json).map_err(|e| format!("writing registry: {e}"))
}
