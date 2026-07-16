//! Refresh orchestration: fetch a feed then apply the result to storage.

use crate::db::Store;
use crate::fetch::{fetch_feed, FetchOutcome};
use serde_json::{json, Value};

/// Refresh a single feed by id, returning the per-feed result object.
///
/// The result always includes `id`, `status` (`"ok"`/`"error"`) and
/// `new_entries`; on failure it also carries `error`.
pub fn refresh_feed(store: &mut Store, id: i64) -> Value {
    let url = match store.feed_url(id) {
        Ok(Some(u)) => u,
        _ => {
            return json!({ "id": id, "status": "error", "new_entries": 0, "error": "feed not found" })
        }
    };
    let (etag, last_modified) = store.conditional_headers(id).unwrap_or((None, None));

    match fetch_feed(&url, etag.as_deref(), last_modified.as_deref()) {
        Ok(FetchOutcome::NotModified) => {
            let _ = store.record_not_modified(id);
            json!({ "id": id, "status": "ok", "new_entries": 0, "not_modified": true })
        }
        Ok(FetchOutcome::Fetched {
            feed,
            etag,
            last_modified,
        }) => match store.apply_fetch(id, &feed, etag.as_deref(), last_modified.as_deref()) {
            Ok(new_entries) => json!({ "id": id, "status": "ok", "new_entries": new_entries }),
            Err(e) => {
                let _ = store.record_error(id, &e.to_string());
                json!({ "id": id, "status": "error", "new_entries": 0, "error": e.to_string() })
            }
        },
        Err(e) => {
            let _ = store.record_error(id, &e);
            json!({ "id": id, "status": "error", "new_entries": 0, "error": e })
        }
    }
}

/// Refresh every registered feed; returns an array of per-feed result objects.
pub fn refresh_all(store: &mut Store) -> Vec<Value> {
    let ids = store.all_feed_ids().unwrap_or_default();
    ids.into_iter().map(|id| refresh_feed(store, id)).collect()
}
