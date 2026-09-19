//! `{{ name }}` substitution. No logic in templates: lists are built in Rust
//! and passed in as a single pre-rendered fragment.

use std::fs;
use std::path::Path;

pub struct Template(String);

impl Template {
    pub fn load(dir: &str, name: &str) -> Template {
        let path = Path::new(dir).join(name);
        match fs::read_to_string(&path) {
            Ok(src) => Template(src),
            Err(e) => {
                eprintln!("error: cannot read template {}: {e}", path.display());
                std::process::exit(1);
            }
        }
    }

    pub fn render(&self, vars: &[(&str, &str)]) -> String {
        let src = &self.0;
        let mut out = String::with_capacity(src.len() + 1024);
        let mut rest = src.as_str();
        while let Some(start) = rest.find("{{") {
            let Some(len) = rest[start..].find("}}") else { break };
            let end = start + len;
            out.push_str(&rest[..start]);
            let key = rest[start + 2..end].trim();
            // An unknown placeholder renders empty rather than leaking braces.
            if let Some((_, value)) = vars.iter().find(|(k, _)| *k == key) {
                out.push_str(value);
            }
            rest = &rest[end + 2..];
        }
        out.push_str(rest);
        out
    }
}

pub fn escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(c),
        }
    }
    out
}
