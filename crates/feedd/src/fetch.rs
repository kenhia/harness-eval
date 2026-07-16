//! On-demand and background feed fetching.
//!
//! A fetch performs a conditional GET (sending `If-None-Match` /
//! `If-Modified-Since` from the stored validators), parses the body, and
//! upserts entries. Failures are isolated: they are recorded on the feed's
//! `last_error` and never propagate to other feeds or crash the server.

use std::io::Read;
use std::time::Duration;

use chrono::Utc;
use serde_json::{json, Value};

use crate::store::Store;

const MAX_BODY_BYTES: u64 = 16 * 1024 * 1024;

/// The outcome of refreshing a single feed.
pub struct RefreshResult {
    pub status: &'static str,
    pub new_entries: usize,
    pub not_modified: bool,
    pub error: Option<String>,
}

impl RefreshResult {
    fn ok(new_entries: usize, not_modified: bool) -> Self {
        RefreshResult {
            status: "ok",
            new_entries,
            not_modified,
            error: None,
        }
    }

    fn err(message: String) -> Self {
        RefreshResult {
            status: "error",
            new_entries: 0,
            not_modified: false,
            error: Some(message),
        }
    }

    /// JSON view, optionally tagged with the feed id (used by `/api/refresh`).
    pub fn to_json(&self, feed_id: Option<i64>) -> Value {
        let mut obj = json!({
            "status": self.status,
            "new_entries": self.new_entries,
        });
        if self.not_modified {
            obj["not_modified"] = Value::Bool(true);
        }
        if let Some(e) = &self.error {
            obj["error"] = Value::String(e.clone());
        }
        if let Some(id) = feed_id {
            obj["feed_id"] = json!(id);
        }
        obj
    }
}

/// Build an HTTP agent with conservative timeouts.
pub fn build_agent() -> ureq::Agent {
    ureq::AgentBuilder::new()
        .timeout_connect(Duration::from_secs(10))
        .timeout_read(Duration::from_secs(30))
        .build()
}

/// Refresh one feed by id. Always returns a result; errors are recorded on the
/// feed rather than surfaced as failures.
pub fn refresh_feed(store: &Store, agent: &ureq::Agent, feed_id: i64) -> RefreshResult {
    match inner_refresh(store, agent, feed_id) {
        Ok(result) => result,
        Err(message) => {
            let now = Utc::now().timestamp();
            let _ = store.record_error(feed_id, &message, now);
            RefreshResult::err(message)
        }
    }
}

/// Refresh every registered feed, isolating per-feed failures.
pub fn refresh_all(store: &Store, agent: &ureq::Agent) -> Vec<(i64, RefreshResult)> {
    let feeds = match store.list_feeds() {
        Ok(f) => f,
        Err(_) => return Vec::new(),
    };
    feeds
        .into_iter()
        .map(|f| (f.id, refresh_feed(store, agent, f.id)))
        .collect()
}

fn inner_refresh(store: &Store, agent: &ureq::Agent, feed_id: i64) -> Result<RefreshResult, String> {
    let feed = store
        .get_feed(feed_id)
        .map_err(|e| format!("database error: {e}"))?
        .ok_or_else(|| "feed not found".to_string())?;

    let now = Utc::now().timestamp();
    let mut req = agent.get(&feed.url);
    if let Some(etag) = &feed.etag {
        req = req.set("If-None-Match", etag);
    }
    if let Some(lm) = &feed.last_modified {
        req = req.set("If-Modified-Since", lm);
    }

    match req.call() {
        Ok(resp) => handle_response(store, feed_id, now, resp),
        Err(ureq::Error::Status(304, _)) => {
            store
                .record_not_modified(feed_id, now)
                .map_err(|e| format!("database error: {e}"))?;
            Ok(RefreshResult::ok(0, true))
        }
        Err(ureq::Error::Status(code, _resp)) => {
            let message = format!("HTTP {code}");
            store
                .record_error(feed_id, &message, now)
                .map_err(|e| format!("database error: {e}"))?;
            Ok(RefreshResult::err(message))
        }
        Err(ureq::Error::Transport(t)) => {
            let message = format!("fetch failed: {t}");
            store
                .record_error(feed_id, &message, now)
                .map_err(|e| format!("database error: {e}"))?;
            Ok(RefreshResult::err(message))
        }
    }
}

fn handle_response(
    store: &Store,
    feed_id: i64,
    now: i64,
    resp: ureq::Response,
) -> Result<RefreshResult, String> {
    // Some servers answer 200 with an unchanged body; treat a 304 here too.
    if resp.status() == 304 {
        store
            .record_not_modified(feed_id, now)
            .map_err(|e| format!("database error: {e}"))?;
        return Ok(RefreshResult::ok(0, true));
    }

    let etag = resp.header("ETag").map(|s| s.to_string());
    let last_modified = resp.header("Last-Modified").map(|s| s.to_string());

    let mut body = Vec::new();
    resp.into_reader()
        .take(MAX_BODY_BYTES)
        .read_to_end(&mut body)
        .map_err(|e| format!("read failed: {e}"))?;

    match feedlib::parse_feed(&body) {
        Ok(parsed) => {
            let mut new_entries = 0;
            for entry in &parsed.entries {
                if store
                    .upsert_entry(feed_id, entry, now)
                    .map_err(|e| format!("database error: {e}"))?
                {
                    new_entries += 1;
                }
            }
            store
                .record_success(
                    feed_id,
                    parsed.title.as_deref(),
                    etag.as_deref(),
                    last_modified.as_deref(),
                    now,
                )
                .map_err(|e| format!("database error: {e}"))?;
            Ok(RefreshResult::ok(new_entries, false))
        }
        Err(pe) => {
            let message = format!("parse error: {pe}");
            store
                .record_error(feed_id, &message, now)
                .map_err(|e| format!("database error: {e}"))?;
            Ok(RefreshResult::err(message))
        }
    }
}
