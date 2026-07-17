//! On-demand feed fetching with conditional GET, dedupe, and failure
//! isolation.

use chrono::Utc;
use reqwest::header::{ETAG, IF_MODIFIED_SINCE, IF_NONE_MATCH, LAST_MODIFIED};
use reqwest::StatusCode;

use feedcore::parse::parse_feed;

use crate::db;
use crate::models::RefreshResult;
use crate::state::AppState;

/// Refresh a single feed by id, fetching over HTTP now.
///
/// Any failure (network, HTTP status, malformed XML) is recorded on the feed
/// as `last_error` and returned as an error result — it never affects other
/// feeds or panics.
pub async fn refresh_feed(state: &AppState, feed_id: i64) -> RefreshResult {
    let row = {
        let conn = state.db.lock().unwrap();
        match db::get_feed_row(&conn, feed_id) {
            Ok(Some(row)) => row,
            Ok(None) => return RefreshResult::error(feed_id, "feed not found"),
            Err(e) => return RefreshResult::error(feed_id, e.to_string()),
        }
    };

    let mut req = state.http.get(&row.url);
    if let Some(etag) = &row.etag {
        req = req.header(IF_NONE_MATCH, etag);
    }
    if let Some(lm) = &row.last_modified {
        req = req.header(IF_MODIFIED_SINCE, lm);
    }

    let resp = match req.send().await {
        Ok(r) => r,
        Err(e) => return record_error(state, feed_id, format!("fetch failed: {e}")),
    };

    let status = resp.status();
    if status == StatusCode::NOT_MODIFIED {
        let conn = state.db.lock().unwrap();
        let _ = db::mark_not_modified(&conn, feed_id, Utc::now());
        return RefreshResult::not_modified(feed_id);
    }
    if !status.is_success() {
        return record_error(state, feed_id, format!("HTTP {}", status.as_u16()));
    }

    let etag = header_string(resp.headers().get(ETAG));
    let last_modified = header_string(resp.headers().get(LAST_MODIFIED));

    let bytes = match resp.bytes().await {
        Ok(b) => b,
        Err(e) => return record_error(state, feed_id, format!("read body failed: {e}")),
    };

    let parsed = match parse_feed(&bytes) {
        Ok(p) => p,
        Err(e) => return record_error(state, feed_id, format!("parse failed: {e}")),
    };

    let now = Utc::now();
    let mut conn = state.db.lock().unwrap();
    let tx = match conn.transaction() {
        Ok(tx) => tx,
        Err(e) => return RefreshResult::error(feed_id, e.to_string()),
    };
    let mut new_entries = 0i64;
    for item in &parsed.items {
        match db::upsert_entry(&tx, feed_id, item, now) {
            Ok(true) => new_entries += 1,
            Ok(false) => {}
            Err(e) => return RefreshResult::error(feed_id, e.to_string()),
        }
    }
    if let Err(e) = db::mark_success(
        &tx,
        feed_id,
        parsed.title.as_deref(),
        etag.as_deref(),
        last_modified.as_deref(),
        now,
    ) {
        return RefreshResult::error(feed_id, e.to_string());
    }
    if let Err(e) = tx.commit() {
        return RefreshResult::error(feed_id, e.to_string());
    }
    RefreshResult::ok(feed_id, new_entries)
}

/// Refresh every registered feed, returning one result per feed.
pub async fn refresh_all(state: &AppState) -> Vec<RefreshResult> {
    let ids = {
        let conn = state.db.lock().unwrap();
        db::all_feed_ids(&conn).unwrap_or_default()
    };
    let mut results = Vec::with_capacity(ids.len());
    for id in ids {
        results.push(refresh_feed(state, id).await);
    }
    results
}

fn record_error(state: &AppState, feed_id: i64, msg: String) -> RefreshResult {
    let conn = state.db.lock().unwrap();
    let _ = db::mark_error(&conn, feed_id, &msg);
    RefreshResult::error(feed_id, msg)
}

fn header_string(v: Option<&reqwest::header::HeaderValue>) -> Option<String> {
    v.and_then(|h| h.to_str().ok()).map(|s| s.to_string())
}
