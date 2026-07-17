//! Common data types serialized by the REST API.

use serde::{Deserialize, Serialize};

/// A registered feed and its last-fetch status.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Feed {
    pub id: i64,
    pub url: String,
    pub title: Option<String>,
    pub last_fetched_at: Option<String>,
    pub last_error: Option<String>,
    pub entry_count: i64,
}

/// A stored feed entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entry {
    pub id: i64,
    pub feed_id: i64,
    pub guid: String,
    pub title: String,
    pub link: Option<String>,
    pub summary: Option<String>,
    pub published_at: Option<String>,
    pub fetched_at: String,
}

/// Result of a single feed refresh, as returned by the refresh endpoints.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefreshResult {
    pub feed_id: i64,
    pub status: String,
    pub new_entries: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub not_modified: Option<bool>,
}
