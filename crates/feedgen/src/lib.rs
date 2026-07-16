//! Test-fixture feed server.
//!
//! `feedgen` exists so that `feedd` can be developed and tested without ever
//! talking to the real internet: [`fixtures`] writes a corpus of feed documents
//! covering the cases the parser has to get right, and [`serve`] serves a
//! directory over HTTP with the conditional-GET machinery `feedd` expects.

pub mod fixtures;
pub mod serve;

pub use fixtures::write_fixtures;
pub use serve::{RunningServer, serve_dir};
