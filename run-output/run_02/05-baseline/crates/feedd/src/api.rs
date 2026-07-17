//! HTTP REST API for `feedd`, served over `tiny_http`.

use serde_json::{json, Value};
use tiny_http::{Header, Method, Request, Response};

use crate::fetch;
use crate::store::{AddOutcome, EntryQuery, Store};

const DEFAULT_LIMIT: i64 = 50;
const MAX_LIMIT: i64 = 500;

/// Handle one HTTP request end to end.
pub fn handle(store: &Store, agent: &ureq::Agent, mut request: Request) {
    let method = request.method().clone();
    let url = request.url().to_string();
    let (path, query) = split_url(&url);
    let segments: Vec<&str> = path
        .trim_matches('/')
        .split('/')
        .filter(|s| !s.is_empty())
        .collect();

    let outcome = route(store, agent, &method, &segments, query, &mut request);
    match outcome {
        Outcome::Json(status, value) => respond_json(request, status, &value),
        Outcome::NoContent => {
            let resp = Response::from_data(Vec::new()).with_status_code(204);
            let _ = request.respond(resp);
        }
    }
}

enum Outcome {
    Json(u16, Value),
    NoContent,
}

fn err(status: u16, message: &str) -> Outcome {
    Outcome::Json(status, json!({ "error": message }))
}

fn route(
    store: &Store,
    agent: &ureq::Agent,
    method: &Method,
    segments: &[&str],
    query: &str,
    request: &mut Request,
) -> Outcome {
    match (method, segments) {
        (Method::Get, ["api", "health"]) => Outcome::Json(200, json!({ "status": "ok" })),

        (Method::Get, ["api", "feeds"]) => list_feeds(store),
        (Method::Post, ["api", "feeds"]) => add_feed(store, request),
        (Method::Get, ["api", "feeds", id]) => get_feed(store, id),
        (Method::Delete, ["api", "feeds", id]) => delete_feed(store, id),
        (Method::Post, ["api", "feeds", id, "refresh"]) => refresh_one(store, agent, id),

        (Method::Post, ["api", "refresh"]) => refresh_all(store, agent),
        (Method::Get, ["api", "entries"]) => entries(store, query),

        _ => err(404, "not found"),
    }
}

fn list_feeds(store: &Store) -> Outcome {
    match store.list_feeds() {
        Ok(feeds) => {
            let mut arr = Vec::new();
            for f in &feeds {
                match store.feed_json(f) {
                    Ok(v) => arr.push(v),
                    Err(e) => return err(500, &format!("database error: {e}")),
                }
            }
            Outcome::Json(200, Value::Array(arr))
        }
        Err(e) => err(500, &format!("database error: {e}")),
    }
}

fn add_feed(store: &Store, request: &mut Request) -> Outcome {
    let mut body = String::new();
    if request.as_reader().read_to_string(&mut body).is_err() {
        return err(422, "could not read request body");
    }
    let parsed: Result<Value, _> = serde_json::from_str(&body);
    let url = match parsed {
        Ok(v) => v.get("url").and_then(|u| u.as_str()).map(|s| s.to_string()),
        Err(_) => return err(422, "invalid JSON body"),
    };
    let url = match url {
        Some(u) => u,
        None => return err(422, "missing 'url' field"),
    };
    if !is_valid_http_url(&url) {
        return err(422, "url must be a valid http(s) URL");
    }
    match store.add_feed(&url) {
        Ok(AddOutcome::Created(feed)) => match store.feed_json(&feed) {
            Ok(v) => Outcome::Json(201, v),
            Err(e) => err(500, &format!("database error: {e}")),
        },
        Ok(AddOutcome::Duplicate) => err(409, "url already registered"),
        Err(e) => err(500, &format!("database error: {e}")),
    }
}

fn get_feed(store: &Store, id: &str) -> Outcome {
    let id = match parse_id(id) {
        Some(i) => i,
        None => return err(404, "feed not found"),
    };
    match store.get_feed(id) {
        Ok(Some(feed)) => match store.feed_json(&feed) {
            Ok(v) => Outcome::Json(200, v),
            Err(e) => err(500, &format!("database error: {e}")),
        },
        Ok(None) => err(404, "feed not found"),
        Err(e) => err(500, &format!("database error: {e}")),
    }
}

