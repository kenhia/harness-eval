//! Argument parsing and command dispatch.

use clap::{Args, Parser, Subcommand, ValueEnum};
use feedhub_core::api::{EntriesPage, Feed, RefreshResult};
use serde::de::DeserializeOwned;
use serde_json::{Value, json};

use crate::CliError;
use crate::client::Api;
use crate::render;

#[derive(Parser)]
#[command(
    name = "feedctl",
    version,
    about = "Command-line client for a feedd server"
)]
pub struct Cli {
    /// Base URL of the feedd server
    #[arg(long, value_name = "URL", default_value = "http://127.0.0.1:8600")]
    pub server: String,
    /// Output format: human-readable text, or the raw API response as JSON
    #[arg(long, value_enum, default_value_t = Format::Text)]
    pub format: Format,
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum Format {
    Text,
    Json,
}

#[derive(Subcommand)]
pub enum Command {
    /// Register a feed
    Add {
        /// http(s) URL of an RSS or Atom feed
        url: String,
    },
    /// List every registered feed
    List,
    /// Show one feed
    Show { id: i64 },
    /// Remove a feed and its entries
    Remove { id: i64 },
    /// Fetch a feed now, or every feed if no id is given
    Refresh { id: Option<i64> },
    /// Query stored entries
    Entries(EntriesArgs),
}

#[derive(Args)]
pub struct EntriesArgs {
    /// Only entries from this feed
    #[arg(long, value_name = "ID")]
    pub feed: Option<i64>,
    /// Only entries published at or after this RFC 3339 instant
    #[arg(long, value_name = "T")]
    pub since: Option<String>,
    /// Only entries published strictly before this RFC 3339 instant
    #[arg(long, value_name = "T")]
    pub until: Option<String>,
    /// Case-insensitive substring of the title
    #[arg(long, value_name = "Q")]
    pub search: Option<String>,
    /// Maximum entries to return (max 500)
    #[arg(long, value_name = "N")]
    pub limit: Option<i64>,
    /// Entries to skip
    #[arg(long, value_name = "N")]
    pub offset: Option<i64>,
}

/// Run a parsed command, printing to stdout.
///
/// In `--format json` the raw API response is printed as one JSON document, so
/// the output is always pipeable into `jq`.
pub async fn run(cli: Cli) -> Result<(), CliError> {
    let api = Api::new(&cli.server);
    let json_output = cli.format == Format::Json;

    match cli.command {
        Command::Add { url } => {
            let body = api.post("/api/feeds", Some(json!({"url": url}))).await?;
            print(json_output, &body, || {
                Ok(render::feed_added(&decode::<Feed>(&body)?))
            })
        }
        Command::List => {
            let body = api.get("/api/feeds", &[]).await?;
            print(json_output, &body, || {
                Ok(render::feed_list(&decode::<Vec<Feed>>(&body)?))
            })
        }
        Command::Show { id } => {
            let body = api.get(&format!("/api/feeds/{id}"), &[]).await?;
            print(json_output, &body, || {
                Ok(render::feed_detail(&decode::<Feed>(&body)?))
            })
        }
        Command::Remove { id } => {
            // 204, so there is no response body to echo; JSON callers still get
            // a document rather than an empty stdout.
            api.delete(&format!("/api/feeds/{id}")).await?;
            let body = json!({"status": "ok", "id": id});
            print(json_output, &body, || Ok(render::feed_removed(id)))
        }
        Command::Refresh { id: Some(id) } => {
            let body = api.post(&format!("/api/feeds/{id}/refresh"), None).await?;
            print(json_output, &body, || {
                Ok(render::refresh_result(&decode::<RefreshResult>(&body)?))
            })
        }
        Command::Refresh { id: None } => {
            let body = api.post("/api/refresh", None).await?;
            print(json_output, &body, || {
                Ok(render::refresh_results(&decode::<Vec<RefreshResult>>(
                    &body,
                )?))
            })
        }
        Command::Entries(args) => {
            let body = api.get("/api/entries", &entries_query(&args)).await?;
            print(json_output, &body, || {
                Ok(render::entries(&decode::<EntriesPage>(&body)?))
            })
        }
    }
}

/// Build the query string. Values are passed through untouched — the server is
/// the one that decides whether a timestamp is valid, and it reports that as a
/// normal API error.
fn entries_query(args: &EntriesArgs) -> Vec<(String, String)> {
    let mut query = Vec::new();
    let mut push = |key: &str, value: Option<String>| {
        if let Some(value) = value {
            query.push((key.to_string(), value));
        }
    };
    push("feed_id", args.feed.map(|v| v.to_string()));
    push("since", args.since.clone());
    push("until", args.until.clone());
    push("q", args.search.clone());
    push("limit", args.limit.map(|v| v.to_string()));
    push("offset", args.offset.map(|v| v.to_string()));
    query
}

fn print(
    json_output: bool,
    body: &Value,
    text: impl FnOnce() -> Result<String, CliError>,
) -> Result<(), CliError> {
    if json_output {
        println!("{body}");
    } else {
        println!("{}", text()?);
    }
    Ok(())
}

/// Interpret a response body. A body that does not match means the server is
/// not the feedd we expect, which is a transport-level problem, not an API
/// error — so it exits 2.
fn decode<T: DeserializeOwned>(body: &Value) -> Result<T, CliError> {
    serde_json::from_value(body.clone())
        .map_err(|e| CliError::unreachable(format!("unexpected response from feedd: {e}")))
}
