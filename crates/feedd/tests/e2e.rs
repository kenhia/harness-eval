//! End-to-end tests: a real `feedd` serving its REST API over local HTTP,
//! fetching real feeds from a real `feedgen` over local HTTP.
//!
//! Both servers bind port 0, so tests never collide on a port, and `feedd` runs
//! with polling disabled so the only fetches are the ones a test asks for.

use feedhub_core::api::{EntriesPage, Entry, Feed, RefreshResult};
use reqwest::{Client, StatusCode};
use serde_json::{Value, json};
use tempfile::TempDir;

struct Harness {
    feedd: feedd::RunningServer,
    feedgen: feedgen::RunningServer,
    /// The served fixture directory; tests edit files in it to simulate a feed
    /// changing upstream.
    fixture_dir: TempDir,
    _db_dir: TempDir,
    client: Client,
}

impl Harness {
    /// A server with background polling off: every fetch is one a test asked
    /// for, so nothing here races the clock.
    async fn start() -> Harness {
        Harness::start_with_poll_interval(0).await
    }

    async fn start_with_poll_interval(poll_interval: u64) -> Harness {
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
            poll_interval,
        })
        .await
        .expect("start feedd");

        Harness {
            feedd,
            feedgen,
            fixture_dir,
            _db_dir: db_dir,
            client: Client::new(),
        }
    }

    /// URL of a fixture as served by feedgen.
    fn fixture_url(&self, name: &str) -> String {
        format!("{}/{name}", self.feedgen.base_url())
    }

    fn api(&self, path: &str) -> String {
        format!("{}{path}", self.feedd.base_url())
    }

    async fn get(&self, path: &str) -> (StatusCode, Value) {
        let response = self.client.get(self.api(path)).send().await.expect("GET");
        let status = response.status();
        (status, response.json().await.unwrap_or(Value::Null))
    }

    async fn post(&self, path: &str, body: Option<Value>) -> (StatusCode, Value) {
        let mut request = self.client.post(self.api(path));
        if let Some(body) = body {
            request = request.json(&body);
        }
        let response = request.send().await.expect("POST");
        let status = response.status();
        (status, response.json().await.unwrap_or(Value::Null))
    }

    async fn delete(&self, path: &str) -> StatusCode {
        self.client
            .delete(self.api(path))
            .send()
            .await
            .expect("DELETE")
            .status()
    }

    /// Register a fixture as a feed and return it.
    async fn add_fixture(&self, name: &str) -> Feed {
        let (status, body) = self
            .post("/api/feeds", Some(json!({"url": self.fixture_url(name)})))
            .await;
        assert_eq!(status, StatusCode::CREATED, "add {name}: {body}");
        serde_json::from_value(body).expect("feed object")
    }

    async fn refresh(&self, id: i64) -> RefreshResult {
        let (status, body) = self.post(&format!("/api/feeds/{id}/refresh"), None).await;
        assert_eq!(status, StatusCode::OK, "refresh {id}: {body}");
        serde_json::from_value(body).expect("refresh result")
    }

    /// Register a fixture and fetch it once.
    async fn add_and_refresh(&self, name: &str) -> (Feed, RefreshResult) {
        let feed = self.add_fixture(name).await;
        let result = self.refresh(feed.id).await;
        (feed, result)
    }

    async fn feed(&self, id: i64) -> Feed {
        let (status, body) = self.get(&format!("/api/feeds/{id}")).await;
        assert_eq!(status, StatusCode::OK, "get feed {id}: {body}");
        serde_json::from_value(body).expect("feed object")
    }

    async fn entries(&self, query: &str) -> EntriesPage {
        let (status, body) = self.get(&format!("/api/entries?{query}")).await;
        assert_eq!(status, StatusCode::OK, "entries?{query}: {body}");
        serde_json::from_value(body).expect("entries page")
    }

    async fn shutdown(self) {
        self.feedd.shutdown().await;
        self.feedgen.shutdown().await;
    }
}

fn guids(page: &EntriesPage) -> Vec<&str> {
    page.items.iter().map(|e| e.guid.as_str()).collect()
}

