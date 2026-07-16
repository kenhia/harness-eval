//! feedctl — a command-line client for the feedd REST API.

use clap::{Parser, Subcommand};
use serde_json::Value;

#[derive(Parser)]
#[command(
    name = "feedctl",
    version,
    about = "Command-line client for the feedd API"
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

#[derive(Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
enum Format {
    Text,
    Json,
}

#[derive(Subcommand)]
enum Command {
    /// Register a new feed by URL.
    Add { url: String },
    /// List all registered feeds.
    List,
    /// Show one feed by id.
    Show { id: i64 },
    /// Remove a feed (and its entries) by id.
    Remove { id: i64 },
    /// Refresh one feed, or all feeds when no id is given.
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

/// Exit codes per the spec.
const EXIT_OK: i32 = 0;
const EXIT_SERVER_ERROR: i32 = 1;
const EXIT_UNREACHABLE: i32 = 2;

enum CallResult {
    /// HTTP 2xx with a parsed JSON body (Null for empty bodies like 204).
    Ok(Value),
    /// HTTP 4xx/5xx; carries the server's error message.
    ServerError(String),
    /// Transport failure (server unreachable, DNS, timeout).
    Unreachable(String),
}

fn main() {
    let cli = Cli::parse();
    std::process::exit(dispatch(&cli));
}

fn dispatch(cli: &Cli) -> i32 {
    let base = cli.server.trim_end_matches('/');
    let agent = ureq::AgentBuilder::new()
        .timeout(std::time::Duration::from_secs(30))
        .build();

    let result = match &cli.command {
        Command::Add { url } => call(
            &agent,
            "POST",
            &format!("{base}/api/feeds"),
            Some(serde_json::json!({ "url": url })),
        ),
        Command::List => call(&agent, "GET", &format!("{base}/api/feeds"), None),
        Command::Show { id } => call(&agent, "GET", &format!("{base}/api/feeds/{id}"), None),
        Command::Remove { id } => call(&agent, "DELETE", &format!("{base}/api/feeds/{id}"), None),
        Command::Refresh { id } => {
            let url = match id {
                Some(id) => format!("{base}/api/feeds/{id}/refresh"),
                None => format!("{base}/api/refresh"),
            };
            call(&agent, "POST", &url, None)
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
                qs.push(format!("since={}", encode(s)));
            }
            if let Some(u) = until {
                qs.push(format!("until={}", encode(u)));
            }
            if let Some(s) = search {
                qs.push(format!("q={}", encode(s)));
            }
            if let Some(l) = limit {
                qs.push(format!("limit={l}"));
            }
            if let Some(o) = offset {
                qs.push(format!("offset={o}"));
            }
            let url = if qs.is_empty() {
                format!("{base}/api/entries")
            } else {
                format!("{base}/api/entries?{}", qs.join("&"))
            };
            call(&agent, "GET", &url, None)
        }
    };

    match result {
        CallResult::Ok(body) => {
            render_success(cli, &body);
            EXIT_OK
        }
        CallResult::ServerError(msg) => {
            eprintln!("error: {msg}");
            EXIT_SERVER_ERROR
        }
        CallResult::Unreachable(msg) => {
            eprintln!("error: {msg}");
            EXIT_UNREACHABLE
        }
    }
}

fn call(agent: &ureq::Agent, method: &str, url: &str, body: Option<Value>) -> CallResult {
    let req = agent.request(method, url);
    let response = match body {
        Some(v) => {
            let payload = serde_json::to_string(&v).unwrap_or_else(|_| "{}".into());
            req.set("Content-Type", "application/json")
                .send_string(&payload)
        }
        None => req.call(),
    };
    match response {
        Ok(resp) => {
            let text = resp.into_string().unwrap_or_default();
            let value = if text.trim().is_empty() {
                Value::Null
            } else {
                serde_json::from_str(&text).unwrap_or(Value::Null)
            };
            CallResult::Ok(value)
        }
        Err(ureq::Error::Status(_, resp)) => {
            let text = resp.into_string().unwrap_or_default();
            let msg = serde_json::from_str::<Value>(&text)
                .ok()
                .and_then(|v| v.get("error").and_then(|e| e.as_str()).map(String::from))
                .unwrap_or_else(|| {
                    if text.is_empty() {
                        "server error".into()
                    } else {
                        text
                    }
                });
            CallResult::ServerError(msg)
        }
        Err(ureq::Error::Transport(t)) => {
            CallResult::Unreachable(format!("server unreachable: {t}"))
        }
    }
}

