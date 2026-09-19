//! Tiny development server: std-only HTTP/1.1 for static files, plus a
//! polling watcher that rebuilds when anything under the source tree changes.
//! Not intended to face the internet — it binds to localhost.

use std::fs;
use std::io::{self, BufRead, BufReader, Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::path::{Component, Path, PathBuf};
use std::sync::mpsc;
use std::time::{Duration, SystemTime};

use crate::config::Config;

const WATCHED: [&str; 5] = ["content", "templates", "static", "include", "site.toml"];

pub fn run(dir: &str, port: u16, build: fn() -> io::Result<usize>) {
    let root = Config::load("site.toml").base_path();
    let listener = match TcpListener::bind(("127.0.0.1", port)) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("error: cannot bind 127.0.0.1:{port}: {e}");
            std::process::exit(1);
        }
    };
    spawn_watcher(build);
    println!("serving {dir}/ on http://127.0.0.1:{port}{root} (ctrl-c to stop)");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                if let Err(e) = handle(stream, Path::new(dir), &root) {
                    if e.kind() != io::ErrorKind::BrokenPipe {
                        eprintln!("request failed: {e}");
                    }
                }
            }
            Err(e) => eprintln!("connection failed: {e}"),
        }
    }
}

/// Rebuilds at most once per change, debounced by the poll interval.
fn spawn_watcher(build: fn() -> io::Result<usize>) {
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let mut last = newest_mtime();
        loop {
            std::thread::sleep(Duration::from_millis(400));
            let now = newest_mtime();
            if now != last {
                last = now;
                let _ = tx.send(());
            }
        }
    });
    std::thread::spawn(move || {
        for () in rx {
            match build() {
                Ok(n) => println!("rebuilt {n} pages"),
                Err(e) => eprintln!("build error: {e}"),
            }
        }
    });
}

fn newest_mtime() -> SystemTime {
    fn walk(path: &Path, newest: &mut SystemTime) {
        let Ok(meta) = fs::metadata(path) else { return };
        if let Ok(modified) = meta.modified() {
            if modified > *newest {
                *newest = modified;
            }
        }
        if meta.is_dir() {
            if let Ok(entries) = fs::read_dir(path) {
                for entry in entries.flatten() {
                    walk(&entry.path(), newest);
                }
            }
        }
    }
    let mut newest = SystemTime::UNIX_EPOCH;
    for path in WATCHED {
        walk(Path::new(path), &mut newest);
    }
    newest
}

fn handle(mut stream: TcpStream, dir: &Path, root: &str) -> io::Result<()> {
    stream.set_read_timeout(Some(Duration::from_secs(5)))?;
    let mut reader = BufReader::new(stream.try_clone()?);
    let mut request_line = String::new();
    if reader.read_line(&mut request_line)? == 0 {
        return Ok(());
    }
    // Drain headers so the client sees a clean response.
    let mut header = String::new();
    while reader.read_line(&mut header)? > 2 {
        header.clear();
    }

    let mut parts = request_line.split_whitespace();
    let method = parts.next().unwrap_or("");
    let target = parts.next().unwrap_or("/");
    if method != "GET" && method != "HEAD" {
        return respond(&mut stream, 405, "text/plain; charset=utf-8", b"method not allowed", method);
    }

    let path = target.split(['?', '#']).next().unwrap_or("/");
    let path = decode(path);
    let path = path.strip_prefix(root).map(|p| p.to_string()).unwrap_or_else(|| {
        path.trim_start_matches('/').to_string()
    });

    match resolve(dir, &path) {
        Some(file) => {
            let body = fs::read(&file)?;
            let ctype = content_type(&file);
            println!("  200 {target}");
            respond(&mut stream, 200, ctype, &body, method)
        }
        None => {
            let body = fs::read(dir.join("404.html")).unwrap_or_else(|_| b"not found".to_vec());
            println!("  404 {target}");
            respond(&mut stream, 404, "text/html; charset=utf-8", &body, method)
        }
    }
}

/// Maps a request path to a file inside `dir`, rejecting traversal.
fn resolve(dir: &Path, path: &str) -> Option<PathBuf> {
    let mut candidate = PathBuf::from(path);
    if path.is_empty() || path.ends_with('/') {
        candidate = candidate.join("index.html");
    }
    if candidate.components().any(|c| !matches!(c, Component::Normal(_))) {
        return None;
    }
    let file = dir.join(&candidate);
    if file.is_file() {
        return Some(file);
    }
    // Extensionless URLs fall back to the directory's index.
    let index = file.join("index.html");
    index.is_file().then_some(index)
}

fn respond(
    stream: &mut TcpStream,
    status: u16,
    content_type: &str,
    body: &[u8],
    method: &str,
) -> io::Result<()> {
    let reason = match status {
        200 => "OK",
        404 => "Not Found",
        _ => "Method Not Allowed",
    };
    let head = format!(
        "HTTP/1.1 {status} {reason}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\n\
         Cache-Control: no-store\r\nConnection: close\r\n\r\n",
        body.len()
    );
    stream.write_all(head.as_bytes())?;
    if method != "HEAD" {
        stream.write_all(body)?;
    }
    stream.flush()?;
    let _ = stream.shutdown(Shutdown::Write);
    // Politely drain anything still in flight before dropping the socket.
    let _ = stream.read(&mut [0u8; 64]);
    Ok(())
}

fn content_type(path: &Path) -> &'static str {
    match path.extension().and_then(|e| e.to_str()).unwrap_or("") {
        "html" => "text/html; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "js" => "text/javascript; charset=utf-8",
        "xml" => "application/xml; charset=utf-8",
        "json" => "application/json",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "webp" => "image/webp",
        "gif" => "image/gif",
        "ico" => "image/x-icon",
        "woff2" => "font/woff2",
        "txt" => "text/plain; charset=utf-8",
        "pdf" => "application/pdf",
        _ => "application/octet-stream",
    }
}

fn decode(path: &str) -> String {
    let bytes = path.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let Ok(byte) = u8::from_str_radix(&path[i + 1..i + 3], 16) {
                out.push(byte);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}
