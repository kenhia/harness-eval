//! `feedctl` — command-line client for the feedd REST API.
//!
//! Exit codes: 0 success; 1 the server answered with an error (message on
//! stderr); 2 the server is unreachable or the usage is invalid (clap emits 2
//! for usage errors automatically).

use clap::{Parser, Subcommand};
use feedcore::api::{EntriesResponse, ErrorBody, FeedDto, RefreshResult};

#[derive(Parser)]
#[command(name = "feedctl", about = "Command-line client for the feedd API.")]
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
    /// List all feeds.
    List,
    /// Show a single feed by id.
    Show { id: i64 },
    /// Remove a feed (and its entries) by id.
    Remove { id: i64 },
    /// Refresh one feed by id, or all feeds when id is omitted.
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

/// Client-side error mapped to a process exit code.
enum ClientError {
    /// Server returned a 4xx/5xx with an error message (exit 1).
    Api(String),
    /// Transport failure — server unreachable (exit 2).
    Unreachable(String),
}

fn main() {
    let cli = Cli::parse();
    match run(&cli) {
        Ok(()) => {}
        Err(ClientError::Api(msg)) => {
            eprintln!("error: {msg}");
            std::process::exit(1);
        }
        Err(ClientError::Unreachable(msg)) => {
            eprintln!("error: {msg}");
            std::process::exit(2);
        }
    }
}

fn run(cli: &Cli) -> Result<(), ClientError> {
    let base = cli.server.trim_end_matches('/');
    match &cli.command {
        Command::Add { url } => {
            let body = serde_json::json!({ "url": url }).to_string();
            let resp = send(cli, "POST", format!("{base}/api/feeds"), Some(body))?;
            output_feed(cli.format, &resp);
            Ok(())
        }
        Command::List => {
            let resp = send(cli, "GET", format!("{base}/api/feeds"), None)?;
            if cli.format == Format::Json {
                println!("{resp}");
            } else {
                match serde_json::from_str::<Vec<FeedDto>>(&resp) {
                    Ok(feeds) => {
                        if feeds.is_empty() {
                            println!("(no feeds)");
                        }
                        for f in feeds {
                            print_feed_line(&f);
                        }
                    }
                    Err(e) => println!("(unparseable response: {e})"),
                }
            }
            Ok(())
        }
        Command::Show { id } => {
            let resp = send(cli, "GET", format!("{base}/api/feeds/{id}"), None)?;
            output_feed(cli.format, &resp);
            Ok(())
        }
        Command::Remove { id } => {
            send(cli, "DELETE", format!("{base}/api/feeds/{id}"), None)?;
            if cli.format == Format::Json {
                println!("{}", serde_json::json!({ "deleted": true, "id": id }));
            } else {
                println!("removed feed {id}");
            }
            Ok(())
        }
        Command::Refresh { id } => {
            let (url, all) = match id {
                Some(i) => (format!("{base}/api/feeds/{i}/refresh"), false),
                None => (format!("{base}/api/refresh"), true),
            };
            let resp = send(cli, "POST", url, None)?;
            if cli.format == Format::Json {
                println!("{resp}");
            } else if all {
                match serde_json::from_str::<Vec<RefreshResult>>(&resp) {
                    Ok(results) => {
                        for r in results {
                            print_refresh(&r);
                        }
                    }
                    Err(e) => println!("(unparseable response: {e})"),
                }
            } else {
                match serde_json::from_str::<RefreshResult>(&resp) {
                    Ok(r) => print_refresh(&r),
                    Err(e) => println!("(unparseable response: {e})"),
                }
            }
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
            let query = encode_query(&params);
            let url = if query.is_empty() {
                format!("{base}/api/entries")
            } else {
                format!("{base}/api/entries?{query}")
            };
            let resp = send(cli, "GET", url, None)?;
            if cli.format == Format::Json {
                println!("{resp}");
            } else {
                match serde_json::from_str::<EntriesResponse>(&resp) {
                    Ok(r) => print_entries(&r),
                    Err(e) => println!("(unparseable response: {e})"),
                }
            }
            Ok(())
        }
    }
}

