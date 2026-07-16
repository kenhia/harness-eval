//! Shared application state.

use std::sync::{Arc, Mutex};
use std::time::Duration;

use anyhow::Result;
use rusqlite::Connection;

/// State shared across request handlers and the background poller.
#[derive(Clone)]
pub struct AppState {
    /// The single SQLite connection, serialized behind a mutex.
    pub db: Arc<Mutex<Connection>>,
    /// HTTP client used to fetch feeds.
    pub http: reqwest::Client,
}

impl AppState {
    pub fn new(conn: Connection) -> Result<Self> {
        let http = reqwest::Client::builder()
            .user_agent(concat!("feedd/", env!("CARGO_PKG_VERSION")))
            .timeout(Duration::from_secs(30))
            .build()?;
        Ok(Self {
            db: Arc::new(Mutex::new(conn)),
            http,
        })
    }
}