fn render_success(cli: &Cli, body: &Value) {
    if cli.format == Format::Json {
        println!(
            "{}",
            serde_json::to_string(body).unwrap_or_else(|_| "null".into())
        );
        return;
    }
    match &cli.command {
        Command::Add { .. } | Command::Show { .. } => print_feed(body),
        Command::List => {
            if let Some(arr) = body.as_array() {
                if arr.is_empty() {
                    println!("(no feeds)");
                }
                for f in arr {
                    print_feed_line(f);
                }
            }
        }
        Command::Remove { id } => println!("removed feed {id}"),
        Command::Refresh { .. } => print_refresh(body),
        Command::Entries { .. } => print_entries(body),
    }
}

fn s<'a>(v: &'a Value, key: &str) -> &'a str {
    v.get(key).and_then(|x| x.as_str()).unwrap_or("")
}

fn null_or<'a>(v: &'a Value, key: &str, dflt: &'a str) -> &'a str {
    match v.get(key) {
        Some(Value::String(s)) => s,
        _ => dflt,
    }
}

fn print_feed(f: &Value) {
    println!("id:              {}", f.get("id").unwrap_or(&Value::Null));
    println!("url:             {}", s(f, "url"));
    println!("title:           {}", null_or(f, "title", "(none)"));
    println!(
        "last_fetched_at: {}",
        null_or(f, "last_fetched_at", "(never)")
    );
    println!("last_error:      {}", null_or(f, "last_error", "(none)"));
    println!(
        "entry_count:     {}",
        f.get("entry_count").unwrap_or(&Value::Null)
    );
}

fn print_feed_line(f: &Value) {
    println!(
        "{:>4}  {:<40}  entries={:<4}  {}",
        f.get("id").unwrap_or(&Value::Null),
        s(f, "url"),
        f.get("entry_count").unwrap_or(&Value::Null),
        null_or(f, "title", "(untitled feed)")
    );
}

fn print_refresh(body: &Value) {
    let results = match body {
        Value::Array(a) => a.clone(),
        other => vec![other.clone()],
    };
    for r in &results {
        let extra = if r
            .get("not_modified")
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
        {
            " (not modified)".to_string()
        } else if let Some(err) = r.get("error").and_then(|e| e.as_str()) {
            format!(" error={err}")
        } else {
            String::new()
        };
        println!(
            "feed {}: {} new_entries={}{}",
            r.get("feed_id").unwrap_or(&Value::Null),
            s(r, "status"),
            r.get("new_entries").unwrap_or(&Value::Null),
            extra
        );
    }
}

fn print_entries(body: &Value) {
    let total = body.get("total").and_then(|v| v.as_i64()).unwrap_or(0);
    println!("total: {total}");
    if let Some(items) = body.get("items").and_then(|v| v.as_array()) {
        for e in items {
            println!(
                "[{}] {} | {} | {}",
                e.get("id").unwrap_or(&Value::Null),
                null_or(e, "published_at", "(no date)"),
                s(e, "title"),
                null_or(e, "link", "")
            );
        }
    }
}

/// Percent-encode a query-parameter value (encode everything outside the
/// unreserved set so RFC 3339 `:` and `+` survive the round trip).
fn encode(s: &str) -> String {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_reserved() {
        assert_eq!(
            encode("2024-01-01T00:00:00+00:00"),
            "2024-01-01T00%3A00%3A00%2B00%3A00"
        );
        assert_eq!(encode("hello world"), "hello%20world");
    }
}
