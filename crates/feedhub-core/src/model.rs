//! Common types shared by the feedhub binaries.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Placeholder stored when an item carries no usable title.
pub const UNTITLED: &str = "(untitled)";

/// Which grammar a document was parsed as.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FeedKind {
    Rss,
    Atom,
}

/// One item/entry lifted out of a feed document, before it reaches storage.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedEntry {
    /// Dedupe identity: RSS `guid` (or `link`), Atom `id`.
    pub guid: String,
    pub title: String,
    pub link: Option<String>,
    pub summary: Option<String>,
    /// `None` when the source date was missing or unparseable. Never a
    /// substituted fetch time.
    pub published_at: Option<DateTime<Utc>>,
}

/// A parsed feed document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedFeed {
    pub kind: FeedKind,
    pub title: Option<String>,
    pub entries: Vec<ParsedEntry>,
}

/// A registered feed, as stored and as returned by the API.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Feed {
    pub id: i64,
    pub url: String,
    /// `null` until the first successful fetch; refreshed on every success.
    pub title: Option<String>,
    pub last_fetched_at: Option<String>,
    /// `null` when the most recent fetch succeeded.
    pub last_error: Option<String>,
    pub entry_count: i64,
}

/// A stored entry, as returned by the API.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

/// Result of refreshing a single feed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RefreshResult {
    pub feed_id: i64,
    /// `"ok"` or `"error"`.
    pub status: String,
    /// Entries newly inserted by this fetch. Always 0 for a 304 and for errors.
    pub new_entries: usize,
    /// Entries that already existed and were updated in place.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_entries: Option<usize>,
    /// True when the origin answered `304 Not Modified`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub not_modified: Option<bool>,
    /// Present only when `status == "error"`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl RefreshResult {
    pub fn ok(feed_id: i64, new_entries: usize, updated_entries: usize, not_modified: bool) -> Self {
        Self {
            feed_id,
            status: "ok".into(),
            new_entries,
            updated_entries: Some(updated_entries),
            not_modified: Some(not_modified),
            error: None,
        }
    }

    pub fn error(feed_id: i64, message: impl Into<String>) -> Self {
        Self {
            feed_id,
            status: "error".into(),
            new_entries: 0,
            updated_entries: None,
            not_modified: None,
            error: Some(message.into()),
        }
    }
}

/// The `{"total": N, "items": [...]}` envelope returned by `GET /api/entries`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntryPage {
    /// Match count ignoring `limit`/`offset`.
    pub total: i64,
    pub items: Vec<Entry>,
}
