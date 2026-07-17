//! The feedhub server.
//!
//! Everything lives here rather than in `main.rs` so integration tests drive
//! exactly the code the binary runs. See [`serve`] for the test entry point:
//! it takes an already-bound listener, so a test can learn the port before the
//! server starts and never has to sleep for readiness.

#![forbid(unsafe_code)]

pub mod api;
pub mod cli;
pub mod fetch;
pub mod poll;
pub mod store;

use std::sync::Arc;
use std::time::Duration;

use api::AppState;
use fetch::Fetcher;
use store::Store;

/// Build the application state from an open store.
pub fn state(store: Arc<Store>) -> anyhow::Result<AppState> {
    Ok(AppState {
        store,
        fetcher: Arc::new(Fetcher::new()?),
    })
}

/// Serve on an already-bound listener until the future is dropped.
pub async fn serve(listener: std::net::TcpListener, state: AppState) -> anyhow::Result<()> {
    listener.set_nonblocking(true)?;
    let listener = tokio::net::TcpListener::from_std(listener)?;
    axum::serve(listener, api::router(state)).await?;
    Ok(())
}

/// Run the server as the binary does: open the database, start polling, serve.
pub async fn run(cli: cli::Cli) -> anyhow::Result<()> {
    let store = Arc::new(
        Store::open(&cli.db)
            .map_err(|e| anyhow::anyhow!("cannot open database {}: {e}", cli.db.display()))?,
    );
    let state = state(store.clone())?;

    // Independent of the API: a zero interval disables the loop, never the
    // refresh endpoints.
    let _poller = poll::spawn(
        store,
        state.fetcher.clone(),
        Duration::from_secs(cli.poll_interval),
    );

    let listener = std::net::TcpListener::bind(cli.listen)
        .map_err(|e| anyhow::anyhow!("cannot bind {}: {e}", cli.listen))?;
    let addr = listener.local_addr()?;
    tracing::info!(%addr, db = %cli.db.display(), "feedd listening");
    println!(
        "feedd listening on http://{addr} (db: {})",
        cli.db.display()
    );

    serve(listener, state).await
}

/// A feedd running in the background on an ephemeral port. Test-facing.
pub struct Spawned {
    pub addr: std::net::SocketAddr,
    pub store: Arc<Store>,
    handle: tokio::task::JoinHandle<()>,
}

impl Spawned {
    pub fn base_url(&self) -> String {
        format!("http://{}", self.addr)
    }

    pub fn url(&self, path: &str) -> String {
        format!("http://{}{}", self.addr, path)
    }
}

impl Drop for Spawned {
    fn drop(&mut self) {
        self.handle.abort();
    }
}

/// Bind `127.0.0.1:0` and serve in a background task, with polling disabled.
///
/// Tests drive the refresh endpoints directly; a timer racing the assertions is
/// exactly the nondeterminism worth avoiding.
pub fn spawn(store: Arc<Store>) -> anyhow::Result<Spawned> {
    let listener = std::net::TcpListener::bind("127.0.0.1:0")?;
    let addr = listener.local_addr()?;
    let state = state(store.clone())?;
    let handle = tokio::spawn(async move {
        let _ = serve(listener, state).await;
    });
    Ok(Spawned {
        addr,
        store,
        handle,
    })
}
