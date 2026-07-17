//! Fetching feed documents, with conditional GET.

use std::time::Duration;

use reqwest::header::{ETAG, IF_MODIFIED_SINCE, IF_NONE_MATCH, LAST_MODIFIED};
use reqwest::{Client, Response, StatusCode};

/// Most a feed document may be before we give up on it.
///
/// feedd fetches whatever URLs it is pointed at, so the body has to be bounded:
/// an endless or enormous response must fail that one feed, not exhaust memory
/// and take the whole server with it.
pub const MAX_FEED_BYTES: usize = 16 * 1024 * 1024;

/// The result of a conditional GET.
pub enum Fetched {
    /// The server sent a body, along with whatever validators it offered.
    Modified {
        body: Vec<u8>,
        etag: Option<String>,
        last_modified: Option<String>,
    },
    /// The server answered 304: what we already stored is current.
    NotModified,
}

/// A fetch that failed. The message is what gets recorded as the feed's
/// `last_error`, so it is written to be read by a person.
#[derive(Debug)]
pub struct FetchError(pub String);

impl std::fmt::Display for FetchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

pub fn client() -> reqwest::Result<Client> {
    Client::builder()
        .timeout(Duration::from_secs(30))
        .user_agent(concat!("feedd/", env!("CARGO_PKG_VERSION")))
        .build()
}

/// GET `url`, sending `If-None-Match` / `If-Modified-Since` when we have the
/// matching validators from a previous fetch.
pub async fn fetch(
    client: &Client,
    url: &str,
    etag: Option<&str>,
    last_modified: Option<&str>,
) -> Result<Fetched, FetchError> {
    let mut request = client.get(url);
    if let Some(etag) = etag {
        request = request.header(IF_NONE_MATCH, etag);
    }
    if let Some(last_modified) = last_modified {
        request = request.header(IF_MODIFIED_SINCE, last_modified);
    }

    let response = request.send().await.map_err(|e| FetchError(describe(&e)))?;

    if response.status() == StatusCode::NOT_MODIFIED {
        return Ok(Fetched::NotModified);
    }
    if !response.status().is_success() {
        return Err(FetchError(format!("HTTP {}", response.status())));
    }

    let header = |name: reqwest::header::HeaderName| {
        response
            .headers()
            .get(name)
            .and_then(|v| v.to_str().ok())
            .map(str::to_string)
    };
    let etag = header(ETAG);
    let last_modified = header(LAST_MODIFIED);
    let body = read_bounded(response).await?;

    Ok(Fetched::Modified {
        body,
        etag,
        last_modified,
    })
}

/// Read the body, refusing anything over [`MAX_FEED_BYTES`].
///
/// The advertised length is only a hint — a chunked or lying response has to be
/// caught as it streams, which is why the running total is checked too.
async fn read_bounded(mut response: Response) -> Result<Vec<u8>, FetchError> {
    if let Some(length) = response.content_length()
        && length > MAX_FEED_BYTES as u64
    {
        return Err(FetchError(format!(
            "feed is too large: {length} bytes, limit is {MAX_FEED_BYTES}"
        )));
    }

    let mut body = Vec::new();
    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|e| FetchError(format!("could not read response body: {e}")))?
    {
        if body.len() + chunk.len() > MAX_FEED_BYTES {
            return Err(FetchError(format!(
                "feed is too large: over {MAX_FEED_BYTES} bytes"
            )));
        }
        body.extend_from_slice(&chunk);
    }
    Ok(body)
}

/// reqwest's own Display is a chain of internal causes; these messages say what
/// happened in the terms an operator cares about.
fn describe(error: &reqwest::Error) -> String {
    if error.is_timeout() {
        return "request timed out".to_string();
    }
    if error.is_connect() {
        return format!("connection failed: {}", root_cause(error));
    }
    if error.is_redirect() {
        return "too many redirects".to_string();
    }
    format!("fetch failed: {}", root_cause(error))
}

fn root_cause(error: &reqwest::Error) -> String {
    let mut source: &dyn std::error::Error = error;
    while let Some(next) = source.source() {
        source = next;
    }
    source.to_string()
}
