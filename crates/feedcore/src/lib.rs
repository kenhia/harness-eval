//! Shared types and feed-parsing logic for feedhub.
//!
//! `feedcore` is deliberately free of any I/O or storage concerns: it turns
//! raw feed bytes into a [`ParsedFeed`] and normalizes dates and text
//! according to the pinned rules in the project spec.

pub mod date;
pub mod parse;
pub mod text;
pub mod types;

pub use date::parse_date;
pub use parse::{parse_feed, ParseError};
pub use types::{ParsedFeed, ParsedItem};