fn send(cli: &Cli, method: &str, url: String, body: Option<String>) -> Result<String, ClientError> {
    let agent = ureq::AgentBuilder::new()
        .timeout(std::time::Duration::from_secs(30))
        .build();
    let req = agent.request(method, &url);
    let result = match body {
        Some(b) => req.set("Content-Type", "application/json").send_string(&b),
        None => req.call(),
    };
    match result {
        Ok(resp) => Ok(resp.into_string().unwrap_or_default()),
        Err(ureq::Error::Status(_code, resp)) => {
            let body = resp.into_string().unwrap_or_default();
            Err(ClientError::Api(extract_error(&body)))
        }
        Err(ureq::Error::Transport(t)) => Err(ClientError::Unreachable(format!(
            "cannot reach server at {}: {t}",
            cli.server
        ))),
    }
}

/// Pull the `error` field out of an error body, falling back to the raw text.
fn extract_error(body: &str) -> String {
    match serde_json::from_str::<ErrorBody>(body) {
        Ok(e) => e.error,
        Err(_) if !body.is_empty() => body.to_string(),
        Err(_) => "server error".to_string(),
    }
}

fn output_feed(format: Format, resp: &str) {
    if format == Format::Json {
        println!("{resp}");
        return;
    }
    match serde_json::from_str::<FeedDto>(resp) {
        Ok(f) => print_feed_detail(&f),
        Err(e) => println!("(unparseable response: {e})"),
    }
}

fn print_feed_line(f: &FeedDto) {
    let title = f.title.as_deref().unwrap_or("(no title yet)");
    let status = match &f.last_error {
        Some(e) => format!("ERROR: {e}"),
        None => "ok".to_string(),
    };
    println!(
        "#{:<3} {:<40} entries={:<4} {}",
        f.id, title, f.entry_count, status
    );
    println!("     {}", f.url);
}

fn print_feed_detail(f: &FeedDto) {
    println!("id:              {}", f.id);
    println!("url:             {}", f.url);
    println!(
        "title:           {}",
        f.title.as_deref().unwrap_or("(none)")
    );
    println!(
        "last_fetched_at: {}",
        f.last_fetched_at.as_deref().unwrap_or("(never)")
    );
    println!(
        "last_error:      {}",
        f.last_error.as_deref().unwrap_or("(none)")
    );
    println!("entry_count:     {}", f.entry_count);
}

fn print_refresh(r: &RefreshResult) {
    let id = r
        .feed_id
        .map(|i| i.to_string())
        .unwrap_or_else(|| "-".to_string());
    match &r.error {
        Some(e) => println!("feed {id}: {} ({e})", r.status),
        None => println!("feed {id}: {} (+{} new)", r.status, r.new_entries),
    }
}

fn print_entries(r: &EntriesResponse) {
    println!("total: {} (showing {})", r.total, r.items.len());
    for e in &r.items {
        let when = e.published_at.as_deref().unwrap_or("(no date)");
        println!("- [{}] {}", when, e.title);
        if let Some(link) = &e.link {
            println!("    {link}");
        }
        println!("    feed={} id={} guid={}", e.feed_id, e.id, e.guid);
    }
}

/// Percent-encode query parameters (RFC 3986 unreserved set kept literal).
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encodes_plus_and_colon() {
        let q = encode_query(&[("since".into(), "2021-09-06T16:20:00+02:00".into())]);
        assert_eq!(q, "since=2021-09-06T16%3A20%3A00%2B02%3A00");
    }

    #[test]
    fn extract_error_reads_field() {
        assert_eq!(extract_error(r#"{"error":"nope"}"#), "nope");
        assert_eq!(extract_error("raw text"), "raw text");
    }
}
