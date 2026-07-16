//! feedgen over real local HTTP.

use feedgen::{fixtures, Options};
use reqwest::header::{ETAG, IF_MODIFIED_SINCE, IF_NONE_MATCH, LAST_MODIFIED};
use reqwest::StatusCode;

/// A corpus in a tempdir, served on an ephemeral port.
fn corpus_server(conditional: bool) -> (tempfile::TempDir, feedgen::Spawned) {
    let dir = tempfile::tempdir().expect("tempdir");
    fixtures::write_corpus(dir.path()).expect("write corpus");
    let server = feedgen::spawn(Options::new(dir.path()).conditional(conditional)).expect("spawn");
    (dir, server)
}

/// A client that never follows redirects and never reaches beyond localhost.
fn client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .expect("client")
}

#[tokio::test]
async fn serves_fixtures_with_content_type_etag_and_last_modified() {
    let (_dir, server) = corpus_server(true);
    let res = client()
        .get(server.url("rss-basic.rss"))
        .send()
        .await
        .expect("request");

    assert_eq!(res.status(), StatusCode::OK);
    assert_eq!(
        res.headers().get(reqwest::header::CONTENT_TYPE).unwrap(),
        "application/rss+xml; charset=utf-8"
    );
    assert!(res.headers().contains_key(ETAG), "ETag is required");
    assert!(
        res.headers().contains_key(LAST_MODIFIED),
        "Last-Modified is required"
    );
    assert!(res.text().await.unwrap().contains("Example RSS Feed"));
}

#[tokio::test]
async fn content_type_tracks_the_extension() {
    let (_dir, server) = corpus_server(true);
    for (file, expected) in [
        ("rss-basic.rss", "application/rss+xml; charset=utf-8"),
        ("atom-basic.atom", "application/atom+xml; charset=utf-8"),
        ("malformed.xml", "application/xml; charset=utf-8"),
        ("README.md", "text/markdown; charset=utf-8"),
    ] {
        let res = client().get(server.url(file)).send().await.unwrap();
        assert_eq!(
            res.headers().get(reqwest::header::CONTENT_TYPE).unwrap(),
            expected,
            "{file}"
        );
    }
}

#[tokio::test]
async fn etag_is_content_derived_and_changes_when_the_file_changes() {
    let (dir, server) = corpus_server(true);

    let first = client().get(server.url("rss-basic.rss")).send().await.unwrap();
    let etag_1 = first.headers().get(ETAG).unwrap().to_str().unwrap().to_string();

    // Same bytes, same ETag.
    let again = client().get(server.url("rss-basic.rss")).send().await.unwrap();
    assert_eq!(again.headers().get(ETAG).unwrap(), etag_1.as_str());

    // Different bytes, different ETag.
    std::fs::write(dir.path().join("rss-basic.rss"), fixtures::ATOM_BASIC).unwrap();
    let changed = client().get(server.url("rss-basic.rss")).send().await.unwrap();
    assert_ne!(changed.headers().get(ETAG).unwrap(), etag_1.as_str());
}

#[tokio::test]
async fn if_none_match_gets_304() {
    let (_dir, server) = corpus_server(true);
    let first = client().get(server.url("rss-basic.rss")).send().await.unwrap();
    let etag = first.headers().get(ETAG).unwrap().to_str().unwrap().to_string();

    let second = client()
        .get(server.url("rss-basic.rss"))
        .header(IF_NONE_MATCH, &etag)
        .send()
        .await
        .unwrap();

    assert_eq!(second.status(), StatusCode::NOT_MODIFIED);
    assert!(second.bytes().await.unwrap().is_empty(), "304 carries no body");
}

#[tokio::test]
async fn if_modified_since_gets_304() {
    let (_dir, server) = corpus_server(true);
    let first = client().get(server.url("rss-basic.rss")).send().await.unwrap();
    let last_modified = first
        .headers()
        .get(LAST_MODIFIED)
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();

    let second = client()
        .get(server.url("rss-basic.rss"))
        .header(IF_MODIFIED_SINCE, &last_modified)
        .send()
        .await
        .unwrap();

    assert_eq!(second.status(), StatusCode::NOT_MODIFIED);
}

#[tokio::test]
async fn stale_if_none_match_gets_200() {
    let (_dir, server) = corpus_server(true);
    let res = client()
        .get(server.url("rss-basic.rss"))
        .header(IF_NONE_MATCH, "\"not-the-current-etag\"")
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn no_conditional_mode_always_answers_200() {
    // The mode that lets feedd's dedupe path be exercised with identical bytes.
    let (_dir, server) = corpus_server(false);
    let first = client().get(server.url("rss-basic.rss")).send().await.unwrap();
    let etag = first.headers().get(ETAG).unwrap().to_str().unwrap().to_string();

    let second = client()
        .get(server.url("rss-basic.rss"))
        .header(IF_NONE_MATCH, &etag)
        .send()
        .await
        .unwrap();
    assert_eq!(
        second.status(),
        StatusCode::OK,
        "conditional GET is disabled, so a matching ETag must still get 200"
    );
}

#[tokio::test]
async fn missing_file_is_404() {
    let (_dir, server) = corpus_server(true);
    let res = client().get(server.url("nope.rss")).send().await.unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn request_log_records_conditional_headers() {
    // This is the mechanism the feedd e2e uses to prove feedd *sends*
    // If-None-Match, rather than merely that feedgen would honor it.
    let (_dir, server) = corpus_server(true);

    client().get(server.url("rss-basic.rss")).send().await.unwrap();
    client()
        .get(server.url("rss-basic.rss"))
        .header(IF_NONE_MATCH, "\"abc\"")
        .send()
        .await
        .unwrap();

    let log = server.log.snapshot();
    assert_eq!(log.len(), 2);
    assert_eq!(log[0].path, "rss-basic.rss");
    assert_eq!(log[0].if_none_match, None);
    assert_eq!(log[1].if_none_match.as_deref(), Some("\"abc\""));
}

#[tokio::test]
async fn path_traversal_is_refused() {
    let (_dir, server) = corpus_server(true);
    // Send a raw, unnormalized path; reqwest would otherwise collapse the `..`.
    let res = client()
        .get(format!("{}/..%2f..%2fetc%2fpasswd", server.base_url()))
        .send()
        .await
        .unwrap();
    assert_ne!(res.status(), StatusCode::OK);
}
