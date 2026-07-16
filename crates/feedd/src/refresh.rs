//! Refreshing feeds: fetch, parse, store, and record what happened.
//!
//! Every failure here is isolated to the feed it happened on — it is written to
//! that feed's `last_error` and returned in its [`RefreshResult`], never
//! propagated to the caller. One unreachable host or one malformed document
//! must not disturb the other feeds or bring the server down.

use chrono::Utc;
use feedhub_core::api::RefreshResult;
use feedhub_core::parse_feed;

use crate::db;
use crate::fetch::{self, Fetched};
use crate::state::SharedState;

fn ok_result(feed_id: i64, counts: db::ApplyCounts, not_modified: bool) -> RefreshResult {
    RefreshResult {
        feed_id,
        status: "ok".to_string(),
        new_entries: counts.new_entries,
        updated_entries: counts.updated_entries,
        not_modified,
        error: None,
    }
}

fn error_result(feed_id: i64, message: String) -> RefreshResult {
    RefreshResult {
        feed_id,
        status: "error".to_string(),
        new_entries: 0,
        updated_entries: 0,
        not_modified: false,
        error: Some(message),
    }
}

/// Fetch one feed now and reconcile what came back.
///
/// Returns `None` only if the feed does not exist; every other outcome is a
/// [`RefreshResult`].
pub async fn refresh_feed(state: &SharedState, feed_id: i64) -> Option<RefreshResult> {
    // Read the validators and drop the lock: the fetch below must not hold it.
    let fetch_state = {
        let conn = state.db.lock().expect("db mutex poisoned");
        match db::fetch_state(&conn, feed_id) {
            Ok(Some(state)) => state,
            Ok(None) => return None,
            Err(e) => return Some(error_result(feed_id, format!("database error: {e}"))),
        }
    };

    let fetched = fetch::fetch(
        &state.client,
        &fetch_state.url,
        fetch_state.etag.as_deref(),
        fetch_state.last_modified.as_deref(),
    )
    .await;

    let now = Utc::now();
    let mut conn = state.db.lock().expect("db mutex poisoned");

    match fetched {
        Err(e) => {
            let message = e.to_string();
            if let Err(e) = db::record_error(&conn, feed_id, &message) {
                return Some(error_result(feed_id, format!("database error: {e}")));
            }
            Some(error_result(feed_id, message))
        }
        // A 304 is a successful fetch that found nothing new: entries are left
        // untouched and no new ones are reported.
        Ok(Fetched::NotModified) => match db::record_not_modified(&conn, feed_id, now) {
            Ok(()) => Some(ok_result(feed_id, db::ApplyCounts::default(), true)),
            Err(e) => Some(error_result(feed_id, format!("database error: {e}"))),
        },
        Ok(Fetched::Modified {
            body,
            etag,
            last_modified,
        }) => {
            let parsed = match parse_feed(&body) {
                Ok(parsed) => parsed,
                Err(e) => {
                    let message = e.to_string();
                    if let Err(e) = db::record_error(&conn, feed_id, &message) {
                        return Some(error_result(feed_id, format!("database error: {e}")));
                    }
                    return Some(error_result(feed_id, message));
                }
            };

            match db::apply_fetch(
                &mut conn,
                feed_id,
                &parsed,
                etag.as_deref(),
                last_modified.as_deref(),
                now,
            ) {
                Ok(counts) => Some(ok_result(feed_id, counts, false)),
                Err(e) => Some(error_result(feed_id, format!("database error: {e}"))),
            }
        }
    }
}

/// Refresh every registered feed, in id order.
///
/// Feeds are refreshed one at a time: the point of this server is to be
/// well-behaved toward the origins it polls, and a handful of sequential
/// fetches is not the bottleneck.
pub async fn refresh_all(state: &SharedState) -> Vec<RefreshResult> {
    let ids = {
        let conn = state.db.lock().expect("db mutex poisoned");
        db::feed_ids(&conn).unwrap_or_default()
    };

    let mut results = Vec::with_capacity(ids.len());
    for id in ids {
        // `None` means the feed was deleted while we were working; skip it.
        if let Some(result) = refresh_feed(state, id).await {
            results.push(result);
        }
    }
    results
}
