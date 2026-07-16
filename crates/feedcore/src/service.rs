//! Feed refresh orchestration: fetch → parse → store, with failure isolation.

use crate::dates;
use crate::fetch::{self, FetchOutcome};
use crate::model::RefreshResult;
use crate::parse;
use crate::store::Store;
use chrono::Utc;

/// Refresh a single feed by id. Never panics; any failure is captured into the
/// returned [`RefreshResult`] and recorded on the feed's `last_error`.
///
/// Returns `None` only if the feed id does not exist.
pub fn refresh_feed(store: &Store, feed_id: i64) -> Option<RefreshResult> {
    let info = match store.fetch_info(feed_id) {
        Ok(Some(i)) => i,
        _ => return None,
    };
    let now = dates::to_rfc3339_z(&Utc::now());

    match fetch::fetch(&info) {
        Err(e) => {
            let msg = e.to_string();
            let _ = store.record_error(feed_id, &now, &msg);
            Some(RefreshResult {
                feed_id,
                status: "error".into(),
                new_entries: 0,
                error: Some(msg),
                not_modified: None,
            })
        }
        Ok(FetchOutcome::NotModified) => {
            let _ = store.record_not_modified(feed_id, &now);
            Some(RefreshResult {
                feed_id,
                status: "ok".into(),
                new_entries: 0,
                error: None,
                not_modified: Some(true),
            })
        }
        Ok(FetchOutcome::Fetched {
            body,
            etag,
            last_modified,
        }) => match parse::parse(&body) {
            Err(e) => {
                let msg = e.to_string();
                let _ = store.record_error(feed_id, &now, &msg);
                Some(RefreshResult {
                    feed_id,
                    status: "error".into(),
                    new_entries: 0,
                    error: Some(msg),
                    not_modified: None,
                })
            }
            Ok(parsed) => {
                let mut new_entries = 0i64;
                for item in &parsed.items {
                    match store.upsert_entry(feed_id, item, &now) {
                        Ok(true) => new_entries += 1,
                        Ok(false) => {}
                        Err(_) => {}
                    }
                }
                let _ = store.record_success(
                    feed_id,
                    parsed.title.as_deref(),
                    &now,
                    etag.as_deref(),
                    last_modified.as_deref(),
                );
                Some(RefreshResult {
                    feed_id,
                    status: "ok".into(),
                    new_entries,
                    error: None,
                    not_modified: None,
                })
            }
        },
    }
}

/// Refresh every registered feed, returning per-feed results in id order.
pub fn refresh_all(store: &Store) -> Vec<RefreshResult> {
    let ids = store.feed_ids().unwrap_or_default();
    ids.into_iter()
        .filter_map(|id| refresh_feed(store, id))
        .collect()
}
