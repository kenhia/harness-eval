//! `feedctl` — command-line client for the feedhub server.

use std::process::ExitCode;
use std::time::Duration;

use clap::{Parser, Subcommand};
use serde_json::{json, Value};

#[derive(Parser)]
#[command(name = "feedctl", about = "Command-line client for the feedhub server.")]
struct Cli {
    /// Base URL of the feedd server.
    #[arg(long, default_value = "http://127.0.0.1:8600")]
    server: String,
    /// Output format.
    #[arg(long, value_enum, default_value_t = Format::Text)]
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
    /// List all registered feeds.
    List,
    /// Show a single feed by id.
    Show { id: i64 },
    /// Remove a feed (and its entries) by id.
    Remove { id: i64 },
    /// Refresh one feed (with ID) or all feeds (without).
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

/// Exit codes: 0 ok, 1 server error, 2 unreachable/usage.
enum CtlError {
    /// Server responded with a 4xx/5xx error (message extracted from body).
    Server(String),
    /// Server unreachable / transport failure.
    Unreachable(String),
}

struct Reply {
    status: u16,
    body: Option<Value>,
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let agent = ureq::AgentBuilder::new()
        .timeout_connect(Duration::from_secs(10))
        .timeout_read(Duration::from_secs(60))
        .build();

