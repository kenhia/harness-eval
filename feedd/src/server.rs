//! REST API server built on tiny_http.

use std::collections::HashMap;
use std::io::Cursor;
use std::sync::Arc;

use feedcore::api::ErrorBody;
use feedcore::dates::parse_rfc3339;
use serde::Serialize;
use serde_json::json;
use tiny_http::{Header, Method, Request, Response, Server};

use crate::refresh::{refresh_all, refresh_feed};
use crate::store::{AddError, EntryFilter, Store};

type Resp = Response<Cursor<Vec<u8>>>;

/// Run the HTTP server loop until the process is terminated.
pub fn run(store: Arc<Store>, listen: &str) -> std::io::Result<()> {
    let server = Server::http(listen)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
    println!("feedd listening on http://{listen}");
    for request in server.incoming_requests() {
        if let Err(e) = dispatch(&store, request) {
            eprintln!("feedd: request error: {e}");
        }
    }
    Ok(())
}

fn dispatch(store: &Arc<Store>, mut request: Request) -> std::io::Result<()> {
    let method = request.method().clone();
    let raw = request.url().to_string();
    let (path, query) = match raw.split_once('?') {
        Some((p, q)) => (p.to_string(), q.to_string()),
        None => (raw, String::new()),
    };
    let segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();

    let response = match (&method, segments.as_slice()) {
        (Method::Get, ["api", "health"]) => json(200, &json!({"status": "ok"})),
        (Method::Post, ["api", "feeds"]) => add_feed(store, &mut request),
        (Method::Get, ["api", "feeds"]) => list_feeds(store),
        (Method::Get, ["api", "feeds", id]) => get_feed(store, id),
        (Method::Delete, ["api", "feeds", id]) => delete_feed(store, id),
        (Method::Post, ["api", "feeds", id, "refresh"]) => refresh_one(store, id),
        (Method::Post, ["api", "refresh"]) => {
            json(200, &refresh_all(store))
        }
        (Method::Get, ["api", "entries"]) => entries(store, &query),
        _ => error(404, "not found"),
    };
    request.respond(response)
}

fn add_feed(store: &Arc<Store>, request: &mut Request) -> Resp {
    let mut body = String::new();
    if request.as_reader().read_to_string(&mut body).is_err() {
        return error(400, "could not read request body");
    }
    let url = match serde_json::from_str::<serde_json::Value>(&body) {
        Ok(v) => v.get("url").and_then(|u| u.as_str()).map(str::to_string),
        Err(_) => return error(400, "invalid JSON body"),
    };
    let url = match url {
        Some(u) if !u.is_empty() => u,
        _ => return error(422, "missing 'url' field"),
    };
    if !is_valid_http_url(&url) {
        return error(422, "url must be a valid http(s) URL");
    }
    match store.add_feed(&url) {
        Ok(feed) => json(201, &feed),
        Err(AddError::Duplicate) => error(409, "feed URL already registered"),
        Err(AddError::Db(e)) => error(500, &e.to_string()),
    }
}

fn list_feeds(store: &Arc<Store>) -> Resp {
    match store.list_feeds() {
        Ok(feeds) => json(200, &feeds),
        Err(e) => error(500, &e.to_string()),
    }
}

fn get_feed(store: &Arc<Store>, id: &str) -> Resp {
    let id = match id.parse::<i64>() {
        Ok(i) => i,
        Err(_) => return error(404, "feed not found"),
    };
    match store.get_feed(id) {
        Ok(Some(feed)) => json(200, &feed),
        Ok(None) => error(404, "feed not found"),
        Err(e) => error(500, &e.to_string()),
    }
}

fn delete_feed(store: &Arc<Store>, id: &str) -> Resp {
    let id = match id.parse::<i64>() {
        Ok(i) => i,
        Err(_) => return error(404, "feed not found"),
    };
    match store.delete_feed(id) {
        Ok(true) => Response::from_data(Vec::new()).with_status_code(204),
        Ok(false) => error(404, "feed not found"),
        Err(e) => error(500, &e.to_string()),
    }
}

fn refresh_one(store: &Arc<Store>, id: &str) -> Resp {
    let id = match id.parse::<i64>() {
        Ok(i) => i,
        Err(_) => return error(404, "feed not found"),
    };
    match refresh_feed(store, id) {
        Some(result) => json(200, &result),
        None => error(404, "feed not found"),
    }
}

