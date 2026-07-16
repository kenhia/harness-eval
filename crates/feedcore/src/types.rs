//! Parser output types.

use chrono::{DateTime, Utc};

/// A parsed feed: an optional channel/feed title plus its items.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedFeed {
    /// Channel (`RSS`) / feed (`Atom`) title, if present.
    pub title: Option<String>,
    /// Items/entries in document order.
    pub items: Vec<ParsedItem>,
}

/// A single parsed item/entry, with identity and text already normalized.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedItem {
    /// Stable identity within the feed: RSS `guid` (fallback `link`), Atom
    /// `id`. When the source provides none, a content-derived key is used so
    /// dedupe stays stable across refetches.
    pub guid: String,
    /// Title after entity unescaping; `(untitled)` when missing.
    pub title: String,
    /// Alternate link, if any.
    pub link: Option<String>,
    /// Summary/description/content after entity unescaping.
    pub summary: Option<String>,
    /// Publication instant normalized to UTC, or `None` when missing or
    /// unparseable.
    pub published_at: Option<DateTime<Utc>>,
}
