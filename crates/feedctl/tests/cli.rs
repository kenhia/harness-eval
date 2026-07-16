//! The real feedctl binary against a real feedd, over local HTTP.
//!
//! Spawning the actual binary is the only way to assert exit codes and the
//! stdout/stderr split, which are the parts of feedctl's contract that a
//! library-level test cannot reach.
//!
//! feedd and feedgen are dev-dependencies here so a live server can be served
//! in-process: `CARGO_BIN_EXE_*` is only injected for this package's own
//! binaries, so `CARGO_BIN_EXE_feedd` does not exist under `-p feedctl`.

use std::process::Output;
use std::sync::Arc;

use feedd::store::Store;
use feedgen::fixtures;
use serde_json::Value;

struct Harness {
    _dir: tempfile::TempDir,
    origin: feedgen::Spawned,
    feedd: feedd::Spawned,
}

fn harness() -> Harness {
    let dir = tempfile::tempdir().expect("tempdir");
    fixtures::write_corpus(dir.path()).expect("corpus");
    let origin = feedgen::spawn(feedgen::Options::new(dir.path())).expect("spawn feedgen");
    let store = Arc::new(Store::open_in_memory().expect("store"));
    let feedd = feedd::spawn(store).expect("spawn feedd");
    Harness {
        _dir: dir,
        origin,
        feedd,
    }
}

/// Run the real binary. `server` is the `--server` value.
fn feedctl(server: &str, args: &[&str]) -> Output {
    std::process::Command::new(env!("CARGO_BIN_EXE_feedctl"))
        .arg("--server")
        .arg(server)
        .args(args)
        .output()
        .expect("run feedctl")
}

fn stdout(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).into_owned()
}

fn stderr(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).into_owned()
}

fn code(out: &Output) -> i32 {
    out.status
        .code()
        .expect("feedctl should exit normally, not by signal")
}

// ------------------------------------------------------------- exit codes

#[tokio::test(flavor = "multi_thread")]
async fn success_exits_0() {
    let h = harness();
    let out = feedctl(&h.feedd.base_url(), &["list"]);
    assert_eq!(code(&out), 0, "stderr: {}", stderr(&out));
}

#[tokio::test(flavor = "multi_thread")]
async fn a_server_error_exits_1_with_the_message_on_stderr() {
    let h = harness();
    let url = h.origin.url("rss-basic.rss");
    assert_eq!(code(&feedctl(&h.feedd.base_url(), &["add", &url])), 0);

    // Adding the same URL again is a 409: the server answered, and said no.
    let out = feedctl(&h.feedd.base_url(), &["add", &url]);
    assert_eq!(code(&out), 1, "a server-side error is exit 1");
    assert!(
        stderr(&out).contains("already registered"),
        "the server's message belongs on stderr; got {:?}",
        stderr(&out)
    );
    assert!(stdout(&out).is_empty(), "nothing on stdout for an error");
}

#[tokio::test(flavor = "multi_thread")]
async fn a_404_from_the_server_exits_1() {
    let h = harness();
    let out = feedctl(&h.feedd.base_url(), &["show", "9999"]);
    assert_eq!(code(&out), 1);
    assert!(stderr(&out).contains("not found"), "got {:?}", stderr(&out));
}

#[tokio::test(flavor = "multi_thread")]
async fn an_unreachable_server_exits_2() {
    // Bind and drop, so the port is almost certainly closed.
    let dead = {
        let l = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        l.local_addr().unwrap().port()
    };
    let out = feedctl(&format!("http://127.0.0.1:{dead}"), &["list"]);
    assert_eq!(code(&out), 2, "unreachable is exit 2, not 1");
    assert!(!stderr(&out).is_empty(), "an explanation belongs on stderr");
}

