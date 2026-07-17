//! Human-readable rendering of API responses.
//!
//! Every function here returns a `String` rather than printing, so the tests can
//! check the output without capturing stdout.

use feedhub_core::api::{EntriesPage, Feed, RefreshResult};

/// Shown in place of a null `published_at`.
const NO_DATE: &str = "(no date)";

pub fn feed_added(feed: &Feed) -> String {
    format!("Added feed {}: {}", feed.id, feed.url)
}

pub fn feed_removed(id: i64) -> String {
    format!("Removed feed {id}")
}

/// A table of feeds: id, title, entry count, last fetch, and status.
pub fn feed_list(feeds: &[Feed]) -> String {
    if feeds.is_empty() {
        return "No feeds registered.".to_string();
    }

    let rows: Vec<[String; 5]> = feeds
        .iter()
        .map(|feed| {
            [
                feed.id.to_string(),
                feed.title.clone().unwrap_or_else(|| feed.url.clone()),
                feed.entry_count.to_string(),
                feed.last_fetched_at
                    .clone()
                    .unwrap_or_else(|| "never".into()),
                status_of(feed),
            ]
        })
        .collect();

    let header = [
        "ID".to_string(),
        "TITLE".to_string(),
        "ENTRIES".to_string(),
        "LAST FETCHED".to_string(),
        "STATUS".to_string(),
    ];

    let mut widths = [0usize; 5];
    for row in std::iter::once(&header).chain(rows.iter()) {
        for (i, cell) in row.iter().enumerate() {
            widths[i] = widths[i].max(cell.chars().count());
        }
    }

    let mut out = String::new();
    for row in std::iter::once(&header).chain(rows.iter()) {
        let mut line = String::new();
        for (i, cell) in row.iter().enumerate() {
            // The last column is not padded, so lines have no trailing blanks.
            if i + 1 == row.len() {
                line.push_str(cell);
            } else {
                line.push_str(&pad(cell, widths[i]));
                line.push_str("  ");
            }
        }
        out.push_str(line.trim_end());
        out.push('\n');
    }
    out.trim_end().to_string()
}

/// One feed in full.
pub fn feed_detail(feed: &Feed) -> String {
    let mut out = String::new();
    out.push_str(&format!("id:            {}\n", feed.id));
    out.push_str(&format!("url:           {}\n", feed.url));
    out.push_str(&format!(
        "title:         {}\n",
        feed.title
            .clone()
            .unwrap_or_else(|| "(not fetched yet)".into())
    ));
    out.push_str(&format!("entries:       {}\n", feed.entry_count));
    out.push_str(&format!(
        "last fetched:  {}\n",
        feed.last_fetched_at
            .clone()
            .unwrap_or_else(|| "never".into())
    ));
    out.push_str(&format!("status:        {}", status_of(feed)));
    out
}

fn status_of(feed: &Feed) -> String {
    match (&feed.last_error, &feed.last_fetched_at) {
        (Some(error), _) => format!("error: {error}"),
        (None, Some(_)) => "ok".to_string(),
        (None, None) => "not fetched yet".to_string(),
    }
}

/// One refresh outcome, one line.
pub fn refresh_result(result: &RefreshResult) -> String {
    if !result.is_ok() {
        let error = result
            .error
            .clone()
            .unwrap_or_else(|| "unknown error".into());
        return format!("feed {}: error: {}", result.feed_id, error);
    }
    if result.not_modified {
        return format!("feed {}: ok, not modified", result.feed_id);
    }
    format!(
        "feed {}: ok, {} new, {} updated",
        result.feed_id, result.new_entries, result.updated_entries
    )
}

/// Several refresh outcomes, with a summary line when there is more than one.
pub fn refresh_results(results: &[RefreshResult]) -> String {
    if results.is_empty() {
        return "No feeds to refresh.".to_string();
    }

    let mut out: String = results
        .iter()
        .map(|r| format!("{}\n", refresh_result(r)))
        .collect();

    if results.len() > 1 {
        let failed = results.iter().filter(|r| !r.is_ok()).count();
        let new: i64 = results.iter().map(|r| r.new_entries).sum();
        out.push_str(&format!(
            "\n{} feeds refreshed, {} new entries, {} failed",
            results.len(),
            new,
            failed
        ));
    }
    out.trim_end().to_string()
}