fn delete_feed(store: &Store, id: &str) -> Outcome {
    let id = match parse_id(id) {
        Some(i) => i,
        None => return err(404, "feed not found"),
    };
    match store.delete_feed(id) {
        Ok(true) => Outcome::NoContent,
        Ok(false) => err(404, "feed not found"),
        Err(e) => err(500, &format!("database error: {e}")),
    }
}

fn refresh_one(store: &Store, agent: &ureq::Agent, id: &str) -> Outcome {
    let id = match parse_id(id) {
        Some(i) => i,
        None => return err(404, "feed not found"),
    };
    match store.get_feed(id) {
        Ok(Some(_)) => {}
        Ok(None) => return err(404, "feed not found"),
        Err(e) => return err(500, &format!("database error: {e}")),
    }
    let result = fetch::refresh_feed(store, agent, id);
    Outcome::Json(200, result.to_json(Some(id)))
}

fn refresh_all(store: &Store, agent: &ureq::Agent) -> Outcome {
    let results = fetch::refresh_all(store, agent);
    let arr: Vec<Value> = results
        .into_iter()
        .map(|(id, r)| r.to_json(Some(id)))
        .collect();
    Outcome::Json(200, Value::Array(arr))
}

fn entries(store: &Store, query: &str) -> Outcome {
    let params = parse_query(query);

    let mut q = EntryQuery {
        limit: DEFAULT_LIMIT,
        ..Default::default()
    };

    if let Some(v) = params.iter().find(|(k, _)| k == "feed_id") {
        match v.1.parse::<i64>() {
            Ok(i) => q.feed_id = Some(i),
            Err(_) => return err(400, "feed_id must be an integer"),
        }
    }
    if let Some(v) = params.iter().find(|(k, _)| k == "since") {
        match feedlib::date::parse_rfc3339(&v.1) {
            Some(dt) => {
                q.since = Some(dt.timestamp());
                q.has_time_bound = true;
            }
            None => return err(400, "since must be an RFC 3339 timestamp"),
        }
    }
    if let Some(v) = params.iter().find(|(k, _)| k == "until") {
        match feedlib::date::parse_rfc3339(&v.1) {
            Some(dt) => {
                q.until = Some(dt.timestamp());
                q.has_time_bound = true;
            }
            None => return err(400, "until must be an RFC 3339 timestamp"),
        }
    }
    if let Some(v) = params.iter().find(|(k, _)| k == "q") {
        if !v.1.is_empty() {
            q.q = Some(v.1.clone());
        }
    }
    if let Some(v) = params.iter().find(|(k, _)| k == "limit") {
        match v.1.parse::<i64>() {
            Ok(i) if i >= 0 => q.limit = i.min(MAX_LIMIT),
            _ => return err(400, "limit must be a non-negative integer"),
        }
    }
    if let Some(v) = params.iter().find(|(k, _)| k == "offset") {
        match v.1.parse::<i64>() {
            Ok(i) if i >= 0 => q.offset = i,
            _ => return err(400, "offset must be a non-negative integer"),
        }
    }

    match store.query_entries(&q) {
        Ok((total, items)) => Outcome::Json(200, json!({ "total": total, "items": items })),
        Err(e) => err(500, &format!("database error: {e}")),
    }
}

fn is_valid_http_url(s: &str) -> bool {
    match url::Url::parse(s) {
        Ok(u) => matches!(u.scheme(), "http" | "https") && u.host().is_some(),
        Err(_) => false,
    }
}

fn parse_id(s: &str) -> Option<i64> {
    s.parse::<i64>().ok()
}

fn split_url(url: &str) -> (&str, &str) {
    match url.split_once('?') {
        Some((p, q)) => (p, q),
        None => (url, ""),
    }
}

/// Parse a query string into decoded key/value pairs.
fn parse_query(query: &str) -> Vec<(String, String)> {
    query
        .split('&')
        .filter(|s| !s.is_empty())
        .map(|pair| match pair.split_once('=') {
            Some((k, v)) => (percent_decode(k), percent_decode(v)),
            None => (percent_decode(pair), String::new()),
        })
        .collect()
}

/// Decode `application/x-www-form-urlencoded` text (`+` => space, `%XX`).
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

fn respond_json(request: Request, status: u16, value: &Value) {
    let body = value.to_string();
    let header =
        Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).expect("valid header");
    let resp = Response::from_string(body)
        .with_status_code(status)
        .with_header(header);
    let _ = request.respond(resp);
}
