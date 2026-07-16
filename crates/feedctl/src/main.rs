//! `feedctl` — drive a feedd server from the command line.
//!
//! Exit codes: `0` success, `1` the server answered with an error, `2` the
//! server is unreachable or the usage is invalid (clap exits `2` itself).

use clap::Parser;

use feedctl::{Cli, run};

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    if let Err(e) = run(cli).await {
        eprintln!("feedctl: {e}");
        std::process::exit(e.code);
    }
}
