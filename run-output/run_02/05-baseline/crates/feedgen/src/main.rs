//! `feedgen` binary entry point.

use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "feedgen",
    about = "Serve local test feeds and generate a fixture corpus."
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Serve the files in a directory over HTTP.
    Serve {
        /// Directory of files to serve.
        #[arg(long)]
        dir: PathBuf,
        /// Address to listen on.
        #[arg(long, default_value = "127.0.0.1:8700")]
        listen: String,
    },
    /// Write the fixture corpus into a directory.
    MakeFixtures {
        /// Destination directory (created if missing).
        dir: PathBuf,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match cli.command {
        Command::Serve { dir, listen } => {
            if !dir.is_dir() {
                eprintln!("feedgen: not a directory: {}", dir.display());
                return ExitCode::from(2);
            }
            let server = match feedgen::bind(&listen) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("feedgen: failed to bind {listen}: {e}");
                    return ExitCode::from(2);
                }
            };
            eprintln!("feedgen: serving {} on {listen}", dir.display());
            feedgen::serve(&server, &dir);
            ExitCode::SUCCESS
        }
        Command::MakeFixtures { dir } => match feedgen::make_fixtures(&dir) {
            Ok(()) => {
                eprintln!("feedgen: wrote fixtures to {}", dir.display());
                ExitCode::SUCCESS
            }
            Err(e) => {
                eprintln!("feedgen: failed to write fixtures: {e}");
                ExitCode::from(1)
            }
        },
    }
}