fn find<'a>(page: &'a EntriesPage, guid: &str) -> &'a Entry {
    page.items
        .iter()
        .find(|e| e.guid == guid)
        .unwrap_or_else(|| panic!("no entry {guid} in {:?}", guids(page)))
}

#[tokio::test]
async fn background_polling_fetches_without_being_asked() {
    // The shortest interval there is, so the test waits a tick, not a minute.
    let h = Harness::start_with_poll_interval(1).await;
    let feed = h.add_fixture("rss-basic.xml").await;

    // Nothing has asked for a fetch, so entries can only come from the poller.
    let mut entries = 0;
    for _ in 0..100 {
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        entries = h.entries("").await.total;
        if entries > 0 {
            break;
        }
    }

    assert_eq!(entries, 3, "the poller should have fetched the feed");
    let feed = h.feed(feed.id).await;
    assert_eq!(feed.title.as_deref(), Some("Basic RSS Feed"));
    assert_eq!(feed.last_error, None);
    h.shutdown().await;
}

#[tokio::test]
async fn a_feeds_title_follows_the_document() {
    let h = Harness::start().await;
    let (feed, _) = h.add_and_refresh("rss-basic.xml").await;
    assert_eq!(
        h.feed(feed.id).await.title.as_deref(),
        Some("Basic RSS Feed")
    );

    let renamed = feedgen::fixtures::RSS_BASIC.replace(
        "<title>Basic RSS Feed</title>",
        "<title>Renamed RSS Feed</title>",
    );
    std::fs::write(h.fixture_dir.path().join("rss-basic.xml"), renamed).expect("rewrite fixture");

    h.refresh(feed.id).await;
    assert_eq!(
        h.feed(feed.id).await.title.as_deref(),
        Some("Renamed RSS Feed")
    );
    h.shutdown().await;
}

#[tokio::test]
async fn health_reports_ok() {
    let h = Harness::start().await;
    let (status, body) = h.get("/api/health").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["status"], "ok");
    h.shutdown().await;
}

#[tokio::test]
async fn feed_registration_lifecycle() {
    let h = Harness::start().await;
    let url = h.fixture_url("rss-basic.xml");

    let (status, body) = h.post("/api/feeds", Some(json!({"url": &url}))).await;
    assert_eq!(status, StatusCode::CREATED);
    let feed: Feed = serde_json::from_value(body).expect("feed object");
    assert_eq!(feed.url, url);
    // Nothing has been fetched yet.
    assert_eq!(feed.title, None);
    assert_eq!(feed.last_fetched_at, None);
    assert_eq!(feed.last_error, None);
    assert_eq!(feed.entry_count, 0);

    // The same URL again is a conflict, not a second feed.
    let (status, body) = h.post("/api/feeds", Some(json!({"url": &url}))).await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert!(body["error"].is_string());

    // Anything that is not an http(s) URL is unprocessable.
    for bad in [
        "not-a-url",
        "ftp://example.invalid/feed.xml",
        "/relative.xml",
    ] {
        let (status, body) = h.post("/api/feeds", Some(json!({"url": bad}))).await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "url {bad}");
        assert!(body["error"].is_string(), "url {bad}");
    }

    // A body without a url is also unprocessable.
    let (status, _) = h.post("/api/feeds", Some(json!({"nope": 1}))).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);

    let (status, body) = h.get("/api/feeds").await;
    assert_eq!(status, StatusCode::OK);
    let feeds: Vec<Feed> = serde_json::from_value(body).expect("feed array");
    assert_eq!(feeds.len(), 1);

    let (status, body) = h.get("/api/feeds/9999").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert!(body["error"].is_string());

    assert_eq!(h.delete("/api/feeds/9999").await, StatusCode::NOT_FOUND);
    h.shutdown().await;
}

#[tokio::test]
async fn deleting_a_feed_deletes_its_entries() {
    let h = Harness::start().await;
    let (feed, result) = h.add_and_refresh("rss-basic.xml").await;
    assert_eq!(result.new_entries, 3);
    assert_eq!(h.entries("").await.total, 3);

    assert_eq!(
        h.delete(&format!("/api/feeds/{}", feed.id)).await,
        StatusCode::NO_CONTENT
    );

    assert_eq!(h.entries("").await.total, 0);
    let (status, _) = h.get(&format!("/api/feeds/{}", feed.id)).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    h.shutdown().await;
}

