//! `feedd` — fetch and store feeds, and serve the REST API.

use std::net::SocketAddr;
use std::path::PathBuf;

use clap::Parser;

use feedd::server::{Config, start};

#[derive(Parser)]
#[command(
    name = "feedd",
    version,
    about = "Feed aggregation server: fetches RSS/Atom feeds into SQLite and serves a REST API"
)]
struct Cli {
    /// SQLite database file; created if it does not exist
    #[arg(long, value_name = "PATH")]
    db: PathBuf,
    /// Address to serve the API on
    #[arg(long, value_name = "ADDR:PORT", default_value = "127.0.0.1:8600")]
    listen: SocketAddr,
    /// Seconds between background refreshes of every feed; 0 disables polling
    #[arg(long, value_name = "SECS", default_value_t = 300)]
    poll_interval: u64,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "feedd=info".into()),
        )
        .init();

    let cli = Cli::parse();
    let server = start(Config {
        db_path: cli.db,
        listen: cli.listen,
        poll_interval: cli.poll_interval,
    })
    .await?;

    tracing::info!(
        "feedd listening on {} (poll interval: {})",
        server.addr,
        match cli.poll_interval {
            0 => "disabled".to_string(),
            secs => format!("{secs}s"),
        }
    );

    tokio::select! {
        result = server.wait() => result,
        _ = tokio::signal::ctrl_c() => Ok(()),
    }
}
