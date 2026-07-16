//! Storage semantics: dedupe, ordering, windows, and failure isolation.

use feedd::store::{ApplyCounts, EntryQuery, FetchSuccess, Store, StoreError};
use feedhub_core::datetime::{parse_rfc3339, to_millis};
use feedhub_core::model::ParsedEntry;

fn store() -> Store {
    Store::open_in_memory().expect("in-memory store")
}

fn at(s: &str) -> chrono::DateTime<chrono::Utc> {
    parse_rfc3339(s).expect("test literal")
}

fn entry(guid: &str, title: &str, published: Option<&str>) -> ParsedEntry {
    ParsedEntry {
        guid: guid.into(),
        title: title.into(),
        link: Some(format!("https://example.com/{guid}")),
        summary: Some(format!("summary of {guid}")),
        published_at: published.map(at),
    }
}

fn success(entries: Vec<ParsedEntry>) -> FetchSuccess {
    FetchSuccess {
        title: Some("Feed Title".into()),
        entries,
        etag: Some("\"v1\"".into()),
        last_modified: None,
    }
}

fn query() -> EntryQuery {
    EntryQuery {
        limit: 50,
        ..Default::default()
    }
}

// ---------------------------------------------------------------- feeds

#[test]
fn add_feed_starts_with_no_title_no_fetch_and_no_entries() {
    let s = store();
    let feed = s.add_feed("https://example.com/f.rss").unwrap();
    assert_eq!(
        feed.title, None,
        "title is null until the first successful fetch"
    );
    assert_eq!(feed.last_fetched_at, None);
    assert_eq!(feed.last_error, None);
    assert_eq!(feed.entry_count, 0);
}

#[test]
fn duplicate_url_is_rejected() {
    let s = store();
    s.add_feed("https://example.com/f.rss").unwrap();
    let err = s.add_feed("https://example.com/f.rss").unwrap_err();
    assert!(matches!(err, StoreError::DuplicateUrl), "got {err:?}");
}

#[test]
fn entry_count_reflects_stored_entries() {
    let s = store();
    let feed = s.add_feed("https://example.com/f.rss").unwrap();
    s.apply_success(
        feed.id,
        &success(vec![entry("a", "A", None), entry("b", "B", None)]),
        at("2020-01-01T00:00:00Z"),
    )
    .unwrap();
    assert_eq!(s.get_feed(feed.id).unwrap().unwrap().entry_count, 2);
}

#[test]
fn deleting_a_feed_deletes_its_entries_and_leaves_others_alone() {
    let s = store();
    let doomed = s.add_feed("https://example.com/a.rss").unwrap();
    let keeper = s.add_feed("https://example.com/b.rss").unwrap();
    s.apply_success(
        doomed.id,
        &success(vec![entry("a", "A", None)]),
        at("2020-01-01T00:00:00Z"),
    )
    .unwrap();
    s.apply_success(
        keeper.id,
        &success(vec![entry("b", "B", None)]),
        at("2020-01-01T00:00:00Z"),
    )
    .unwrap();

    assert!(s.delete_feed(doomed.id).unwrap());
    assert!(s.get_feed(doomed.id).unwrap().is_none());

    // ON DELETE CASCADE only works with PRAGMA foreign_keys ON; without it this
    // assertion is what fails.
    let all = s.query_entries(&query()).unwrap();
    assert_eq!(all.total, 1, "the deleted feed's entries must go with it");
    assert_eq!(all.items[0].feed_id, keeper.id);
}

#[test]
fn deleting_a_missing_feed_reports_false() {
    assert!(!store().delete_feed(999).unwrap());
}

// ---------------------------------------------------------------- dedupe

#[test]
fn re_fetching_identical_content_inserts_nothing() {
    // The regression that a 304-only test cannot catch: if new_entries were
    // derived from ON CONFLICT + changes(), this would report 2 forever.
    let s = store();
    let feed = s.add_feed("https://example.com/f.rss").unwrap();
    let fetch = success(vec![
        entry("a", "A", Some("2020-01-01T00:00:00Z")),
        entry("b", "B", Some("2020-01-02T00:00:00Z")),
    ]);

    let first = s
        .apply_success(feed.id, &fetch, at("2020-01-01T00:00:00Z"))
        .unwrap();
    assert_eq!(first, ApplyCounts { new: 2, updated: 0 });

    let second = s
        .apply_success(feed.id, &fetch, at("2020-01-03T00:00:00Z"))
        .unwrap();
    assert_eq!(
        second,
        ApplyCounts { new: 0, updated: 2 },
        "a second fetch of the same content must insert nothing"
    );
    assert_eq!(s.get_feed(feed.id).unwrap().unwrap().entry_count, 2);
}

