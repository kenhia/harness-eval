//! `feedd` — the feedhub server: fetches/stores RSS & Atom feeds and exposes a
//! REST API. Background polling is optional; refresh endpoints fetch on demand.

mod fetch;
mod refresh;
mod server;
mod store;

use std::sync::Arc;
use std::thread;
use std::time::Duration;

use clap::Parser;

use crate::refresh::refresh_all;
use crate::store::Store;

#[derive(Parser)]
#[command(
    name = "feedd",
    about = "feedhub server: fetch and store RSS/Atom feeds and serve a REST API."
)]
struct Cli {
    /// Path to the SQLite database file (created if missing).
    #[arg(long)]
    db: String,
    /// Address to listen on.
    #[arg(long, default_value = "127.0.0.1:8600")]
    listen: String,
    /// Background poll interval in seconds; 0 disables background polling.
    #[arg(long, default_value_t = 300)]
    poll_interval: u64,
}

fn main() {
    let cli = Cli::parse();
    let store = match Store::open(&cli.db) {
        Ok(s) => Arc::new(s),
        Err(e) => {
            eprintln!("feedd: failed to open database {}: {e}", cli.db);
            std::process::exit(1);
        }
    };

    if cli.poll_interval > 0 {
        spawn_poller(Arc::clone(&store), cli.poll_interval);
    }

    if let Err(e) = server::run(store, &cli.listen) {
        eprintln!("feedd: server error: {e}");
        std::process::exit(1);
    }
}

/// Spawn a background thread that refreshes all feeds on a fixed interval.
/// Failures are isolated per feed inside `refresh_all`, so the loop never dies.
fn spawn_poller(store: Arc<Store>, interval_secs: u64) {
    thread::spawn(move || loop {
        thread::sleep(Duration::from_secs(interval_secs));
        let _ = refresh_all(&store);
    });
}
