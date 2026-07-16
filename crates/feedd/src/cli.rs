//! `feedd` command line.

use std::net::SocketAddr;
use std::path::PathBuf;

use clap::Parser;

pub const DEFAULT_LISTEN: &str = "127.0.0.1:8600";
pub const DEFAULT_POLL_INTERVAL: u64 = 300;

#[derive(Debug, Parser)]
#[command(
    name = "feedd",
    version,
    about = "The feedhub server: fetches and stores RSS/Atom feeds, and serves a REST API.",
    long_about = "feedd polls registered RSS 2.0 and Atom feeds into a SQLite database and \
                  exposes them over a JSON REST API.\n\n\
                  Refresh endpoints always fetch on demand, independently of background \
                  polling, so --poll-interval 0 disables the loop without disabling refresh."
)]
pub struct Cli {
    /// SQLite database file. Created if it does not exist.
    #[arg(long, value_name = "PATH")]
    pub db: PathBuf,

    /// Address to listen on.
    #[arg(long, value_name = "ADDR:PORT", default_value = DEFAULT_LISTEN)]
    pub listen: SocketAddr,

    /// Seconds between background polls. 0 disables background polling;
    /// refresh endpoints still fetch on demand.
    #[arg(long, value_name = "SECS", default_value_t = DEFAULT_POLL_INTERVAL)]
    pub poll_interval: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn cli_definition_is_valid() {
        Cli::command().debug_assert();
    }

    #[test]
    fn defaults_match_the_spec() {
        let cli = Cli::parse_from(["feedd", "--db", "/tmp/f.db"]);
        assert_eq!(cli.db, PathBuf::from("/tmp/f.db"));
        assert_eq!(cli.listen.to_string(), "127.0.0.1:8600");
        assert_eq!(cli.poll_interval, 300);
    }

    #[test]
    fn db_is_required() {
        assert!(Cli::try_parse_from(["feedd"]).is_err(), "--db has no default");
    }

    #[test]
    fn flags_override_the_defaults() {
        let cli = Cli::parse_from([
            "feedd",
            "--db",
            "/tmp/f.db",
            "--listen",
            "0.0.0.0:9000",
            "--poll-interval",
            "0",
        ]);
        assert_eq!(cli.listen.to_string(), "0.0.0.0:9000");
        assert_eq!(cli.poll_interval, 0, "0 is a valid value, meaning no polling");
    }
}
