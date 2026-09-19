//! Minimal `key = "value"` config reader, so we don't pull in a TOML crate.

use std::fs;
use std::path::Path;

pub struct Config(Vec<(String, String)>);

impl Config {
    pub fn load(path: impl AsRef<Path>) -> Config {
        let src = fs::read_to_string(path).unwrap_or_default();
        let mut pairs = Vec::new();
        for line in src.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let Some((key, value)) = line.split_once('=') else { continue };
            let value = value.trim().trim_matches('"').to_string();
            pairs.push((key.trim().to_string(), value));
        }
        Config(pairs)
    }

    pub fn get(&self, key: &str) -> &str {
        self.0
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.as_str())
            .unwrap_or("")
    }

    /// Origin without a trailing slash, e.g. `https://example.com`.
    pub fn base_url(&self) -> String {
        self.get("base_url").trim_end_matches('/').to_string()
    }

    /// Path prefix with exactly one leading and one trailing slash.
    pub fn base_path(&self) -> String {
        let raw = self.get("base_path").trim().trim_matches('/').to_string();
        if raw.is_empty() {
            "/".into()
        } else {
            format!("/{raw}/")
        }
    }
}
