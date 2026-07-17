//! Static file server with the conditional-GET behavior `feedd` is tested
//! against: a content-derived `ETag`, a `Last-Modified` from the file's mtime,
//! and `304 Not Modified` for `If-None-Match` / `If-Modified-Since`.

use std::net::SocketAddr;
use std::path::{Component, Path, PathBuf};

use anyhow::Context;
use axum::Router;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode, Uri, header};
use axum::response::{IntoResponse, Response};
use axum::routing::any;
use chrono::{DateTime, Timelike, Utc};
use sha2::{Digest, Sha256};
use tokio::net::TcpListener;
use tokio::sync::oneshot;
use tokio::task::JoinHandle;

use feedhub_core::parse_rfc822;

/// A server that is listening, with the address it actually bound to.
///
/// Tests bind port 0 and read `addr` back, so they never collide on a fixed
/// port or race against a port that is still in TIME_WAIT.
pub struct RunningServer {
    pub addr: SocketAddr,
    shutdown: Option<oneshot::Sender<()>>,
    handle: JoinHandle<()>,
}

impl RunningServer {
    /// The base URL to fetch from, e.g. `http://127.0.0.1:41234`.
    pub fn base_url(&self) -> String {
        format!("http://{}", self.addr)
    }

    /// Ask the server to stop and wait for it to finish.
    pub async fn shutdown(mut self) {
        if let Some(tx) = self.shutdown.take() {
            let _ = tx.send(());
        }
        let _ = (&mut self.handle).await;
    }
}

/// Start serving `dir` on `addr`. Binding happens before this returns, so a
/// caller that gets `Ok` can immediately connect.
pub async fn serve_dir(dir: PathBuf, addr: SocketAddr) -> anyhow::Result<RunningServer> {
    let dir = dir
        .canonicalize()
        .with_context(|| format!("fixture directory {} is not readable", dir.display()))?;

    let app = Router::new()
        .fallback(any(handle))
        .with_state(std::sync::Arc::new(dir));

    let listener = TcpListener::bind(addr)
        .await
        .with_context(|| format!("cannot listen on {addr}"))?;
    let addr = listener.local_addr()?;

    let (tx, rx) = oneshot::channel();
    let handle = tokio::spawn(async move {
        let server = axum::serve(listener, app).with_graceful_shutdown(async {
            let _ = rx.await;
        });
        if let Err(e) = server.await {
            eprintln!("feedgen: server error: {e}");
        }
    });

    Ok(RunningServer {
        addr,
        shutdown: Some(tx),
        handle,
    })
}

type Dir = std::sync::Arc<PathBuf>;

async fn handle(State(dir): State<Dir>, uri: Uri, headers: HeaderMap) -> Response {
    let path = uri.path();
    if path == "/" {
        return index(&dir).await;
    }

    let Some(file) = resolve(&dir, path) else {
        return error(StatusCode::NOT_FOUND, "not found");
    };

    let Ok(bytes) = tokio::fs::read(&file).await else {
        return error(StatusCode::NOT_FOUND, "not found");
    };
    let modified = match tokio::fs::metadata(&file).await.and_then(|m| m.modified()) {
        Ok(t) => DateTime::<Utc>::from(t),
        Err(_) => Utc::now(),
    };
    // HTTP dates have one-second resolution; comparing against an mtime with
    // sub-second precision would make a just-written file look newer forever.
    let modified = modified
        .with_timezone(&Utc)
        .with_nanosecond(0)
        .unwrap_or(modified);
    let etag = etag_for(&bytes);

    let mut response_headers = HeaderMap::new();
    insert(&mut response_headers, header::ETAG, &etag);
    insert(
        &mut response_headers,
        header::LAST_MODIFIED,
        &http_date(modified),
    );
    insert(
        &mut response_headers,
        header::CONTENT_TYPE,
        content_type(&file, &bytes),
    );

    if not_modified(&headers, &etag, modified) {
        return (StatusCode::NOT_MODIFIED, response_headers).into_response();
    }

    (StatusCode::OK, response_headers, bytes).into_response()
}

