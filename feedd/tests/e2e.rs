//! End-to-end test: drives the real `feedd` binary against the real `feedgen`
//! binary over local HTTP. No network access beyond these two local servers.

use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::{Child, Command};
use std::time::Duration;

use serde_json::Value;

/// Kills a child process when dropped so tests never leak servers.
struct Guard(Child);
impl Drop for Guard {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn bin(name: &str) -> PathBuf {
    // feedd's own binary path is provided by cargo; sibling bins share the dir.
    let feedd = env!("CARGO_BIN_EXE_feedd");
    let dir = Path::new(feedd).parent().expect("target bin dir");
    let path = dir.join(name);
    if !path.exists() {
        // Ensure the sibling binary exists even under `cargo test -p feedd`.
        let status = Command::new(env!("CARGO"))
            .args(["build", "-p", name])
            .status()
            .expect("cargo build");
        assert!(status.success(), "failed to build {name}");
    }
    path
}

fn free_port() -> u16 {
    let l = TcpListener::bind("127.0.0.1:0").expect("bind ephemeral");
    l.local_addr().unwrap().port()
}

fn wait_ready(url: &str) {
    for _ in 0..100 {
        if ureq::get(url)
            .timeout(Duration::from_millis(200))
            .call()
            .is_ok()
        {
            return;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    panic!("server never became ready: {url}");
}

fn get_json(url: &str) -> (u16, Value) {
    match ureq::get(url).call() {
        Ok(r) => (r.status(), parse_body(r)),
        Err(ureq::Error::Status(c, r)) => (c, parse_body(r)),
        Err(e) => panic!("transport error: {e}"),
    }
}

fn parse_body(r: ureq::Response) -> Value {
    let text = r.into_string().unwrap_or_default();
    serde_json::from_str(&text).unwrap_or(Value::Null)
}

fn post_json(url: &str, body: Option<Value>) -> (u16, Value) {
    let req = ureq::post(url);
    let res = match body {
        Some(b) => req
            .set("Content-Type", "application/json")
            .send_string(&b.to_string()),
        None => req.call(),
    };
    match res {
        Ok(r) => (r.status(), parse_body(r)),
        Err(ureq::Error::Status(c, r)) => (c, parse_body(r)),
        Err(e) => panic!("transport error: {e}"),
    }
}

#[test]
fn feedd_against_feedgen_over_http() {
    let feedgen = bin("feedgen");
    let feedd = bin("feedd");

    let fixtures = tempfile::tempdir().unwrap();
    let db = tempfile::NamedTempFile::new().unwrap();

    // Generate the fixture corpus.
    let status = Command::new(&feedgen)
        .args(["make-fixtures", fixtures.path().to_str().unwrap()])
        .status()
        .unwrap();
    assert!(status.success());

    let gen_port = free_port();
    let srv_port = free_port();

    let _gen = Guard(
        Command::new(&feedgen)
            .args([
                "serve",
                "--dir",
                fixtures.path().to_str().unwrap(),
                "--listen",
                &format!("127.0.0.1:{gen_port}"),
            ])
            .spawn()
            .unwrap(),
    );
    let _srv = Guard(
        Command::new(&feedd)
            .args([
                "--db",
                db.path().to_str().unwrap(),
                "--listen",
                &format!("127.0.0.1:{srv_port}"),
                "--poll-interval",
                "0",
            ])
            .spawn()
            .unwrap(),
    );

    let gen = format!("http://127.0.0.1:{gen_port}");
    let api = format!("http://127.0.0.1:{srv_port}/api");
    wait_ready(&format!("{gen}/rss.xml"));
    wait_ready(&format!("{api}/health"));

    // Health.
    let (code, body) = get_json(&format!("{api}/health"));
    assert_eq!(code, 200);
    assert_eq!(body["status"], "ok");

    // Register the RSS feed.
    let (code, feed) = post_json(
        &format!("{api}/feeds"),
        Some(serde_json::json!({ "url": format!("{gen}/rss.xml") })),
    );
    assert_eq!(code, 201);
    let feed_id = feed["id"].as_i64().unwrap();

    // Duplicate -> 409; invalid scheme -> 422.
    let (code, _) = post_json(
        &format!("{api}/feeds"),
        Some(serde_json::json!({ "url": format!("{gen}/rss.xml") })),
    );
    assert_eq!(code, 409);
    let (code, _) = post_json(
        &format!("{api}/feeds"),
        Some(serde_json::json!({ "url": "ftp://nope/x" })),
    );
    assert_eq!(code, 422);

    // First refresh: two new entries.
    let (code, res) = post_json(&format!("{api}/feeds/{feed_id}/refresh"), None);
    assert_eq!(code, 200);
    assert_eq!(res["status"], "ok");
    assert_eq!(res["new_entries"], 2);

    // Feed metadata updated.
    let (_, feed) = get_json(&format!("{api}/feeds/{feed_id}"));
    assert_eq!(feed["title"], "Example RSS Feed");
    assert_eq!(feed["entry_count"], 2);
    assert!(feed["last_error"].is_null());

    // Entries ordered by published_at desc; RFC 822 -0500 normalized to UTC.
    let (_, entries) = get_json(&format!("{api}/entries?feed_id={feed_id}"));
    assert_eq!(entries["total"], 2);
    let items = entries["items"].as_array().unwrap();
    assert_eq!(items[0]["title"], "Second Post");
    assert_eq!(items[0]["published_at"], "2021-09-07T13:00:00Z");
    assert_eq!(items[1]["title"], "First Post");

    // Second refresh hits conditional GET (304): no new entries, no duplicates.
    let (_, res) = post_json(&format!("{api}/feeds/{feed_id}/refresh"), None);
    assert_eq!(res["new_entries"], 0);
    let (_, feed) = get_json(&format!("{api}/feeds/{feed_id}"));
    assert_eq!(feed["entry_count"], 2);

    // Failure isolation: a malformed feed records an error but does not crash.
    let (_, bad) = post_json(
        &format!("{api}/feeds"),
        Some(serde_json::json!({ "url": format!("{gen}/malformed.xml") })),
    );
    let bad_id = bad["id"].as_i64().unwrap();
    let (_, res) = post_json(&format!("{api}/feeds/{bad_id}/refresh"), None);
    assert_eq!(res["status"], "error");
    let (_, bad_feed) = get_json(&format!("{api}/feeds/{bad_id}"));
    assert!(!bad_feed["last_error"].is_null());

    // The good feed is still healthy after the bad one failed.
    let (_, res) = post_json(&format!("{api}/refresh"), None);
    let results = res.as_array().unwrap();
    assert!(results
        .iter()
        .any(|r| r["feed_id"] == feed_id && r["status"] == "ok"));

    // Null published_at excluded when a window bound is present.
    let (_, dates) = post_json(
        &format!("{api}/feeds"),
        Some(serde_json::json!({ "url": format!("{gen}/dates.xml") })),
    );
    let dates_id = dates["id"].as_i64().unwrap();
    post_json(&format!("{api}/feeds/{dates_id}/refresh"), None);
    let (_, all) = get_json(&format!("{api}/entries?feed_id={dates_id}"));
    assert_eq!(all["total"], 3);
    let (_, windowed) = get_json(&format!(
        "{api}/entries?feed_id={dates_id}&since=2000-01-01T00:00:00Z"
    ));
    assert_eq!(windowed["total"], 2, "null-date entry excluded by window");

    // Delete cascades to entries.
    let code = ureq::delete(&format!("{api}/feeds/{feed_id}"))
        .call()
        .unwrap()
        .status();
    assert_eq!(code, 204);
    let (_, entries) = get_json(&format!("{api}/entries?feed_id={feed_id}"));
    assert_eq!(entries["total"], 0);
    let (code, _) = get_json(&format!("{api}/feeds/{feed_id}"));
    assert_eq!(code, 404);
}
