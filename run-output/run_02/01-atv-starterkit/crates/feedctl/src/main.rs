//! `feedctl` — the feedhub CLI client.

use clap::{Parser, Subcommand};
use serde_json::Value;
use std::process::ExitCode;

#[derive(Parser, Debug)]
#[command(name = "feedctl", version, about)]
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

#[derive(clap::ValueEnum, Clone, Copy, Debug, PartialEq)]
enum Format {
    Text,
    Json,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Register a new feed by URL.
    Add { url: String },
    /// List all registered feeds.
    List,
    /// Show one feed by id.
    Show { id: i64 },
    /// Remove a feed and its entries.
    Remove { id: i64 },
    /// Refresh one feed (by id) or all feeds.
    Refresh { id: Option<i64> },
    /// Query stored entries.
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

/// Exit codes: 0 success, 1 server error, 2 transport/usage error.
fn main() -> ExitCode {
    let cli = Cli::parse();
    match run(&cli) {
        Ok(()) => ExitCode::from(0),
        Err(AppError::Server(msg)) => {
            eprintln!("error: {msg}");
            ExitCode::from(1)
        }
        Err(AppError::Transport(msg)) => {
            eprintln!("error: {msg}");
            ExitCode::from(2)
        }
    }
}

enum AppError {
    /// The server answered with a JSON error (4xx/5xx).
    Server(String),
    /// The server was unreachable.
    Transport(String),
}

struct ApiResponse {
    body: Option<Value>,
}

fn run(cli: &Cli) -> Result<(), AppError> {
    let base = cli.server.trim_end_matches('/');
    match &cli.command {
        Command::Add { url } => {
            let resp = request(
                "POST",
                &format!("{base}/api/feeds"),
                Some(serde_json::json!({ "url": url })),
            )?;
            emit(cli.format, &resp, |v| {
                println!(
                    "added feed {} ({})",
                    field_i64(v, "id"),
                    field_str(v, "url")
                )
            })
        }
        Command::List => {
            let resp = request("GET", &format!("{base}/api/feeds"), None)?;
            emit(cli.format, &resp, print_feed_list)
        }
        Command::Show { id } => {
            let resp = request("GET", &format!("{base}/api/feeds/{id}"), None)?;
            emit(cli.format, &resp, print_feed)
        }
        Command::Remove { id } => {
            let resp = request("DELETE", &format!("{base}/api/feeds/{id}"), None)?;
            emit(cli.format, &resp, |_| println!("removed feed {id}"))
        }
        Command::Refresh { id } => {
            let url = match id {
                Some(id) => format!("{base}/api/feeds/{id}/refresh"),
                None => format!("{base}/api/refresh"),
            };
            let resp = request("POST", &url, None)?;
            emit(cli.format, &resp, print_refresh)
        }
        Command::Entries {
            feed,
            since,
            until,
            search,
            limit,
            offset,
        } => {
            let mut params: Vec<(String, String)> = Vec::new();
            if let Some(f) = feed {
                params.push(("feed_id".into(), f.to_string()));
            }
            if let Some(s) = since {
                params.push(("since".into(), s.clone()));
            }
            if let Some(u) = until {
                params.push(("until".into(), u.clone()));
            }
            if let Some(q) = search {
                params.push(("q".into(), q.clone()));
            }
            if let Some(l) = limit {
                params.push(("limit".into(), l.to_string()));
            }
            if let Some(o) = offset {
                params.push(("offset".into(), o.to_string()));
            }
            let qs = encode_query(&params);
            let url = if qs.is_empty() {
                format!("{base}/api/entries")
            } else {
                format!("{base}/api/entries?{qs}")
            };
            let resp = request("GET", &url, None)?;
            emit(cli.format, &resp, print_entries)
        }
    }
}

fn request(method: &str, url: &str, body: Option<Value>) -> Result<ApiResponse, AppError> {
    let req = ureq::request(method, url);
    let result = match body {
        Some(v) => req
            .set("Content-Type", "application/json")
            .send_string(&serde_json::to_string(&v).unwrap()),
        None => req.call(),
    };
    match result {
        Ok(resp) => Ok(read_response(resp)),
        Err(ureq::Error::Status(status, resp)) => {
            let parsed = read_response(resp);
            let msg = parsed
                .body
                .as_ref()
                .and_then(|b| b.get("error"))
                .and_then(Value::as_str)
                .map(str::to_string)
                .unwrap_or_else(|| format!("server returned HTTP {status}"));
            Err(AppError::Server(msg))
        }
        Err(ureq::Error::Transport(t)) => {
            Err(AppError::Transport(format!("cannot reach server: {t}")))
        }
    }
}

fn read_response(resp: ureq::Response) -> ApiResponse {
    let text = resp.into_string().unwrap_or_default();
    let body = serde_json::from_str::<Value>(&text).ok();
    ApiResponse { body }
}

fn emit(format: Format, resp: &ApiResponse, text: impl FnOnce(&Value)) -> Result<(), AppError> {
    let body = resp.body.clone().unwrap_or(Value::Null);
    match format {
        Format::Json => println!("{}", serde_json::to_string(&body).unwrap()),
        Format::Text => text(&body),
    }
    Ok(())
}

fn field_str(v: &Value, key: &str) -> String {
    v.get(key)
        .and_then(Value::as_str)
        .map(str::to_string)
        .unwrap_or_else(|| "-".to_string())
}

fn field_i64(v: &Value, key: &str) -> i64 {
    v.get(key).and_then(Value::as_i64).unwrap_or_default()
}

fn opt_str(v: &Value, key: &str) -> String {
    match v.get(key) {
        Some(Value::String(s)) => s.clone(),
        _ => "-".to_string(),
    }
}

fn print_feed_list(v: &Value) {
    let Some(feeds) = v.as_array() else { return };
    if feeds.is_empty() {
        println!("(no feeds)");
        return;
    }
    for f in feeds {
        print_feed(f);
    }
}

fn print_feed(v: &Value) {
    println!("[{}] {}", field_i64(v, "id"), opt_str(v, "title"));
    println!("     url:        {}", field_str(v, "url"));
    println!("     entries:    {}", field_i64(v, "entry_count"));
    println!("     fetched:    {}", opt_str(v, "last_fetched_at"));
    let err = opt_str(v, "last_error");
    if err != "-" {
        println!("     last_error: {err}");
    }
}

fn print_refresh(v: &Value) {
    let results: Vec<Value> = match v {
        Value::Array(a) => a.clone(),
        other => vec![other.clone()],
    };
    for r in &results {
        let status = field_str(r, "status");
        let base = format!(
            "feed {}: {} ({} new)",
            field_i64(r, "id"),
            status,
            field_i64(r, "new_entries")
        );
        match r.get("error").and_then(Value::as_str) {
            Some(e) => println!("{base} - {e}"),
            None => println!("{base}"),
        }
    }
}

fn print_entries(v: &Value) {
    let total = field_i64(v, "total");
    let items = v
        .get("items")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    println!("{total} entries (showing {})", items.len());
    for e in &items {
        println!(
            "- [{}] {}",
            opt_str(e, "published_at"),
            field_str(e, "title")
        );
        let link = opt_str(e, "link");
        if link != "-" {
            println!("      {link}");
        }
    }
}

/// Percent-encode query parameters (encodes everything but unreserved chars).
fn encode_query(params: &[(String, String)]) -> String {
    params
        .iter()
        .map(|(k, v)| format!("{}={}", percent_encode(k), percent_encode(v)))
        .collect::<Vec<_>>()
        .join("&")
}

fn percent_encode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for &b in s.as_bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char)
            }
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}