#[test]
fn a_known_identity_updates_in_place_keeping_id_and_fetched_at() {
    let s = store();
    let feed = s.add_feed("https://example.com/f.rss").unwrap();

    s.apply_success(
        feed.id,
        &success(vec![entry(
            "a",
            "Original title",
            Some("2020-01-01T00:00:00Z"),
        )]),
        at("2020-01-01T00:00:00Z"),
    )
    .unwrap();
    let before = s.query_entries(&query()).unwrap().items[0].clone();

    // Same guid, everything else changed, fetched much later.
    let mut changed = entry("a", "Revised title", Some("2020-06-01T00:00:00Z"));
    changed.summary = Some("revised summary".into());
    changed.link = Some("https://example.com/moved".into());
    let counts = s
        .apply_success(feed.id, &success(vec![changed]), at("2020-12-25T00:00:00Z"))
        .unwrap();

    assert_eq!(counts, ApplyCounts { new: 0, updated: 1 });
    let after = s.query_entries(&query()).unwrap();
    assert_eq!(after.total, 1, "no duplicate row");

    let after = &after.items[0];
    assert_eq!(after.id, before.id, "the entry keeps its internal id");
    assert_eq!(
        after.fetched_at, before.fetched_at,
        "the entry keeps its original fetched_at"
    );
    assert_eq!(after.title, "Revised title");
    assert_eq!(after.summary.as_deref(), Some("revised summary"));
    assert_eq!(after.link.as_deref(), Some("https://example.com/moved"));
    assert_eq!(after.published_at.as_deref(), Some("2020-06-01T00:00:00Z"));
}

#[test]
fn the_same_guid_twice_in_one_document_yields_one_row() {
    let s = store();
    let feed = s.add_feed("https://example.com/f.rss").unwrap();
    let counts = s
        .apply_success(
            feed.id,
            &success(vec![
                entry("dup", "First", None),
                entry("dup", "Second", None),
            ]),
            at("2020-01-01T00:00:00Z"),
        )
        .unwrap();

    assert_eq!(counts.new, 1, "a repeated guid must not be counted twice");
    assert_eq!(s.get_feed(feed.id).unwrap().unwrap().entry_count, 1);
}

#[test]
fn the_same_guid_in_different_feeds_is_two_entries() {
    // Identity is (feed, guid), not guid alone.
    let s = store();
    let a = s.add_feed("https://example.com/a.rss").unwrap();
    let b = s.add_feed("https://example.com/b.rss").unwrap();
    s.apply_success(
        a.id,
        &success(vec![entry("shared", "A", None)]),
        at("2020-01-01T00:00:00Z"),
    )
    .unwrap();
    let counts = s
        .apply_success(
            b.id,
            &success(vec![entry("shared", "B", None)]),
            at("2020-01-01T00:00:00Z"),
        )
        .unwrap();
    assert_eq!(counts.new, 1);
    assert_eq!(s.query_entries(&query()).unwrap().total, 2);
}

// ---------------------------------------------------------- feed metadata

#[test]
fn title_appears_on_first_success_and_updates_on_refresh() {
    let s = store();
    let feed = s.add_feed("https://example.com/f.rss").unwrap();
    assert_eq!(feed.title, None);

    s.apply_success(feed.id, &success(vec![]), at("2020-01-01T00:00:00Z"))
        .unwrap();
    assert_eq!(
        s.get_feed(feed.id).unwrap().unwrap().title.as_deref(),
        Some("Feed Title")
    );

    let mut renamed = success(vec![]);
    renamed.title = Some("Renamed Feed".into());
    s.apply_success(feed.id, &renamed, at("2020-01-02T00:00:00Z"))
        .unwrap();
    assert_eq!(
        s.get_feed(feed.id).unwrap().unwrap().title.as_deref(),
        Some("Renamed Feed")
    );
}