#[tokio::test]
async fn rss_refresh_stores_mapped_entries() {
    let h = Harness::start().await;
    let (feed, result) = h.add_and_refresh("rss-basic.xml").await;

    assert!(result.is_ok(), "{result:?}");
    assert_eq!(result.new_entries, 3);
    assert_eq!(result.updated_entries, 0);
    assert!(!result.not_modified);

    // The feed picked up its title and a successful-fetch timestamp.
    let feed = h.feed(feed.id).await;
    assert_eq!(feed.title.as_deref(), Some("Basic RSS Feed"));
    assert_eq!(feed.last_error, None);
    assert!(feed.last_fetched_at.is_some());
    assert_eq!(feed.entry_count, 3);

    let page = h.entries("").await;
    let entry = find(&page, "rss-1");
    assert_eq!(entry.title, "Rust release notes");
    assert_eq!(entry.link.as_deref(), Some("http://feedgen.invalid/rss/1"));
    assert_eq!(
        entry.summary.as_deref(),
        Some("What shipped in the latest release.")
    );
    assert_eq!(entry.published_at.as_deref(), Some("2024-03-04T09:00:00Z"));
    assert!(entry.fetched_at.ends_with('Z'));

    // A -0500 pubDate is stored as the equivalent UTC instant.
    assert_eq!(
        find(&page, "rss-2").published_at.as_deref(),
        Some("2024-03-03T17:00:00Z")
    );
    h.shutdown().await;
}

#[tokio::test]
async fn atom_refresh_stores_mapped_entries() {
    let h = Harness::start().await;
    let (feed, result) = h.add_and_refresh("atom-basic.xml").await;
    assert_eq!(result.new_entries, 2);

    let feed = h.feed(feed.id).await;
    assert_eq!(feed.title.as_deref(), Some("Basic Atom Feed"));

    let page = h.entries("").await;
    let first = find(&page, "atom-1");
    // rel="alternate" wins over the rel="edit" link that comes first.
    assert_eq!(first.link.as_deref(), Some("http://feedgen.invalid/atom/1"));
    // summary wins over content, and published wins over updated.
    assert_eq!(
        first.summary.as_deref(),
        Some("The summary is preferred over the content.")
    );
    assert_eq!(first.published_at.as_deref(), Some("2024-03-05T08:15:00Z"));

    let second = find(&page, "atom-2");
    assert_eq!(
        second.link.as_deref(),
        Some("http://feedgen.invalid/atom/2")
    );
    assert_eq!(
        second.summary.as_deref(),
        Some("No summary here, so the content is the summary.")
    );
    assert_eq!(second.published_at.as_deref(), Some("2024-03-01T04:00:00Z"));
    h.shutdown().await;
}

#[tokio::test]
async fn conditional_get_reports_not_modified_and_touches_nothing() {
    let h = Harness::start().await;
    let (feed, _) = h.add_and_refresh("rss-basic.xml").await;
    let before = h.entries("").await;

    // Nothing changed upstream, so feedd's If-None-Match must earn a 304.
    let result = h.refresh(feed.id).await;
    assert!(result.is_ok(), "{result:?}");
    assert!(result.not_modified, "expected a 304: {result:?}");
    assert_eq!(result.new_entries, 0);
    assert_eq!(result.updated_entries, 0);

    // A successful fetch, so still no error recorded.
    let feed = h.feed(feed.id).await;
    assert_eq!(feed.last_error, None);
    assert_eq!(feed.entry_count, 3);
    assert_eq!(h.entries("").await, before);
    h.shutdown().await;
}

