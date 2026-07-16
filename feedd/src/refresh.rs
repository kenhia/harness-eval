//! Refresh orchestration: fetch → parse → store, with failure isolation.

use std::sync::Arc;

use feedcore::api::RefreshResult;
use feedcore::parse_feed;

use crate::fetch::{fetch, FetchOutcome};
use crate::store::Store;

/// Refresh a single feed by id. Any fetch/parse failure is recorded on the feed
/// and returned as an error result; it never propagates or affects other feeds.
/// Returns `None` if the feed id does not exist.
pub fn refresh_feed(store: &Arc<Store>, id: i64) -> Option<RefreshResult> {
    let info = match store.fetch_info(id) {
        Ok(Some(i)) => i,
        Ok(None) => return None,
        Err(e) => return Some(err_result(id, e.to_string())),
    };

    let outcome = fetch(&info.url, info.etag.as_deref(), info.last_modified.as_deref());
    match outcome {
        Err(message) => {
            let _ = store.record_error(id, &message);
            Some(err_result(id, message))
        }
        Ok(FetchOutcome::NotModified) => {
            let _ = store.record_not_modified(id);
            Some(ok_result(id, 0))
        }
        Ok(FetchOutcome::Fetched {
            body,
            etag,
            last_modified,
        }) => match parse_feed(&body) {
            Err(e) => {
                let message = e.to_string();
                let _ = store.record_error(id, &message);
                Some(err_result(id, message))
            }
            Ok(parsed) => match store.apply_parsed(id, &parsed, etag, last_modified) {
                Ok(new_entries) => Some(ok_result(id, new_entries)),
                Err(e) => {
                    let message = e.to_string();
                    let _ = store.record_error(id, &message);
                    Some(err_result(id, message))
                }
            },
        },
    }
}

/// Refresh every registered feed, returning one result per feed (each carries
/// its `feed_id`).
pub fn refresh_all(store: &Arc<Store>) -> Vec<RefreshResult> {
    let ids = store.all_feed_ids().unwrap_or_default();
    ids.into_iter()
        .filter_map(|id| refresh_feed(store, id))
        .collect()
}

fn ok_result(id: i64, new_entries: i64) -> RefreshResult {
    RefreshResult {
        feed_id: Some(id),
        status: "ok".to_string(),
        new_entries,
        error: None,
    }
}

fn err_result(id: i64, message: String) -> RefreshResult {
    RefreshResult {
        feed_id: Some(id),
        status: "error".to_string(),
        new_entries: 0,
        error: Some(message),
    }
}
