//! End-to-end: a real feedd fetching a real feedgen over local HTTP.
//!
//! Both servers bind `127.0.0.1:0` and the test learns each port before the
//! task spawns, so nothing here polls or sleeps for readiness. The only URLs
//! fetched are feedgen's.

use std::sync::Arc;

use feedd::store::Store;
use feedgen::fixtures;
use reqwest::StatusCode;
use serde_json::{json, Value};

struct Harness {
    _dir: tempfile::TempDir,
    origin: feedgen::Spawned,
    feedd: feedd::Spawned,
}

/// feedd + feedgen, with the corpus written to a tempdir and served from disk.
///
/// Going through `make-fixtures` -> `serve --dir` on real files exercises both
/// halves of feedgen the way an operator uses them, rather than serving a
/// compiled-in corpus that `serve --dir` would never touch.
fn harness(conditional: bool) -> Harness {
    let dir = tempfile::tempdir().expect("tempdir");
    fixtures::write_corpus(dir.path()).expect("write corpus");
    let origin = feedgen::spawn(feedgen::Options::new(dir.path()).conditional(conditional))
        .expect("spawn feedgen");
    let store = Arc::new(Store::open_in_memory().expect("store"));
    let feedd = feedd::spawn(store).expect("spawn feedd");
    Harness {
        _dir: dir,
        origin,
        feedd,
    }
}

fn client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .expect("client")
}

impl Harness {
    /// Register a feedgen-served fixture with feedd. Returns the feed id.
    async fn add_fixture(&self, name: &str) -> i64 {
        self.add_url(&self.origin.url(name)).await
    }

    async fn add_url(&self, url: &str) -> i64 {
        let res = client()
            .post(self.feedd.url("/api/feeds"))
            .json(&json!({ "url": url }))
            .send()
            .await
            .expect("add feed");
        assert_eq!(res.status(), StatusCode::CREATED, "adding {url}");
        let feed: Value = res.json().await.unwrap();
        feed["id"].as_i64().unwrap()
    }

    async fn refresh(&self, id: i64) -> Value {
        client()
            .post(self.feedd.url(&format!("/api/feeds/{id}/refresh")))
            .send()
            .await
            .expect("refresh")
            .json()
            .await
            .expect("refresh result")
    }

    async fn refresh_all(&self) -> Vec<Value> {
        let body: Value = client()
            .post(self.feedd.url("/api/refresh"))
            .send()
            .await
            .expect("refresh all")
            .json()
            .await
            .expect("results");
        body.as_array().cloned().expect("an array of results")
    }

    async fn feed(&self, id: i64) -> Value {
        client()
            .get(self.feedd.url(&format!("/api/feeds/{id}")))
            .send()
            .await
            .expect("get feed")
            .json()
            .await
            .expect("feed object")
    }

    async fn entries(&self, query: &str) -> Value {
        client()
            .get(self.feedd.url(&format!("/api/entries?{query}")))
            .send()
            .await
            .expect("entries")
            .json()
            .await
            .expect("entry page")
    }
}

// -------------------------------------------------------- the happy path

