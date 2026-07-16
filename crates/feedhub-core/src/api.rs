//! The JSON shapes exchanged over the REST API.
//!
//! `feedd` serializes these and `feedctl` deserializes them, so the two cannot
//! drift apart. All timestamps are strings in the pinned storage format (see
//! [`crate::dates`]).

use serde::{Deserialize, Serialize};

/// A registered feed.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Feed {
    pub id: i64,
    pub url: String,
    /// Null until the first successful fetch; refreshed from the document.
    pub title: Option<String>,
    pub last_fetched_at: Option<String>,
    /// Null when the last fetch succeeded.
    pub last_error: Option<String>,
    pub entry_count: i64,
}

/// One stored entry.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Entry {
    pub id: i64,
    pub feed_id: i64,
    pub guid: String,
    pub title: String,
    pub link: Option<String>,
    pub summary: Option<String>,
    /// Null when the feed gave no parseable date.
    pub published_at: Option<String>,
    pub fetched_at: String,
}

/// Response of `GET /api/entries`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EntriesPage {
    /// Number of matches ignoring `limit`/`offset`.
    pub total: i64,
    pub items: Vec<Entry>,
}

/// Outcome of refreshing a single feed. Returned on its own by
/// `POST /api/feeds/{id}/refresh` and in an array by `POST /api/refresh`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RefreshResult {
    pub feed_id: i64,
    /// `"ok"` or `"error"`.
    pub status: String,
    /// Entries stored for the first time by this fetch.
    pub new_entries: i64,
    /// Known entries whose contents this fetch updated in place.
    pub updated_entries: i64,
    /// True when the server answered `304 Not Modified`.
    pub not_modified: bool,
    /// Set when `status` is `"error"`.
    pub error: Option<String>,
}

impl RefreshResult {
    pub fn is_ok(&self) -> bool {
        self.status == "ok"
    }
}

/// The body of every error response: `{"error": "<message>"}`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ErrorBody {
    pub error: String,
}

/// Body of `POST /api/feeds`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AddFeedRequest {
    pub url: String,
}

/// Default and maximum for `GET /api/entries?limit=`.
pub const DEFAULT_LIMIT: i64 = 50;
pub const MAX_LIMIT: i64 = 500;
