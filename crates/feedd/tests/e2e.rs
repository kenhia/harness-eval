//! End-to-end test: drive a real `feedd` process against an in-process
//! `feedgen` HTTP server serving the fixture corpus over local HTTP.

use std::net::TcpListener;
use std::path::Path;
use std::process::{Child, Command};
use std::thread;
use std::time::{Duration, Instant};

use serde_json::Value;

/// Grab an ephemeral port by binding and immediately releasing it.
fn free_port() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    listener.local_addr().unwrap().port()
}

/// Start feedgen serving `dir` on a fresh port; returns the base URL.
fn start_feedgen(dir: &Path) -> String {
    let port = free_port();
    let addr = format!("127.0.0.1:{port}");
    let server = feedgen::bind(&addr).expect("bind feedgen");
    let dir = dir.to_path_buf();
    thread::spawn(move || feedgen::serve(&server, &dir));
    format!("http://{addr}")
}

struct FeeddProc {
    child: Child,
    base: String,
}

impl Drop for FeeddProc {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// Launch the built `feedd` binary and wait until it answers `/api/health`.
fn start_feedd(db: &Path) -> FeeddProc {
    let port = free_port();
    let listen = format!("127.0.0.1:{port}");
    let child = Command::new(env!("CARGO_BIN_EXE_feedd"))
        .args([
            "--db",
            db.to_str().unwrap(),
            "--listen",
            &listen,
            "--poll-interval",
            "0",
        ])
        .spawn()
        .expect("spawn feedd");
    let base = format!("http://{listen}");

    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        if let Ok(resp) = ureq::get(&format!("{base}/api/health")).call() {
            if resp.status() == 200 {
                break;
            }
        }
        if Instant::now() > deadline {
            panic!("feedd did not become ready");
        }
        thread::sleep(Duration::from_millis(100));
    }

    FeeddProc { child, base }
}

fn get_json(url: &str) -> (u16, Value) {
    match ureq::get(url).call() {
        Ok(resp) => {
            let status = resp.status();
            let v: Value = serde_json::from_str(&resp.into_string().unwrap_or_default())
                .unwrap_or(Value::Null);
            (status, v)
        }
        Err(ureq::Error::Status(code, resp)) => (
            code,
            serde_json::from_str(&resp.into_string().unwrap_or_default()).unwrap_or(Value::Null),
        ),
        Err(e) => panic!("request failed: {e}"),
    }
}

fn post_json(url: &str, body: Option<Value>) -> (u16, Value) {
    let req = ureq::post(url);
    let result = match body {
        Some(b) => req.send_string(&b.to_string()),
        None => req.call(),
    };
    match result {
        Ok(resp) => {
            let status = resp.status();
            let v: Value = serde_json::from_str(&resp.into_string().unwrap_or_default())
                .unwrap_or(Value::Null);
            (status, v)
        }
        Err(ureq::Error::Status(code, resp)) => (
            code,
            serde_json::from_str(&resp.into_string().unwrap_or_default()).unwrap_or(Value::Null),
        ),
        Err(e) => panic!("request failed: {e}"),
    }
}

fn add_feed(base: &str, url: &str) -> (u16, Value) {
    post_json(
        &format!("{base}/api/feeds"),
        Some(serde_json::json!({ "url": url })),
    )
}

