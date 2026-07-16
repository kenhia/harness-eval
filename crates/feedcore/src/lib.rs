//! Shared types and feed-parsing logic for feedhub.

pub mod date;
pub mod model;
pub mod parse;

pub use date::parse_date;
pub use model::{ParsedEntry, ParsedFeed};
pub use parse::{parse_feed, ParseError};
