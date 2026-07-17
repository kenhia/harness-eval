//! HTTP fetching with conditional GET.

use std::io::Read;
use std::time::Duration;

/// Outcome of a conditional fetch.
pub enum FetchOutcome {
    /// 200 with a fresh body and optional new validators.
    Fetched {
        body: Vec<u8>,
        etag: Option<String>,
        last_modified: Option<String>,
    },
    /// 304 Not Modified.
    NotModified,
}

const MAX_BODY: u64 = 16 * 1024 * 1024;

/// Fetch `url`, sending `If-None-Match` / `If-Modified-Since` from stored
/// validators. Returns `Err(message)` on connection/HTTP/read failure.
pub fn fetch(
    url: &str,
    etag: Option<&str>,
    last_modified: Option<&str>,
) -> Result<FetchOutcome, String> {
    let agent = ureq::AgentBuilder::new()
        .timeout_connect(Duration::from_secs(10))
        .timeout_read(Duration::from_secs(30))
        .build();

    let mut req = agent.get(url);
    if let Some(e) = etag {
        req = req.set("If-None-Match", e);
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
            let mut body = Vec::new();
            resp.into_reader()
                .take(MAX_BODY)
                .read_to_end(&mut body)
                .map_err(|e| format!("read error: {e}"))?;
            Ok(FetchOutcome::Fetched {
                body,
                etag,
                last_modified,
            })
        }
        Err(ureq::Error::Status(304, _)) => Ok(FetchOutcome::NotModified),
        Err(ureq::Error::Status(code, _)) => Err(format!("HTTP {code}")),
        Err(ureq::Error::Transport(t)) => Err(format!("connection error: {t}")),
    }
}