#[test]
fn end_to_end_against_feedgen() {
    let tmp = tempfile::tempdir().unwrap();
    let fixtures = tmp.path().join("fixtures");
    feedgen::make_fixtures(&fixtures).unwrap();
    let feed_base = start_feedgen(&fixtures);

    let db = tmp.path().join("feedhub.db");
    let feedd = start_feedd(&db);
    let base = &feedd.base;

    // Register the RSS fixture.
    let (status, feed) = add_feed(base, &format!("{feed_base}/rss.xml"));
    assert_eq!(status, 201, "add feed: {feed}");
    let rss_id = feed["id"].as_i64().unwrap();

    // Duplicate URL is rejected.
    let (status, _) = add_feed(base, &format!("{feed_base}/rss.xml"));
    assert_eq!(status, 409);

    // Invalid URL is rejected.
    let (status, _) = add_feed(base, "ftp://example.com/x");
    assert_eq!(status, 422);

    // First refresh pulls both items.
    let (status, r) = post_json(&format!("{base}/api/feeds/{rss_id}/refresh"), None);
    assert_eq!(status, 200);
    assert_eq!(r["status"], "ok");
    assert_eq!(r["new_entries"], 2);

    // Second refresh hits the conditional-GET path: 304, no new entries.
    let (_s, r) = post_json(&format!("{base}/api/feeds/{rss_id}/refresh"), None);
    assert_eq!(r["new_entries"], 0);
    assert_eq!(r["not_modified"], true);

    // The feed now has a title and no error.
    let (_s, feed) = get_json(&format!("{base}/api/feeds/{rss_id}"));
    assert_eq!(feed["title"], "Example RSS Feed");
    assert_eq!(feed["last_error"], Value::Null);
    assert_eq!(feed["entry_count"], 2);

    // Entries: ordered published_at desc, and entity unescaping applied.
    let (_s, entries) = get_json(&format!("{base}/api/entries?feed_id={rss_id}"));
    assert_eq!(entries["total"], 2);
    let items = entries["items"].as_array().unwrap();
    assert_eq!(items[0]["title"], "Second Post");
    assert_eq!(items[0]["summary"], "Another post & more.");
    assert_eq!(items[1]["title"], "Hello World");
    // RSS pubDate -0500 normalized to UTC.
    assert_eq!(items[1]["published_at"], "2006-01-02T20:04:05Z");

    // Add the edge-case dates fixture: zone names + a missing date.
    let (_s, dfeed) = add_feed(base, &format!("{feed_base}/dates.xml"));
    let dates_id = dfeed["id"].as_i64().unwrap();
    let (_s, r) = post_json(&format!("{base}/api/feeds/{dates_id}/refresh"), None);
    assert_eq!(r["new_entries"], 5);

    // Window is half-open and excludes null-dated entries.
    let (_s, win) = get_json(&format!(
        "{base}/api/entries?feed_id={dates_id}&since=2005-01-01T00:00:00Z&until=2006-01-01T00:00:00Z"
    ));
    let titles: Vec<&str> = win["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|i| i["title"].as_str().unwrap())
        .collect();
    assert_eq!(titles, vec!["Eastern Daylight", "Pacific Standard"]);

    // Case-insensitive title search.
    let (_s, search) = get_json(&format!("{base}/api/entries?q=UNIVERSAL"));
    assert_eq!(search["total"], 1);

    // Malformed feed: failure is isolated to that feed.
    let (_s, mfeed) = add_feed(base, &format!("{feed_base}/malformed.xml"));
    let mal_id = mfeed["id"].as_i64().unwrap();
    let (_s, r) = post_json(&format!("{base}/api/feeds/{mal_id}/refresh"), None);
    assert_eq!(r["status"], "error");
    let (_s, mfeed) = get_json(&format!("{base}/api/feeds/{mal_id}"));
    assert!(mfeed["last_error"].is_string());
    // Other feeds remain healthy and their entries intact.
    let (_s, feed) = get_json(&format!("{base}/api/feeds/{rss_id}"));
    assert_eq!(feed["entry_count"], 2);
    assert_eq!(feed["last_error"], Value::Null);

    // refresh-all returns a per-feed result array including feed ids.
    let (_s, all) = post_json(&format!("{base}/api/refresh"), None);
    let arr = all.as_array().unwrap();
    assert_eq!(arr.len(), 3);
    assert!(arr.iter().all(|r| r.get("feed_id").is_some()));

    // Deleting a feed removes its entries.
    let resp = ureq::delete(&format!("{base}/api/feeds/{dates_id}"))
        .call()
        .unwrap();
    assert_eq!(resp.status(), 204);
    let (_s, entries) = get_json(&format!("{base}/api/entries?feed_id={dates_id}"));
    assert_eq!(entries["total"], 0);
}
