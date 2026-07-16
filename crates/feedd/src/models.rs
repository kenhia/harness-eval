//! API data-transfer types (the JSON shapes the REST API returns).

use serde::Serialize;

/// A feed as exposed by the API.
#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct FeedDto {
    pub id: i64,
    pub url: String,
    pub title: Option<String>,
    pub last_fetched_at: Option<String>,
    pub last_error: Option<String>,
    pub entry_count: i64,
}

/// An entry as exposed by the API.
#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct EntryDto {
    pub id: i64,
    pub feed_id: i64,
    pub guid: String,
    pub title: String,
    pub link: Option<String>,
    pub summary: Option<String>,
    pub published_at: Option<String>,
    pub fetched_at: String,
}

/// Internal feed row including conditional-GET bookkeeping.
#[derive(Debug, Clone)]
pub struct FeedRow {
    pub id: i64,
    pub url: String,
    pub etag: Option<String>,
    pub last_modified: Option<String>,
}

/// Result of refreshing a single feed (also the API refresh payload).
#[derive(Debug, Serialize)]
pub struct RefreshResult {
    pub feed_id: i64,
    pub status: &'static str,
    pub new_entries: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub not_modified: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl RefreshResult {
    pub fn ok(feed_id: i64, new_entries: i64) -> Self {
        Self { feed_id, status: "ok", new_entries, not_modified: None, error: None }
    }
    pub fn not_modified(feed_id: i64) -> Self {
        Self { feed_id, status: "ok", new_entries: 0, not_modified: Some(true), error: None }
    }
    pub fn error(feed_id: i64, msg: impl Into<String>) -> Self {
        Self { feed_id, status: "error", new_entries: 0, not_modified: None, error: Some(msg.into()) }
    }
}
