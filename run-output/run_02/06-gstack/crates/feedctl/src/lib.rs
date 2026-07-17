//! Command line client for the feedhub API.
//!
//! # Exit codes
//!
//! | code | meaning |
//! |---|---|
//! | 0 | success |
//! | 1 | the server answered with an error (its message goes to stderr) |
//! | 2 | the server is unreachable, or the usage is invalid |
//!
//! Code 2 covers both halves of "we never got an answer": clap emits it for
//! usage errors on its own, and [`CtlError::Unreachable`] carries it for
//! transport failures.

#![forbid(unsafe_code)]

use clap::{Parser, Subcommand};
use serde_json::Value;

pub const DEFAULT_SERVER: &str = "http://127.0.0.1:8600";

#[derive(Debug, Parser)]
#[command(
    name = "feedctl",
    version,
    about = "Command line client for the feedhub API.",
    long_about = "feedctl drives a running feedd over its REST API.\n\n\
                  Exit codes: 0 success, 1 the server answered with an error \
                  (message on stderr), 2 the server is unreachable or the usage \
                  is invalid."
)]
pub struct Cli {
    /// Base URL of the feedd server.
    #[arg(long, global = true, value_name = "URL", default_value = DEFAULT_SERVER)]
    pub server: String,

    /// Output format. `json` prints the API response as one JSON document.
    #[arg(long, global = true, value_enum, default_value_t = Format::Text)]
    pub format: Format,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum Format {
    /// Human-readable.
    Text,
    /// The API response, as a single JSON document on stdout.
    Json,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Register a feed.
    Add {
        /// Feed URL. Must be http or https.
        url: String,
    },
    /// List registered feeds.
    List,
    /// Show one feed.
    Show { id: i64 },
    /// Unregister a feed and delete its entries.
    Remove { id: i64 },
    /// Fetch now. With no ID, refreshes every feed.
    Refresh { id: Option<i64> },
    /// Query stored entries.
    Entries {
        /// Restrict to one feed.
        #[arg(long, value_name = "ID")]
        feed: Option<i64>,
        /// Inclusive lower bound, RFC 3339.
        #[arg(long, value_name = "T")]
        since: Option<String>,
        /// Exclusive upper bound, RFC 3339.
        #[arg(long, value_name = "T")]
        until: Option<String>,
        /// Case-insensitive substring of the title.
        #[arg(long, value_name = "Q")]
        search: Option<String>,
        /// Maximum entries to return (max 500).
        #[arg(long, value_name = "N")]
        limit: Option<i64>,
        /// Entries to skip.
        #[arg(long, value_name = "N")]
        offset: Option<i64>,
    },
}

#[derive(Debug)]
pub enum CtlError {
    /// The server answered, and the answer was an error. Exit 1.
    Server(String),
    /// We never got an answer. Exit 2.
    Unreachable(String),
}

impl CtlError {
    pub fn exit_code(&self) -> u8 {
        match self {
            CtlError::Server(_) => 1,
            CtlError::Unreachable(_) => 2,
        }
    }

    pub fn message(&self) -> &str {
        match self {
            CtlError::Server(m) | CtlError::Unreachable(m) => m,
        }
    }
}

type CtlResult<T> = Result<T, CtlError>;

/// Percent-encode a query parameter value.
fn encode(s: &str) -> String {
    s.bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                (b as char).to_string()
            }
            _ => format!("%{b:02X}"),
        })
        .collect()
}

/// Pull the server's `{"error": "..."}` message out of a failed response.
fn server_error(status: reqwest::StatusCode, body: &str) -> CtlError {
    let message = serde_json::from_str::<Value>(body)
        .ok()
        .and_then(|v| v["error"].as_str().map(str::to_string))
        .unwrap_or_else(|| {
            // The server broke its own contract; say what we actually saw
            // rather than inventing a tidy message.
            let body = body.trim();
            if body.is_empty() {
                format!("server returned {status}")
            } else {
                format!("server returned {status}: {body}")
            }
        });
    CtlError::Server(message)
}