    match run(&cli, &agent) {
        Ok(()) => ExitCode::SUCCESS,
        Err(CtlError::Server(msg)) => {
            eprintln!("error: {msg}");
            ExitCode::from(1)
        }
        Err(CtlError::Unreachable(msg)) => {
            eprintln!("error: {msg}");
            ExitCode::from(2)
        }
    }
}

fn run(cli: &Cli, agent: &ureq::Agent) -> Result<(), CtlError> {
    let base = cli.server.trim_end_matches('/');
    match &cli.command {
        Command::Add { url } => {
            let reply = send(
                agent,
                "POST",
                &format!("{base}/api/feeds"),
                Some(json!({ "url": url })),
                &[],
            )?;
            output_feed(cli.format, &reply, "added");
            Ok(())
        }
        Command::List => {
            let reply = send(agent, "GET", &format!("{base}/api/feeds"), None, &[])?;
            output_feed_list(cli.format, &reply);
            Ok(())
        }
        Command::Show { id } => {
            let reply = send(agent, "GET", &format!("{base}/api/feeds/{id}"), None, &[])?;
            output_feed(cli.format, &reply, "feed");
            Ok(())
        }
        Command::Remove { id } => {
            let reply = send(agent, "DELETE", &format!("{base}/api/feeds/{id}"), None, &[])?;
            if cli.format == Format::Json {
                println!("{}", reply.body.clone().unwrap_or(json!({ "status": "ok" })));
            } else {
                println!("Removed feed {id}");
            }
            Ok(())
        }
        Command::Refresh { id } => {
            let url = match id {
                Some(id) => format!("{base}/api/feeds/{id}/refresh"),
                None => format!("{base}/api/refresh"),
            };
            let reply = send(agent, "POST", &url, None, &[])?;
            output_refresh(cli.format, &reply);
            Ok(())
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
            let reply = send(agent, "GET", &format!("{base}/api/entries"), None, &params)?;
            output_entries(cli.format, &reply);
            Ok(())
        }
    }
}

fn send(
    agent: &ureq::Agent,
    method: &str,
    url: &str,
    body: Option<Value>,
    query: &[(String, String)],
) -> Result<Reply, CtlError> {
    let mut req = agent.request(method, url);
    for (k, v) in query {
        req = req.query(k, v);
    }

    let result = match body {
        Some(v) => req.send_string(&v.to_string()),
        None => req.call(),
    };

    match result {
        Ok(resp) => {
            let status = resp.status();
            let text = resp.into_string().unwrap_or_default();
            let value = serde_json::from_str::<Value>(&text).ok();
            Ok(Reply {
                status,
                body: value,
            })
        }
        Err(ureq::Error::Status(code, resp)) => {
            let text = resp.into_string().unwrap_or_default();
            let msg = serde_json::from_str::<Value>(&text)
                .ok()
                .and_then(|v| v.get("error").and_then(|e| e.as_str()).map(|s| s.to_string()))
                .unwrap_or_else(|| format!("server returned status {code}"));
            Err(CtlError::Server(msg))
        }
        Err(ureq::Error::Transport(t)) => {
            Err(CtlError::Unreachable(format!("could not reach server: {t}")))
        }
    }
}

fn print_json(reply: &Reply) {
    match &reply.body {
        Some(v) => println!("{v}"),
        None => println!("{}", json!({ "status": reply.status })),
    }
}

fn output_feed(format: Format, reply: &Reply, label: &str) {
    if format == Format::Json {
        print_json(reply);
        return;
    }
    if let Some(feed) = &reply.body {
        println!("{label}: {}", feed_line(feed));
    }
}

fn output_feed_list(format: Format, reply: &Reply) {
    if format == Format::Json {
        print_json(reply);
        return;
    }
    match reply.body.as_ref().and_then(|v| v.as_array()) {
        Some(feeds) if !feeds.is_empty() => {
            for feed in feeds {
                println!("{}", feed_line(feed));
            }
        }
        _ => println!("(no feeds)"),
    }
}

fn feed_line(feed: &Value) -> String {
    let id = feed.get("id").and_then(|v| v.as_i64()).unwrap_or(0);
    let url = feed.get("url").and_then(|v| v.as_str()).unwrap_or("");
    let title = feed
        .get("title")
        .and_then(|v| v.as_str())
        .unwrap_or("(untitled)");
    let count = feed.get("entry_count").and_then(|v| v.as_i64()).unwrap_or(0);
    let error = feed.get("last_error").and_then(|v| v.as_str());
    let mut line = format!("[{id}] {title} <{url}> ({count} entries)");
    if let Some(e) = error {
        line.push_str(&format!(" ERROR: {e}"));
    }
    line
}

fn output_refresh(format: Format, reply: &Reply) {
    if format == Format::Json {
        print_json(reply);
        return;
    }
    let Some(body) = &reply.body else { return };
    match body {
        Value::Array(results) => {
            for r in results {
                println!("{}", refresh_line(r));
            }
        }
        obj => println!("{}", refresh_line(obj)),
    }
}

fn refresh_line(r: &Value) -> String {
    let status = r.get("status").and_then(|v| v.as_str()).unwrap_or("?");
    let new = r.get("new_entries").and_then(|v| v.as_i64()).unwrap_or(0);
    let id = r.get("feed_id").and_then(|v| v.as_i64());
    let prefix = match id {
        Some(id) => format!("feed {id}"),
        None => "feed".to_string(),
    };
    if status == "ok" {
        let nm = r
            .get("not_modified")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        if nm {
            format!("{prefix}: ok (not modified, 0 new)")
        } else {
            format!("{prefix}: ok ({new} new)")
        }
    } else {
        let err = r.get("error").and_then(|v| v.as_str()).unwrap_or("unknown error");
        format!("{prefix}: error: {err}")
    }
}

fn output_entries(format: Format, reply: &Reply) {
    if format == Format::Json {
        print_json(reply);
        return;
    }
    let Some(body) = &reply.body else { return };
    let total = body.get("total").and_then(|v| v.as_i64()).unwrap_or(0);
    let empty = Vec::new();
    let items = body.get("items").and_then(|v| v.as_array()).unwrap_or(&empty);
    println!("{total} entries (showing {})", items.len());
    for e in items {
        let id = e.get("id").and_then(|v| v.as_i64()).unwrap_or(0);
        let title = e.get("title").and_then(|v| v.as_str()).unwrap_or("(untitled)");
        let published = e
            .get("published_at")
            .and_then(|v| v.as_str())
            .unwrap_or("(no date)");
        let link = e.get("link").and_then(|v| v.as_str()).unwrap_or("");
        println!("[{id}] {published}  {title}");
        if !link.is_empty() {
            println!("      {link}");
        }
    }
}
