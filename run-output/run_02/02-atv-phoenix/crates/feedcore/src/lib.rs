//! feedcore — shared feed parsing, storage, fetching, and refresh logic for
//! the feedhub tools (`feedd`, `feedctl`, `feedgen`).

pub mod dates;
pub mod error;
pub mod fetch;
pub mod model;
pub mod parse;
pub mod service;
pub mod store;
pub mod text;

pub use error::{FeedError, Result};
pub use model::{Entry, Feed, RefreshResult};
pub use parse::{ParsedFeed, ParsedItem};
pub use store::{EntryQuery, Store};