#[tokio::test]
async fn known_entries_update_in_place() {
    let h = Harness::start().await;
    let (feed, _) = h.add_and_refresh("rss-basic.xml").await;

    let before = h.entries("").await;
    let original = find(&before, "rss-1").clone();

    // Same guid, new title/summary/date — plus one entirely new item.
    let edited = feedgen::fixtures::RSS_BASIC
        .replace(
            "<title>Rust release notes</title>",
            "<title>Rust release notes, revised</title>",
        )
        .replace(
            "<description>What shipped in the latest release.</description>",
            "<description>Now with corrections.</description>",
        )
        .replace(
            "<pubDate>Mon, 04 Mar 2024 09:00:00 +0000</pubDate>",
            "<pubDate>Tue, 05 Mar 2024 09:00:00 +0000</pubDate>",
        )
        .replace(
            "  </channel>",
            "    <item>
      <title>A brand new post</title>
      <link>http://feedgen.invalid/rss/4</link>
      <description>Fresh.</description>
      <guid isPermaLink=\"false\">rss-4</guid>
      <pubDate>Wed, 06 Mar 2024 09:00:00 +0000</pubDate>
    </item>
  </channel>",
        );
    std::fs::write(h.fixture_dir.path().join("rss-basic.xml"), edited).expect("rewrite fixture");

    let result = h.refresh(feed.id).await;
    assert!(result.is_ok(), "{result:?}");
    assert_eq!(result.new_entries, 1, "only rss-4 is new: {result:?}");
    assert_eq!(result.updated_entries, 3);

    // Four rows, not seven: the three known guids were updated, not duplicated.
    let after = h.entries("").await;
    assert_eq!(after.total, 4);
    assert_eq!(h.feed(feed.id).await.entry_count, 4);

    let updated = find(&after, "rss-1");
    assert_eq!(updated.title, "Rust release notes, revised");
    assert_eq!(updated.summary.as_deref(), Some("Now with corrections."));
    assert_eq!(
        updated.published_at.as_deref(),
        Some("2024-03-05T09:00:00Z")
    );
    // The identity of the row survives the update.
    assert_eq!(updated.id, original.id);
    assert_eq!(updated.fetched_at, original.fetched_at);
    h.shutdown().await;
}

#[tokio::test]
async fn rss_date_edge_cases_map_to_utc_and_null() {
    let h = Harness::start().await;
    h.add_and_refresh("dates-edge.xml").await;
    let page = h.entries("limit=500").await;
    assert_eq!(page.total, 15);

    let published = |guid: &str| find(&page, guid).published_at.clone();
    // Each item is noon on 2024-03-01 in its own zone.
    for (guid, expected) in [
        ("date-gmt", "2024-03-01T12:00:00Z"),
        ("date-ut", "2024-03-01T12:00:00Z"),
        ("date-z", "2024-03-01T12:00:00Z"),
        ("date-est", "2024-03-01T17:00:00Z"),
        ("date-edt", "2024-03-01T16:00:00Z"),
        ("date-cst", "2024-03-01T18:00:00Z"),
        ("date-cdt", "2024-03-01T17:00:00Z"),
        ("date-mst", "2024-03-01T19:00:00Z"),
        ("date-mdt", "2024-03-01T18:00:00Z"),
        ("date-pst", "2024-03-01T20:00:00Z"),
        ("date-pdt", "2024-03-01T19:00:00Z"),
        ("date-plus", "2024-03-01T06:30:00Z"),
        ("date-minus", "2024-03-01T20:00:00Z"),
    ] {
        assert_eq!(published(guid).as_deref(), Some(expected), "{guid}");
    }

    // A missing or unparseable date is null — never the fetch time.
    assert_eq!(published("date-missing"), None);
    assert_eq!(published("date-junk"), None);
    // ...and the entries are still stored.
    assert_eq!(find(&page, "date-junk").title, "Unparseable date");
    h.shutdown().await;
}

#[tokio::test]
async fn atom_date_edge_cases() {
    let h = Harness::start().await;
    h.add_and_refresh("atom-dates-edge.xml").await;
    let page = h.entries("limit=500").await;

    // Fractional seconds are truncated to the stored second precision.
    assert_eq!(
        find(&page, "atom-frac").published_at.as_deref(),
        Some("2024-03-01T12:00:00Z")
    );
    assert_eq!(
        find(&page, "atom-offset").published_at.as_deref(),
        Some("2024-03-01T06:30:00Z")
    );
    // No published element, so updated is used.
    assert_eq!(
        find(&page, "atom-updated-only").published_at.as_deref(),
        Some("2024-03-02T16:00:00Z")
    );
    assert_eq!(find(&page, "atom-junk").published_at, None);
    h.shutdown().await;
}