/// A page of entries: date and title, with the link underneath.
pub fn entries(page: &EntriesPage) -> String {
    if page.items.is_empty() {
        return "No matching entries.".to_string();
    }

    let width = page
        .items
        .iter()
        .map(|e| e.published_at.as_deref().unwrap_or(NO_DATE).chars().count())
        .max()
        .unwrap_or(0);

    let mut out = String::new();
    for entry in &page.items {
        let date = entry.published_at.as_deref().unwrap_or(NO_DATE);
        out.push_str(&format!("{}  {}\n", pad(date, width), entry.title));
        if let Some(link) = &entry.link {
            out.push_str(&format!("{}  {}\n", " ".repeat(width), link));
        }
    }

    out.push_str(&format!(
        "\nShowing {} of {} entries.",
        page.items.len(),
        page.total
    ));
    out
}

fn pad(text: &str, width: usize) -> String {
    let len = text.chars().count();
    format!("{text}{}", " ".repeat(width.saturating_sub(len)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use feedhub_core::api::Entry;

    fn feed(id: i64) -> Feed {
        Feed {
            id,
            url: format!("http://example.invalid/{id}"),
            title: Some(format!("Feed {id}")),
            last_fetched_at: Some("2024-03-01T12:00:00Z".into()),
            last_error: None,
            entry_count: 2,
        }
    }

    #[test]
    fn feed_list_is_aligned_and_has_no_trailing_space() {
        let feeds = vec![feed(1), feed(200)];
        let text = feed_list(&feeds);
        let lines: Vec<&str> = text.lines().collect();

        assert!(lines[0].starts_with("ID"));
        assert_eq!(lines.len(), 3);
        for line in &lines {
            assert_eq!(*line, line.trim_end(), "no trailing whitespace: {line:?}");
        }
        // The URL column starts at the same offset on every row.
        let title_column = lines[0].find("TITLE").expect("a TITLE column");
        assert_eq!(lines[1].find("Feed 1"), Some(title_column));
        assert_eq!(lines[2].find("Feed 200"), Some(title_column));
    }

    #[test]
    fn empty_results_say_so_rather_than_printing_nothing() {
        assert_eq!(feed_list(&[]), "No feeds registered.");
        assert_eq!(refresh_results(&[]), "No feeds to refresh.");
        assert_eq!(
            entries(&EntriesPage {
                total: 0,
                items: vec![]
            }),
            "No matching entries."
        );
    }

    #[test]
    fn a_feeds_error_shows_up_in_its_status() {
        let mut broken = feed(1);
        broken.last_error = Some("HTTP 404 Not Found".into());
        assert!(feed_list(&[broken.clone()]).contains("error: HTTP 404 Not Found"));
        assert!(feed_detail(&broken).contains("error: HTTP 404 Not Found"));

        let fresh = Feed {
            last_fetched_at: None,
            title: None,
            ..feed(2)
        };
        assert!(feed_detail(&fresh).contains("not fetched yet"));
    }

    #[test]
    fn refresh_lines_distinguish_the_three_outcomes() {
        let base = RefreshResult {
            feed_id: 1,
            status: "ok".into(),
            new_entries: 2,
            updated_entries: 1,
            not_modified: false,
            error: None,
        };
        assert_eq!(refresh_result(&base), "feed 1: ok, 2 new, 1 updated");

        let not_modified = RefreshResult {
            not_modified: true,
            new_entries: 0,
            updated_entries: 0,
            ..base.clone()
        };
        assert_eq!(refresh_result(&not_modified), "feed 1: ok, not modified");

        let failed = RefreshResult {
            status: "error".into(),
            error: Some("connection failed".into()),
            new_entries: 0,
            updated_entries: 0,
            ..base.clone()
        };
        assert_eq!(refresh_result(&failed), "feed 1: error: connection failed");

        let summary = refresh_results(&[base, failed]);
        assert!(summary.contains("2 feeds refreshed, 2 new entries, 1 failed"));
    }

    #[test]
    fn entries_show_dates_titles_and_links() {
        let page = EntriesPage {
            total: 5,
            items: vec![
                Entry {
                    id: 1,
                    feed_id: 1,
                    guid: "a".into(),
                    title: "Dated".into(),
                    link: Some("http://example.invalid/a".into()),
                    summary: None,
                    published_at: Some("2024-03-01T12:00:00Z".into()),
                    fetched_at: "2024-03-02T00:00:00Z".into(),
                },
                Entry {
                    id: 2,
                    feed_id: 1,
                    guid: "b".into(),
                    title: "Undated".into(),
                    link: None,
                    summary: None,
                    published_at: None,
                    fetched_at: "2024-03-02T00:00:00Z".into(),
                },
            ],
        };

        let text = entries(&page);
        assert!(text.contains("2024-03-01T12:00:00Z  Dated"));
        assert!(text.contains("http://example.invalid/a"));
        assert!(text.contains(NO_DATE));
        // total is the match count, not the page size.
        assert!(text.ends_with("Showing 2 of 5 entries."));
    }
}
