//! feedctl end to end: the real binary, against a real feedd, which fetches
//! real feeds from a real feedgen — all over local HTTP.

use std::process::Output;

use serde_json::Value;
use tempfile::TempDir;

struct Harness {
    feedd: feedd::RunningServer,
    feedgen: feedgen::RunningServer,
    _fixture_dir: TempDir,
    _db_dir: TempDir,
}

impl Harness {
    async fn start() -> Harness {
        let fixture_dir = tempfile::tempdir().expect("fixture tempdir");
        feedgen::write_fixtures(fixture_dir.path()).expect("write fixtures");
        let feedgen = feedgen::serve_dir(
            fixture_dir.path().to_path_buf(),
            "127.0.0.1:0".parse().unwrap(),
        )
        .await
        .expect("start feedgen");

        let db_dir = tempfile::tempdir().expect("db tempdir");
        let feedd = feedd::start(feedd::Config {
            db_path: db_dir.path().join("feedd.sqlite"),
            listen: "127.0.0.1:0".parse().unwrap(),
            poll_interval: 0,
        })
        .await
        .expect("start feedd");

        Harness {
            feedd,
            feedgen,
            _fixture_dir: fixture_dir,
            _db_dir: db_dir,
        }
    }

    fn fixture_url(&self, name: &str) -> String {
        format!("{}/{name}", self.feedgen.base_url())
    }

    /// Run the feedctl binary against this harness's server.
    ///
    /// This blocks the calling thread, which is why every test that uses the
    /// harness runs on a multi-threaded runtime: feedd is a task on the same
    /// runtime, and a current-thread runtime could not poll it while this
    /// thread waits for the subprocess that is trying to talk to it.
    fn run(&self, args: &[&str]) -> Output {
        run_against(&self.feedd.base_url(), args)
    }

