//! HTTP fetching with conditional GET support.

use feedcore::{parse_feed, ParsedFeed};
use std::io::Read;

/// Outcome of a successful HTTP fetch attempt.
pub enum FetchOutcome {
    /// Server returned 304; stored entries are untouched.
    NotModified,
    /// A fresh body was fetched and parsed.
    Fetched {
        feed: ParsedFeed,
        etag: Option<String>,
        last_modified: Option<String>,
    },
}

/// Fetch and parse a feed, sending conditional-GET headers when available.
///
/// Returns `Err(message)` for connection failures, HTTP error statuses, and
/// malformed XML — the caller records this on the feed without affecting others.
pub fn fetch_feed(
    url: &str,
    etag: Option<&str>,
    last_modified: Option<&str>,
) -> Result<FetchOutcome, String> {
    let mut req = ureq::get(url);
    if let Some(tag) = etag {
        req = req.set("If-None-Match", tag);
    }
    if let Some(lm) = last_modified {
        req = req.set("If-Modified-Since", lm);
    }

    match req.call() {
        Ok(resp) => {
            if resp.status() == 304 {
                return Ok(FetchOutcome::NotModified);
            }
            let etag = resp.header("ETag").map(str::to_string);
            let last_modified = resp.header("Last-Modified").map(str::to_string);
            let mut bytes: Vec<u8> = Vec::new();
            resp.into_reader()
                .take(64 * 1024 * 1024)
                .read_to_end(&mut bytes)
                .map_err(|e| format!("read error: {e}"))?;
            let feed = parse_feed(&bytes).map_err(|e| e.to_string())?;
            Ok(FetchOutcome::Fetched {
                feed,
                etag,
                last_modified,
            })
        }
        Err(ureq::Error::Status(304, _)) => Ok(FetchOutcome::NotModified),
        Err(ureq::Error::Status(code, resp)) => {
            let status_text = resp.status_text().to_string();
            Err(format!("HTTP {code} {status_text}"))
        }
        Err(ureq::Error::Transport(t)) => Err(format!("connection error: {t}")),
    }
}
