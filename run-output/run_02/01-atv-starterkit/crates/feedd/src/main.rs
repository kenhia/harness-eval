//! `feedd` — the feedhub server binary.

mod db;
mod fetch;
mod refresh;
mod server;

use clap::Parser;
use db::Store;
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// feedhub server: fetches and stores RSS/Atom feeds and serves a REST API.
#[derive(Parser, Debug)]
#[command(name = "feedd", version, about)]
struct Args {
    /// Path to the SQLite database file.
    #[arg(long)]
    db: String,

    /// Address to listen on.
    #[arg(long, default_value = "127.0.0.1:8600")]
    listen: String,

    /// Background poll interval in seconds; 0 disables polling.
    #[arg(long, default_value_t = 300)]
    poll_interval: u64,
}

fn main() {
    let args = Args::parse();

    let store = match Store::open(&args.db) {
        Ok(s) => Arc::new(Mutex::new(s)),
        Err(e) => {
            eprintln!("feedd: failed to open database {}: {e}", args.db);
            std::process::exit(1);
        }
    };

    if args.poll_interval > 0 {
        let store = Arc::clone(&store);
        let interval = Duration::from_secs(args.poll_interval);
        std::thread::spawn(move || loop {
            std::thread::sleep(interval);
            let mut guard = store.lock().unwrap();
            let _ = refresh::refresh_all(&mut guard);
        });
    }

    println!("feedd listening on {}", args.listen);
    if let Err(e) = server::serve(store, &args.listen) {
        eprintln!("feedd: server error: {e}");
        std::process::exit(1);
    }
}
