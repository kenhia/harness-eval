//! feedctl: a CLI client for the feedd REST API.

use std::process::ExitCode;
use std::time::Duration;

use clap::{Parser, Subcommand};
use reqwest::blocking::Client;
use reqwest::StatusCode;
use serde_json::{json, Value};

#[derive(Parser)]
#[command(
    name = "feedctl",
    about = "Command-line client for the feedd feed aggregator"
)]
struct Cli {
    /// Base URL of the feedd server.
    #[arg(long, default_value = "http://127.0.0.1:8600", global = true)]
    server: String,
    /// Output format.
    #[arg(long, value_enum, default_value_t = Format::Text, global = true)]
    format: Format,
    #[command(subcommand)]
    command: Command,
}

#[derive(Clone, Copy, PartialEq, clap::ValueEnum)]
enum Format {
    Text,
    Json,
}

#[derive(Subcommand)]
enum Command {
    /// Register a feed by URL.
    Add { url: String },
    /// List all feeds.
    List,
    /// Show a single feed.
    Show { id: i64 },
    /// Remove a feed (and its entries).
    Remove { id: i64 },
    /// Refresh one feed, or all feeds when no ID is given.
    Refresh { id: Option<i64> },
    /// Query entries.
    Entries {
        #[arg(long)]
        feed: Option<i64>,
        #[arg(long)]
        since: Option<String>,
        #[arg(long)]
        until: Option<String>,
        #[arg(long)]
        search: Option<String>,
        #[arg(long)]
        limit: Option<i64>,
        #[arg(long)]
        offset: Option<i64>,
    },
}

/// A client-side error, carrying the process exit code it maps to.
enum CtlError {
    /// The server responded with an error (4xx/5xx). Exit code 1.
    Api(String),
    /// The server was unreachable. Exit code 2.
    Unreachable(String),
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match run(&cli) {
        Ok(()) => ExitCode::from(0),
        Err(CtlError::Api(msg)) => {
            eprintln!("error: {msg}");
            ExitCode::from(1)
        }
        Err(CtlError::Unreachable(msg)) => {
            eprintln!("error: {msg}");
            ExitCode::from(2)
        }
    }
}

fn run(cli: &Cli) -> Result<(), CtlError> {
    let client = Client::builder()
        .timeout(Duration::from_secs(60))
        .build()
        .map_err(|e| CtlError::Unreachable(e.to_string()))?;
    let base = cli.server.trim_end_matches('/');

    match &cli.command {
        Command::Add { url } => {
            let v = send(
                &client,
                Method::Post,
                &format!("{base}/api/feeds"),
                Some(json!({"url": url})),
            )?;
            output(cli.format, &v, |v| {
                format!("Added feed {}: {}", v["id"], as_str(&v["url"]))
            });
        }
        Command::List => {
            let v = send(&client, Method::Get, &format!("{base}/api/feeds"), None)?;
            output(cli.format, &v, render_feed_list);
        }
        Command::Show { id } => {
            let v = send(
                &client,
                Method::Get,
                &format!("{base}/api/feeds/{id}"),
                None,
            )?;
            output(cli.format, &v, render_feed);
        }
        Command::Remove { id } => {
            let v = send(
                &client,
                Method::Delete,
                &format!("{base}/api/feeds/{id}"),
                None,
            )?;
            output(cli.format, &v, |_| format!("Removed feed {id}"));
        }
        Command::Refresh { id } => {
            let path = match id {
                Some(id) => format!("{base}/api/feeds/{id}/refresh"),
                None => format!("{base}/api/refresh"),
            };
            let v = send(&client, Method::Post, &path, None)?;
            output(cli.format, &v, render_refresh);
        }
        Command::Entries {
            feed,
            since,
            until,
            search,
            limit,
            offset,
        } => {
            let mut qs: Vec<String> = Vec::new();
            if let Some(f) = feed {
                qs.push(format!("feed_id={f}"));
            }
            if let Some(s) = since {
                qs.push(format!("since={}", urlencode(s)));
            }
            if let Some(u) = until {
                qs.push(format!("until={}", urlencode(u)));
            }
            if let Some(s) = search {
                qs.push(format!("q={}", urlencode(s)));
            }
            if let Some(l) = limit {
                qs.push(format!("limit={l}"));
            }
            if let Some(o) = offset {
                qs.push(format!("offset={o}"));
            }
            let path = if qs.is_empty() {
                format!("{base}/api/entries")
            } else {
                format!("{base}/api/entries?{}", qs.join("&"))
            };
            let v = send(&client, Method::Get, &path, None)?;
            output(cli.format, &v, render_entries);
        }
    }
    Ok(())
}