#[tokio::test(flavor = "multi_thread")]
async fn invalid_usage_exits_2() {
    for args in [
        vec!["frobnicate"],
        vec!["show", "not-a-number"],
        vec!["--format", "yaml", "list"],
        vec!["entries", "--limit"],
    ] {
        let out = feedctl("http://127.0.0.1:1", &args);
        assert_eq!(code(&out), 2, "{args:?} is a usage error, so exit 2");
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn help_and_version_exit_0() {
    for args in [vec!["--help"], vec!["--version"]] {
        let out = feedctl("http://127.0.0.1:1", &args);
        assert_eq!(code(&out), 0, "{args:?} is not an error");
        assert!(!stdout(&out).is_empty());
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn a_client_side_invalid_url_is_a_server_error_exit_1() {
    // The server is reachable and rejects the URL with a 422, so this is a
    // server answer (1), not a transport failure (2).
    let h = harness();
    let out = feedctl(&h.feedd.base_url(), &["add", "file:///etc/passwd"]);
    assert_eq!(code(&out), 1);
    assert!(!stderr(&out).is_empty());
}

// ---------------------------------------------------------------- formats

#[tokio::test(flavor = "multi_thread")]
async fn json_format_emits_exactly_one_valid_json_document() {
    let h = harness();
    let url = h.origin.url("rss-basic.rss");
    feedctl(&h.feedd.base_url(), &["add", &url]);

    for args in [
        vec!["list"],
        vec!["show", "1"],
        vec!["refresh", "1"],
        vec!["refresh"],
        vec!["entries"],
    ] {
        let mut full = vec!["--format", "json"];
        full.extend_from_slice(&args);
        let out = feedctl(&h.feedd.base_url(), &full);
        assert_eq!(code(&out), 0, "{args:?}: {}", stderr(&out));

        let body = stdout(&out);
        // Exactly one document: serde's parser rejects trailing content.
        let parsed: Value = serde_json::from_str(body.trim())
            .unwrap_or_else(|e| panic!("{args:?} did not emit one JSON document: {e}\n{body}"));
        assert!(!parsed.is_null(), "{args:?} emitted null");
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn json_format_passes_the_api_response_through_unchanged() {
    let h = harness();
    let url = h.origin.url("rss-basic.rss");
    let out = feedctl(&h.feedd.base_url(), &["--format", "json", "add", &url]);
    let feed: Value = serde_json::from_str(stdout(&out).trim()).unwrap();

    // The API's feed object, not a reshaped one.
    for field in [
        "id",
        "url",
        "title",
        "last_fetched_at",
        "last_error",
        "entry_count",
    ] {
        assert!(
            feed.get(field).is_some(),
            "the API's {field} field is missing"
        );
    }
    assert_eq!(feed["url"], url);
}

#[tokio::test(flavor = "multi_thread")]
async fn remove_emits_valid_json_despite_the_204_having_no_body() {
    let h = harness();
    let url = h.origin.url("rss-basic.rss");
    feedctl(&h.feedd.base_url(), &["add", &url]);

    let out = feedctl(&h.feedd.base_url(), &["--format", "json", "remove", "1"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let v: Value = serde_json::from_str(stdout(&out).trim()).expect("valid JSON for a 204");
    assert_eq!(v["status"], "ok");
    assert_eq!(v["id"], 1);
}

#[tokio::test(flavor = "multi_thread")]
async fn text_format_is_human_readable_and_is_the_default() {
    let h = harness();
    let url = h.origin.url("rss-basic.rss");
    feedctl(&h.feedd.base_url(), &["add", &url]);

    let out = feedctl(&h.feedd.base_url(), &["list"]);
    let body = stdout(&out);
    assert!(body.contains("ID"), "expected a table header, got {body:?}");
    assert!(body.contains(&url));
    assert!(
        serde_json::from_str::<Value>(body.trim()).is_err(),
        "the default format is text, not JSON"
    );
}

// ------------------------------------------------------- the whole flow

#[tokio::test(flavor = "multi_thread")]
async fn add_refresh_entries_remove_round_trip() {
    let h = harness();
    let server = h.feedd.base_url();
    let url = h.origin.url("rss-basic.rss");

    let out = feedctl(&server, &["add", &url]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    assert!(stdout(&out).contains("Added feed 1"));

    let out = feedctl(&server, &["refresh", "1"]);
    assert_eq!(code(&out), 0);
    assert!(
        stdout(&out).contains("3 new entries"),
        "got {:?}",
        stdout(&out)
    );

    // A second refresh gets a 304 from feedgen and adds nothing.
    let out = feedctl(&server, &["refresh"]);
    assert!(
        stdout(&out).contains("not modified"),
        "got {:?}",
        stdout(&out)
    );

    let out = feedctl(&server, &["show", "1"]);
    assert!(stdout(&out).contains("Example RSS Feed"));

    let out = feedctl(&server, &["entries", "--search", "second"]);
    assert_eq!(code(&out), 0);
    assert!(
        stdout(&out).contains("Second post"),
        "got {:?}",
        stdout(&out)
    );
    assert!(stdout(&out).contains("Showing 1 of 1"));

    let out = feedctl(&server, &["entries", "--limit", "2"]);
    assert!(
        stdout(&out).contains("Showing 2 of 3"),
        "got {:?}",
        stdout(&out)
    );

    let out = feedctl(&server, &["remove", "1"]);
    assert_eq!(code(&out), 0);

    let out = feedctl(&server, &["--format", "json", "entries"]);
    let page: Value = serde_json::from_str(stdout(&out).trim()).unwrap();
    assert_eq!(page["total"], 0, "removing the feed removed its entries");
}

#[tokio::test(flavor = "multi_thread")]
async fn entries_flags_reach_the_api() {
    let h = harness();
    let server = h.feedd.base_url();
    feedctl(&server, &["add", &h.origin.url("rss-basic.rss")]);
    feedctl(&server, &["refresh"]);

    // --since / --until, exercising the half-open window through the CLI.
    let out = feedctl(
        &server,
        &[
            "--format",
            "json",
            "entries",
            "--since",
            "2003-06-11T00:00:00Z",
            "--until",
            "2003-06-12T00:00:00Z",
        ],
    );
    let page: Value = serde_json::from_str(stdout(&out).trim()).unwrap();
    assert_eq!(page["total"], 1);
    assert_eq!(page["items"][0]["title"], "Second post");

    // --feed and --offset.
    let out = feedctl(
        &server,
        &[
            "--format", "json", "entries", "--feed", "1", "--offset", "1",
        ],
    );
    let page: Value = serde_json::from_str(stdout(&out).trim()).unwrap();
    assert_eq!(page["total"], 3, "total ignores paging");
    assert_eq!(page["items"].as_array().unwrap().len(), 2);
}