#[tokio::test]
async fn text_handling_matches_the_spec() {
    let h = Harness::start().await;
    h.add_and_refresh("cdata-entities.xml").await;
    let page = h.entries("limit=500").await;

    // CDATA is stored exactly as written: markup and `&amp;` are not touched.
    let cdata = find(&page, "cdata-1");
    assert_eq!(cdata.title, "Raw <b>markup</b> & ampersands");
    assert_eq!(
        cdata.summary.as_deref(),
        Some("<p>CDATA is stored verbatim, including &amp; as written.</p>")
    );

    // Entity references are unescaped on the way in.
    let entity = find(&page, "entity-1");
    assert_eq!(entity.title, "Fish & Chips <battered>");
    assert_eq!(
        entity.summary.as_deref(),
        Some("Café & crème, unescaped on the way in.")
    );

    assert_eq!(find(&page, "untitled-1").title, "(untitled)");
    h.shutdown().await;
}

#[tokio::test]
async fn a_broken_feed_does_not_disturb_the_others() {
    let h = Harness::start().await;
    let (good, good_result) = h.add_and_refresh("rss-basic.xml").await;
    assert!(good_result.is_ok());

    // Malformed XML, an HTTP error, and a refused connection.
    let (malformed, _) = h.add_and_refresh("malformed.xml").await;
    let (missing, _) = h.add_and_refresh("no-such-feed.xml").await;
    let unreachable: Feed = {
        let (status, body) = h
            .post(
                "/api/feeds",
                // Port 1 on loopback: nothing is listening, and nothing will be.
                Some(json!({"url": "http://127.0.0.1:1/feed.xml"})),
            )
            .await;
        assert_eq!(status, StatusCode::CREATED);
        serde_json::from_value(body).expect("feed object")
    };

    for id in [malformed.id, missing.id, unreachable.id] {
        let result = h.refresh(id).await;
        assert_eq!(result.status, "error", "feed {id}: {result:?}");
        assert_eq!(result.new_entries, 0);
        assert!(result.error.is_some(), "feed {id} needs a message");

        let feed = h.feed(id).await;
        let error = feed.last_error.expect("last_error recorded");
        assert!(!error.is_empty());
        // A failed fetch stores nothing.
        assert_eq!(feed.entry_count, 0);
    }

    // The healthy feed is untouched, and the server is still up.
    let good = h.feed(good.id).await;
    assert_eq!(good.last_error, None);
    assert_eq!(good.entry_count, 3);
    assert_eq!(h.get("/api/health").await.0, StatusCode::OK);

    // A feed that starts working again clears its error.
    std::fs::write(
        h.fixture_dir.path().join("malformed.xml"),
        feedgen::fixtures::RSS_BASIC,
    )
    .expect("repair fixture");
    let result = h.refresh(malformed.id).await;
    assert!(result.is_ok(), "{result:?}");
    assert_eq!(h.feed(malformed.id).await.last_error, None);
    h.shutdown().await;
}

#[tokio::test]
async fn an_oversized_feed_is_refused_rather_than_buffered() {
    let h = Harness::start().await;

    // One item padded past the limit: feedd must reject it on size, not try to
    // hold it all in memory.
    let padding = "x".repeat(feedd::fetch::MAX_FEED_BYTES + 1);
    let huge = format!(
        "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n\
         <rss version=\"2.0\"><channel><title>Huge</title>\
         <item><guid>huge-1</guid><title>Huge</title>\
         <description>{padding}</description></item></channel></rss>"
    );
    std::fs::write(h.fixture_dir.path().join("huge.xml"), huge).expect("write huge fixture");

    let (feed, result) = h.add_and_refresh("huge.xml").await;
    assert_eq!(result.status, "error", "{result:?}");
    assert!(
        result
            .error
            .as_deref()
            .unwrap_or_default()
            .contains("too large"),
        "{result:?}"
    );
    assert_eq!(h.feed(feed.id).await.entry_count, 0);
    // The server is still serving.
    assert_eq!(h.get("/api/health").await.0, StatusCode::OK);
    h.shutdown().await;
}

