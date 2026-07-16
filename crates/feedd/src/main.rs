//! Thin shim over the `feedd` library.
//!
//! Everything lives in the lib target so integration tests drive the same code
//! the binary runs. `main.rs` must never declare modules of its own: that would
//! compile the tree a second time into types the lib's callers can't use.

use clap::Parser;
use feedd::cli::Cli;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "feedd=info".into()),
        )
        .init();

    feedd::run(Cli::parse()).await
}
