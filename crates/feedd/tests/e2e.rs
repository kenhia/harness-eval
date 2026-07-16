//! End-to-end test: drive a real `feedd` process against a real `feedgen`
//! fixture server over local HTTP. Exercises registration, on-demand refresh,
//! conditional GET (304), dedupe, date/window semantics, search, failure
//! isolation, and deletion cascade.

use serde_json::Value;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::Duration;

struct Kid(Child);
impl Drop for Kid {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn bindir() -> PathBuf {
    // CARGO_BIN_EXE_feedd points at target/<profile>/feedd; feedgen is a sibling.
    PathBuf::from(env!("CARGO_BIN_EXE_feedd"))
        .parent()
        .unwrap()
        .to_path_buf()
}

fn ensure_built() {
    // Guarantee both debug binaries exist before we spawn them.
    let status = Command::new(env!("CARGO"))
        .args(["build", "-p", "feedgen", "-p", "feedd"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .expect("run cargo build");
    assert!(status.success(), "cargo build failed");
}

fn wait_ready(url: &str) {
    for _ in 0..100 {
        if ureq::get(url)
            .timeout(Duration::from_millis(300))
            .call()
            .is_ok()
        {
            return;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    panic!("server never became ready: {url}");
}

fn read_body(r: ureq::Response) -> Value {
    let text = r.into_string().unwrap_or_default();
    if text.trim().is_empty() {
        Value::Null
    } else {
        serde_json::from_str(&text).unwrap_or(Value::Null)
    }
}

fn get_json(url: &str) -> (u16, Value) {
    match ureq::get(url).call() {
        Ok(r) => {
            let code = r.status();
            (code, read_body(r))
        }
        Err(ureq::Error::Status(code, r)) => (code, read_body(r)),
        Err(e) => panic!("GET {url} failed: {e}"),
    }
}

fn post_json(url: &str, body: Value) -> (u16, Value) {
    let payload = serde_json::to_string(&body).unwrap();
    match ureq::post(url)
        .set("Content-Type", "application/json")
        .send_string(&payload)
    {
        Ok(r) => {
            let code = r.status();
            (code, read_body(r))
        }
        Err(ureq::Error::Status(code, r)) => (code, read_body(r)),
        Err(e) => panic!("POST {url} failed: {e}"),
    }
}

fn post_empty(url: &str) -> (u16, Value) {
    match ureq::post(url).call() {
        Ok(r) => {
            let code = r.status();
            (code, read_body(r))
        }
        Err(ureq::Error::Status(code, r)) => (code, read_body(r)),
        Err(e) => panic!("POST {url} failed: {e}"),
    }
}

fn delete(url: &str) -> u16 {
    match ureq::delete(url).call() {
        Ok(r) => r.status(),
        Err(ureq::Error::Status(code, _)) => code,
        Err(e) => panic!("DELETE {url} failed: {e}"),
    }
}

fn add_feed(api: &str, gen_base: &str, file: &str) -> i64 {
    let (code, body) = post_json(
        &format!("{api}/api/feeds"),
        serde_json::json!({ "url": format!("{gen_base}/{file}") }),
    );
    assert_eq!(code, 201, "add feed {file}: {body}");
    body["id"].as_i64().unwrap()
}

fn refresh(api: &str, id: i64) -> Value {
    let (code, body) = post_empty(&format!("{api}/api/feeds/{id}/refresh"));
    assert_eq!(code, 200, "refresh {id}: {body}");
    body
}

fn spawn(bin: &Path, args: &[&str]) -> Child {
    Command::new(bin)
        .args(args)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap_or_else(|e| panic!("spawn {}: {e}", bin.display()))
}

fn p(path: &Path) -> &str {
    path.to_str().unwrap()
}

#[test]
fn end_to_end_feedd_against_feedgen() {
    ensure_built();
    let dir = bindir();
    let feedd = dir.join("feedd");
    let feedgen = dir.join("feedgen");

    let pid = std::process::id();
    let gen_port = 20000 + (pid % 15000) as u16;
    let api_port = gen_port + 1;
    let gen_base = format!("http://127.0.0.1:{gen_port}");
    let api = format!("http://127.0.0.1:{api_port}");

    let tmp = std::env::temp_dir().join(format!("feedhub-e2e-{pid}"));
    let fixtures = tmp.join("fixtures");
    std::fs::create_dir_all(&fixtures).unwrap();
    let db = tmp.join("feedhub.db");

    // Generate the fixture corpus.
    let status = Command::new(&feedgen)
        .arg("make-fixtures")
        .arg(&fixtures)
        .status()
        .expect("run make-fixtures");
    assert!(status.success());
    for f in [
        "rss.xml",
        "atom.xml",
        "dates.xml",
        "cdata.xml",
        "malformed.xml",
    ] {
        assert!(fixtures.join(f).exists(), "missing fixture {f}");
    }

    // Start feedgen and feedd.
    let gen_listen = format!("127.0.0.1:{gen_port}");
    let api_listen = format!("127.0.0.1:{api_port}");
    let _gen = Kid(spawn(
        &feedgen,
        &["serve", "--dir", p(&fixtures), "--listen", &gen_listen],
    ));
    let _srv = Kid(spawn(
        &feedd,
        &[
            "--db",
            p(&db),
            "--listen",
            &api_listen,
            "--poll-interval",
            "0",
        ],
    ));

    wait_ready(&format!("{gen_base}/rss.xml"));
    wait_ready(&format!("{api}/api/health"));

    // Health.
    let (code, body) = get_json(&format!("{api}/api/health"));
    assert_eq!(code, 200);
    assert_eq!(body["status"], "ok");

    // Register the RSS feed.
    let rss_id = add_feed(&api, &gen_base, "rss.xml");

    // Duplicate URL -> 409.
    let (code, _) = post_json(
        &format!("{api}/api/feeds"),
        serde_json::json!({ "url": format!("{gen_base}/rss.xml") }),
    );
    assert_eq!(code, 409, "duplicate must be 409");

    // Invalid URL -> 422.
    let (code, _) = post_json(
        &format!("{api}/api/feeds"),
        serde_json::json!({ "url": "ftp://nope" }),
    );
    assert_eq!(code, 422, "invalid url must be 422");

    // First refresh: two new entries.
    let r = refresh(&api, rss_id);
    assert_eq!(r["status"], "ok", "{r}");
    assert_eq!(r["new_entries"], 2, "first refresh should ingest 2: {r}");

    // Second refresh: conditional GET -> 304, no new entries.
    let r = refresh(&api, rss_id);
    assert_eq!(r["status"], "ok");
    assert_eq!(
        r["new_entries"], 0,
        "second refresh should be 0 (304/dedupe): {r}"
    );

    // Feed object reflects title + entry_count and a clear last_error.
    let (_c, feed) = get_json(&format!("{api}/api/feeds/{rss_id}"));
    assert_eq!(feed["title"], "Example RSS Feed");
    assert_eq!(feed["entry_count"], 2);
    assert!(feed["last_error"].is_null());
    assert!(!feed["last_fetched_at"].is_null());

    // Entries ordering: published_at desc. The 2024 post sorts before the 2002.
    let (_c, entries) = get_json(&format!("{api}/api/entries?feed_id={rss_id}"));
    assert_eq!(entries["total"], 2);
    let items = entries["items"].as_array().unwrap();
    assert_eq!(items[0]["published_at"], "2024-07-15T15:30:00Z");
    assert_eq!(items[1]["published_at"], "2002-10-02T13:00:00Z");
    // Entity unescaping in titles.
    assert_eq!(items[1]["title"], "Hello RSS & World");

    // Atom feed.
    let atom_id = add_feed(&api, &gen_base, "atom.xml");
    let r = refresh(&api, atom_id);
    assert_eq!(r["new_entries"], 2, "atom refresh: {r}");
    let (_c, entries) = get_json(&format!("{api}/api/entries?feed_id={atom_id}"));
    let items = entries["items"].as_array().unwrap();
    // Entry two has no published, falls back to updated (2024-07-16), sorts first.
    assert_eq!(items[0]["published_at"], "2024-07-16T00:00:00Z");
    assert_eq!(items[0]["summary"], "Content used as the summary fallback.");

    // Dates feed: zone names + a missing date (null published_at, still stored).
    let dates_id = add_feed(&api, &gen_base, "dates.xml");
    let r = refresh(&api, dates_id);
    assert_eq!(r["new_entries"], 3, "dates refresh: {r}");
    let (_c, entries) = get_json(&format!("{api}/api/entries?feed_id={dates_id}"));
    let items = entries["items"].as_array().unwrap();
    assert_eq!(entries["total"], 3);
    // Nulls last: the missing-date entry is last.
    assert!(items[2]["published_at"].is_null());
    // EST 10:30 -> 15:30Z, PDT 10:30 -> 17:30Z present.
    let dates: Vec<&str> = items
        .iter()
        .filter_map(|e| e["published_at"].as_str())
        .collect();
    assert!(dates.contains(&"2024-07-15T15:30:00Z"));
    assert!(dates.contains(&"2024-07-15T17:30:00Z"));

    // Window is half-open and excludes null published_at.
    let (_c, win) = get_json(&format!(
        "{api}/api/entries?feed_id={dates_id}&since=2024-07-15T15:30:00Z&until=2024-07-15T17:30:00Z"
    ));
    assert_eq!(win["total"], 1, "half-open window: {win}");
    assert_eq!(win["items"][0]["published_at"], "2024-07-15T15:30:00Z");

    // Case-insensitive title search across all feeds.
    let (_c, found) = get_json(&format!("{api}/api/entries?q=eastern"));
    assert_eq!(found["total"], 1);
    assert_eq!(found["items"][0]["title"], "Eastern Standard Time");

    // Failure isolation: malformed feed records last_error, others unaffected.
    let bad_id = add_feed(&api, &gen_base, "malformed.xml");
    let r = refresh(&api, bad_id);
    assert_eq!(r["status"], "error", "malformed should error: {r}");
    let (_c, badfeed) = get_json(&format!("{api}/api/feeds/{bad_id}"));
    assert!(!badfeed["last_error"].is_null(), "last_error must be set");
    // Other feeds still intact.
    let (_c, rssfeed) = get_json(&format!("{api}/api/feeds/{rss_id}"));
    assert_eq!(rssfeed["entry_count"], 2);
    assert!(rssfeed["last_error"].is_null());

    // refresh-all returns per-feed results including the feed id.
    let (code, all) = post_empty(&format!("{api}/api/refresh"));
    assert_eq!(code, 200);
    let arr = all.as_array().unwrap();
    assert_eq!(arr.len(), 4);
    assert!(arr.iter().all(|r| r.get("feed_id").is_some()));

    // Delete cascade: removing the feed removes its entries.
    assert_eq!(delete(&format!("{api}/api/feeds/{rss_id}")), 204);
    assert_eq!(delete(&format!("{api}/api/feeds/999999")), 404);
    let (_c, entries) = get_json(&format!("{api}/api/entries?feed_id={rss_id}"));
    assert_eq!(entries["total"], 0, "entries deleted with feed");

    // Cleanup best-effort.
    let _ = std::fs::remove_dir_all(&tmp);
}