    /// Run feedctl and require it to succeed, returning stdout.
    fn ok(&self, args: &[&str]) -> String {
        let output = self.run(args);
        assert!(
            output.status.success(),
            "feedctl {args:?} failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8(output.stdout).expect("utf-8 stdout")
    }

    /// Run feedctl with --format json and parse what it printed.
    fn json(&self, args: &[&str]) -> Value {
        let mut all = vec!["--format", "json"];
        all.extend_from_slice(args);
        let stdout = self.ok(&all);
        serde_json::from_str(&stdout)
            .unwrap_or_else(|e| panic!("stdout is not one JSON document ({e}): {stdout:?}"))
    }

    async fn shutdown(self) {
        self.feedd.shutdown().await;
        self.feedgen.shutdown().await;
    }
}

fn run_against(server: &str, args: &[&str]) -> Output {
    std::process::Command::new(env!("CARGO_BIN_EXE_feedctl"))
        .arg("--server")
        .arg(server)
        .args(args)
        .output()
        .expect("run feedctl")
}

fn code(output: &Output) -> i32 {
    output.status.code().expect("feedctl exited normally")
}

#[tokio::test(flavor = "multi_thread")]
async fn full_workflow_in_text_format() {
    let h = Harness::start().await;
    let url = h.fixture_url("rss-basic.xml");

    let added = h.ok(&["add", &url]);
    assert!(added.contains("Added feed 1"), "{added}");

    let refreshed = h.ok(&["refresh", "1"]);
    assert!(refreshed.contains("feed 1: ok, 3 new"), "{refreshed}");

    let listed = h.ok(&["list"]);
    assert!(listed.contains("Basic RSS Feed"), "{listed}");
    assert!(listed.contains("ID"), "{listed}");

    let shown = h.ok(&["show", "1"]);
    assert!(shown.contains(&url), "{shown}");
    assert!(shown.contains("Basic RSS Feed"), "{shown}");

    let entries = h.ok(&["entries"]);
    assert!(entries.contains("Rust release notes"), "{entries}");
    assert!(entries.contains("2024-03-04T09:00:00Z"), "{entries}");
    assert!(entries.contains("Showing 3 of 3 entries."), "{entries}");

    // Refreshing everything reports each feed.
    let all = h.ok(&["refresh"]);
    assert!(all.contains("feed 1: ok, not modified"), "{all}");

    let removed = h.ok(&["remove", "1"]);
    assert!(removed.contains("Removed feed 1"), "{removed}");
    assert!(h.ok(&["list"]).contains("No feeds registered."));
    assert!(h.ok(&["entries"]).contains("No matching entries."));

    h.shutdown().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn json_format_prints_the_api_response() {
    let h = Harness::start().await;
    let url = h.fixture_url("rss-basic.xml");

    let feed = h.json(&["add", &url]);
    assert_eq!(feed["id"], 1);
    assert_eq!(feed["url"], url);
    assert!(feed["title"].is_null(), "not fetched yet");

    let result = h.json(&["refresh", "1"]);
    assert_eq!(result["status"], "ok");
    assert_eq!(result["new_entries"], 3);
    assert_eq!(result["feed_id"], 1);

    let feeds = h.json(&["list"]);
    assert!(feeds.is_array());
    assert_eq!(feeds[0]["title"], "Basic RSS Feed");
    assert_eq!(feeds[0]["entry_count"], 3);

    assert_eq!(h.json(&["show", "1"])["id"], 1);

    let page = h.json(&["entries", "--limit", "2"]);
    assert_eq!(page["total"], 3);
    assert_eq!(page["items"].as_array().expect("items").len(), 2);

    let all = h.json(&["refresh"]);
    assert!(all.is_array());
    assert_eq!(all[0]["feed_id"], 1);

    // DELETE has no body, so feedctl still prints a document of its own.
    let removed = h.json(&["remove", "1"]);
    assert_eq!(removed["status"], "ok");

    h.shutdown().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn entries_flags_reach_the_api() {
    let h = Harness::start().await;
    h.ok(&["add", &h.fixture_url("rss-basic.xml")]);
    h.ok(&["add", &h.fixture_url("dates-edge.xml")]);
    h.ok(&["refresh"]);

    assert_eq!(h.json(&["entries", "--feed", "1"])["total"], 3);
    assert_eq!(h.json(&["entries", "--search", "notes"])["total"], 3);
    // The search value is a substring, not a pattern, and is URL-encoded on the
    // way out.
    assert_eq!(h.json(&["entries", "--search", "rust release"])["total"], 1);

    // A window with a non-UTC offset, which has to survive query encoding.
    let page = h.json(&[
        "entries",
        "--since",
        "2024-03-01T13:00:00+01:00",
        "--until",
        "2024-03-01T17:00:00Z",
        "--limit",
        "500",
    ]);
    assert_eq!(page["total"], 4);

    let page = h.json(&["entries", "--limit", "2", "--offset", "1"]);
    assert_eq!(page["items"].as_array().expect("items").len(), 2);

    h.shutdown().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn server_errors_exit_1_with_the_message_on_stderr() {
    let h = Harness::start().await;

    // 404 from the API.
    let output = h.run(&["show", "999"]);
    assert_eq!(code(&output), 1);
    assert!(output.stdout.is_empty(), "errors do not print to stdout");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("no feed with id 999"), "{stderr}");

    assert_eq!(code(&h.run(&["remove", "999"])), 1);
    assert_eq!(code(&h.run(&["refresh", "999"])), 1);

    // 422: not an http(s) URL.
    let output = h.run(&["add", "ftp://example.invalid/feed.xml"]);
    assert_eq!(code(&output), 1);
    assert!(String::from_utf8_lossy(&output.stderr).contains("http"));

    // 409: already registered.
    let url = h.fixture_url("rss-basic.xml");
    assert_eq!(code(&h.run(&["add", &url])), 0);
    let output = h.run(&["add", &url]);
    assert_eq!(code(&output), 1);
    assert!(String::from_utf8_lossy(&output.stderr).contains("already registered"));

    // 422: a bad timestamp, rejected by the server.
    let output = h.run(&["entries", "--since", "yesterday"]);
    assert_eq!(code(&output), 1);
    assert!(String::from_utf8_lossy(&output.stderr).contains("RFC 3339"));

    h.shutdown().await;
}

#[test]
fn an_unreachable_server_exits_2() {
    // Port 1 on loopback: nothing is listening there.
    let output = run_against("http://127.0.0.1:1", &["list"]);
    assert_eq!(code(&output), 2);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("cannot reach feedd"), "{stderr}");

    // A URL that is not a server at all fails the same way.
    let output = run_against("http://feedctl.invalid.test", &["list"]);
    assert_eq!(code(&output), 2);
}

#[test]
fn invalid_usage_exits_2() {
    for args in [
        vec!["nonsense"],                   // no such subcommand
        vec!["show"],                       // missing required id
        vec!["show", "not-a-number"],       // id is not an integer
        vec!["entries", "--limit", "many"], // limit is not an integer
        vec!["--format", "yaml", "list"],   // not a known format
    ] {
        let output = run_against("http://127.0.0.1:1", &args);
        assert_eq!(code(&output), 2, "expected exit 2 for {args:?}");
    }

    // --help and --version are successes, not usage errors.
    for args in [vec!["--help"], vec!["--version"], vec!["entries", "--help"]] {
        let output = std::process::Command::new(env!("CARGO_BIN_EXE_feedctl"))
            .args(&args)
            .output()
            .expect("run feedctl");
        assert_eq!(code(&output), 0, "expected exit 0 for {args:?}");
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn a_feed_that_fails_to_fetch_is_reported_not_fatal() {
    let h = Harness::start().await;
    h.ok(&["add", &h.fixture_url("malformed.xml")]);

    // The API call succeeded, so feedctl succeeds; the failure is in the result.
    let result = h.json(&["refresh", "1"]);
    assert_eq!(result["status"], "error");
    assert!(result["error"].is_string());

    let text = h.ok(&["refresh", "1"]);
    assert!(text.contains("feed 1: error:"), "{text}");

    let shown = h.ok(&["show", "1"]);
    assert!(shown.contains("error:"), "{shown}");

    h.shutdown().await;
}
