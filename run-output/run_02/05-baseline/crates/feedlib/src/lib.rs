//! Shared library for feedhub: feed parsing, date handling, and common types.

pub mod date;
pub mod model;
pub mod parse;

pub use model::{Feed, ParsedEntry, ParsedFeed};
pub use parse::{parse_feed, ParseError};