/// A response body, already known to be a success.
async fn send(request: reqwest::RequestBuilder) -> CtlResult<(reqwest::StatusCode, String)> {
    let response = request
        .send()
        .await
        .map_err(|e| CtlError::Unreachable(transport_error(&e)))?;
    let status = response.status();
    let body = response
        .text()
        .await
        .map_err(|e| CtlError::Unreachable(transport_error(&e)))?;

    if status.is_success() {
        Ok((status, body))
    } else {
        Err(server_error(status, &body))
    }
}

/// Say plainly that the server isn't there, rather than echoing reqwest's
/// "error sending request for url (...)" chain.
fn transport_error(e: &reqwest::Error) -> String {
    let base = if e.is_connect() {
        "cannot reach the server"
    } else if e.is_timeout() {
        "the server timed out"
    } else if e.is_builder() {
        "invalid server URL"
    } else {
        "request failed"
    };
    let mut source: &dyn std::error::Error = e;
    while let Some(next) = source.source() {
        source = next;
    }
    format!("{base}: {source}")
}

fn json(body: &str) -> CtlResult<Value> {
    serde_json::from_str(body)
        .map_err(|e| CtlError::Server(format!("server sent invalid JSON: {e}")))
}

/// Run a command, writing output to `out`.
pub async fn run(cli: Cli, out: &mut impl std::io::Write) -> CtlResult<()> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| CtlError::Unreachable(format!("cannot build HTTP client: {e}")))?;

    let base = cli.server.trim_end_matches('/').to_string();
    let url = |path: &str| format!("{base}{path}");

    // Each arm yields the JSON document to print, plus a text rendering.
    let (value, text) = match &cli.command {
        Command::Add { url: feed_url } => {
            let (_, body) = send(
                client
                    .post(url("/api/feeds"))
                    .json(&serde_json::json!({ "url": feed_url })),
            )
            .await?;
            let v = json(&body)?;
            let text = format!(
                "Added feed {}: {}",
                v["id"],
                v["url"].as_str().unwrap_or("")
            );
            (v, text)
        }

        Command::List => {
            let (_, body) = send(client.get(url("/api/feeds"))).await?;
            let v = json(&body)?;
            (v.clone(), render_feed_list(&v))
        }

        Command::Show { id } => {
            let (_, body) = send(client.get(url(&format!("/api/feeds/{id}")))).await?;
            let v = json(&body)?;
            (v.clone(), render_feed_detail(&v))
        }

        Command::Remove { id } => {
            // 204 has no body, so there is no raw document to pass through.
            // Synthesize one so `--format json` always emits valid JSON.
            send(client.delete(url(&format!("/api/feeds/{id}")))).await?;
            let v = serde_json::json!({ "status": "ok", "id": id });
            (v, format!("Removed feed {id}"))
        }

        Command::Refresh { id: Some(id) } => {
            let (_, body) = send(client.post(url(&format!("/api/feeds/{id}/refresh")))).await?;
            let v = json(&body)?;
            (v.clone(), render_refresh(&v))
        }

        Command::Refresh { id: None } => {
            let (_, body) = send(client.post(url("/api/refresh"))).await?;
            let v = json(&body)?;
            let text = match v.as_array() {
                Some(rs) if rs.is_empty() => "No feeds registered.".to_string(),
                Some(rs) => rs.iter().map(render_refresh).collect::<Vec<_>>().join("\n"),
                None => render_refresh(&v),
            };
            (v, text)
        }

        Command::Entries {
            feed,
            since,
            until,
            search,
            limit,
            offset,
        } => {
            let mut params: Vec<String> = Vec::new();
            if let Some(v) = feed {
                params.push(format!("feed_id={v}"));
            }
            if let Some(v) = since {
                params.push(format!("since={}", encode(v)));
            }
            if let Some(v) = until {
                params.push(format!("until={}", encode(v)));
            }
            // --search is the CLI spelling of the API's `q`.
            if let Some(v) = search {
                params.push(format!("q={}", encode(v)));
            }
            if let Some(v) = limit {
                params.push(format!("limit={v}"));
            }
            if let Some(v) = offset {
                params.push(format!("offset={v}"));
            }
            let query = if params.is_empty() {
                String::new()
            } else {
                format!("?{}", params.join("&"))
            };

            let (_, body) = send(client.get(url(&format!("/api/entries{query}")))).await?;
            let v = json(&body)?;
            (v.clone(), render_entries(&v))
        }
    };

    let rendered = match cli.format {
        Format::Json => serde_json::to_string(&value)
            .map_err(|e| CtlError::Server(format!("cannot serialize response: {e}")))?,
        Format::Text => text,
    };
    writeln!(out, "{rendered}").map_err(|e| CtlError::Unreachable(format!("cannot write: {e}")))
}