enum Method {
    Get,
    Post,
    Delete,
}

/// Perform a request and return the parsed JSON body (or a synthetic object
/// for empty `204` responses). Maps transport failures to
/// [`CtlError::Unreachable`] and error statuses to [`CtlError::Api`].
fn send(
    client: &Client,
    method: Method,
    url: &str,
    body: Option<Value>,
) -> Result<Value, CtlError> {
    let mut req = match method {
        Method::Get => client.get(url),
        Method::Post => client.post(url),
        Method::Delete => client.delete(url),
    };
    if let Some(b) = body {
        req = req.json(&b);
    }
    let resp = req
        .send()
        .map_err(|e| CtlError::Unreachable(e.to_string()))?;
    let status = resp.status();
    if status == StatusCode::NO_CONTENT {
        return Ok(json!({ "status": "ok" }));
    }
    let text = resp
        .text()
        .map_err(|e| CtlError::Unreachable(e.to_string()))?;
    let value: Value = serde_json::from_str(&text).unwrap_or_else(|_| json!({ "raw": text }));
    if status.is_success() {
        Ok(value)
    } else {
        let msg = value
            .get("error")
            .and_then(|e| e.as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| format!("server returned HTTP {}", status.as_u16()));
        Err(CtlError::Api(msg))
    }
}

fn output(format: Format, value: &Value, render: impl Fn(&Value) -> String) {
    match format {
        Format::Json => println!("{}", serde_json::to_string(value).unwrap()),
        Format::Text => println!("{}", render(value)),
    }
}

fn as_str(v: &Value) -> String {
    v.as_str().map(|s| s.to_string()).unwrap_or_else(|| {
        if v.is_null() {
            "-".to_string()
        } else {
            v.to_string()
        }
    })
}

fn render_feed(v: &Value) -> String {
    format!(
        "feed {}\n  url:        {}\n  title:      {}\n  entries:    {}\n  last fetch: {}\n  last error: {}",
        v["id"],
        as_str(&v["url"]),
        as_str(&v["title"]),
        v["entry_count"],
        as_str(&v["last_fetched_at"]),
        as_str(&v["last_error"]),
    )
}

fn render_feed_list(v: &Value) -> String {
    let empty = vec![];
    let feeds = v.as_array().unwrap_or(&empty);
    if feeds.is_empty() {
        return "(no feeds)".to_string();
    }
    feeds
        .iter()
        .map(|f| {
            format!(
                "{:>4}  {:<28}  entries={:<4}  {}",
                f["id"],
                truncate(&as_str(&f["title"]), 28),
                f["entry_count"],
                as_str(&f["url"]),
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn render_refresh(v: &Value) -> String {
    match v {
        Value::Array(items) => items
            .iter()
            .map(render_one_refresh)
            .collect::<Vec<_>>()
            .join("\n"),
        other => render_one_refresh(other),
    }
}

fn render_one_refresh(v: &Value) -> String {
    let base = format!(
        "feed {}: {} ({} new)",
        v["feed_id"],
        as_str(&v["status"]),
        v["new_entries"]
    );
    match v.get("error").and_then(|e| e.as_str()) {
        Some(err) => format!("{base} — {err}"),
        None => base,
    }
}

fn render_entries(v: &Value) -> String {
    let total = v["total"].as_i64().unwrap_or(0);
    let empty = vec![];
    let items = v["items"].as_array().unwrap_or(&empty);
    let mut out = format!("{total} entries total, showing {}", items.len());
    for item in items {
        out.push_str(&format!(
            "\n  [{}] {}\n      {}",
            as_str(&item["published_at"]),
            as_str(&item["title"]),
            as_str(&item["link"]),
        ));
    }
    out
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let mut t: String = s.chars().take(max.saturating_sub(1)).collect();
        t.push('…');
        t
    }
}

/// Minimal percent-encoding for query values (encodes the characters that
/// matter for RFC 3339 timestamps and free-text search).
fn urlencode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char)
            }
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}
