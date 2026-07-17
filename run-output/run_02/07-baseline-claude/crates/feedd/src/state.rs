//! Shared server state.

use std::sync::{Arc, Mutex};

use rusqlite::Connection;

/// The one database connection, plus the HTTP client used to fetch feeds.
///
/// A single connection behind a mutex is enough here: SQLite serializes writes
/// anyway, and every query in this server is small. The rule that keeps it
/// honest is that the lock is never held across an `.await` — see
/// [`crate::refresh::refresh_feed`], which reads what it needs, drops the lock,
/// fetches, and only then takes it again.
pub struct AppState {
    pub db: Mutex<Connection>,
    pub client: reqwest::Client,
}

pub type SharedState = Arc<AppState>;
