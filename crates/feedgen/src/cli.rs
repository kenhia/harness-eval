//! `feedgen` command line.

use std::net::SocketAddr;
use std::path::PathBuf;

use clap::{Parser, Subcommand};

/// Default listen address. Sits next to feedd's 8600 so the two can run side by
/// side without a flag.
pub const DEFAULT_LISTEN: &str = "127.0.0.1:8601";

#[derive(Debug, Parser)]
#[command(
    name = "feedgen",
    version,
    about = "Serve feed fixtures over local HTTP, and generate the fixture corpus.",
    long_about = "feedgen serves a directory of feed files over HTTP with correct \
                  Content-Types, content-derived ETags, and Last-Modified stamps, \
                  honoring If-None-Match and If-Modified-Since with 304. It exists so \
                  feedd can be developed and tested without touching the real internet.\n\n\
                  The generated corpus covers a valid RSS 2.0 feed, a valid Atom feed, \
                  edge-case dates (RFC 822 zone names, a missing date, an unparseable \
                  date), CDATA and entity handling, and a malformed-XML file. \
                  `make-fixtures` also writes a README.md cataloging them."
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Serve the files in DIR over HTTP.
    Serve {
        /// Directory of feed files to serve.
        #[arg(long, value_name = "DIR")]
        dir: PathBuf,

        /// Address to listen on.
        #[arg(long, value_name = "ADDR:PORT", default_value = DEFAULT_LISTEN)]
        listen: SocketAddr,

        /// Ignore If-None-Match / If-Modified-Since and always answer 200.
        ///
        /// Useful for exercising a client's re-parse path with unchanged
        /// content, which a 304 would otherwise short-circuit.
        #[arg(long)]
        no_conditional: bool,
    },

    /// Write the fixture corpus (and a README cataloging it) into DIR.
    MakeFixtures {
        /// Destination directory. Created if it does not exist.
        #[arg(value_name = "DIR")]
        dir: PathBuf,
    },
}

/// Run the CLI.
pub async fn run(cli: Cli) -> anyhow::Result<()> {
    match cli.command {
        Command::Serve {
            dir,
            listen,
            no_conditional,
        } => {
            if !dir.is_dir() {
                anyhow::bail!("--dir {} is not a directory", dir.display());
            }
            let listener = std::net::TcpListener::bind(listen)
                .map_err(|e| anyhow::anyhow!("cannot bind {listen}: {e}"))?;
            let addr = listener.local_addr()?;
            tracing::info!(%addr, dir = %dir.display(), "feedgen serving");
            println!("feedgen serving {} on http://{}", dir.display(), addr);

            let opts = crate::Options::new(dir).conditional(!no_conditional);
            crate::serve(listener, opts, crate::RequestLog::new()).await
        }

        Command::MakeFixtures { dir } => {
            let written = crate::fixtures::write_corpus(&dir)
                .map_err(|e| anyhow::anyhow!("cannot write corpus to {}: {e}", dir.display()))?;
            for name in &written {
                println!("{}", dir.join(name).display());
            }
            println!("\n{} files written. Serve them with:", written.len());
            println!("  feedgen serve --dir {}", dir.display());
            Ok(())
        }
    }
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
    fn serve_defaults_to_8601() {
        let cli = Cli::parse_from(["feedgen", "serve", "--dir", "/tmp/x"]);
        match cli.command {
            Command::Serve {
                listen,
                dir,
                no_conditional,
            } => {
                assert_eq!(listen.to_string(), DEFAULT_LISTEN);
                assert_eq!(dir, PathBuf::from("/tmp/x"));
                assert!(!no_conditional, "conditional GET is on by default");
            }
            other => panic!("expected serve, got {other:?}"),
        }
    }

    #[test]
    fn make_fixtures_takes_a_positional_dir() {
        let cli = Cli::parse_from(["feedgen", "make-fixtures", "/tmp/corpus"]);
        match cli.command {
            Command::MakeFixtures { dir } => assert_eq!(dir, PathBuf::from("/tmp/corpus")),
            other => panic!("expected make-fixtures, got {other:?}"),
        }
    }
}