#[test]
fn a_failure_preserves_the_title_and_entries() {
    let s = store();
    let feed = s.add_feed("https://example.com/f.rss").unwrap();
    s.apply_success(
        feed.id,
        &success(vec![entry("a", "A", None)]),
        at("2020-01-01T00:00:00Z"),
    )
    .unwrap();

    s.apply_error(feed.id, "connection refused", at("2020-01-02T00:00:00Z"))
        .unwrap();

    let feed = s.get_feed(feed.id).unwrap().unwrap();
    assert_eq!(
        feed.title.as_deref(),
        Some("Feed Title"),
        "title survives a failure"
    );
    assert_eq!(feed.entry_count, 1, "entries survive a failure");
    assert_eq!(feed.last_error.as_deref(), Some("connection refused"));
}

#[test]
fn a_title_less_fetch_does_not_blank_a_known_title() {
    let s = store();
    let feed = s.add_feed("https://example.com/f.rss").unwrap();
    s.apply_success(feed.id, &success(vec![]), at("2020-01-01T00:00:00Z"))
        .unwrap();

    let mut untitled = success(vec![]);
    untitled.title = None;
    s.apply_success(feed.id, &untitled, at("2020-01-02T00:00:00Z"))
        .unwrap();

    assert_eq!(
        s.get_feed(feed.id).unwrap().unwrap().title.as_deref(),
        Some("Feed Title")
    );
}

#[test]
fn any_successful_fetch_clears_last_error() {
    let s = store();
    let feed = s.add_feed("https://example.com/f.rss").unwrap();
    s.apply_error(feed.id, "boom", at("2020-01-01T00:00:00Z"))
        .unwrap();
    assert!(s.get_feed(feed.id).unwrap().unwrap().last_error.is_some());

    s.apply_success(feed.id, &success(vec![]), at("2020-01-02T00:00:00Z"))
        .unwrap();
    assert_eq!(s.get_feed(feed.id).unwrap().unwrap().last_error, None);
}

#[test]
fn a_304_counts_as_success_and_leaves_entries_untouched() {
    let s = store();
    let feed = s.add_feed("https://example.com/f.rss").unwrap();
    s.apply_success(
        feed.id,
        &success(vec![entry("a", "A", None)]),
        at("2020-01-01T00:00:00Z"),
    )
    .unwrap();
    s.apply_error(feed.id, "transient", at("2020-01-02T00:00:00Z"))
        .unwrap();

    s.apply_not_modified(feed.id, at("2020-01-03T00:00:00Z"))
        .unwrap();

    let feed = s.get_feed(feed.id).unwrap().unwrap();
    assert_eq!(feed.last_error, None, "304 is a successful fetch");
    assert_eq!(
        feed.last_fetched_at.as_deref(),
        Some("2020-01-03T00:00:00Z")
    );
    assert_eq!(feed.entry_count, 1, "304 leaves entries untouched");
}

#[test]
fn an_error_on_one_feed_does_not_touch_another() {
    let s = store();
    let broken = s.add_feed("https://example.com/broken.rss").unwrap();
    let healthy = s.add_feed("https://example.com/healthy.rss").unwrap();
    s.apply_success(
        healthy.id,
        &success(vec![entry("h", "H", None)]),
        at("2020-01-01T00:00:00Z"),
    )
    .unwrap();

    s.apply_error(broken.id, "malformed XML", at("2020-01-02T00:00:00Z"))
        .unwrap();

    let healthy = s.get_feed(healthy.id).unwrap().unwrap();
    assert_eq!(healthy.last_error, None);
    assert_eq!(healthy.entry_count, 1);
    assert_eq!(healthy.title.as_deref(), Some("Feed Title"));
}

// ---------------------------------------------------------------- queries

/// Seed a feed whose entries cover the ordering edge cases.
fn seeded() -> (Store, i64) {
    let s = store();
    let feed = s.add_feed("https://example.com/f.rss").unwrap();
    s.apply_success(
        feed.id,
        &success(vec![
            entry("old", "Old post", Some("2020-01-01T00:00:00Z")),
            entry("new", "New post", Some("2020-03-01T00:00:00Z")),
            entry("mid", "Mid post", Some("2020-02-01T00:00:00Z")),
            entry("undated-1", "Undated one", None),
            entry("undated-2", "Undated two", None),
        ]),
        at("2020-06-01T00:00:00Z"),
    )
    .unwrap();
    (s, feed.id)
}

