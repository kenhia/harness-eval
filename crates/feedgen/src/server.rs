//! Minimal HTTP/1.1 static-file server with conditional-GET support.

use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use anyhow::{Context, Result};
use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::task::JoinHandle;

/// A running feedgen server bound to a local address.
pub struct Server {
    /// The actual bound address (useful when binding to port 0 in tests).
    pub addr: SocketAddr,
    handle: JoinHandle<()>,
}

impl Server {
    /// The base URL (`http://ADDR`) the server is reachable at.
    pub fn base_url(&self) -> String {
        format!("http://{}", self.addr)
    }

    /// Stop the server task.
    pub fn shutdown(self) {
        self.handle.abort();
    }
}

/// Bind and start serving `dir` on `listen`, returning once the socket is
/// bound. The server runs on a background task.
pub async fn spawn(dir: impl Into<PathBuf>, listen: &str) -> Result<Server> {
    let dir = dir.into();
    let listener = TcpListener::bind(listen)
        .await
        .with_context(|| format!("binding {listen}"))?;
    let addr = listener.local_addr()?;
    let handle = tokio::spawn(async move {
        accept_loop(listener, dir).await;
    });
    Ok(Server { addr, handle })
}

/// Bind and serve `dir` on `listen`, running until the task is cancelled.
pub async fn serve_forever(dir: impl Into<PathBuf>, listen: &str) -> Result<()> {
    let dir = dir.into();
    let listener = TcpListener::bind(listen)
        .await
        .with_context(|| format!("binding {listen}"))?;
    accept_loop(listener, dir).await;
    Ok(())
}

async fn accept_loop(listener: TcpListener, dir: PathBuf) {
    while let Ok((stream, _peer)) = listener.accept().await {
        let dir = dir.clone();
        tokio::spawn(async move {
            let _ = handle_conn(stream, &dir).await;
        });
    }
}

struct Request {
    method: String,
    path: String,
    headers: HashMap<String, String>,
}

async fn read_request(stream: &mut BufReader<&mut TcpStream>) -> Result<Option<Request>> {
    let mut line = String::new();
    let n = stream.read_line(&mut line).await?;
    if n == 0 {
        return Ok(None);
    }
    let mut parts = line.split_whitespace();
    let method = parts.next().unwrap_or_default().to_string();
    let raw_path = parts.next().unwrap_or_default().to_string();
    let path = raw_path.split(['?', '#']).next().unwrap_or("").to_string();

    let mut headers = HashMap::new();
    loop {
        let mut hline = String::new();
        let hn = stream.read_line(&mut hline).await?;
        if hn == 0 || hline == "\r\n" || hline == "\n" {
            break;
        }
        if let Some((k, v)) = hline.split_once(':') {
            headers.insert(k.trim().to_ascii_lowercase(), v.trim().to_string());
        }
    }
    Ok(Some(Request {
        method,
        path,
        headers,
    }))
}

async fn handle_conn(mut stream: TcpStream, dir: &Path) -> Result<()> {
    let req = {
        let mut reader = BufReader::new(&mut stream);
        let req = match read_request(&mut reader).await? {
            Some(r) => r,
            None => return Ok(()),
        };
        // Drain any request body (none expected for GET/HEAD).
        if let Some(len) = req
            .headers
            .get("content-length")
            .and_then(|v| v.parse::<usize>().ok())
        {
            let mut body = vec![0u8; len];
            let _ = reader.read_exact(&mut body).await;
        }
        req
    };

    let response = build_response(&req, dir);
    stream.write_all(&response).await?;
    stream.flush().await?;
    Ok(())
}

fn build_response(req: &Request, dir: &Path) -> Vec<u8> {
    if req.method != "GET" && req.method != "HEAD" {
        return simple(405, "Method Not Allowed", "method not allowed");
    }
    let Some(path) = safe_path(dir, &req.path) else {
        return simple(403, "Forbidden", "forbidden");
    };
    let bytes = match std::fs::read(&path) {
        Ok(b) => b,
        Err(_) => return simple(404, "Not Found", "not found"),
    };
    let etag = etag_for(&bytes);
    let last_modified = std::fs::metadata(&path)
        .and_then(|m| m.modified())
        .ok()
        .map(system_time_to_http_date);

    if is_not_modified(req, &etag, &last_modified) {
        return not_modified(&etag, last_modified.as_deref());
    }

    let ctype = mime_for(&path);
    let mut head = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: {ctype}\r\nContent-Length: {}\r\nETag: {etag}\r\nConnection: close\r\n",
        bytes.len()
    );
    if let Some(lm) = &last_modified {
        head.push_str(&format!("Last-Modified: {lm}\r\n"));
    }
    head.push_str("\r\n");
    let mut out = head.into_bytes();
    if req.method == "GET" {
        out.extend_from_slice(&bytes);
    }
    out
}

