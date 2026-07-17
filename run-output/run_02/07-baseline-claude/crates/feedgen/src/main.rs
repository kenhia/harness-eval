//! `feedgen` — serve test feeds over local HTTP, and generate the corpus.

use std::net::SocketAddr;
use std::path::PathBuf;

use clap::{Parser, Subcommand};

use feedgen::fixtures;
use feedgen::serve::serve_dir;

#[derive(Parser)]
#[command(
    name = "feedgen",
    version,
    about = "Serve fixture feeds over local HTTP for developing and testing feedd"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Serve the files in DIR over HTTP, with ETag/Last-Modified and 304 support
    Serve {
        /// Directory of feed files to serve
        #[arg(long)]
        dir: PathBuf,
        /// Address to listen on
        #[arg(long, default_value = "127.0.0.1:8601")]
        listen: SocketAddr,
    },
    /// Write the fixture corpus, and a README describing it, into DIR
    MakeFixtures {
        /// Directory to write into; created if it does not exist
        dir: PathBuf,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    match Cli::parse().command {
        Command::Serve { dir, listen } => {
            let server = serve_dir(dir.clone(), listen).await?;
            println!(
                "feedgen: serving {} on http://{}",
                dir.display(),
                server.addr
            );
            tokio::signal::ctrl_c().await?;
            server.shutdown().await;
            Ok(())
        }
        Command::MakeFixtures { dir } => {
            for path in fixtures::write_fixtures(&dir)? {
                println!("{}", path.display());
            }
            Ok(())
        }
    }
}
