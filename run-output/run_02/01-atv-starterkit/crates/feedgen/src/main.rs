//! `feedgen` — fixture tool: serve a directory of feeds over HTTP and
//! generate a fixture corpus for testing feedhub without the real internet.

use clap::{Parser, Subcommand};
use feedgen::{fixtures, serve};

#[derive(Parser, Debug)]
#[command(name = "feedgen", version, about)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Serve the files in DIR over HTTP with ETag/Last-Modified support.
    Serve {
        /// Directory of files to serve.
        #[arg(long)]
        dir: String,
        /// Address to listen on.
        #[arg(long, default_value = "127.0.0.1:8700")]
        listen: String,
    },
    /// Write the fixture corpus into DIR.
    MakeFixtures {
        /// Target directory (created if missing).
        dir: String,
    },
}

fn main() {
    let args = Args::parse();
    let result = match args.command {
        Command::Serve { dir, listen } => serve::run(&dir, &listen),
        Command::MakeFixtures { dir } => fixtures::write(&dir),
    };
    if let Err(e) = result {
        eprintln!("feedgen: {e}");
        std::process::exit(1);
    }
}
