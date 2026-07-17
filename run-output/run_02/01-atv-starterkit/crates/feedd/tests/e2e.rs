//! End-to-end test: drive the real `feedd` binary against an in-process
//! `feedgen` static server, entirely over local HTTP.

use serde_json::Value;
use std::net::TcpListener;
use std::process::{Child, Command};
use std::thread;
use std::time::{Duration, Instant};

/// Grab a free localhost port by binding to :0 and releasing it.
fn free_port() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    listener.local_addr().unwrap().port()
}

fn get(url: &str) -> (u16, Value) {
    call("GET", url, None)
}

fn call(method: &str, url: &str, body: Option<Value>) -> (u16, Value) {
    let req = ureq::request(method, url);
    let res = match body {
        Some(v) => req
            .set("Content-Type", "application/json")
            .send_string(&v.to_string()),
        None => req.call(),
    };
    match res {
        Ok(resp) => {
            let status = resp.status();
            let text = resp.into_string().unwrap_or_default();
            (status, serde_json::from_str(&text).unwrap_or(Value::Null))
        }
        Err(ureq::Error::Status(status, resp)) => {
            let text = resp.into_string().unwrap_or_default();
            (status, serde_json::from_str(&text).unwrap_or(Value::Null))
        }
        Err(e) => panic!("transport error: {e}"),
    }
}

fn wait_ready(base: &str) {
    let deadline = Instant::now() + Duration::from_secs(10);
    while Instant::now() < deadline {
        if let Ok(resp) = ureq::get(&format!("{base}/api/health")).call() {
            if resp.status() == 200 {
                return;
            }
        }
        thread::sleep(Duration::from_millis(50));
    }
    panic!("feedd did not become ready");
}

struct Feedd(Child);
impl Drop for Feedd {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

#[test]
fn end_to_end_feedd_against_feedgen() {
    // Prepare a fixture corpus in a temp dir and serve it in-process.
    let dir = std::env::temp_dir().join(format!("feedhub-e2e-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let corpus = dir.join("corpus");
    feedgen::fixtures::write(corpus.to_str().unwrap()).unwrap();

    let gen_port = free_port();
    let gen_addr = format!("127.0.0.1:{gen_port}");
    {
        let corpus = corpus.clone();
        let gen_addr = gen_addr.clone();
        thread::spawn(move || {
            feedgen::serve::run(corpus.to_str().unwrap(), &gen_addr).unwrap();
        });
    }

    // Launch the real feedd binary with on-demand refresh only.
    let db = dir.join("feed.db");
    let feedd_port = free_port();
    let base = format!("http://127.0.0.1:{feedd_port}");
    let child = Command::new(env!("CARGO_BIN_EXE_feedd"))
        .args([
            "--db",
            db.to_str().unwrap(),
            "--listen",
            &format!("127.0.0.1:{feedd_port}"),
            "--poll-interval",
            "0",
        ])
        .spawn()
        .expect("spawn feedd");
    let _guard = Feedd(child);
    wait_ready(&base);

    let feed_base = format!("http://127.0.0.1:{gen_port}");

    // Register the RSS fixture.
    let (s, rss) = call(
        "POST",
        &format!("{base}/api/feeds"),
        Some(serde_json::json!({ "url": format!("{feed_base}/rss.xml") })),
    );
    assert_eq!(s, 201, "create rss feed");
    let rss_id = rss["id"].as_i64().unwrap();
    assert!(rss["title"].is_null(), "title null before first fetch");

    // Duplicate registration is 409.
    let (s, _) = call(
        "POST",
        &format!("{base}/api/feeds"),
        Some(serde_json::json!({ "url": format!("{feed_base}/rss.xml") })),
    );
    assert_eq!(s, 409, "duplicate feed rejected");

    // Non-http url is 422.
    let (s, _) = call(
        "POST",
        &format!("{base}/api/feeds"),
        Some(serde_json::json!({ "url": "ftp://nope" })),
    );
    assert_eq!(s, 422, "non-http url rejected");

    let (_, atom) = call(
        "POST",
        &format!("{base}/api/feeds"),
        Some(serde_json::json!({ "url": format!("{feed_base}/atom.xml") })),
    );
    let atom_id = atom["id"].as_i64().unwrap();

    let (_, bad) = call(
        "POST",
        &format!("{base}/api/feeds"),
        Some(serde_json::json!({ "url": format!("{feed_base}/malformed.xml") })),
    );
    let bad_id = bad["id"].as_i64().unwrap();

    // Refresh all: rss+atom succeed, malformed records an error but does not crash.
    let (s, results) = call("POST", &format!("{base}/api/refresh"), None);
    assert_eq!(s, 200);
    let results = results.as_array().unwrap();
    assert_eq!(results.len(), 3);
    for r in results {
        let id = r["id"].as_i64().unwrap();
        if id == bad_id {
            assert_eq!(r["status"], "error");
            assert_eq!(r["new_entries"].as_i64().unwrap(), 0);
        } else {
            assert_eq!(r["status"], "ok");
            assert_eq!(r["new_entries"].as_i64().unwrap(), 2);
        }
    }

    // Feed title populated after successful fetch; malformed feed has last_error.
    let (_, rss_feed) = get(&format!("{base}/api/feeds/{rss_id}"));
    assert_eq!(rss_feed["title"], "Example RSS Feed");
    assert_eq!(rss_feed["last_error"], Value::Null);
    let (_, bad_feed) = get(&format!("{base}/api/feeds/{bad_id}"));
    assert!(bad_feed["last_error"].is_string());

    // Conditional GET: a second refresh reports 0 new entries (304).
    let (_, again) = call("POST", &format!("{base}/api/feeds/{rss_id}/refresh"), None);
    assert_eq!(again["new_entries"].as_i64().unwrap(), 0);

    // Entries: total across rss+atom is 4.
    let (_, all) = get(&format!("{base}/api/entries"));
    assert_eq!(all["total"].as_i64().unwrap(), 4);

    // Ordering is published_at descending; the newest is the Atom entry.
    let items = all["items"].as_array().unwrap();
    assert_eq!(items[0]["published_at"], "2021-02-11T13:45:00Z");

    // Per-feed filter.
    let (_, atom_only) = get(&format!("{base}/api/entries?feed_id={atom_id}"));
    assert_eq!(atom_only["total"].as_i64().unwrap(), 2);

    // Window semantics: half-open, since <= published_at < until.
    let (_, win) = get(&format!(
        "{base}/api/entries?feed_id={rss_id}&since=2021-01-05T00:00:00Z&until=2021-01-06T00:00:00Z"
    ));
    assert_eq!(win["total"].as_i64().unwrap(), 1);
    assert_eq!(win["items"][0]["guid"], "post-0002");

    // Case-insensitive title search.
    let (_, q) = get(&format!("{base}/api/entries?q=SECOND"));
    assert_eq!(q["total"].as_i64().unwrap(), 1);

    // Delete cascades: feed and its entries disappear.
    let (s, _) = call("DELETE", &format!("{base}/api/feeds/{rss_id}"), None);
    assert_eq!(s, 204);
    let (s, _) = get(&format!("{base}/api/feeds/{rss_id}"));
    assert_eq!(s, 404);
    let (_, after) = get(&format!("{base}/api/entries"));
    assert_eq!(
        after["total"].as_i64().unwrap(),
        2,
        "rss entries removed with feed"
    );

    let _ = std::fs::remove_dir_all(&dir);
}