#[tokio::test]
async fn refresh_all_reports_every_feed() {
    let h = Harness::start().await;
    let rss = h.add_fixture("rss-basic.xml").await;
    let atom = h.add_fixture("atom-basic.xml").await;
    let broken = h.add_fixture("malformed.xml").await;

    let (status, body) = h.post("/api/refresh", None).await;
    assert_eq!(status, StatusCode::OK);
    let results: Vec<RefreshResult> = serde_json::from_value(body).expect("result array");
    assert_eq!(results.len(), 3);

    let by_id = |id: i64| -> &RefreshResult {
        results
            .iter()
            .find(|r| r.feed_id == id)
            .expect("result carries its feed id")
    };
    assert_eq!(by_id(rss.id).new_entries, 3);
    assert_eq!(by_id(atom.id).new_entries, 2);
    assert_eq!(by_id(broken.id).status, "error");

    assert_eq!(h.entries("").await.total, 5);
    h.shutdown().await;
}

#[tokio::test]
async fn entries_are_ordered_newest_first_nulls_last_ties_by_id() {
    let h = Harness::start().await;
    h.add_and_refresh("dates-edge.xml").await;

    let page = h.entries("limit=500").await;
    // Descending by published_at; the three zone spellings of noon UTC and the
    // two 20:00Z entries tie, and are broken by insertion order.
    assert_eq!(
        guids(&page),
        vec![
            "date-pst",     // 20:00Z, inserted first of the two
            "date-minus",   // 20:00Z
            "date-mst",     // 19:00Z
            "date-pdt",     // 19:00Z
            "date-cst",     // 18:00Z
            "date-mdt",     // 18:00Z
            "date-est",     // 17:00Z
            "date-cdt",     // 17:00Z
            "date-edt",     // 16:00Z
            "date-gmt",     // 12:00Z
            "date-ut",      // 12:00Z
            "date-z",       // 12:00Z
            "date-plus",    // 06:30Z
            "date-missing", // null dates sort last
            "date-junk",
        ]
    );
    h.shutdown().await;
}

#[tokio::test]
async fn entries_window_is_half_open_and_offset_aware() {
    let h = Harness::start().await;
    h.add_and_refresh("dates-edge.xml").await;

    // since <= published_at < until: 12:00Z is in, 17:00Z is out.
    let page = h
        .entries("since=2024-03-01T12:00:00Z&until=2024-03-01T17:00:00Z&limit=500")
        .await;
    assert_eq!(
        guids(&page),
        vec!["date-edt", "date-gmt", "date-ut", "date-z"]
    );
    assert_eq!(page.total, 4);

    // The same instants written with a different offset select the same window.
    let same = h
        .entries("since=2024-03-01T13:00:00%2B01:00&until=2024-03-01T12:00:00-05:00&limit=500")
        .await;
    assert_eq!(same, page);

    // Either bound alone excludes entries with no date.
    let since_only = h.entries("since=2024-03-01T00:00:00Z&limit=500").await;
    assert_eq!(since_only.total, 13);
    assert!(!guids(&since_only).contains(&"date-missing"));
    let until_only = h.entries("until=2024-03-02T00:00:00Z&limit=500").await;
    assert_eq!(until_only.total, 13);
    assert!(!guids(&until_only).contains(&"date-junk"));

    // An empty window is empty, not an error.
    let empty = h
        .entries("since=2024-03-01T12:00:00Z&until=2024-03-01T12:00:00Z")
        .await;
    assert_eq!(empty.total, 0);
    assert!(empty.items.is_empty());
    h.shutdown().await;
}

#[tokio::test]
async fn sub_second_bounds_round_in_the_direction_that_keeps_the_window_honest() {
    let h = Harness::start().await;
    h.add_and_refresh("dates-edge.xml").await;
    // The three noon-UTC entries are the ones a bound just past noon can slice.
    let noon = ["date-gmt", "date-ut", "date-z"];

    // An entry at 12:00:00 is before 12:00:00.5, so this window must keep it.
    let page = h
        .entries("since=2024-03-01T12:00:00Z&until=2024-03-01T12:00:00.500Z&limit=500")
        .await;
    assert_eq!(guids(&page), noon);

    // An entry at 12:00:00 is not at or after 12:00:00.5, so this must drop it.
    let page = h
        .entries("since=2024-03-01T12:00:00.500Z&until=2024-03-01T13:00:00Z&limit=500")
        .await;
    assert_eq!(page.total, 0);
    h.shutdown().await;
}

