//! feedgen's HTTP behavior: content types, validators, and 304 handling.

use std::net::SocketAddr;

use feedgen::fixtures;
use feedgen::serve::{RunningServer, serve_dir};
use reqwest::StatusCode;
use reqwest::header::{ETAG, IF_MODIFIED_SINCE, IF_NONE_MATCH, LAST_MODIFIED};

/// A server over a fresh copy of the corpus, plus the directory it serves.
async fn fixture_server() -> (RunningServer, tempfile::TempDir) {
    let dir = tempfile::tempdir().expect("tempdir");
    fixtures::write_fixtures(dir.path()).expect("write fixtures");
    let addr: SocketAddr = "127.0.0.1:0".parse().unwrap();
    let server = serve_dir(dir.path().to_path_buf(), addr)
        .await
        .expect("serve");
    (server, dir)
}

#[tokio::test]
async fn serves_feeds_with_validators_and_content_types() {
    let (server, _dir) = fixture_server().await;
    let client = reqwest::Client::new();

    let response = client
        .get(format!("{}/rss-basic.xml", server.base_url()))
        .send()
        .await
        .expect("request");
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok()),
        Some("application/rss+xml; charset=utf-8")
    );
    assert!(response.headers().contains_key(ETAG));
    assert!(response.headers().contains_key(LAST_MODIFIED));
    assert!(
        response
            .text()
            .await
            .unwrap()
            .contains("Rust release notes")
    );

    let response = client
        .get(format!("{}/atom-basic.xml", server.base_url()))
        .send()
        .await
        .expect("request");
    assert_eq!(
        response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok()),
        Some("application/atom+xml; charset=utf-8")
    );

    server.shutdown().await;
}

#[tokio::test]
async fn matching_validators_get_304() {
    let (server, _dir) = fixture_server().await;
    let client = reqwest::Client::new();
    let url = format!("{}/rss-basic.xml", server.base_url());

    let first = client.get(&url).send().await.expect("request");
    let etag = first.headers()[ETAG].clone();
    let last_modified = first.headers()[LAST_MODIFIED].clone();

    let response = client
        .get(&url)
        .header(IF_NONE_MATCH, &etag)
        .send()
        .await
        .expect("request");
    assert_eq!(response.status(), StatusCode::NOT_MODIFIED);
    assert!(response.bytes().await.unwrap().is_empty());

    let response = client
        .get(&url)
        .header(IF_MODIFIED_SINCE, &last_modified)
        .send()
        .await
        .expect("request");
    assert_eq!(response.status(), StatusCode::NOT_MODIFIED);

    server.shutdown().await;
}

#[tokio::test]
async fn stale_validators_get_the_new_body() {
    let (server, dir) = fixture_server().await;
    let client = reqwest::Client::new();
    let url = format!("{}/rss-basic.xml", server.base_url());

    let first = client.get(&url).send().await.expect("request");
    let stale_etag = first.headers()[ETAG].clone();

    // Rewriting within the same second leaves the mtime alone, so only the
    // content-derived ETag can catch this.
    let edited = fixtures::RSS_BASIC.replace("Rust release notes", "Rewritten title");
    std::fs::write(dir.path().join("rss-basic.xml"), edited).expect("rewrite fixture");

    let response = client
        .get(&url)
        .header(IF_NONE_MATCH, &stale_etag)
        .send()
        .await
        .expect("request");
    assert_eq!(response.status(), StatusCode::OK);
    assert!(response.text().await.unwrap().contains("Rewritten title"));

    server.shutdown().await;
}

#[tokio::test]
async fn unknown_and_escaping_paths_are_404() {
    let (server, _dir) = fixture_server().await;
    let client = reqwest::Client::new();

    for path in ["/nope.xml", "/../Cargo.toml", "/%2e%2e/Cargo.toml"] {
        let response = client
            .get(format!("{}{path}", server.base_url()))
            .send()
            .await
            .expect("request");
        assert_eq!(
            response.status(),
            StatusCode::NOT_FOUND,
            "expected 404 for {path}"
        );
    }

    server.shutdown().await;
}

#[tokio::test]
async fn root_lists_the_corpus() {
    let (server, _dir) = fixture_server().await;
    let body = reqwest::get(server.base_url())
        .await
        .expect("request")
        .text()
        .await
        .expect("body");

    for fixture in fixtures::FIXTURES {
        assert!(
            body.contains(fixture.name),
            "index should list {}",
            fixture.name
        );
    }

    server.shutdown().await;
}

#[test]
fn make_fixtures_writes_the_documented_corpus() {
    let dir = tempfile::tempdir().expect("tempdir");
    let written = fixtures::write_fixtures(dir.path()).expect("write fixtures");
    assert_eq!(written.len(), fixtures::FIXTURES.len() + 1);

    let readme = std::fs::read_to_string(dir.path().join("README.md")).expect("readme");
    for fixture in fixtures::FIXTURES {
        assert!(dir.path().join(fixture.name).is_file(), "{}", fixture.name);
        assert!(
            readme.contains(fixture.name),
            "README documents {}",
            fixture.name
        );
    }
}
