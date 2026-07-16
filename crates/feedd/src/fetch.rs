//! Fetching feeds over HTTP, with conditional GET.

use std::time::Duration;

use feedhub_core::parser::{self, ParseError};
use reqwest::header::{ETAG, IF_MODIFIED_SINCE, IF_NONE_MATCH, LAST_MODIFIED};
use reqwest::{StatusCode, Url};

use crate::store::{FetchState, FetchSuccess};

/// Cap on a feed body. A hostile or broken origin must not be able to exhaust
/// memory.
const MAX_BODY_BYTES: usize = 32 * 1024 * 1024;

/// Whole-request budget. reqwest has no default timeout, so without this one
/// unresponsive origin would wedge the poll loop indefinitely.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);
const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Debug, thiserror::Error)]
pub enum FetchError {
    #[error("request failed: {0}")]
    Network(String),
    #[error("HTTP {0}")]
    Http(u16),
    #[error("response too large: {0} bytes exceeds the {1} byte limit")]
    TooLarge(usize, usize),
    #[error("{0}")]
    Parse(#[from] ParseError),
}

/// What a fetch produced.
#[derive(Debug)]
pub enum Fetched {
    /// A 200 with a parsed body.
    Body(Box<FetchSuccess>),
    /// A 304. Entries stay as they are, and this still counts as a success.
    NotModified,
}

/// Is this a URL we are willing to register and fetch?
///
/// Only http(s), and only with a host. Rejecting everything else here is what
/// keeps `file://` and friends out of the fetch path.
pub fn validate_feed_url(raw: &str) -> Result<(), String> {
    let url = Url::parse(raw).map_err(|e| format!("not a valid URL: {e}"))?;
    match url.scheme() {
        "http" | "https" => {}
        other => return Err(format!("unsupported URL scheme {other:?}: expected http or https")),
    }
    if url.host().is_none() {
        return Err("URL has no host".to_string());
    }
    Ok(())
}

pub struct Fetcher {
    client: reqwest::Client,
}

impl Fetcher {
    pub fn new() -> anyhow::Result<Self> {
        let client = reqwest::Client::builder()
            .timeout(REQUEST_TIMEOUT)
            .connect_timeout(CONNECT_TIMEOUT)
            .redirect(reqwest::redirect::Policy::limited(5))
            .user_agent(concat!("feedhub/", env!("CARGO_PKG_VERSION")))
            .build()?;
        Ok(Self { client })
    }

    /// Fetch one feed, sending the stored validators.
    pub async fn fetch(&self, state: &FetchState) -> Result<Fetched, FetchError> {
        let mut req = self.client.get(&state.url);
        // Conditional GET: these are the whole point of storing the validators.
        if let Some(etag) = &state.etag {
            req = req.header(IF_NONE_MATCH, etag);
        }
        if let Some(last_modified) = &state.last_modified {
            req = req.header(IF_MODIFIED_SINCE, last_modified);
        }

        let res = req
            .send()
            .await
            .map_err(|e| FetchError::Network(friendly_error(&e)))?;

        if res.status() == StatusCode::NOT_MODIFIED {
            return Ok(Fetched::NotModified);
        }
        if !res.status().is_success() {
            return Err(FetchError::Http(res.status().as_u16()));
        }

        let header = |name: &reqwest::header::HeaderName| {
            res.headers()
                .get(name)
                .and_then(|v| v.to_str().ok())
                .map(str::to_string)
        };
        let etag = header(&ETAG);
        let last_modified = header(&LAST_MODIFIED);

        // Refuse an oversized body before buffering it, when the origin says so.
        if let Some(len) = res.content_length() {
            if len > MAX_BODY_BYTES as u64 {
                return Err(FetchError::TooLarge(len as usize, MAX_BODY_BYTES));
            }
        }

        // bytes(), never text(): text() guesses an encoding from the
        // Content-Type charset and would mangle the body before the parser sees
        // it. The spec pins UTF-8, and the parser is what validates that.
        let body = res
            .bytes()
            .await
            .map_err(|e| FetchError::Network(friendly_error(&e)))?;
        if body.len() > MAX_BODY_BYTES {
            return Err(FetchError::TooLarge(body.len(), MAX_BODY_BYTES));
        }

        let outcome = parser::parse_feed(&body)?;
        if outcome.skipped_without_identity > 0 {
            tracing::warn!(
                url = %state.url,
                skipped = outcome.skipped_without_identity,
                "dropped items with no guid/id: they have no stable dedupe key"
            );
        }

        Ok(Fetched::Body(Box::new(FetchSuccess {
            title: outcome.feed.title,
            entries: outcome.feed.entries,
            etag,
            last_modified,
        })))
    }
}

/// reqwest's Display is a chain of "error sending request for url (...)". Pull
/// out the useful part so `last_error` reads like something a human wrote.
fn friendly_error(e: &reqwest::Error) -> String {
    if e.is_timeout() {
        return "timed out".to_string();
    }
    if e.is_connect() {
        return "connection failed".to_string();
    }
    let mut source: &dyn std::error::Error = e;
    while let Some(next) = source.source() {
        source = next;
    }
    source.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn http_and_https_urls_are_accepted() {
        assert!(validate_feed_url("http://example.com/feed.rss").is_ok());
        assert!(validate_feed_url("https://example.com/feed.rss").is_ok());
        assert!(validate_feed_url("http://127.0.0.1:8601/a.rss").is_ok());
    }

    #[test]
    fn non_http_urls_are_rejected() {
        for bad in [
            "file:///etc/passwd",
            "ftp://example.com/feed.rss",
            "javascript:alert(1)",
            "not a url at all",
            "",
            "example.com/feed.rss", // no scheme
        ] {
            assert!(validate_feed_url(bad).is_err(), "{bad:?} must be rejected");
        }
    }
}
