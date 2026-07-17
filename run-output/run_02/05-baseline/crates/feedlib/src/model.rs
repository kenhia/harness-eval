//! Common data types shared across the workspace.

use chrono::{DateTime, Utc};

/// A feed item after parsing and normalization, ready to be stored.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedEntry {
    /// Stable identity within a feed: RSS `guid` (falling back to `link`) or
    /// Atom `id`.
    pub guid: String,
    /// Title after entity unescaping; `(untitled)` when absent.
    pub title: String,
    /// Item link, if any.
    pub link: Option<String>,
    /// Summary/description after entity unescaping, if any.
    pub summary: Option<String>,
    /// Publication instant normalized to UTC, or `None` when missing or
    /// unparseable.
    pub published_at: Option<DateTime<Utc>>,
}

/// A parsed feed document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedFeed {
    /// Feed/channel title after entity unescaping, if present.
    pub title: Option<String>,
    /// The feed's entries in document order.
    pub entries: Vec<ParsedEntry>,
}

/// A registered feed row as stored by the server.
#[derive(Debug, Clone)]
pub struct Feed {
    pub id: i64,
    pub url: String,
    pub title: Option<String>,
    pub last_fetched_at: Option<DateTime<Utc>>,
    pub last_error: Option<String>,
    pub etag: Option<String>,
    pub last_modified: Option<String>,
}