fn is_not_modified(req: &Request, etag: &str, last_modified: &Option<String>) -> bool {
    if let Some(inm) = req.headers.get("if-none-match") {
        if inm.trim() == "*" || inm.split(',').any(|t| t.trim() == etag) {
            return true;
        }
    }
    if let (Some(ims), Some(lm)) = (req.headers.get("if-modified-since"), last_modified) {
        if let (Some(ims_t), Some(lm_t)) = (parse_http_date(ims), parse_http_date(lm)) {
            if lm_t <= ims_t {
                return true;
            }
        }
    }
    false
}

/// Compute a stable, content-derived ETag (quoted per HTTP).
pub fn etag_for(bytes: &[u8]) -> String {
    // FNV-1a 64-bit — deterministic across runs.
    let mut hash: u64 = 0xcbf29ce484222325;
    for &b in bytes {
        hash ^= b as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("\"{hash:016x}\"")
}

/// Guess a content type from the file extension.
pub fn mime_for(path: &Path) -> &'static str {
    match path.extension().and_then(|e| e.to_str()) {
        Some("xml") => "application/xml; charset=utf-8",
        Some("rss") => "application/rss+xml; charset=utf-8",
        Some("atom") => "application/atom+xml; charset=utf-8",
        Some("json") => "application/json; charset=utf-8",
        Some("txt") => "text/plain; charset=utf-8",
        Some("html") => "text/html; charset=utf-8",
        _ => "application/octet-stream",
    }
}

fn safe_path(dir: &Path, req_path: &str) -> Option<PathBuf> {
    let rel = req_path.trim_start_matches('/');
    let rel = if rel.is_empty() { "index.html" } else { rel };
    // Reject traversal.
    if rel.split('/').any(|seg| seg == "..") {
        return None;
    }
    Some(dir.join(rel))
}

fn simple(code: u16, reason: &str, body: &str) -> Vec<u8> {
    format!(
        "HTTP/1.1 {code} {reason}\r\nContent-Type: text/plain; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    )
    .into_bytes()
}

fn not_modified(etag: &str, last_modified: Option<&str>) -> Vec<u8> {
    let mut head = format!("HTTP/1.1 304 Not Modified\r\nETag: {etag}\r\nConnection: close\r\n");
    if let Some(lm) = last_modified {
        head.push_str(&format!("Last-Modified: {lm}\r\n"));
    }
    head.push_str("\r\n");
    head.into_bytes()
}

fn system_time_to_http_date(t: SystemTime) -> String {
    let dt: DateTime<Utc> = t.into();
    dt.format("%a, %d %b %Y %H:%M:%S GMT").to_string()
}

fn parse_http_date(s: &str) -> Option<DateTime<Utc>> {
    let s = s.trim();
    let naive = NaiveDateTime::parse_from_str(s, "%a, %d %b %Y %H:%M:%S GMT").ok()?;
    Some(Utc.from_utc_datetime(&naive))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn etag_stable_and_content_derived() {
        assert_eq!(etag_for(b"abc"), etag_for(b"abc"));
        assert_ne!(etag_for(b"abc"), etag_for(b"abd"));
    }

    #[test]
    fn mime_by_ext() {
        assert_eq!(
            mime_for(&PathBuf::from("a.xml")),
            "application/xml; charset=utf-8"
        );
        assert_eq!(
            mime_for(&PathBuf::from("a.atom")),
            "application/atom+xml; charset=utf-8"
        );
    }

    #[test]
    fn rejects_traversal() {
        assert!(safe_path(Path::new("/srv"), "/../etc/passwd").is_none());
        assert!(safe_path(Path::new("/srv"), "/feed.xml").is_some());
    }
}
