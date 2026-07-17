//! A local static HTTP server for feed fixtures, plus the fixture corpus.
//!
//! `feedgen` exists so feedhub can be developed and tested without touching the
//! real internet. It serves a directory with content-derived ETags and real
//! `Last-Modified` stamps, and honors conditional GET — which is what makes
//! `feedd`'s 304 path testable.

#![forbid(unsafe_code)]

pub mod cli;
pub mod fixtures;

use std::net::SocketAddr;
use std::path::{Path as FsPath, PathBuf};
use std::sync::{Arc, Mutex};

use axum::extract::{Path, State};
use axum::http::header::{CONTENT_TYPE, ETAG, IF_MODIFIED_SINCE, IF_NONE_MATCH, LAST_MODIFIED};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::Router;
use chrono::{DateTime, Utc};

/// A request as feedgen received it.
///
/// Recorded so tests can assert what a client *sent* — verifying that `feedd`
/// actually emits `If-None-Match`/`If-Modified-Since` on refetch, rather than
/// only that feedgen would answer 304 if it did.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecordedRequest {
    pub path: String,
    pub if_none_match: Option<String>,
    pub if_modified_since: Option<String>,
}

/// Shared, cloneable log of received requests.
#[derive(Clone, Default)]
pub struct RequestLog(Arc<Mutex<Vec<RecordedRequest>>>);

impl RequestLog {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn snapshot(&self) -> Vec<RecordedRequest> {
        self.0.lock().expect("request log poisoned").clone()
    }

    pub fn len(&self) -> usize {
        self.0.lock().expect("request log poisoned").len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn record(&self, req: RecordedRequest) {
        self.0.lock().expect("request log poisoned").push(req);
    }
}

#[derive(Clone, Debug)]
pub struct Options {
    pub dir: PathBuf,
    /// When false, ignore `If-None-Match`/`If-Modified-Since` and always answer
    /// `200`.
    ///
    /// Tests use this to force `feedd` down its re-parse and dedupe path with
    /// identical content — the path a 304 would otherwise hide.
    pub conditional: bool,
}

impl Options {
    pub fn new(dir: impl Into<PathBuf>) -> Self {
        Self {
            dir: dir.into(),
            conditional: true,
        }
    }