#[test]
fn ordering_is_newest_first_then_nulls_last_by_id() {
    let (s, _) = seeded();
    let page = s.query_entries(&query()).unwrap();
    let titles: Vec<&str> = page.items.iter().map(|e| e.title.as_str()).collect();
    assert_eq!(
        titles,
        [
            "New post",
            "Mid post",
            "Old post",
            "Undated one",
            "Undated two"
        ]
    );

    // The two undated entries tie, so entry id ascending breaks it.
    let undated: Vec<i64> = page.items[3..].iter().map(|e| e.id).collect();
    assert!(undated[0] < undated[1], "ties break by ascending entry id");
}

#[test]
fn equal_dates_break_ties_by_ascending_entry_id() {
    let s = store();
    let feed = s.add_feed("https://example.com/f.rss").unwrap();
    s.apply_success(
        feed.id,
        &success(vec![
            entry("first", "First inserted", Some("2020-01-01T00:00:00Z")),
            entry("second", "Second inserted", Some("2020-01-01T00:00:00Z")),
        ]),
        at("2020-06-01T00:00:00Z"),
    )
    .unwrap();

    let items = s.query_entries(&query()).unwrap().items;
    assert_eq!(items[0].title, "First inserted");
    assert!(items[0].id < items[1].id);
}

#[test]
fn since_is_inclusive_and_until_is_exclusive() {
    let (s, _) = seeded();

    // since == the Mid post's exact instant: Mid is included.
    let page = s
        .query_entries(&EntryQuery {
            since: Some(to_millis(at("2020-02-01T00:00:00Z"))),
            ..query()
        })
        .unwrap();
    let titles: Vec<&str> = page.items.iter().map(|e| e.title.as_str()).collect();
    assert_eq!(titles, ["New post", "Mid post"], "since is inclusive");

    // until == the Mid post's exact instant: Mid is excluded.
    let page = s
        .query_entries(&EntryQuery {
            until: Some(to_millis(at("2020-02-01T00:00:00Z"))),
            ..query()
        })
        .unwrap();
    let titles: Vec<&str> = page.items.iter().map(|e| e.title.as_str()).collect();
    assert_eq!(titles, ["Old post"], "until is exclusive");
}

#[test]
fn a_sub_second_bound_compares_as_a_true_instant() {
    // The case that a truncate-then-ceil scheme gets wrong: an entry published
    // at .900 must satisfy since=.500 and fail until=.500.
    let s = store();
    let feed = s.add_feed("https://example.com/f.rss").unwrap();
    s.apply_success(
        feed.id,
        &success(vec![entry(
            "frac",
            "Fractional",
            Some("2020-01-01T00:00:00.900Z"),
        )]),
        at("2020-06-01T00:00:00Z"),
    )
    .unwrap();

    let since_half = s
        .query_entries(&EntryQuery {
            since: Some(to_millis(at("2020-01-01T00:00:00.500Z"))),
            ..query()
        })
        .unwrap();
    assert_eq!(since_half.total, 1, ".900 >= .500 must match");

    let until_half = s
        .query_entries(&EntryQuery {
            until: Some(to_millis(at("2020-01-01T00:00:00.500Z"))),
            ..query()
        })
        .unwrap();
    assert_eq!(until_half.total, 0, ".900 < .500 must not match");
}

#[test]
fn undated_entries_drop_out_as_soon_as_either_bound_is_given() {
    let (s, _) = seeded();

    let unbounded = s.query_entries(&query()).unwrap();
    assert_eq!(
        unbounded.total, 5,
        "no bounds: undated entries are included"
    );

    for bounded in [
        EntryQuery {
            since: Some(to_millis(at("1970-01-01T00:00:00Z"))),
            ..query()
        },
        EntryQuery {
            until: Some(to_millis(at("2999-01-01T00:00:00Z"))),
            ..query()
        },
    ] {
        let page = s.query_entries(&bounded).unwrap();
        assert_eq!(page.total, 3, "a bound must exclude undated entries");
        assert!(page.items.iter().all(|e| e.published_at.is_some()));
    }
}

