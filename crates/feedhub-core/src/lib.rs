//! Shared types and feed parsing for feedhub.
//!
//! This crate owns the parts of the system whose behavior the spec pins
//! exactly: date grammars and normalization ([`datetime`]), the RSS/Atom
//! element mapping ([`parser`]), and the types that cross binary boundaries
//! ([`model`]).

#![forbid(unsafe_code)]

pub mod datetime;
pub mod model;
pub mod parser;

pub use model::{
    Entry, EntryPage, Feed, FeedKind, ParsedEntry, ParsedFeed, RefreshResult, UNTITLED,
};
pub use parser::{parse_feed, ParseError, ParseOutcome};
