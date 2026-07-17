//! Thin shim over the `feedctl` library.
//!
//! Exit codes are the contract here, so `main` owns them and nothing else does:
//! 0 success, 1 the server answered with an error, 2 unreachable or bad usage.
//! clap already exits 2 on a usage error, which is why `parse()` needs no
//! special handling.

use std::io::Write;
use std::process::ExitCode;

use clap::Parser;
use feedctl::Cli;

#[tokio::main]
async fn main() -> ExitCode {
    let cli = Cli::parse();
    let stdout = std::io::stdout();
    let mut out = stdout.lock();

    match feedctl::run(cli, &mut out).await {
        Ok(()) => {
            let _ = out.flush();
            ExitCode::SUCCESS
        }
        Err(e) => {
            let _ = out.flush();
            eprintln!("feedctl: {}", e.message());
            ExitCode::from(e.exit_code())
        }
    }
}
