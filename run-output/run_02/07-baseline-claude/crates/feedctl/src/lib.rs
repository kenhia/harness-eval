//! `feedctl` — the command-line client for `feedd`.
//!
//! The library holds the whole implementation so that tests can exercise the
//! argument parsing and the rendering directly; `main.rs` only maps the outcome
//! onto a process exit code.

pub mod cli;
pub mod client;
pub mod render;

pub use cli::{Cli, run};

/// A command that did not succeed, and the exit code it should produce.
///
/// The codes are part of the interface: `1` means the server answered with an
/// error, `2` means it could not be reached at all (invalid usage also exits
/// `2`, by way of clap).
#[derive(Debug)]
pub struct CliError {
    pub code: i32,
    pub message: String,
}

impl CliError {
    /// The server answered, and what it said was an error.
    pub fn server(message: impl Into<String>) -> Self {
        CliError {
            code: 1,
            message: message.into(),
        }
    }

    /// The server could not be reached, or the request could not be made.
    pub fn unreachable(message: impl Into<String>) -> Self {
        CliError {
            code: 2,
            message: message.into(),
        }
    }
}

impl std::fmt::Display for CliError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for CliError {}
