//! feedd: the feedhub server library.
//!
//! Exposes the storage layer, REST router, and fetch logic so both the
//! `feedd` binary and integration tests can drive the service.

pub mod api;
pub mod db;
pub mod fetch;
pub mod models;
pub mod state;

use std::time::Duration;

use anyhow::Result;
use tokio::net::TcpListener;

use crate::state::AppState;

/// Open the database, build state, optionally start background polling, and
/// serve the REST API on `listen` until the process is interrupted.
pub async fn run(db_path: &str, listen: &str, poll_interval: u64) -> Result<()> {
    let conn = db::open(db_path)?;
    let state = AppState::new(conn)?;

    if poll_interval > 0 {
        spawn_poller(state.clone(), poll_interval);
    }

    let listener = TcpListener::bind(listen).await?;
    tracing::info!("feedd listening on http://{}", listener.local_addr()?);
    axum::serve(listener, api::router(state))
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    Ok(())
}

fn spawn_poller(state: AppState, poll_interval: u64) {
    tokio::spawn(async move {
        let mut ticker = tokio::time::interval(Duration::from_secs(poll_interval));
        // Consume the immediate first tick so polling starts one interval in.
        ticker.tick().await;
        loop {
            ticker.tick().await;
            let results = fetch::refresh_all(&state).await;
            let errors = results.iter().filter(|r| r.status == "error").count();
            tracing::info!(feeds = results.len(), errors, "background poll complete");
        }
    });
}

async fn shutdown_signal() {
    let _ = tokio::signal::ctrl_c().await;
}
