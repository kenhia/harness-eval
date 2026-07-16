//! `feedgen` — fixture generator and static feed server for testing feedhub
//! without touching the real internet.

mod fixtures;

use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use chrono::{DateTime, TimeZone, Utc};
use clap::{Parser, Subcommand};
use tiny_http::{Header, Method, Request, Response, Server};

#[derive(Parser)]
#[command(
    name = "feedgen",
    about = "Serve test feeds over local HTTP and generate a fixture corpus."
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Serve the files in DIR over HTTP with ETag / Last-Modified / 304 support.
    Serve {
        #[arg(long)]
        dir: PathBuf,
        #[arg(long, default_value = "127.0.0.1:8700")]
        listen: String,
    },
    /// Write the fixture corpus into DIR (created if missing).
    MakeFixtures {
        /// Target directory for the fixture files.
        dir: PathBuf,
    },
}

fn main() {
    let cli = Cli::parse();
    let result = match cli.command {
        Command::MakeFixtures { dir } => make_fixtures(&dir),
        Command::Serve { dir, listen } => serve(&dir, &listen),
    };
    if let Err(e) = result {
        eprintln!("feedgen: {e}");
        std::process::exit(1);
    }
}

fn make_fixtures(dir: &Path) -> std::io::Result<()> {
    fs::create_dir_all(dir)?;
    for (name, contents) in fixtures::CORPUS {
        fs::write(dir.join(name), contents)?;
    }
    println!(
        "wrote {} fixture files to {}",
        fixtures::CORPUS.len(),
        dir.display()
    );
    Ok(())
}

fn serve(dir: &Path, listen: &str) -> std::io::Result<()> {
    let dir = dir.to_path_buf();
    let server = Server::http(listen).map_err(|e| std::io::Error::other(e.to_string()))?;
    println!("feedgen serving {} on http://{listen}", dir.display());
    for request in server.incoming_requests() {
        if let Err(e) = handle(&dir, request) {
            eprintln!("feedgen: request error: {e}");
        }
    }
    Ok(())
}

/// Map a request path to a file inside `dir`, rejecting traversal.
fn resolve(dir: &Path, url: &str) -> Option<PathBuf> {
    let path = url.split('?').next().unwrap_or(url);
    let trimmed = path.trim_start_matches('/');
    if trimmed.is_empty() {
        return None;
    }
    let mut out = dir.to_path_buf();
    for segment in trimmed.split('/') {
        if segment == ".." || segment == "." || segment.is_empty() {
            return None;
        }
        out.push(segment);
    }
    Some(out)
}

fn content_type(path: &Path) -> &'static str {
    match path.extension().and_then(|e| e.to_str()) {
        Some("xml") => "application/xml; charset=utf-8",
        Some("rss") => "application/rss+xml; charset=utf-8",
        Some("atom") => "application/atom+xml; charset=utf-8",
        Some("html") => "text/html; charset=utf-8",
        Some("json") => "application/json; charset=utf-8",
        Some("txt") | Some("md") => "text/plain; charset=utf-8",
        _ => "application/octet-stream",
    }
}

fn etag_for(bytes: &[u8]) -> String {
    let mut hasher = DefaultHasher::new();
    bytes.hash(&mut hasher);
    format!("\"{:016x}\"", hasher.finish())
}

fn http_date(t: SystemTime) -> String {
    let dt: DateTime<Utc> = match t.duration_since(SystemTime::UNIX_EPOCH) {
        Ok(d) => Utc
            .timestamp_opt(d.as_secs() as i64, 0)
            .single()
            .unwrap_or_else(Utc::now),
        Err(_) => Utc::now(),
    };
    dt.format("%a, %d %b %Y %H:%M:%S GMT").to_string()
}

fn header(name: &str, value: &str) -> Header {
    Header::from_bytes(name.as_bytes(), value.as_bytes())
        .expect("static header names/values are valid")
}

fn find_header<'a>(req: &'a Request, name: &str) -> Option<&'a str> {
    req.headers()
        .iter()
        .find(|h| h.field.as_str().as_str().eq_ignore_ascii_case(name))
        .map(|h| h.value.as_str())
}

fn handle(dir: &Path, request: Request) -> std::io::Result<()> {
    if !matches!(request.method(), Method::Get | Method::Head) {
        let resp = Response::from_string("method not allowed").with_status_code(405);
        return request.respond(resp);
    }

    let path = match resolve(dir, request.url()) {
        Some(p) => p,
        None => return request.respond(Response::from_string("bad path").with_status_code(400)),
    };

    let bytes = match fs::read(&path) {
        Ok(b) => b,
        Err(_) => return request.respond(Response::from_string("not found").with_status_code(404)),
    };

    let etag = etag_for(&bytes);
    let last_modified = fs::metadata(&path)
        .and_then(|m| m.modified())
        .map(http_date)
        .unwrap_or_else(|_| http_date(SystemTime::now()));

    let inm = find_header(&request, "If-None-Match").map(str::to_string);
    let ims = find_header(&request, "If-Modified-Since").map(str::to_string);
    let not_modified = match (&inm, &ims) {
        (Some(tag), _) => tag == &etag || tag == "*",
        (None, Some(since)) => match_modified_since(&path, since),
        _ => false,
    };

    let common = [
        header("ETag", &etag),
        header("Last-Modified", &last_modified),
        header("Cache-Control", "no-cache"),
    ];

    if not_modified {
        let mut resp = Response::from_data(Vec::new()).with_status_code(304);
        for h in common {
            resp.add_header(h);
        }
        return request.respond(resp);
    }

    let mut resp = Response::new(
        200.into(),
        vec![header("Content-Type", content_type(&path))],
        Cursor::new(bytes),
        None,
        None,
    );
    for h in common {
        resp.add_header(h);
    }
    request.respond(resp)
}

/// True when the file's mtime is not newer than the `If-Modified-Since` date.
fn match_modified_since(path: &Path, since: &str) -> bool {
    let since = match DateTime::parse_from_str(since.trim(), "%a, %d %b %Y %H:%M:%S GMT") {
        Ok(d) => d.with_timezone(&Utc),
        Err(_) => return false,
    };
    let modified = match fs::metadata(path).and_then(|m| m.modified()) {
        Ok(m) => m,
        Err(_) => return false,
    };
    let modified: DateTime<Utc> = match modified.duration_since(SystemTime::UNIX_EPOCH) {
        Ok(d) => match Utc.timestamp_opt(d.as_secs() as i64, 0).single() {
            Some(dt) => dt,
            None => return false,
        },
        Err(_) => return false,
    };
    modified <= since
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_rejects_traversal() {
        let dir = Path::new("/srv");
        assert!(resolve(dir, "/../etc/passwd").is_none());
        assert!(resolve(dir, "/").is_none());
        assert_eq!(
            resolve(dir, "/rss.xml"),
            Some(PathBuf::from("/srv/rss.xml"))
        );
        assert_eq!(
            resolve(dir, "/rss.xml?x=1"),
            Some(PathBuf::from("/srv/rss.xml"))
        );
    }

    #[test]
    fn etag_is_content_derived() {
        assert_eq!(etag_for(b"abc"), etag_for(b"abc"));
        assert_ne!(etag_for(b"abc"), etag_for(b"abd"));
    }

    #[test]
    fn content_type_by_extension() {
        assert_eq!(
            content_type(Path::new("a.xml")),
            "application/xml; charset=utf-8"
        );
        assert_eq!(content_type(Path::new("a.md")), "text/plain; charset=utf-8");
    }
}
