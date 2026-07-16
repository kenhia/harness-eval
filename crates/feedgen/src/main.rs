//! feedgen CLI: serve a fixture directory over HTTP, or generate the corpus.

use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "feedgen", about = "Serve and generate local test feeds for feedhub")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Serve files in DIR over HTTP with ETag/Last-Modified + conditional GET.
    Serve {
        /// Directory of files to serve.
        #[arg(long)]
        dir: PathBuf,
        /// Address to listen on.
        #[arg(long, default_value = "127.0.0.1:8700")]
        listen: String,
    },
    /// Write the fixture corpus into DIR.
    MakeFixtures {
        /// Target directory (created if missing).
        dir: PathBuf,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Serve { dir, listen } => {
            eprintln!("feedgen serving {} on http://{listen}", dir.display());
            feedgen::serve_forever(dir, &listen).await?;
        }
        Command::MakeFixtures { dir } => {
            feedgen::fixtures::make_fixtures(&dir)?;
            println!("wrote fixture corpus to {}", dir.display());
        }
    }
    Ok(())
}
