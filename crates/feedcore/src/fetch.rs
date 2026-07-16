//! HTTP fetching with conditional GET (ETag / Last-Modified).

use crate::error::{FeedError, Result};
use crate::store::FeedFetchInfo;
use std::io::Read;
use std::time::Duration;

/// Outcome of a conditional fetch.
pub enum FetchOutcome {
    /// Server returned 304; nothing changed.
    NotModified,
    /// Server returned a fresh body plus (optional) new validators.
    Fetched {
        body: Vec<u8>,
        etag: Option<String>,
        last_modified: Option<String>,
    },
}

fn agent() -> ureq::Agent {
    ureq::AgentBuilder::new()
        .timeout(Duration::from_secs(30))
        .user_agent("feedhub/0.1 (+https://example.invalid/feedhub)")
        .build()
}

/// Fetch a feed, sending `If-None-Match` / `If-Modified-Since` when validators
/// are known. Network, DNS, and HTTP (>= 400) failures map to
/// [`FeedError::Fetch`].
pub fn fetch(info: &FeedFetchInfo) -> Result<FetchOutcome> {
    fetch_with_agent(&agent(), info)
}

fn fetch_with_agent(agent: &ureq::Agent, info: &FeedFetchInfo) -> Result<FetchOutcome> {
    let mut req = agent.get(&info.url);
    if let Some(etag) = &info.etag {
        req = req.set("If-None-Match", etag);
    }
    if let Some(lm) = &info.last_modified {
        req = req.set("If-Modified-Since", lm);
    }

    match req.call() {
        Ok(resp) => build_outcome(resp),
        Err(ureq::Error::Status(304, _)) => Ok(FetchOutcome::NotModified),
        Err(ureq::Error::Status(code, resp)) => Err(FeedError::Fetch(format!(
            "HTTP {code} {}",
            resp.status_text()
        ))),
        Err(ureq::Error::Transport(t)) => Err(FeedError::Fetch(format!("transport error: {t}"))),
    }
}

fn build_outcome(resp: ureq::Response) -> Result<FetchOutcome> {
    if resp.status() == 304 {
        return Ok(FetchOutcome::NotModified);
    }
    let etag = resp.header("ETag").map(|s| s.to_string());
    let last_modified = resp.header("Last-Modified").map(|s| s.to_string());
    let mut body = Vec::new();
    resp.into_reader()
        .take(50 * 1024 * 1024)
        .read_to_end(&mut body)
        .map_err(|e| FeedError::Fetch(format!("read error: {e}")))?;
    Ok(FetchOutcome::Fetched {
        body,
        etag,
        last_modified,
    })
}
