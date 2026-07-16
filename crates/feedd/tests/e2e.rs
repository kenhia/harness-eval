//! End-to-end test: drive `feedd`'s REST API against `feedgen` serving the
//! fixture corpus over local HTTP. No real network is touched.

use std::net::SocketAddr;

use feedd::state::AppState;
use serde_json::Value;
use tokio::net::TcpListener;

/// Bring up feedgen (serving the corpus) and feedd (REST API) on ephemeral
/// local ports. Returns their base URLs and a tempdir kept alive for the DB.
async fn setup() -> (String, String, tempfile::TempDir, feedgen::Server) {
    let tmp = tempfile::tempdir().unwrap();
    // Generate and serve the fixture corpus.
    let corpus = tmp.path().join("corpus");
    feedgen::fixtures::make_fixtures(&corpus).unwrap();
    let gen = feedgen::spawn(corpus, "127.0.0.1:0").await.unwrap();
    let gen_url = gen.base_url();

    // Start feedd against a fresh SQLite file.
    let db_path = tmp.path().join("feedd.sqlite");
    let state = AppState::new(feedd::db::open(db_path.to_str().unwrap()).unwrap()).unwrap();
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr: SocketAddr = listener.local_addr().unwrap();
    let app = feedd::api::router(state);
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    let feedd_url = format!("http://{addr}");
    (feedd_url, gen_url, tmp, gen)
}

fn client() -> reqwest::Client {
    reqwest::Client::new()
}

#[tokio::test]
async fn end_to_end_flow() {
    let (feedd, gen, _tmp, _server) = setup().await;
    let http = client();

    // Health.
    let resp = http.get(format!("{feedd}/api/health")).send().await.unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["status"], "ok");

    // Invalid URL -> 422.
    let resp = http
        .post(format!("{feedd}/api/feeds"))
        .json(&serde_json::json!({ "url": "ftp://nope" }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 422);

    // Register the RSS fixture.
    let rss_url = format!("{gen}/rss.xml");
    let resp = http
        .post(format!("{feedd}/api/feeds"))
        .json(&serde_json::json!({ "url": rss_url }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 201);
    let feed: Value = resp.json().await.unwrap();
    let rss_id = feed["id"].as_i64().unwrap();
    assert!(feed["title"].is_null(), "title null before first fetch");

    // Duplicate registration -> 409.
    let resp = http
        .post(format!("{feedd}/api/feeds"))
        .json(&serde_json::json!({ "url": rss_url }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 409);

    // Refresh the feed: 2 new entries.
    let resp = http
        .post(format!("{feedd}/api/feeds/{rss_id}/refresh"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let r: Value = resp.json().await.unwrap();
    assert_eq!(r["status"], "ok");
    assert_eq!(r["new_entries"], 2);

    // Title now populated.
    let feed: Value = http
        .get(format!("{feedd}/api/feeds/{rss_id}"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(feed["title"], "Example RSS Feed");
    assert_eq!(feed["entry_count"], 2);
    assert!(feed["last_error"].is_null());

    // Second refresh: conditional GET -> 304 -> 0 new entries, no dupes.
    let r: Value = http
        .post(format!("{feedd}/api/feeds/{rss_id}/refresh"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(r["new_entries"], 0);
    assert_eq!(r["status"], "ok");

    // Add Atom + dates + malformed fixtures, refresh all.
    for name in ["atom.xml", "dates.xml", "malformed.xml"] {
        http.post(format!("{feedd}/api/feeds"))
            .json(&serde_json::json!({ "url": format!("{gen}/{name}") }))
            .send()
            .await
            .unwrap();
    }
    let results: Value = http
        .post(format!("{feedd}/api/refresh"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let arr = results.as_array().unwrap();
    assert_eq!(arr.len(), 4);
    // The malformed feed must report an error but not break the others.
    let statuses: Vec<&str> = arr.iter().map(|r| r["status"].as_str().unwrap()).collect();
    assert!(statuses.contains(&"error"), "malformed feed should error");
    assert!(statuses.contains(&"ok"));

    // The malformed feed records last_error.
    let feeds: Value = http
        .get(format!("{feedd}/api/feeds"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let malformed = feeds
        .as_array()
        .unwrap()
        .iter()
        .find(|f| f["url"].as_str().unwrap().ends_with("malformed.xml"))
        .unwrap();
    assert!(!malformed["last_error"].is_null());

    // Entries: search filter (case-insensitive).
    let entries: Value = http
        .get(format!("{feedd}/api/entries?q=hello"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(entries["total"], 1);
    assert_eq!(entries["items"][0]["title"], "Hello RSS");

    // Entries: window semantics — only Jan 2024 RSS entries, null dates excluded.
    let entries: Value = http
        .get(format!(
            "{feedd}/api/entries?since=2024-01-01T00:00:00Z&until=2024-01-03T00:00:00Z"
        ))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    // rss hello (Jan 2 EST->..), atom entry one (Jan 10 excluded). Just assert
    // all returned items have a non-null published_at inside the window.
    for item in entries["items"].as_array().unwrap() {
        let p = item["published_at"].as_str().unwrap();
        assert!(p.as_bytes() < b"2024-01-03T00:00:00Z".as_ref());
    }

    // Ordering: published_at descending across all entries, nulls last.
    let entries: Value = http
        .get(format!("{feedd}/api/entries?limit=500"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let items = entries["items"].as_array().unwrap();
    let mut seen_null = false;
    let mut prev: Option<String> = None;
    for item in items {
        match item["published_at"].as_str() {
            Some(p) => {
                assert!(!seen_null, "non-null published_at after a null (nulls must be last)");
                if let Some(prev) = &prev {
                    assert!(prev >= &p.to_string(), "not descending: {prev} then {p}");
                }
                prev = Some(p.to_string());
            }
            None => seen_null = true,
        }
    }

    // Delete a feed removes its entries.
    let before = count_entries(&http, &feedd).await;
    let resp = http
        .delete(format!("{feedd}/api/feeds/{rss_id}"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 204);
    let after = count_entries(&http, &feedd).await;
    assert_eq!(after, before - 2, "deleting RSS feed drops its 2 entries");

    // Deleting again -> 404.
    let resp = http
        .delete(format!("{feedd}/api/feeds/{rss_id}"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 404);
}

async fn count_entries(http: &reqwest::Client, feedd: &str) -> i64 {
    let entries: Value = http
        .get(format!("{feedd}/api/entries?limit=500"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    entries["total"].as_i64().unwrap()
}
