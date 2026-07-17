//! feedd — the feedhub server. Fetches and stores RSS/Atom feeds and exposes a
//! JSON REST API. Fetching happens both on a background poll timer and on
//! demand via the refresh endpoints.

use clap::Parser;
use feedcore::store::EntryQuery;
use feedcore::{service, Store};
use serde_json::{json, Value};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tiny_http::{Header, Method, Request, Response, Server};

#[derive(Parser)]
#[command(
    name = "feedd",
    version,
    about = "feedhub server: fetch, store, and serve feeds"
)]
struct Cli {
    /// Path to the SQLite database file (created if missing).
    #[arg(long)]
    db: String,
    /// Address to listen on.
    #[arg(long, default_value = "127.0.0.1:8600")]
    listen: String,
    /// Background poll interval in seconds; 0 disables background polling.
    #[arg(long, default_value_t = 300)]
    poll_interval: u64,
}

fn main() {
    let cli = Cli::parse();
    std::process::exit(run(cli));
}

fn run(cli: Cli) -> i32 {
    let store = match Store::open(&cli.db) {
        Ok(s) => Arc::new(Mutex::new(s)),
        Err(e) => {
            eprintln!("feedd: cannot open database {}: {e}", cli.db);
            return 2;
        }
    };

    if cli.poll_interval > 0 {
        let db = cli.db.clone();
        let interval = cli.poll_interval;
        thread::spawn(move || background_poll(db, interval));
    }

    let server = match Server::http(&cli.listen) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("feedd: cannot listen on {}: {e}", cli.listen);
            return 2;
        }
    };
    println!("feedd listening on http://{}", cli.listen);

    for request in server.incoming_requests() {
        let store = Arc::clone(&store);
        handle(request, &store);
    }
    0
}

fn background_poll(db: String, interval: u64) {
    // A dedicated connection avoids contending with the request handler's lock.
    let store = match Store::open(&db) {
        Ok(s) => s,
        Err(_) => return,
    };
    loop {
        thread::sleep(Duration::from_secs(interval));
        let _ = service::refresh_all(&store);
    }
}

fn json_response(status: u16, body: &Value) -> Response<std::io::Cursor<Vec<u8>>> {
    let data = serde_json::to_vec(body).unwrap_or_else(|_| b"{}".to_vec());
    let mut resp = Response::from_data(data).with_status_code(status);
    resp.add_header(Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap());
    resp
}

fn error_response(status: u16, message: &str) -> Response<std::io::Cursor<Vec<u8>>> {
    json_response(status, &json!({ "error": message }))
}

fn handle(mut request: Request, store: &Mutex<Store>) {
    let method = request.method().clone();
    let url = request.url().to_string();
    let (path, query) = match url.split_once('?') {
        Some((p, q)) => (p.to_string(), q.to_string()),
        None => (url.clone(), String::new()),
    };
    let segments: Vec<&str> = path.trim_matches('/').split('/').collect();

    let response = route(&method, &segments, &query, &mut request, store);
    let _ = request.respond(response);
}

fn route(
    method: &Method,
    segments: &[&str],
    query: &str,
    request: &mut Request,
    store: &Mutex<Store>,
) -> Response<std::io::Cursor<Vec<u8>>> {
    // All routes are under /api.
    match (method, segments) {
        (Method::Get, ["api", "health"]) => json_response(200, &json!({ "status": "ok" })),
        (Method::Get, ["api", "feeds"]) => {
            let store = store.lock().unwrap();
            match store.list_feeds() {
                Ok(feeds) => json_response(200, &serde_json::to_value(feeds).unwrap()),
                Err(e) => error_response(500, &e.to_string()),
            }
        }
        (Method::Post, ["api", "feeds"]) => add_feed(request, store),
        (Method::Get, ["api", "feeds", id]) => match id.parse::<i64>() {
            Ok(id) => {
                let store = store.lock().unwrap();
                match store.get_feed(id) {
                    Ok(Some(f)) => json_response(200, &serde_json::to_value(f).unwrap()),
                    Ok(None) => error_response(404, "feed not found"),
                    Err(e) => error_response(500, &e.to_string()),
                }
            }
            Err(_) => error_response(404, "feed not found"),
        },
        (Method::Delete, ["api", "feeds", id]) => match id.parse::<i64>() {
            Ok(id) => {
                let store = store.lock().unwrap();
                match store.delete_feed(id) {
                    Ok(true) => Response::from_data(Vec::new()).with_status_code(204),
                    Ok(false) => error_response(404, "feed not found"),
                    Err(e) => error_response(500, &e.to_string()),
                }
            }
            Err(_) => error_response(404, "feed not found"),
        },
        (Method::Post, ["api", "feeds", id, "refresh"]) => match id.parse::<i64>() {
            Ok(id) => {
                let store = store.lock().unwrap();
                match service::refresh_feed(&store, id) {
                    Some(result) => json_response(200, &serde_json::to_value(result).unwrap()),
                    None => error_response(404, "feed not found"),
                }
            }
            Err(_) => error_response(404, "feed not found"),
        },
        (Method::Post, ["api", "refresh"]) => {
            let store = store.lock().unwrap();
            let results = service::refresh_all(&store);
            json_response(200, &serde_json::to_value(results).unwrap())
        }
        (Method::Get, ["api", "entries"]) => entries(query, store),
        _ => error_response(404, "not found"),
    }
}

