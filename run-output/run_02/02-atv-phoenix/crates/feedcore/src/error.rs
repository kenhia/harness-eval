//! Error type shared across feedcore.

use std::fmt;

/// Errors produced by feedcore operations.
#[derive(Debug)]
pub enum FeedError {
    /// Underlying SQLite failure.
    Db(rusqlite::Error),
    /// Feed content could not be parsed.
    Parse(String),
    /// Network / HTTP failure while fetching a feed.
    Fetch(String),
    /// A requested feed id does not exist.
    NotFound,
    /// The URL is already registered.
    Duplicate,
    /// The URL is not a valid http(s) URL.
    InvalidUrl,
}

impl fmt::Display for FeedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FeedError::Db(e) => write!(f, "database error: {e}"),
            FeedError::Parse(m) => write!(f, "parse error: {m}"),
            FeedError::Fetch(m) => write!(f, "fetch error: {m}"),
            FeedError::NotFound => write!(f, "not found"),
            FeedError::Duplicate => write!(f, "feed already registered"),
            FeedError::InvalidUrl => write!(f, "invalid url: must be http(s)"),
        }
    }
}

impl std::error::Error for FeedError {}

impl From<rusqlite::Error> for FeedError {
    fn from(e: rusqlite::Error) -> Self {
        FeedError::Db(e)
    }
}

/// Convenience result alias.
pub type Result<T> = std::result::Result<T, FeedError>;
