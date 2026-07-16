//! The REST API contract, over real local HTTP.

use std::sync::Arc;

use feedd::store::Store;
use reqwest::StatusCode;
use serde_json::{json, Value};

fn server() -> feedd::Spawned {
    let store = Arc::new(Store::open_in_memory().expect("store"));
    feedd::spawn(store).expect("spawn feedd")
}

fn client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .expect("client")
}

/// Register a feed and return its id. The URL never gets fetched here.
async fn add(server: &feedd::Spawned, url: &str) -> Value {
    let res = client()
        .post(server.url("/api/feeds"))
        .json(&json!({ "url": url }))
        .send()
        .await
        .expect("add feed");
    assert_eq!(res.status(), StatusCode::CREATED);
    res.json().await.expect("feed object")
}

// ------------------------------------------------------------------ health

#[tokio::test]
async fn health_reports_ok() {
    let s = server();
    let res = client().get(s.url("/api/health")).send().await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body: Value = res.json().await.unwrap();
    assert_eq!(body["status"], "ok");
}

#[tokio::test]
async fn an_unknown_endpoint_is_404_in_the_pinned_error_shape() {
    let s = server();
    let res = client().get(s.url("/api/nope")).send().await.unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
    let body: Value = res.json().await.unwrap();
    assert!(body["error"].is_string(), "errors are {{\"error\": ...}}");
}

// ------------------------------------------------------------------- feeds

#[tokio::test]
async fn adding_a_feed_returns_201_and_the_feed_object() {
    let s = server();
    let feed = add(&s, "https://example.com/feed.rss").await;

    assert!(feed["id"].is_i64());
    assert_eq!(feed["url"], "https://example.com/feed.rss");
    assert!(
        feed["title"].is_null(),
        "title is null until the first fetch"
    );
    assert!(feed["last_fetched_at"].is_null());
    assert!(feed["last_error"].is_null());
    assert_eq!(feed["entry_count"], 0);
}

#[tokio::test]
async fn adding_the_same_url_twice_is_409() {
    let s = server();
    add(&s, "https://example.com/feed.rss").await;

    let res = client()
        .post(s.url("/api/feeds"))
        .json(&json!({ "url": "https://example.com/feed.rss" }))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::CONFLICT);
    let body: Value = res.json().await.unwrap();
    assert!(body["error"].is_string());
}

#[tokio::test]
async fn a_non_http_url_is_422() {
    let s = server();
    for bad in [
        "file:///etc/passwd",
        "ftp://example.com/f.rss",
        "not a url",
        "",
    ] {
        let res = client()
            .post(s.url("/api/feeds"))
            .json(&json!({ "url": bad }))
            .send()
            .await
            .unwrap();
        assert_eq!(
            res.status(),
            StatusCode::UNPROCESSABLE_ENTITY,
            "{bad:?} must be rejected"
        );
        let body: Value = res.json().await.unwrap();
        assert!(
            body["error"].is_string(),
            "{bad:?} needs the error envelope"
        );
    }
}

#[tokio::test]
async fn a_malformed_body_is_422_in_the_pinned_error_shape() {
    let s = server();
    for body in [
        json!({}),
        json!({ "uri": "https://example.com" }),
        json!("nope"),
    ] {
        let res = client()
            .post(s.url("/api/feeds"))
            .json(&body)
            .send()
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::UNPROCESSABLE_ENTITY, "{body:?}");
        let parsed: Value = res.json().await.unwrap();
        assert!(
            parsed["error"].is_string(),
            "{body:?} needs the error envelope"
        );
    }
}

#[tokio::test]
async fn listing_feeds_returns_an_array() {
    let s = server();
    let empty: Value = client()
        .get(s.url("/api/feeds"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(empty, json!([]));

    add(&s, "https://example.com/a.rss").await;
    add(&s, "https://example.com/b.rss").await;

    let feeds: Value = client()
        .get(s.url("/api/feeds"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(feeds.as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn getting_one_feed_works_and_a_missing_one_is_404() {
    let s = server();
    let feed = add(&s, "https://example.com/a.rss").await;
    let id = feed["id"].as_i64().unwrap();

    let res = client()
        .get(s.url(&format!("/api/feeds/{id}")))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body: Value = res.json().await.unwrap();
    assert_eq!(body["id"], id);

    let res = client()
        .get(s.url("/api/feeds/99999"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
    let body: Value = res.json().await.unwrap();
    assert!(body["error"].is_string());
}

#[tokio::test]
async fn deleting_a_feed_is_204_and_a_missing_one_is_404() {
    let s = server();
    let feed = add(&s, "https://example.com/a.rss").await;
    let id = feed["id"].as_i64().unwrap();

    let res = client()
        .delete(s.url(&format!("/api/feeds/{id}")))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::NO_CONTENT);

    let res = client()
        .get(s.url(&format!("/api/feeds/{id}")))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND, "the feed is gone");

    let res = client()
        .delete(s.url(&format!("/api/feeds/{id}")))
        .send()
        .await
        .unwrap();
    assert_eq!(
        res.status(),
        StatusCode::NOT_FOUND,
        "deleting it again is 404"
    );
}

#[tokio::test]
async fn refreshing_a_missing_feed_is_404() {
    let s = server();
    let res = client()
        .post(s.url("/api/feeds/99999/refresh"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn refreshing_all_with_no_feeds_is_an_empty_array() {
    let s = server();
    let res = client().post(s.url("/api/refresh")).send().await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body: Value = res.json().await.unwrap();
    assert_eq!(body, json!([]));
}

// ----------------------------------------------------------------- entries

#[tokio::test]
async fn entries_returns_the_total_items_envelope() {
    let s = server();
    let res = client().get(s.url("/api/entries")).send().await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body: Value = res.json().await.unwrap();
    assert_eq!(body["total"], 0);
    assert_eq!(body["items"], json!([]));
}

#[tokio::test]
async fn bad_entry_parameters_are_422_in_the_pinned_error_shape() {
    let s = server();
    for query in [
        "limit=abc",
        "offset=nope",
        "feed_id=many",
        "since=yesterday",
        "until=2020-13-45",
        "limit=-1",
        "offset=-1",
    ] {
        let res = client()
            .get(s.url(&format!("/api/entries?{query}")))
            .send()
            .await
            .unwrap();
        assert_eq!(
            res.status(),
            StatusCode::UNPROCESSABLE_ENTITY,
            "?{query} should be unprocessable"
        );
        let body: Value = res.json().await.unwrap();
        assert!(
            body["error"].is_string(),
            "?{query} needs the error envelope"
        );
    }
}

#[tokio::test]
async fn valid_entry_parameters_are_accepted() {
    let s = server();
    for query in [
        "limit=10&offset=5",
        "limit=100000",
        "since=2020-01-01T00:00:00Z",
        "until=2020-01-01T00:00:00%2B05:30",
        "since=2020-01-01T00:00:00.500Z",
        "q=anything",
        "feed_id=1",
    ] {
        let res = client()
            .get(s.url(&format!("/api/entries?{query}")))
            .send()
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::OK, "?{query} should be accepted");
    }
}