// ------------------------------------------------------------- rendering

fn or_dash(v: &Value) -> String {
    v.as_str().map(str::to_string).unwrap_or_else(|| "-".into())
}

fn render_feed_list(v: &Value) -> String {
    let Some(feeds) = v.as_array() else {
        return "-".into();
    };
    if feeds.is_empty() {
        return "No feeds registered.".into();
    }
    let mut out = format!("{:>4}  {:>7}  {:<30}  {}", "ID", "ENTRIES", "TITLE", "URL");
    for f in feeds {
        let title = f["title"].as_str().unwrap_or("(not yet fetched)");
        let flag = if f["last_error"].is_null() {
            ""
        } else {
            "  [error]"
        };
        // `.to_string()` first: a width spec like `{:>4}` is silently ignored
        // when applied straight to a serde_json::Value, because Value's Display
        // impl never consults the formatter's width or fill.
        out.push_str(&format!(
            "\n{:>4}  {:>7}  {:<30}  {}{}",
            f["id"].to_string(),
            f["entry_count"].to_string(),
            truncate(title, 30),
            or_dash(&f["url"]),
            flag
        ));
    }
    out
}

fn render_feed_detail(f: &Value) -> String {
    let mut out = String::new();
    out.push_str(&format!("id:              {}\n", f["id"]));
    out.push_str(&format!("url:             {}\n", or_dash(&f["url"])));
    out.push_str(&format!(
        "title:           {}\n",
        f["title"].as_str().unwrap_or("(not yet fetched)")
    ));
    out.push_str(&format!("entries:         {}\n", f["entry_count"]));
    out.push_str(&format!(
        "last fetched at: {}\n",
        or_dash(&f["last_fetched_at"])
    ));
    out.push_str(&format!("last error:      {}", or_dash(&f["last_error"])));
    out
}

fn render_refresh(r: &Value) -> String {
    let id = &r["feed_id"];
    if r["status"] == "error" {
        return format!("feed {id}: error: {}", or_dash(&r["error"]));
    }
    let new = r["new_entries"].as_u64().unwrap_or(0);
    let suffix = if r["not_modified"] == Value::Bool(true) {
        " (not modified)"
    } else {
        ""
    };
    format!(
        "feed {id}: ok, {new} new {}{suffix}",
        if new == 1 { "entry" } else { "entries" }
    )
}

fn render_entries(page: &Value) -> String {
    let total = page["total"].as_i64().unwrap_or(0);
    let Some(items) = page["items"].as_array() else {
        return "-".into();
    };
    if items.is_empty() {
        return format!("No entries. ({total} total)");
    }

    let mut out = String::new();
    for e in items {
        let published = e["published_at"].as_str().unwrap_or("(undated)");
        out.push_str(&format!(
            "{:<24}  [feed {}]  {}\n",
            published,
            e["feed_id"],
            or_dash(&e["title"])
        ));
        if let Some(link) = e["link"].as_str() {
            out.push_str(&format!("{:<24}  {}\n", "", link));
        }
    }
    out.push_str(&format!(
        "\nShowing {} of {} matching {}.",
        items.len(),
        total,
        if total == 1 { "entry" } else { "entries" }
    ));
    out
}

