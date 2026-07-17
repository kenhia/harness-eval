//! `feedd` — the feedhub server binary.
//!
//! Fetches and stores RSS/Atom feeds in SQLite and exposes a REST API. A
//! background poller refreshes feeds on an interval (disabled with
//! `--poll-interval 0`); refresh endpoints always fetch on demand.

mod api;
mod fetch;
mod store;

use std::process::ExitCode;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use clap::Parser;

use store::Store;

#[derive(Parser)]
#[command(name = "feedd", about = "feedhub feed-aggregation server.")]
struct Cli {
    /// Path to the SQLite database file (created if missing).
    #[arg(long)]
    db: String,
    /// Address to listen on.
    #[arg(long, default_value = "127.0.0.1:8600")]
    listen: String,
    /// Background poll interval in seconds; 0 disables polling.
    #[arg(long, default_value_t = 300)]
    poll_interval: u64,
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    let store = match Store::open(&cli.db) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("feedd: failed to open database {}: {e}", cli.db);
            return ExitCode::from(2);
        }
    };

    let agent = fetch::build_agent();

    let server = match tiny_http::Server::http(&cli.listen) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("feedd: failed to bind {}: {e}", cli.listen);
            return ExitCode::from(2);
        }
    };
    eprintln!("feedd: listening on {}", cli.listen);

    // Background poller.
    if cli.poll_interval > 0 {
        let poll_store = store.clone();
        let poll_agent = agent.clone();
        let interval = Duration::from_secs(cli.poll_interval);
        thread::spawn(move || loop {
            thread::sleep(interval);
            let _ = fetch::refresh_all(&poll_store, &poll_agent);
        });
    }

    let server = Arc::new(server);
    for request in server.incoming_requests() {
        api::handle(&store, &agent, request);
    }

    ExitCode::SUCCESS
}
