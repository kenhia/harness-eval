//! Shared types and feed parsing for the feedhub tools.
//!
//! This crate is the single source of truth for the parts of the system that
//! `feedd`, `feedctl` and the tests all have to agree on:
//!
//! * [`api`] — the JSON shapes exchanged over the REST API.
//! * [`dates`] — RFC 822 / RFC 3339 parsing and the pinned UTC serialization.
//! * [`parse`] — RSS 2.0 and Atom parsing into a common [`ParsedFeed`].

pub mod api;
pub mod dates;
pub mod parse;

pub use dates::{format_utc, format_utc_ceil, parse_rfc822, parse_rfc3339};
pub use parse::{ParseError, ParsedEntry, ParsedFeed, parse_feed};

/// Title stored for an entry whose feed provided no title.
pub const UNTITLED: &str = "(untitled)";