fn entries(store: &Arc<Store>, query: &str) -> Resp {
    let params = parse_query(query);
    let mut filter = EntryFilter {
        limit: 50,
        ..Default::default()
    };

    if let Some(v) = params.get("feed_id") {
        match v.parse::<i64>() {
            Ok(i) => filter.feed_id = Some(i),
            Err(_) => return error(400, "feed_id must be an integer"),
        }
    }
    match parse_instant_ms(params.get("since")) {
        Ok(v) => filter.since_ms = v,
        Err(_) => return error(400, "since must be an RFC 3339 timestamp"),
    }
    match parse_instant_ms(params.get("until")) {
        Ok(v) => filter.until_ms = v,
        Err(_) => return error(400, "until must be an RFC 3339 timestamp"),
    }
    if let Some(q) = params.get("q") {
        if !q.is_empty() {
            filter.q = Some(q.clone());
        }
    }
    if let Some(v) = params.get("limit") {
        match v.parse::<i64>() {
            Ok(n) => filter.limit = n.clamp(0, 500),
            Err(_) => return error(400, "limit must be an integer"),
        }
    }
    if let Some(v) = params.get("offset") {
        match v.parse::<i64>() {
            Ok(n) => filter.offset = n.max(0),
            Err(_) => return error(400, "offset must be an integer"),
        }
    }

    match store.query_entries(&filter) {
        Ok((total, items)) => json(200, &json!({"total": total, "items": items})),
        Err(e) => error(500, &e.to_string()),
    }
}

/// Parse an optional RFC 3339 param to epoch millis. `Ok(None)` when absent;
/// `Err(())` when present but unparseable.
fn parse_instant_ms(value: Option<&String>) -> Result<Option<i64>, ()> {
    match value {
        None => Ok(None),
        Some(s) if s.is_empty() => Ok(None),
        Some(s) => parse_rfc3339(s).map(|d| Some(d.timestamp_millis())).ok_or(()),
    }
}

fn is_valid_http_url(url: &str) -> bool {
    let rest = match url.split_once("://") {
        Some((scheme, rest)) if scheme.eq_ignore_ascii_case("http") || scheme.eq_ignore_ascii_case("https") => rest,
        _ => return false,
    };
    let host = rest.split(['/', '?', '#']).next().unwrap_or("");
    !host.is_empty()
}

fn parse_query(query: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for pair in query.split('&') {
        if pair.is_empty() {
            continue;
        }
        let (k, v) = match pair.split_once('=') {
            Some((k, v)) => (k, v),
            None => (pair, ""),
        };
        map.insert(percent_decode(k), percent_decode(v));
    }
    map
}

/// Decode `%XX` escapes. `+` is left as-is so RFC 3339 offsets survive.
fn percent_decode(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            let hi = (bytes[i + 1] as char).to_digit(16);
            let lo = (bytes[i + 2] as char).to_digit(16);
            if let (Some(h), Some(l)) = (hi, lo) {
                out.push((h * 16 + l) as u8);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn json_header() -> Header {
    Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..])
        .expect("valid static header")
}

fn json<T: Serialize>(status: u16, value: &T) -> Resp {
    let body = serde_json::to_vec(value).unwrap_or_else(|_| b"{}".to_vec());
    Response::from_data(body)
        .with_status_code(status)
        .with_header(json_header())
}

fn error(status: u16, message: &str) -> Resp {
    let body = ErrorBody {
        error: message.to_string(),
    };
    json(status, &body)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_http_urls() {
        assert!(is_valid_http_url("http://example.com/feed"));
        assert!(is_valid_http_url("https://example.com"));
        assert!(is_valid_http_url("HTTP://Example.com/x"));
        assert!(!is_valid_http_url("ftp://example.com"));
        assert!(!is_valid_http_url("example.com"));
        assert!(!is_valid_http_url("http://"));
    }

    #[test]
    fn percent_decode_keeps_plus() {
        assert_eq!(percent_decode("a%20b"), "a b");
        assert_eq!(percent_decode("2021-09-06T16:20:00%2B02:00"), "2021-09-06T16:20:00+02:00");
        assert_eq!(percent_decode("x+y"), "x+y");
    }

    #[test]
    fn parse_query_pairs() {
        let m = parse_query("feed_id=3&q=hello%20world&limit=10");
        assert_eq!(m.get("feed_id").unwrap(), "3");
        assert_eq!(m.get("q").unwrap(), "hello world");
        assert_eq!(m.get("limit").unwrap(), "10");
    }

    #[test]
    fn parse_instant_ms_cases() {
        assert_eq!(parse_instant_ms(None), Ok(None));
        assert_eq!(parse_instant_ms(Some(&String::new())), Ok(None));
        assert!(parse_instant_ms(Some(&"2021-09-06T16:20:00Z".to_string())).unwrap().is_some());
        assert_eq!(parse_instant_ms(Some(&"nope".to_string())), Err(()));
    }
}
