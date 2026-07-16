//! feedd CLI entry point.

use anyhow::Result;
use clap::Parser;

#[derive(Parser)]
#[command(name = "feedd", about = "feedhub feed-aggregation server")]
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

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "feedd=info".into()),
        )
        .init();

    let cli = Cli::parse();
    feedd::run(&cli.db, &cli.listen, cli.poll_interval).await
}
