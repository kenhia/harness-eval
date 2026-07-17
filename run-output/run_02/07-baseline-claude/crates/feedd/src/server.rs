//! Starting the server and its background poller.

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use anyhow::{Context, Result};
use tokio::net::TcpListener;
use tokio::sync::oneshot;
use tokio::task::JoinHandle;

use crate::api;
use crate::db;
use crate::fetch;
use crate::refresh;
use crate::state::{AppState, SharedState};

/// How to run the server.
pub struct Config {
    pub db_path: PathBuf,
    pub listen: SocketAddr,
    /// Seconds between background refreshes; `0` disables polling entirely.
    pub poll_interval: u64,
}

/// A server that is listening, with the address it actually bound to.
pub struct RunningServer {
    pub addr: SocketAddr,
    state: SharedState,
    shutdown: Option<oneshot::Sender<()>>,
    server: JoinHandle<()>,
    poller: Option<JoinHandle<()>>,
}

impl RunningServer {
    /// The base URL the API is reachable at.
    pub fn base_url(&self) -> String {
        format!("http://{}", self.addr)
    }

    pub fn state(&self) -> &SharedState {
        &self.state
    }

    /// Run until the server stops on its own (it will not, unless it fails).
    pub async fn wait(self) -> Result<()> {
        self.server.await.context("server task panicked")
    }

    /// Stop the poller and the listener, and wait for both to finish.
    pub async fn shutdown(mut self) {
        if let Some(poller) = self.poller.take() {
            poller.abort();
            let _ = poller.await;
        }
        if let Some(tx) = self.shutdown.take() {
            let _ = tx.send(());
        }
        let _ = (&mut self.server).await;
    }
}

/// Open the database, bind the listener, and start serving.
///
/// Binding happens before this returns, so tests can bind port 0 and connect to
/// `addr` immediately without polling for readiness.
pub async fn start(config: Config) -> Result<RunningServer> {
    let conn = db::open(&config.db_path)?;
    let state: SharedState = Arc::new(AppState {
        db: Mutex::new(conn),
        client: fetch::client().context("cannot build HTTP client")?,
    });

    let listener = TcpListener::bind(config.listen)
        .await
        .with_context(|| format!("cannot listen on {}", config.listen))?;
    let addr = listener.local_addr()?;

    let (tx, rx) = oneshot::channel();
    let app = api::router(state.clone());
    let server = tokio::spawn(async move {
        let server = axum::serve(listener, app).with_graceful_shutdown(async {
            let _ = rx.await;
        });
        if let Err(e) = server.await {
            tracing::error!("server error: {e}");
        }
    });

    let poller = spawn_poller(state.clone(), config.poll_interval);

    Ok(RunningServer {
        addr,
        state,
        shutdown: Some(tx),
        server,
        poller,
    })
}

/// Refresh everything every `interval` seconds. `0` means no polling at all —
/// which is what the tests use, so that the only fetches are the ones they ask
/// for through the refresh endpoints.
fn spawn_poller(state: SharedState, interval: u64) -> Option<JoinHandle<()>> {
    if interval == 0 {
        return None;
    }

    Some(tokio::spawn(async move {
        let mut ticker = tokio::time::interval(Duration::from_secs(interval));
        // A poll that overruns the interval must not be followed by a burst of
        // queued ticks hammering every origin back-to-back; wait a full interval
        // after each poll instead.
        ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
        // The first tick fires immediately; skip it so that starting the server
        // does not itself cause a fetch.
        ticker.tick().await;
        loop {
            ticker.tick().await;
            let results = refresh::refresh_all(&state).await;
            let failed = results.iter().filter(|r| !r.is_ok()).count();
            tracing::info!(
                feeds = results.len(),
                failed,
                new_entries = results.iter().map(|r| r.new_entries).sum::<i64>(),
                "poll finished"
            );
        }
    }))
}
