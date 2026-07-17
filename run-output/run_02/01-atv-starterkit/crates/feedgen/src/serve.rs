//! Static file server with content-derived ETag and Last-Modified support.

use chrono::{DateTime, Utc};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use tiny_http::{Header, Response, Server, StatusCode};

pub fn run(dir: &str, listen: &str) -> Result<(), String> {
    let root = fs::canonicalize(dir).map_err(|e| format!("invalid --dir {dir}: {e}"))?;
    let server = Server::http(listen).map_err(|e| e.to_string())?;
    println!("feedgen serving {} on {}", root.display(), listen);

    for request in server.incoming_requests() {
        let url_path = request.url().split('?').next().unwrap_or("/");
        let inm = header_value(&request, "If-None-Match");
        let ims = header_value(&request, "If-Modified-Since");

        match resolve(&root, url_path) {
            Some(path) => match load(&path) {
                Ok(file) => {
                    if inm.as_deref() == Some(&file.etag)
                        || ims.as_deref() == Some(&file.last_modified)
                    {
                        let resp = Response::empty(StatusCode(304))
                            .with_header(etag_header(&file.etag))
                            .with_header(last_modified_header(&file.last_modified));
                        let _ = request.respond(resp);
                    } else {
                        let resp = Response::from_data(file.body)
                            .with_header(content_type_header(&path))
                            .with_header(etag_header(&file.etag))
                            .with_header(last_modified_header(&file.last_modified));
                        let _ = request.respond(resp);
                    }
                }
                Err(_) => respond_status(request, 404, "not found"),
            },
            None => respond_status(request, 404, "not found"),
        }
    }
    Ok(())
}

struct LoadedFile {
    body: Vec<u8>,
    etag: String,
    last_modified: String,
}

fn load(path: &Path) -> std::io::Result<LoadedFile> {
    let body = fs::read(path)?;
    let etag = format!("\"{}\"", content_hash(&body));
    let mtime = fs::metadata(path)?
        .modified()
        .unwrap_or(SystemTime::UNIX_EPOCH);
    let last_modified = http_date(mtime);
    Ok(LoadedFile {
        body,
        etag,
        last_modified,
    })
}

/// Resolve a request path to a file within `root`, rejecting traversal.
fn resolve(root: &Path, url_path: &str) -> Option<PathBuf> {
    let rel = url_path.trim_start_matches('/');
    if rel.is_empty() {
        return None;
    }
    if rel.split('/').any(|seg| seg == ".." || seg == ".") {
        return None;
    }
    let candidate = root.join(rel);
    let canonical = fs::canonicalize(&candidate).ok()?;
    if canonical.starts_with(root) && canonical.is_file() {
        Some(canonical)
    } else {
        None
    }
}

/// A stable, content-derived hash (FNV-1a, 64-bit).
fn content_hash(bytes: &[u8]) -> String {
    let mut hash: u64 = 0xcbf29ce484222325;
    for &b in bytes {
        hash ^= b as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("{hash:016x}")
}

fn http_date(t: SystemTime) -> String {
    let dt: DateTime<Utc> = t.into();
    dt.format("%a, %d %b %Y %H:%M:%S GMT").to_string()
}

fn content_type(path: &Path) -> &'static str {
    match path.extension().and_then(|e| e.to_str()) {
        Some("rss") => "application/rss+xml; charset=utf-8",
        Some("atom") => "application/atom+xml; charset=utf-8",
        Some("xml") => "application/xml; charset=utf-8",
        Some("json") => "application/json",
        Some("txt") => "text/plain; charset=utf-8",
        _ => "application/octet-stream",
    }
}

fn header(name: &str, value: &str) -> Header {
    Header::from_bytes(name.as_bytes(), value.as_bytes()).unwrap()
}

fn content_type_header(path: &Path) -> Header {
    header("Content-Type", content_type(path))
}

fn etag_header(etag: &str) -> Header {
    header("ETag", etag)
}

fn last_modified_header(lm: &str) -> Header {
    header("Last-Modified", lm)
}

fn header_value(request: &tiny_http::Request, name: &str) -> Option<String> {
    request
        .headers()
        .iter()
        .find(|h| h.field.as_str().as_str().eq_ignore_ascii_case(name))
        .map(|h| h.value.as_str().to_string())
}

fn respond_status(request: tiny_http::Request, code: u16, msg: &str) {
    let resp = Response::from_string(msg).with_status_code(code);
    let _ = request.respond(resp);
}
