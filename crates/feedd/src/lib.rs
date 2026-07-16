//! `feedd` — the feed-aggregation server.
//!
//! The binary is a thin wrapper around [`server::start`]; the library exists so
//! that tests can run a real server, over real local HTTP, in-process.

pub mod api;
pub mod db;
pub mod fetch;
pub mod refresh;
pub mod server;
pub mod state;

pub use server::{Config, RunningServer, start};