#[tokio::test]
async fn entries_filter_by_feed_and_search_title() {
    let h = Harness::start().await;
    let (rss, _) = h.add_and_refresh("rss-basic.xml").await;
    let (atom, _) = h.add_and_refresh("atom-basic.xml").await;

    let page = h.entries(&format!("feed_id={}", rss.id)).await;
    assert_eq!(page.total, 3);
    assert!(page.items.iter().all(|e| e.feed_id == rss.id));
    assert_eq!(h.entries(&format!("feed_id={}", atom.id)).await.total, 2);
    assert_eq!(h.entries("feed_id=9999").await.total, 0);

    // Case-insensitive substring of the title, in either direction.
    assert_eq!(h.entries("q=notes").await.total, 3);
    assert_eq!(h.entries("q=NOTES").await.total, 3);
    assert_eq!(h.entries("q=rUsT").await.total, 1);
    // Matches the title only, not the summary.
    assert_eq!(h.entries("q=fsync").await.total, 0);
    assert_eq!(h.entries("q=nothing-matches-this").await.total, 0);
    // Wildcards are matched literally, not interpreted.
    assert_eq!(h.entries("q=%25").await.total, 0);

    // Filters compose: only the Atom feed has an entry titled "entry".
    let page = h.entries(&format!("feed_id={}&q=entry", atom.id)).await;
    assert_eq!(page.total, 2);
    let page = h.entries(&format!("feed_id={}&q=entry", rss.id)).await;
    assert_eq!(page.total, 0);
    h.shutdown().await;
}

#[tokio::test]
async fn entries_paginate_with_total_ignoring_the_page() {
    let h = Harness::start().await;
    h.add_and_refresh("dates-edge.xml").await;
    let all = h.entries("limit=500").await;

    let page = h.entries("limit=2").await;
    assert_eq!(page.total, 15, "total counts matches, not the page");
    assert_eq!(guids(&page), guids(&all)[..2]);

    let page = h.entries("limit=2&offset=2").await;
    assert_eq!(page.total, 15);
    assert_eq!(guids(&page), guids(&all)[2..4]);

    // Past the end is empty, not an error.
    let page = h.entries("offset=100").await;
    assert_eq!(page.total, 15);
    assert!(page.items.is_empty());

    // limit is capped at 500 rather than rejected.
    assert_eq!(h.entries("limit=100000").await.items.len(), 15);
    h.shutdown().await;
}

#[tokio::test]
async fn entries_defaults_to_fifty() {
    let h = Harness::start().await;
    let (feed, _) = h.add_and_refresh("rss-basic.xml").await;

    // 60 entries in one feed, so the default limit has something to cut.
    let mut items = String::new();
    for i in 0..60 {
        items.push_str(&format!(
            "<item><guid>bulk-{i}</guid><title>Bulk {i}</title>\
             <pubDate>Fri, 01 Mar 2024 12:00:00 GMT</pubDate></item>\n"
        ));
    }
    let bulk = format!(
        "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n\
         <rss version=\"2.0\"><channel><title>Bulk</title>\n{items}</channel></rss>"
    );
    std::fs::write(h.fixture_dir.path().join("rss-basic.xml"), bulk).expect("write bulk fixture");
    h.refresh(feed.id).await;

    let page = h.entries("").await;
    assert_eq!(page.total, 63);
    assert_eq!(page.items.len(), 50);
    h.shutdown().await;
}

#[tokio::test]
async fn bad_query_parameters_are_rejected() {
    let h = Harness::start().await;

    for query in [
        "feed_id=abc",
        "limit=abc",
        "offset=abc",
        "limit=-1",
        "offset=-1",
        "since=yesterday",
        "until=2024-03-01",
    ] {
        let (status, body) = h.get(&format!("/api/entries?{query}")).await;
        assert_eq!(
            status,
            StatusCode::UNPROCESSABLE_ENTITY,
            "expected 422 for {query}"
        );
        assert!(body["error"].is_string(), "{query} needs an error message");
    }

    // Unknown routes are errors in the same shape.
    let (status, body) = h.get("/api/nonsense").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert!(body["error"].is_string());
    h.shutdown().await;
}
