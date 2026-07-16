//! REST API server built on `tiny_http`.

use crate::db::{EntryQuery, Store};
use crate::refresh::{refresh_all, refresh_feed};
use serde_json::{json, Value};
use std::sync::{Arc, Mutex};
use tiny_http::{Header, Method, Request, Response, Server};

const DEFAULT_LIMIT: i64 = 50;
const MAX_LIMIT: i64 = 500;

type Handled = Result<(u16, Value), (u16, String)>;

/// Run the blocking request loop until the process is terminated.
pub fn serve(store: Arc<Mutex<Store>>, addr: &str) -> Result<(), String> {
    let server = Server::http(addr).map_err(|e| e.to_string())?;
    for request in server.incoming_requests() {
        handle(&store, request);
    }
    Ok(())
}

fn handle(store: &Arc<Mutex<Store>>, mut request: Request) {
    let method = request.method().clone();
    let raw_url = request.url().to_string();
    let (path, query) = split_url(&raw_url);

    let mut body = String::new();
    let _ = request.as_reader().read_to_string(&mut body);

    let result = route(store, &method, &path, &query, &body);
    let (status, value) = match result {
        Ok((code, v)) => (code, v),
        Err((code, msg)) => (code, json!({ "error": msg })),
    };

    let data = serde_json::to_string(&value).unwrap_or_else(|_| "{}".to_string());
    let header = Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap();
    let response = if status == 204 {
        Response::from_string(String::new()).with_status_code(204)
    } else {
        Response::from_string(data)
            .with_status_code(status)
            .with_header(header)
    };
    let _ = request.respond(response);
}

fn route(
    store: &Arc<Mutex<Store>>,
    method: &Method,
    path: &str,
    query: &[(String, String)],
    body: &str,
) -> Handled {
    let segs: Vec<&str> = path
        .trim_matches('/')
        .split('/')
        .filter(|s| !s.is_empty())
        .collect();

    match (method, segs.as_slice()) {
        (Method::Get, ["api", "health"]) => Ok((200, json!({ "status": "ok" }))),

        (Method::Post, ["api", "feeds"]) => add_feed(store, body),
        (Method::Get, ["api", "feeds"]) => {
            let s = store.lock().unwrap();
            let feeds = s.all_feeds_json().map_err(internal)?;
            Ok((200, Value::Array(feeds)))
        }
        (Method::Get, ["api", "feeds", id]) => {
            let id = parse_id(id)?;
            let s = store.lock().unwrap();
            match s.feed_json(id).map_err(internal)? {
                Some(f) => Ok((200, f)),
                None => Err((404, "feed not found".into())),
            }
        }
        (Method::Delete, ["api", "feeds", id]) => {
            let id = parse_id(id)?;
            let s = store.lock().unwrap();
            if s.delete_feed(id).map_err(internal)? {
                Ok((204, Value::Null))
            } else {
                Err((404, "feed not found".into()))
            }
        }
        (Method::Post, ["api", "feeds", id, "refresh"]) => {
            let id = parse_id(id)?;
            let mut s = store.lock().unwrap();
            if !s.feed_exists(id).map_err(internal)? {
                return Err((404, "feed not found".into()));
            }
            Ok((200, refresh_feed(&mut s, id)))
        }
        (Method::Post, ["api", "refresh"]) => {
            let mut s = store.lock().unwrap();
            Ok((200, Value::Array(refresh_all(&mut s))))
        }
        (Method::Get, ["api", "entries"]) => entries(store, query),

        _ => Err((404, "not found".into())),
    }
}

fn add_feed(store: &Arc<Mutex<Store>>, body: &str) -> Handled {
    let parsed: Value =
        serde_json::from_str(body).map_err(|_| (422, "invalid JSON body".to_string()))?;
    let url = parsed
        .get("url")
        .and_then(Value::as_str)
        .ok_or((422, "missing 'url'".to_string()))?
        .trim()
        .to_string();
    if !(url.starts_with("http://") || url.starts_with("https://")) {
        return Err((422, "url must be http(s)".into()));
    }
    let s = store.lock().unwrap();
    if s.feed_id_for_url(&url).map_err(internal)?.is_some() {
        return Err((409, "url already registered".into()));
    }
    let id = s.add_feed(&url).map_err(internal)?;
    let feed = s.feed_json(id).map_err(internal)?.unwrap_or(Value::Null);
    Ok((201, feed))
}

fn entries(store: &Arc<Mutex<Store>>, query: &[(String, String)]) -> Handled {
    let mut q = EntryQuery {
        limit: DEFAULT_LIMIT,
        ..Default::default()
    };
    for (k, v) in query {
        match k.as_str() {
            "feed_id" => {
                q.feed_id = Some(
                    v.parse()
                        .map_err(|_| (400, "invalid feed_id".to_string()))?,
                )
            }
            "since" => q.since = Some(normalize_instant(v)?),
            "until" => q.until = Some(normalize_instant(v)?),
            "q" => q.q = Some(v.clone()),
            "limit" => {
                let n: i64 = v.parse().map_err(|_| (400, "invalid limit".to_string()))?;
                q.limit = n.clamp(0, MAX_LIMIT);
            }
            "offset" => {
                let n: i64 = v.parse().map_err(|_| (400, "invalid offset".to_string()))?;
                q.offset = n.max(0);
            }
            _ => {}
        }
    }
    let s = store.lock().unwrap();
    let result = s.query_entries(&q).map_err(internal)?;
    Ok((200, result))
}

/// Normalize an RFC 3339 bound to the fixed-width UTC storage form so string
/// comparison in SQL matches instant comparison.
fn normalize_instant(v: &str) -> Result<String, (u16, String)> {
    feedcore::date::parse_rfc3339(v)
        .map(|dt| crate::db::fmt_utc(&dt))
        .ok_or((400, format!("invalid RFC 3339 instant: {v}")))
}

fn parse_id(s: &str) -> Result<i64, (u16, String)> {
    s.parse().map_err(|_| (404, "feed not found".to_string()))
}

fn internal(e: impl std::fmt::Display) -> (u16, String) {
    (500, e.to_string())
}

/// Split a raw request target into (path, decoded query pairs).
fn split_url(raw: &str) -> (String, Vec<(String, String)>) {
    match raw.split_once('?') {
        Some((path, qs)) => (path.to_string(), parse_query(qs)),
        None => (raw.to_string(), Vec::new()),
    }
}

fn parse_query(qs: &str) -> Vec<(String, String)> {
    qs.split('&')
        .filter(|s| !s.is_empty())
        .map(|pair| match pair.split_once('=') {
            Some((k, v)) => (percent_decode(k), percent_decode(v)),
            None => (percent_decode(pair), String::new()),
        })
        .collect()
}

/// Minimal percent-decoding (no `+`-as-space, so RFC 3339 `+` offsets survive).
fn percent_decode(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            let hi = (bytes[i + 1] as char).to_digit(16);
            let lo = (bytes[i + 2] as char).to_digit(16);
            if let (Some(hi), Some(lo)) = (hi, lo) {
                out.push((hi * 16 + lo) as u8);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}