/// RFC 9110: when both validators are present `If-None-Match` decides, because
/// it is the stronger one. A file rewritten within the same second has an
/// unchanged mtime but a different ETag, and must not answer 304.
fn not_modified(headers: &HeaderMap, etag: &str, modified: DateTime<Utc>) -> bool {
    if let Some(inm) = headers
        .get(header::IF_NONE_MATCH)
        .and_then(|v| v.to_str().ok())
    {
        return inm.trim() == "*"
            || inm
                .split(',')
                .map(|candidate| candidate.trim().trim_start_matches("W/"))
                .any(|candidate| candidate == etag);
    }

    if let Some(ims) = headers
        .get(header::IF_MODIFIED_SINCE)
        .and_then(|v| v.to_str().ok())
        .and_then(parse_rfc822)
    {
        return modified <= ims;
    }

    false
}

/// A strong ETag derived from the bytes, so any edit changes it even when the
/// mtime does not.
fn etag_for(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let hex: String = digest.iter().map(|byte| format!("{byte:02x}")).collect();
    format!("\"{hex}\"")
}

fn http_date(dt: DateTime<Utc>) -> String {
    dt.format("%a, %d %b %Y %H:%M:%S GMT").to_string()
}

/// Map a URL path to a file inside `dir`, rejecting anything that tries to
/// escape it.
fn resolve(dir: &Path, url_path: &str) -> Option<PathBuf> {
    let relative = Path::new(url_path.trim_start_matches('/'));
    if relative.as_os_str().is_empty() {
        return None;
    }
    if !relative
        .components()
        .all(|c| matches!(c, Component::Normal(_)))
    {
        return None;
    }
    let candidate = dir.join(relative).canonicalize().ok()?;
    if !candidate.starts_with(dir) || !candidate.is_file() {
        return None;
    }
    Some(candidate)
}

/// Prefer what the document actually is over what it is named: fixtures are
/// `.xml` files, but an RSS document should still be served as
/// `application/rss+xml`.
fn content_type(file: &Path, bytes: &[u8]) -> &'static str {
    let extension = file
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();

    let is_xmlish = matches!(extension.as_str(), "xml" | "rss" | "atom");
    if is_xmlish {
        let head = &bytes[..bytes.len().min(512)];
        let head = String::from_utf8_lossy(head);
        if head.contains("<rss") {
            return "application/rss+xml; charset=utf-8";
        }
        if head.contains("<feed") {
            return "application/atom+xml; charset=utf-8";
        }
    }

    match extension.as_str() {
        "xml" => "application/xml; charset=utf-8",
        "rss" => "application/rss+xml; charset=utf-8",
        "atom" => "application/atom+xml; charset=utf-8",
        "json" => "application/json",
        "txt" => "text/plain; charset=utf-8",
        "md" => "text/markdown; charset=utf-8",
        "html" => "text/html; charset=utf-8",
        _ => "application/octet-stream",
    }
}

/// A plain-text listing of the corpus, so `curl`ing the root tells you what is
/// available to fetch.
async fn index(dir: &Path) -> Response {
    let Ok(mut entries) = tokio::fs::read_dir(dir).await else {
        return error(StatusCode::INTERNAL_SERVER_ERROR, "cannot read directory");
    };

    let mut names = Vec::new();
    while let Ok(Some(entry)) = entries.next_entry().await {
        if entry.path().is_file() {
            names.push(entry.file_name().to_string_lossy().into_owned());
        }
    }
    names.sort();

    let body = names
        .into_iter()
        .map(|n| format!("/{n}\n"))
        .collect::<String>();
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "text/plain; charset=utf-8")],
        body,
    )
        .into_response()
}

fn error(status: StatusCode, message: &str) -> Response {
    (
        status,
        [(header::CONTENT_TYPE, "text/plain; charset=utf-8")],
        format!("{message}\n"),
    )
        .into_response()
}

fn insert(headers: &mut HeaderMap, name: header::HeaderName, value: &str) {
    if let Ok(value) = value.parse() {
        headers.insert(name, value);
    }
}
