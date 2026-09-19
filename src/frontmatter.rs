//! `---` delimited front matter: one `key: value` per line.
//!
//! Keys may repeat (used for `link:` on the resume), so this keeps a Vec
//! rather than a map.

pub struct Doc {
    meta: Vec<(String, String)>,
    pub body: String,
}

impl Doc {
    pub fn parse(src: &str) -> Doc {
        let src = src.trim_start_matches('\u{feff}');
        let Some(rest) = src.strip_prefix("---") else {
            return Doc { meta: Vec::new(), body: src.to_string() };
        };
        let rest = rest.trim_start_matches(['\r', '\n']);
        let Some(end) = find_fence(rest) else {
            return Doc { meta: Vec::new(), body: src.to_string() };
        };

        let mut meta = Vec::new();
        for line in rest[..end].lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some((key, value)) = line.split_once(':') {
                let value = value.trim().trim_matches('"').to_string();
                meta.push((key.trim().to_lowercase(), value));
            }
        }
        let body = rest[end..]
            .trim_start_matches("---")
            .trim_start_matches(['\r', '\n'])
            .to_string();
        Doc { meta, body }
    }

    pub fn get(&self, key: &str) -> &str {
        self.meta
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.as_str())
            .unwrap_or("")
    }

    /// Every value for a repeated key, in document order.
    pub fn all(&self, key: &str) -> Vec<&str> {
        self.meta
            .iter()
            .filter(|(k, _)| k == key)
            .map(|(_, v)| v.as_str())
            .collect()
    }

    pub fn flag(&self, key: &str) -> bool {
        matches!(self.get(key), "true" | "yes" | "1")
    }

    /// Comma separated value, e.g. `tags: rust, web`.
    pub fn list(&self, key: &str) -> Vec<String> {
        self.get(key)
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string)
            .collect()
    }
}

/// Byte offset of the closing `---` line.
fn find_fence(src: &str) -> Option<usize> {
    let mut offset = 0;
    for line in src.split_inclusive('\n') {
        if line.trim_end() == "---" {
            return Some(offset);
        }
        offset += line.len();
    }
    None
}