#[tokio::test]
async fn add_refresh_and_read_back_an_rss_feed() {
    let h = harness(true);
    let id = h.add_fixture("rss-basic.rss").await;

    let result = h.refresh(id).await;
    assert_eq!(result["status"], "ok");
    assert_eq!(result["new_entries"], 3);
    assert_eq!(result["feed_id"], id);

    let feed = h.feed(id).await;
    assert_eq!(
        feed["title"], "Example RSS Feed",
        "title arrives on first fetch"
    );
    assert!(feed["last_error"].is_null());
    assert!(feed["last_fetched_at"].is_string());
    assert_eq!(feed["entry_count"], 3);

    let page = h.entries(&format!("feed_id={id}")).await;
    assert_eq!(page["total"], 3);

    // Ordering: newest first.
    let titles: Vec<&str> = page["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| e["title"].as_str().unwrap())
        .collect();
    assert_eq!(titles, ["Third post, no guid", "Second post", "First post"]);

    // The third item has no guid, so its identity fell back to its link.
    let third = &page["items"][0];
    assert_eq!(third["guid"], third["link"]);
    // Dates normalize to UTC with Z regardless of the source offset.
    assert_eq!(page["items"][1]["published_at"], "2003-06-11T14:30:00Z");
    assert_eq!(page["items"][2]["published_at"], "2003-06-10T04:00:00Z");
}

#[tokio::test]
async fn add_refresh_and_read_back_an_atom_feed() {
    let h = harness(true);
    let id = h.add_fixture("atom-basic.atom").await;

    let result = h.refresh(id).await;
    assert_eq!(result["status"], "ok");
    assert_eq!(result["new_entries"], 2);

    let feed = h.feed(id).await;
    assert_eq!(feed["title"], "Example Atom Feed");

    let page = h.entries(&format!("feed_id={id}")).await;
    let items = page["items"].as_array().unwrap();

    // rel=alternate wins over the earlier rel=self.
    let alt = items
        .iter()
        .find(|e| e["guid"] == "urn:uuid:1225c695-cfb8-4ebb-aaaa-80da344efa6a")
        .expect("the alternate-link entry");
    assert_eq!(alt["link"], "https://example.com/entries/1");
    assert_eq!(
        alt["published_at"], "2003-12-13T12:29:29Z",
        "published, as UTC"
    );

    // No summary element, so content became the summary; no published, so
    // updated supplied the date.
    let fallback = items
        .iter()
        .find(|e| e["guid"] == "urn:uuid:2225c695-cfb8-4ebb-aaaa-80da344efa6b")
        .expect("the fallback entry");
    assert_eq!(
        fallback["summary"],
        "No summary element, so content becomes the summary."
    );
    assert_eq!(fallback["published_at"], "2006-01-02T15:04:05Z");
}

// ------------------------------------------------------- conditional GET

#[tokio::test]
async fn a_second_refresh_sends_if_none_match_and_gets_a_304() {
    let h = harness(true);
    let id = h.add_fixture("rss-basic.rss").await;

    let first = h.refresh(id).await;
    assert_eq!(first["new_entries"], 3);
    assert_eq!(first["not_modified"], false);

    let second = h.refresh(id).await;
    assert_eq!(second["status"], "ok", "a 304 is a successful fetch");
    assert_eq!(second["new_entries"], 0, "a 304 adds nothing");
    assert_eq!(second["not_modified"], true);

    // Assert what feedd *sent*. Without this the etag column could be
    // write-only and every other assertion here would still pass.
    let requests = h.origin.log.snapshot();
    assert_eq!(requests.len(), 2);
    assert_eq!(
        requests[0].if_none_match, None,
        "nothing to validate against yet"
    );
    assert!(
        requests[1].if_none_match.is_some(),
        "the refetch must carry If-None-Match; got {:?}",
        requests[1]
    );

    let feed = h.feed(id).await;
    assert!(feed["last_error"].is_null(), "304 clears any prior error");
    assert_eq!(feed["entry_count"], 3, "304 leaves entries untouched");
}

#[tokio::test]
async fn refetching_identical_content_without_a_304_still_adds_nothing() {
    // The path a 304 hides. feedgen is in --no-conditional mode, so feedd gets
    // a full 200 with identical bytes and must dedupe on identity alone.
    //
    // This is the test that catches deriving new_entries from
    // `INSERT .. ON CONFLICT DO UPDATE` + `changes()`: SQLite reports 1 row
    // changed for both branches, so new_entries would report 3 here, forever.
    let h = harness(false);
    let id = h.add_fixture("rss-basic.rss").await;

    let first = h.refresh(id).await;
    assert_eq!(first["new_entries"], 3);

    let second = h.refresh(id).await;
    assert_eq!(second["status"], "ok");
    assert_eq!(second["not_modified"], false, "feedgen answered a full 200");
    assert_eq!(
        second["new_entries"], 0,
        "identical content must insert nothing on a 200"
    );
    assert_eq!(
        second["updated_entries"], 3,
        "the known entries updated in place"
    );
    assert_eq!(
        h.feed(id).await["entry_count"],
        3,
        "still no duplicate rows"
    );
}

#[tokio::test]
async fn an_upstream_edit_updates_the_entry_in_place() {
    let h = harness(true);
    let dir = h._dir.path().to_path_buf();
    let id = h.add_fixture("rss-basic.rss").await;
    h.refresh(id).await;

    let before = h.entries("q=First post").await;
    assert_eq!(before["total"], 1);
    let original_id = before["items"][0]["id"].as_i64().unwrap();
    let original_fetched_at = before["items"][0]["fetched_at"]
        .as_str()
        .unwrap()
        .to_string();

    // Retitle post-0001 upstream, keeping its guid.
    let edited = fixtures::RSS_BASIC.replace(
        "<title>First post</title>",
        "<title>First post, revised</title>",
    );
    std::fs::write(dir.join("rss-basic.rss"), &edited).unwrap();

    let result = h.refresh(id).await;
    assert_eq!(result["new_entries"], 0, "a retitle is not a new entry");
    assert_eq!(h.feed(id).await["entry_count"], 3, "no duplicate row");

    let after = h.entries("q=First post, revised").await;
    assert_eq!(after["total"], 1);
    let entry = &after["items"][0];
    assert_eq!(
        entry["id"].as_i64().unwrap(),
        original_id,
        "keeps its internal id"
    );
    assert_eq!(
        entry["fetched_at"].as_str().unwrap(),
        original_fetched_at,
        "keeps its original fetched_at"
    );
}

// ------------------------------------------------------ failure isolation

#[tokio::test]
async fn a_malformed_feed_records_last_error_and_spares_its_neighbours() {
    let h = harness(true);
    let healthy = h.add_fixture("rss-basic.rss").await;
    let broken = h.add_fixture("malformed.xml").await;

    let results = h.refresh_all().await;
    assert_eq!(results.len(), 2, "one failure must not truncate the array");

    let by_id = |id: i64| {
        results
            .iter()
            .find(|r| r["feed_id"] == id)
            .unwrap_or_else(|| panic!("no result for feed {id}"))
    };
    assert_eq!(by_id(healthy)["status"], "ok");
    assert_eq!(by_id(healthy)["new_entries"], 3);
    assert_eq!(by_id(broken)["status"], "error");
    assert_eq!(by_id(broken)["new_entries"], 0);
    assert!(by_id(broken)["error"].is_string());

    // The broken feed carries its error; the healthy one is untouched.
    let broken_feed = h.feed(broken).await;
    assert!(
        broken_feed["last_error"]
            .as_str()
            .unwrap()
            .contains("malformed"),
        "got {:?}",
        broken_feed["last_error"]
    );
    assert_eq!(broken_feed["entry_count"], 0);

    let healthy_feed = h.feed(healthy).await;
    assert!(healthy_feed["last_error"].is_null());
    assert_eq!(healthy_feed["entry_count"], 3);
    assert_eq!(healthy_feed["title"], "Example RSS Feed");
}

#[tokio::test]
async fn an_unreachable_origin_records_last_error_without_crashing_the_server() {
    let h = harness(true);
    // Port 1, not an ephemeral port obtained by bind-then-drop: the OS is free
    // to hand a released ephemeral port to a concurrent test's server, and then
    // this "dead" origin answers and the test flakes. Ports below 1024 need
    // privileges no test has, so this connection is refused deterministically.
    let id = h.add_url("http://127.0.0.1:1/gone.rss").await;

    let result = h.refresh(id).await;
    assert_eq!(result["status"], "error");
    assert_eq!(result["new_entries"], 0);
    assert!(h.feed(id).await["last_error"].is_string());

    // The server is still answering.
    let health = client()
        .get(h.feedd.url("/api/health"))
        .send()
        .await
        .unwrap();
    assert_eq!(health.status(), StatusCode::OK);
}

#[tokio::test]
async fn a_404_from_the_origin_is_recorded_as_an_error() {
    let h = harness(true);
    let id = h.add_url(&h.origin.url("does-not-exist.rss")).await;
    let result = h.refresh(id).await;
    assert_eq!(result["status"], "error");
    assert!(
        h.feed(id).await["last_error"]
            .as_str()
            .unwrap()
            .contains("404"),
        "the HTTP status belongs in the error"
    );
}

#[tokio::test]
async fn a_feed_recovers_from_an_error_on_the_next_success() {
    let h = harness(true);
    let dir = h._dir.path().to_path_buf();
    let id = h.add_fixture("recovering.rss").await;

    std::fs::write(dir.join("recovering.rss"), fixtures::MALFORMED).unwrap();
    h.refresh(id).await;
    assert!(h.feed(id).await["last_error"].is_string());

    std::fs::write(dir.join("recovering.rss"), fixtures::RSS_BASIC).unwrap();
    let result = h.refresh(id).await;

    assert_eq!(result["status"], "ok");
    let feed = h.feed(id).await;
    assert!(feed["last_error"].is_null(), "a success clears last_error");
    assert_eq!(feed["entry_count"], 3);
}

#[tokio::test]
async fn a_reported_error_always_matches_the_feed_row() {
    // The response and the feed row must never disagree about what happened.
    let h = harness(true);
    let id = h.add_fixture("malformed.xml").await;

    let result = h.refresh(id).await;
    assert_eq!(result["status"], "error");

    let feed = h.feed(id).await;
    assert!(
        feed["last_error"].is_string(),
        "status was 'error', so last_error must say so, not stay null"
    );
    assert!(
        feed["last_fetched_at"].is_string(),
        "a failed attempt is still a completed fetch attempt"
    );
}

// ---------------------------------------------------- pinned date + text

#[tokio::test]
async fn edge_case_dates_land_exactly_where_the_spec_says() {
    let h = harness(true);
    let id = h.add_fixture("dates-edge.rss").await;
    h.refresh(id).await;

    let page = h.entries(&format!("feed_id={id}&limit=500")).await;
    assert_eq!(page["total"], 6, "every item is stored, dated or not");

    let published = |guid: &str| -> Value {
        page["items"]
            .as_array()
            .unwrap()
            .iter()
            .find(|e| e["guid"] == guid)
            .unwrap_or_else(|| panic!("no entry {guid}"))["published_at"]
            .clone()
    };

    assert_eq!(published("date-est"), "2020-01-01T17:00:00Z");
    assert_eq!(published("date-pdt"), "2020-01-01T19:00:00Z");
    assert_eq!(published("date-ut"), "2020-01-01T12:00:00Z");
    assert_eq!(published("date-numeric"), "2020-01-01T17:00:00Z");
    assert!(
        published("date-missing").is_null(),
        "no date -> null, never fetch time"
    );
    assert!(published("date-garbage").is_null(), "unparseable -> null");
}

#[tokio::test]
async fn cdata_and_entities_survive_the_round_trip() {
    let h = harness(true);
    let id = h.add_fixture("cdata-entities.rss").await;
    h.refresh(id).await;

    let page = h.entries(&format!("feed_id={id}&limit=500")).await;
    let items = page["items"].as_array().unwrap();
    let by_guid = |guid: &str| items.iter().find(|e| e["guid"] == guid).unwrap();

    let first = by_guid("cdata-entities-1");
    assert_eq!(
        first["title"], "Tom & Jerry <the sequel>",
        "entities unescape"
    );
    assert_eq!(
        first["summary"], "Verbatim CDATA: &amp; stays literal, <b>markup</b> is not parsed.",
        "CDATA is verbatim"
    );

    assert_eq!(
        by_guid("cdata-entities-2")["title"],
        "A CDATA title with <angle> brackets"
    );
    assert_eq!(by_guid("cdata-entities-3")["title"], "(untitled)");
}

// --------------------------------------------------------- entry queries

#[tokio::test]
async fn entries_span_feeds_and_filter_by_feed_id() {
    let h = harness(true);
    let rss = h.add_fixture("rss-basic.rss").await;
    let atom = h.add_fixture("atom-basic.atom").await;
    h.refresh_all().await;

    assert_eq!(h.entries("").await["total"], 5, "3 RSS + 2 Atom");
    assert_eq!(h.entries(&format!("feed_id={rss}")).await["total"], 3);
    assert_eq!(h.entries(&format!("feed_id={atom}")).await["total"], 2);
}

#[tokio::test]
async fn the_since_until_window_is_half_open() {
    let h = harness(true);
    let id = h.add_fixture("rss-basic.rss").await;
    h.refresh(id).await;

    // The three posts are at 04:00, 14:30, and 12:00 UTC on Jun 10/11/12 2003.
    let first_post = "2003-06-10T04:00:00Z";

    let inclusive = h.entries(&format!("since={first_post}")).await;
    assert_eq!(
        inclusive["total"], 3,
        "since is inclusive of an exact match"
    );

    let exclusive = h.entries(&format!("until={first_post}")).await;
    assert_eq!(
        exclusive["total"], 0,
        "until is exclusive of an exact match"
    );

    let window = h
        .entries("since=2003-06-11T00:00:00Z&until=2003-06-12T00:00:00Z")
        .await;
    assert_eq!(window["total"], 1);
    assert_eq!(window["items"][0]["title"], "Second post");
}

#[tokio::test]
async fn a_bound_expressed_in_another_offset_means_the_same_instant() {
    let h = harness(true);
    let id = h.add_fixture("rss-basic.rss").await;
    h.refresh(id).await;

    let utc = h.entries("since=2003-06-11T00:00:00Z").await;
    // The same instant, written +05:30.
    let offset = h.entries("since=2003-06-11T05:30:00%2B05:30").await;
    assert_eq!(utc["total"], offset["total"]);
    assert_eq!(utc["items"], offset["items"]);
}

#[tokio::test]
async fn undated_entries_drop_out_once_a_bound_is_given() {
    let h = harness(true);
    let id = h.add_fixture("dates-edge.rss").await;
    h.refresh(id).await;

    assert_eq!(h.entries("limit=500").await["total"], 6, "unbounded: all 6");
    let bounded = h.entries("since=1970-01-01T00:00:00Z&limit=500").await;
    assert_eq!(bounded["total"], 4, "the 2 undated entries drop out");
    assert!(bounded["items"]
        .as_array()
        .unwrap()
        .iter()
        .all(|e| !e["published_at"].is_null()));
}

#[tokio::test]
async fn search_is_case_insensitive_over_titles() {
    let h = harness(true);
    let id = h.add_fixture("rss-basic.rss").await;
    h.refresh(id).await;

    for term in ["second post", "SECOND POST", "Second Post"] {
        let page = h.entries(&format!("q={}", urlencode(term))).await;
        assert_eq!(page["total"], 1, "{term:?}");
        assert_eq!(page["items"][0]["title"], "Second post");
    }
}

#[tokio::test]
async fn limit_and_offset_page_while_total_stays_whole() {
    let h = harness(true);
    let id = h.add_fixture("rss-basic.rss").await;
    h.refresh(id).await;

    let page = h.entries("limit=1&offset=1").await;
    assert_eq!(page["total"], 3, "total ignores paging");
    assert_eq!(page["items"].as_array().unwrap().len(), 1);
    assert_eq!(page["items"][0]["title"], "Second post");
}

#[tokio::test]
async fn deleting_a_feed_takes_its_entries_with_it() {
    let h = harness(true);
    let rss = h.add_fixture("rss-basic.rss").await;
    let atom = h.add_fixture("atom-basic.atom").await;
    h.refresh_all().await;
    assert_eq!(h.entries("").await["total"], 5);

    let res = client()
        .delete(h.feedd.url(&format!("/api/feeds/{rss}")))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::NO_CONTENT);

    let remaining = h.entries("").await;
    assert_eq!(remaining["total"], 2, "only the Atom feed's entries remain");
    assert!(remaining["items"]
        .as_array()
        .unwrap()
        .iter()
        .all(|e| e["feed_id"] == atom));
}

// ------------------------------------------------------------ the poller

#[tokio::test]
async fn a_poll_tick_refreshes_every_feed_and_isolates_failures() {
    // The poll loop is the one component that runs unattended, so it gets a
    // deterministic tick rather than a race against a timer.
    let h = harness(true);
    let healthy = h.add_fixture("rss-basic.rss").await;
    let broken = h.add_fixture("malformed.xml").await;

    let fetcher = feedd::fetch::Fetcher::new().unwrap();
    let results = feedd::poll::poll_once(&h.feedd.store, &fetcher).await;

    assert_eq!(results.len(), 2);
    assert_eq!(h.feed(healthy).await["entry_count"], 3);
    assert!(h.feed(broken).await["last_error"].is_string());
    assert!(
        h.feed(healthy).await["last_error"].is_null(),
        "a broken feed must not poison a healthy one"
    );
}

fn urlencode(s: &str) -> String {
    s.bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                (b as char).to_string()
            }
            _ => format!("%{b:02X}"),
        })
        .collect()
}
