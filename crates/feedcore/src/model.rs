//! Common data types produced by feed parsing.

use chrono::{DateTime, Utc};

/// A single parsed entry/item, before it is persisted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedEntry {
    /// Stable identity within a feed (RSS `guid`/`link`, Atom `id`).
    pub guid: String,
    /// Title, `None` when absent (caller substitutes `(untitled)`).
    pub title: Option<String>,
    pub link: Option<String>,
    pub summary: Option<String>,
    /// Publication instant normalized to UTC, `None` when missing/unparseable.
    pub published_at: Option<DateTime<Utc>>,
}

/// A parsed feed document.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ParsedFeed {
    pub title: Option<String>,
    pub entries: Vec<ParsedEntry>,
}