fn add_feed(request: &mut Request, store: &Mutex<Store>) -> Response<std::io::Cursor<Vec<u8>>> {
    let mut body = String::new();
    if request.as_reader().read_to_string(&mut body).is_err() {
        return error_response(400, "could not read request body");
    }
    let parsed: Value = match serde_json::from_str(&body) {
        Ok(v) => v,
        Err(_) => return error_response(422, "body must be JSON with a 'url' field"),
    };
    let url = match parsed.get("url").and_then(|u| u.as_str()) {
        Some(u) => u,
        None => return error_response(422, "missing 'url' field"),
    };
    let store = store.lock().unwrap();
    match store.add_feed(url) {
        Ok(feed) => json_response(201, &serde_json::to_value(feed).unwrap()),
        Err(feedcore::FeedError::Duplicate) => error_response(409, "url already registered"),
        Err(feedcore::FeedError::InvalidUrl) => {
            error_response(422, "url must be a valid http(s) URL")
        }
        Err(e) => error_response(500, &e.to_string()),
    }
}

fn entries(query: &str, store: &Mutex<Store>) -> Response<std::io::Cursor<Vec<u8>>> {
    let params = parse_query(query);
    let mut q = EntryQuery::new();

    if let Some(v) = params.get("feed_id") {
        match v.parse::<i64>() {
            Ok(id) => q.feed_id = Some(id),
            Err(_) => return error_response(400, "feed_id must be an integer"),
        }
    }
    if let Some(v) = params.get("since") {
        q.since = Some(v.clone());
    }
    if let Some(v) = params.get("until") {
        q.until = Some(v.clone());
    }
    if let Some(v) = params.get("q") {
        q.q = Some(v.clone());
    }
    if let Some(v) = params.get("limit") {
        match v.parse::<i64>() {
            Ok(n) if n >= 0 => q.limit = n.min(500),
            _ => return error_response(400, "limit must be a non-negative integer"),
        }
    }
    if let Some(v) = params.get("offset") {
        match v.parse::<i64>() {
            Ok(n) if n >= 0 => q.offset = n,
            _ => return error_response(400, "offset must be a non-negative integer"),
        }
    }

    let store = store.lock().unwrap();
    match store.query_entries(&q) {
        Ok((total, items)) => json_response(200, &json!({ "total": total, "items": items })),
        Err(feedcore::FeedError::Parse(m)) => error_response(400, &m),
        Err(e) => error_response(500, &e.to_string()),
    }
}

/// Parse a URL query string into a key→value map, percent-decoding both.
fn parse_query(query: &str) -> std::collections::HashMap<String, String> {
    let mut map = std::collections::HashMap::new();
    for pair in query.split('&').filter(|s| !s.is_empty()) {
        let (k, v) = match pair.split_once('=') {
            Some((k, v)) => (k, v),
            None => (pair, ""),
        };
        map.insert(percent_decode(k), percent_decode(v));
    }
    map
}

/// Decode `application/x-www-form-urlencoded` text (`+` → space, `%XX` bytes).
fn percent_decode(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'+' => {
                out.push(b' ');
                i += 1;
            }
            b'%' if i + 2 < bytes.len() => {
                let hi = (bytes[i + 1] as char).to_digit(16);
                let lo = (bytes[i + 2] as char).to_digit(16);
                match (hi, lo) {
                    (Some(h), Some(l)) => {
                        out.push((h * 16 + l) as u8);
                        i += 3;
                    }
                    _ => {
                        out.push(bytes[i]);
                        i += 1;
                    }
                }
            }
            b => {
                out.push(b);
                i += 1;
            }
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn query_parsing_and_decoding() {
        let m = parse_query("feed_id=3&q=hello%20world&since=2024-01-01T00%3A00%3A00%2B00%3A00");
        assert_eq!(m.get("feed_id").unwrap(), "3");
        assert_eq!(m.get("q").unwrap(), "hello world");
        assert_eq!(m.get("since").unwrap(), "2024-01-01T00:00:00+00:00");
    }

    #[test]
    fn percent_decode_plus_is_space() {
        assert_eq!(percent_decode("a+b"), "a b");
        assert_eq!(percent_decode("100%25"), "100%");
    }
}