fn truncate(s: &str, width: usize) -> String {
    // char_indices, not byte slicing: a non-ASCII title must not panic here.
    match s.char_indices().nth(width) {
        None => s.to_string(),
        Some((idx, _)) => format!("{}…", &s[..idx]),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;
    use serde_json::json;

    #[test]
    fn cli_definition_is_valid() {
        Cli::command().debug_assert();
    }

    #[test]
    fn defaults_match_the_spec() {
        let cli = Cli::parse_from(["feedctl", "list"]);
        assert_eq!(cli.server, "http://127.0.0.1:8600");
        assert_eq!(cli.format, Format::Text);
    }

    #[test]
    fn global_flags_work_before_and_after_the_subcommand() {
        for args in [
            vec!["feedctl", "--format", "json", "list"],
            vec!["feedctl", "list", "--format", "json"],
        ] {
            let cli = Cli::parse_from(&args);
            assert_eq!(cli.format, Format::Json, "{args:?}");
        }
    }

    #[test]
    fn every_spec_command_parses() {
        assert!(matches!(
            Cli::parse_from(["feedctl", "add", "https://e.com/f.rss"]).command,
            Command::Add { .. }
        ));
        assert!(matches!(
            Cli::parse_from(["feedctl", "list"]).command,
            Command::List
        ));
        assert!(matches!(
            Cli::parse_from(["feedctl", "show", "1"]).command,
            Command::Show { id: 1 }
        ));
        assert!(matches!(
            Cli::parse_from(["feedctl", "remove", "1"]).command,
            Command::Remove { id: 1 }
        ));
        assert!(matches!(
            Cli::parse_from(["feedctl", "refresh"]).command,
            Command::Refresh { id: None }
        ));
        assert!(matches!(
            Cli::parse_from(["feedctl", "refresh", "2"]).command,
            Command::Refresh { id: Some(2) }
        ));
    }

    #[test]
    fn entries_accepts_every_documented_flag() {
        let cli = Cli::parse_from([
            "feedctl",
            "entries",
            "--feed",
            "1",
            "--since",
            "2020-01-01T00:00:00Z",
            "--until",
            "2021-01-01T00:00:00Z",
            "--search",
            "rust",
            "--limit",
            "10",
            "--offset",
            "5",
        ]);
        match cli.command {
            Command::Entries {
                feed,
                since,
                until,
                search,
                limit,
                offset,
            } => {
                assert_eq!(feed, Some(1));
                assert_eq!(since.as_deref(), Some("2020-01-01T00:00:00Z"));
                assert_eq!(until.as_deref(), Some("2021-01-01T00:00:00Z"));
                assert_eq!(search.as_deref(), Some("rust"));
                assert_eq!(limit, Some(10));
                assert_eq!(offset, Some(5));
            }
            other => panic!("expected entries, got {other:?}"),
        }
    }

    #[test]
    fn unknown_usage_is_rejected() {
        assert!(
            Cli::try_parse_from(["feedctl"]).is_err(),
            "a command is required"
        );
        assert!(Cli::try_parse_from(["feedctl", "frobnicate"]).is_err());
        assert!(Cli::try_parse_from(["feedctl", "--format", "yaml", "list"]).is_err());
        assert!(Cli::try_parse_from(["feedctl", "show", "abc"]).is_err());
    }

    #[test]
    fn exit_codes_follow_the_spec() {
        assert_eq!(CtlError::Server("x".into()).exit_code(), 1);
        assert_eq!(CtlError::Unreachable("x".into()).exit_code(), 2);
    }

    #[test]
    fn server_errors_surface_the_servers_own_message() {
        let e = server_error(
            reqwest::StatusCode::CONFLICT,
            r#"{"error":"a feed with that URL is already registered"}"#,
        );
        assert_eq!(e.message(), "a feed with that URL is already registered");
        assert_eq!(e.exit_code(), 1);
    }

    #[test]
    fn a_non_conforming_error_body_still_reports_something_useful() {
        let e = server_error(reqwest::StatusCode::BAD_GATEWAY, "<html>nginx</html>");
        assert!(e.message().contains("502"), "got {:?}", e.message());
        assert_eq!(e.exit_code(), 1);
    }

    #[test]
    fn query_values_are_percent_encoded() {
        assert_eq!(encode("hello world"), "hello%20world");
        assert_eq!(encode("a&b=c"), "a%26b%3Dc");
        assert_eq!(
            encode("2020-01-01T00:00:00+05:30"),
            "2020-01-01T00%3A00%3A00%2B05%3A30"
        );
        assert_eq!(encode("plain-Text_1.0~"), "plain-Text_1.0~");
    }

    #[test]
    fn truncate_does_not_split_a_multibyte_character() {
        assert_eq!(truncate("abc", 10), "abc");
        assert_eq!(truncate("abcdef", 3), "abc…");
        // Byte slicing at index 3 would panic mid-character here.
        assert_eq!(truncate("日本語のタイトル", 3), "日本語…");
    }

    #[test]
    fn refresh_rendering_covers_ok_not_modified_and_error() {
        assert_eq!(
            render_refresh(
                &json!({"feed_id":1,"status":"ok","new_entries":3,"not_modified":false})
            ),
            "feed 1: ok, 3 new entries"
        );
        assert_eq!(
            render_refresh(
                &json!({"feed_id":1,"status":"ok","new_entries":1,"not_modified":false})
            ),
            "feed 1: ok, 1 new entry"
        );
        assert_eq!(
            render_refresh(&json!({"feed_id":2,"status":"ok","new_entries":0,"not_modified":true})),
            "feed 2: ok, 0 new entries (not modified)"
        );
        assert_eq!(
            render_refresh(&json!({"feed_id":3,"status":"error","error":"HTTP 404"})),
            "feed 3: error: HTTP 404"
        );
    }

    #[test]
    fn feed_list_marks_an_unfetched_title_and_an_errored_feed() {
        let rendered = render_feed_list(&json!([
            {"id":1,"url":"https://e.com/a","title":null,"entry_count":0,"last_error":null},
            {"id":2,"url":"https://e.com/b","title":"B","entry_count":5,"last_error":"boom"},
        ]));
        assert!(rendered.contains("(not yet fetched)"));
        assert!(rendered.contains("[error]"));
    }

    #[test]
    fn feed_list_columns_line_up_with_the_header() {
        // A `contains` assertion cannot see misalignment. `{:>4}` applied to a
        // serde_json::Value is a no-op — Value's Display ignores width — so the
        // id and entry-count columns silently lost their padding. Compare
        // column offsets against the header instead.
        let rendered = render_feed_list(&json!([
            {"id":1,"url":"https://e.com/a","title":"A","entry_count":3,"last_error":null},
            {"id":1234,"url":"https://e.com/b","title":"B","entry_count":9999,"last_error":null},
        ]));
        let lines: Vec<&str> = rendered.lines().collect();
        let header_title = lines[0].find("TITLE").expect("a TITLE column");

        for row in &lines[1..] {
            assert_eq!(
                row.find('A').or_else(|| row.find('B')),
                Some(header_title),
                "row {row:?} should start its title at column {header_title}"
            );
        }
        assert!(
            lines[1].starts_with("   1  "),
            "id is right-aligned: {:?}",
            lines[1]
        );
        assert!(
            lines[2].starts_with("1234  "),
            "id is right-aligned: {:?}",
            lines[2]
        );
    }

    #[test]
    fn empty_list_says_so() {
        assert_eq!(render_feed_list(&json!([])), "No feeds registered.");
    }
}