    pub fn conditional(mut self, on: bool) -> Self {
        self.conditional = on;
        self
    }
}

#[derive(Clone)]
struct AppState {
    opts: Options,
    log: RequestLog,
}

/// FNV-1a. Small, dependency-free, and deterministic across runs — an ETag that
/// changed on restart would defeat the whole point of conditional GET.
fn fnv1a(bytes: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for b in bytes {
        hash ^= u64::from(*b);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

/// A strong, content-derived ETag.
pub fn etag_for(bytes: &[u8]) -> String {
    format!("\"{:016x}\"", fnv1a(bytes))
}

/// Map a file extension to a content type.
pub fn content_type_for(path: &FsPath) -> &'static str {
    match path.extension().and_then(|e| e.to_str()) {
        Some("rss") => "application/rss+xml; charset=utf-8",
        Some("atom") => "application/atom+xml; charset=utf-8",
        Some("xml") => "application/xml; charset=utf-8",
        Some("md") => "text/markdown; charset=utf-8",
        Some("txt") => "text/plain; charset=utf-8",
        Some("json") => "application/json",
        _ => "application/octet-stream",
    }
}

/// Format an instant as an HTTP-date (RFC 9110 IMF-fixdate).
pub fn http_date(t: DateTime<Utc>) -> String {
    t.format("%a, %d %b %Y %H:%M:%S GMT").to_string()
}

/// Join `rel` onto `dir`, refusing anything that escapes the directory.
fn safe_join(dir: &FsPath, rel: &str) -> Option<PathBuf> {
    let rel = FsPath::new(rel);
    // Only plain names. No `..`, no absolute paths, no root/prefix components.
    if !rel
        .components()
        .all(|c| matches!(c, std::path::Component::Normal(_)))
    {
        return None;
    }
    Some(dir.join(rel))
}

/// Does an `If-None-Match` header match our ETag?
///
/// Handles `*`, comma-separated lists, and the weak `W/` prefix.
fn etag_matches(header: &str, etag: &str) -> bool {
    if header.trim() == "*" {
        return true;
    }
    header.split(',').map(str::trim).any(|candidate| {
        let candidate = candidate.strip_prefix("W/").unwrap_or(candidate);
        candidate == etag
    })
}

fn error_response(status: StatusCode, message: &str) -> Response {
    (status, message.to_string()).into_response()
}

async fn serve_file(
    State(state): State<AppState>,
    Path(path): Path<String>,
    headers: HeaderMap,
) -> Response {
    let header_str = |name: &axum::http::HeaderName| {
        headers
            .get(name)
            .and_then(|v| v.to_str().ok())
            .map(str::to_string)
    };
    let if_none_match = header_str(&IF_NONE_MATCH);
    let if_modified_since = header_str(&IF_MODIFIED_SINCE);

    state.log.record(RecordedRequest {
        path: path.clone(),
        if_none_match: if_none_match.clone(),
        if_modified_since: if_modified_since.clone(),
    });

    let Some(full) = safe_join(&state.opts.dir, &path) else {
        return error_response(StatusCode::BAD_REQUEST, "invalid path");
    };
    let Ok(bytes) = tokio::fs::read(&full).await else {
        return error_response(StatusCode::NOT_FOUND, "not found");
    };

    let etag = etag_for(&bytes);
    let modified: Option<DateTime<Utc>> = tokio::fs::metadata(&full)
        .await
        .ok()
        .and_then(|m| m.modified().ok())
        .map(DateTime::<Utc>::from);

    let mut base = HeaderMap::new();
    base.insert(ETAG, etag.parse().expect("etag is header-safe"));
    if let Some(m) = modified {
        if let Ok(v) = http_date(m).parse() {
            base.insert(LAST_MODIFIED, v);
        }
    }

    if state.opts.conditional {
        let etag_hit = if_none_match
            .as_deref()
            .is_some_and(|h| etag_matches(h, &etag));

        // Per RFC 9110 §13.1.3, If-None-Match takes precedence and
        // If-Modified-Since is only consulted in its absence.
        let time_hit = !etag_hit
            && if_none_match.is_none()
            && match (if_modified_since.as_deref(), modified) {
                (Some(raw), Some(m)) => feedhub_core::datetime::parse_rfc822(raw)
                    .is_some_and(|since| m.timestamp() <= since.timestamp()),
                _ => false,
            };

        if etag_hit || time_hit {
            return (StatusCode::NOT_MODIFIED, base).into_response();
        }
    }

    base.insert(
        CONTENT_TYPE,
        content_type_for(&full)
            .parse()
            .expect("content type is header-safe"),
    );
    (StatusCode::OK, base, bytes).into_response()
}

/// Build the router for a fixture directory.
pub fn router(opts: Options, log: RequestLog) -> Router {
    Router::new()
        .route("/{*path}", get(serve_file))
        .with_state(AppState { opts, log })
}

/// Serve `listener` until the future is dropped.
///
/// Takes an already-bound listener so the caller can read `local_addr()` before
/// the server starts. That is what lets tests use an ephemeral port with no
/// readiness polling and no sleeps.
pub async fn serve(
    listener: std::net::TcpListener,
    opts: Options,
    log: RequestLog,
) -> anyhow::Result<()> {
    listener.set_nonblocking(true)?;
    let listener = tokio::net::TcpListener::from_std(listener)?;
    axum::serve(listener, router(opts, log)).await?;
    Ok(())
}

/// A feedgen running in the background on an ephemeral port.
pub struct Spawned {
    pub addr: SocketAddr,
    pub log: RequestLog,
    pub handle: tokio::task::JoinHandle<()>,
}

impl Spawned {
    pub fn base_url(&self) -> String {
        format!("http://{}", self.addr)
    }

    /// URL for a file in the served directory.
    pub fn url(&self, name: &str) -> String {
        format!("http://{}/{}", self.addr, name)
    }
}

impl Drop for Spawned {
    fn drop(&mut self) {
        self.handle.abort();
    }
}

/// Bind `127.0.0.1:0` and serve in a background task.
pub fn spawn(opts: Options) -> anyhow::Result<Spawned> {
    let listener = std::net::TcpListener::bind("127.0.0.1:0")?;
    let addr = listener.local_addr()?;
    let log = RequestLog::new();
    let log_for_task = log.clone();
    let handle = tokio::spawn(async move {
        let _ = serve(listener, opts, log_for_task).await;
    });
    Ok(Spawned { addr, log, handle })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn etag_is_content_derived_and_stable() {
        assert_eq!(
            etag_for(b"hello"),
            etag_for(b"hello"),
            "stable for equal content"
        );
        assert_ne!(
            etag_for(b"hello"),
            etag_for(b"hellp"),
            "differs on content change"
        );
        assert!(
            etag_for(b"hello").starts_with('"'),
            "must be a quoted-string"
        );
    }

    #[test]
    fn etag_matching_handles_star_lists_and_weak_prefixes() {
        let etag = etag_for(b"x");
        assert!(etag_matches("*", &etag));
        assert!(etag_matches(&etag, &etag));
        assert!(etag_matches(&format!("W/{etag}"), &etag));
        assert!(etag_matches(&format!("\"other\", {etag}"), &etag));
        assert!(!etag_matches("\"nope\"", &etag));
    }

    #[test]
    fn content_types_follow_the_extension() {
        let ct = |n: &str| content_type_for(FsPath::new(n));
        assert_eq!(ct("a.rss"), "application/rss+xml; charset=utf-8");
        assert_eq!(ct("a.atom"), "application/atom+xml; charset=utf-8");
        assert_eq!(ct("a.xml"), "application/xml; charset=utf-8");
        assert_eq!(ct("a.md"), "text/markdown; charset=utf-8");
        assert_eq!(ct("a.bin"), "application/octet-stream");
    }

    #[test]
    fn safe_join_refuses_traversal() {
        let dir = FsPath::new("/srv/fixtures");
        assert!(safe_join(dir, "a.rss").is_some());
        assert!(safe_join(dir, "sub/a.rss").is_some());
        assert!(safe_join(dir, "../secrets").is_none());
        assert!(safe_join(dir, "a/../../secrets").is_none());
        assert!(safe_join(dir, "/etc/passwd").is_none());
    }

    #[test]
    fn http_date_is_imf_fixdate() {
        let t = feedhub_core::datetime::parse_rfc3339("1994-11-15T12:45:26Z").unwrap();
        assert_eq!(http_date(t), "Tue, 15 Nov 1994 12:45:26 GMT");
        // Round-trips through the RFC 822 parser feedd uses.
        assert_eq!(feedhub_core::datetime::parse_rfc822(&http_date(t)), Some(t));
    }
}