#[test]
fn search_folds_ascii_case_and_matches_substrings() {
    let (s, _) = seeded();
    for term in ["old post", "OLD POST", "Old Post", "ld po"] {
        let page = s
            .query_entries(&EntryQuery {
                q: Some(term.into()),
                ..query()
            })
            .unwrap();
        assert_eq!(page.total, 1, "{term:?} should match Old post");
        assert_eq!(page.items[0].title, "Old post");
    }
}

#[test]
fn search_treats_like_metacharacters_literally() {
    // With LIKE '%..%' these terms would behave as wildcards and match
    // everything. instr() has no metacharacters.
    let (s, _) = seeded();
    for term in ["%", "_", "%post%"] {
        let page = s
            .query_entries(&EntryQuery {
                q: Some(term.into()),
                ..query()
            })
            .unwrap();
        assert_eq!(page.total, 0, "{term:?} must be a literal, not a wildcard");
    }

    let s2 = store();
    let feed = s2.add_feed("https://example.com/f.rss").unwrap();
    s2.apply_success(
        feed.id,
        &success(vec![entry("pct", "100% certain", None)]),
        at("2020-01-01T00:00:00Z"),
    )
    .unwrap();
    let page = s2
        .query_entries(&EntryQuery {
            q: Some("100%".into()),
            ..query()
        })
        .unwrap();
    assert_eq!(page.total, 1, "a literal % must match a literal %");
}

#[test]
fn feed_id_restricts_to_one_feed() {
    let s = store();
    let a = s.add_feed("https://example.com/a.rss").unwrap();
    let b = s.add_feed("https://example.com/b.rss").unwrap();
    s.apply_success(
        a.id,
        &success(vec![entry("a", "From A", None)]),
        at("2020-01-01T00:00:00Z"),
    )
    .unwrap();
    s.apply_success(
        b.id,
        &success(vec![entry("b", "From B", None)]),
        at("2020-01-01T00:00:00Z"),
    )
    .unwrap();

    let page = s
        .query_entries(&EntryQuery {
            feed_id: Some(a.id),
            ..query()
        })
        .unwrap();
    assert_eq!(page.total, 1);
    assert_eq!(page.items[0].title, "From A");
}

#[test]
fn total_ignores_limit_and_offset_while_items_respect_them() {
    let (s, _) = seeded();
    let page = s
        .query_entries(&EntryQuery {
            limit: 2,
            offset: 1,
            ..query()
        })
        .unwrap();

    assert_eq!(page.total, 5, "total is the match count, ignoring paging");
    assert_eq!(page.items.len(), 2);
    let titles: Vec<&str> = page.items.iter().map(|e| e.title.as_str()).collect();
    assert_eq!(titles, ["Mid post", "Old post"], "offset skips the newest");
}

#[test]
fn an_offset_past_the_end_yields_no_items_but_a_real_total() {
    let (s, _) = seeded();
    let page = s
        .query_entries(&EntryQuery {
            limit: 50,
            offset: 999,
            ..query()
        })
        .unwrap();
    assert_eq!(page.total, 5);
    assert!(page.items.is_empty());
}

#[test]
fn filters_combine() {
    let (s, feed_id) = seeded();
    let page = s
        .query_entries(&EntryQuery {
            feed_id: Some(feed_id),
            since: Some(to_millis(at("2020-01-15T00:00:00Z"))),
            until: Some(to_millis(at("2020-02-15T00:00:00Z"))),
            q: Some("post".into()),
            limit: 50,
            offset: 0,
        })
        .unwrap();
    assert_eq!(page.total, 1);
    assert_eq!(page.items[0].title, "Mid post");
}

#[test]
fn entries_survive_reopening_the_database_file() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("feedhub.db");

    let feed_id = {
        let s = Store::open(&path).unwrap();
        let feed = s.add_feed("https://example.com/f.rss").unwrap();
        s.apply_success(
            feed.id,
            &success(vec![entry("a", "A", None)]),
            at("2020-01-01T00:00:00Z"),
        )
        .unwrap();
        feed.id
    };

    // Reopening must be idempotent: CREATE TABLE IF NOT EXISTS, data intact.
    let reopened = Store::open(&path).unwrap();
    let feed = reopened.get_feed(feed_id).unwrap().unwrap();
    assert_eq!(feed.entry_count, 1);
    assert_eq!(feed.title.as_deref(), Some("Feed Title"));
}
