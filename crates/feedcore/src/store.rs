//! SQLite storage: feeds, entries, dedupe/update, and the entries query.

use crate::dates;
use crate::error::{FeedError, Result};
use crate::model::{Entry, Feed};
use crate::parse::ParsedItem;
use rusqlite::{params, Connection, OptionalExtension};

/// Cached conditional-GET validators for a feed.
#[derive(Debug, Clone, Default)]
pub struct FeedFetchInfo {
    pub url: String,
    pub etag: Option<String>,
    pub last_modified: Option<String>,
}

/// Filter for [`Store::query_entries`].
#[derive(Debug, Default, Clone)]
pub struct EntryQuery {
    pub feed_id: Option<i64>,
    pub since: Option<String>,
    pub until: Option<String>,
    pub q: Option<String>,
    pub limit: i64,
    pub offset: i64,
}

impl EntryQuery {
    pub fn new() -> Self {
        EntryQuery {
            limit: 50,
            offset: 0,
            ..Default::default()
        }
    }
}

/// A handle to the SQLite database.
pub struct Store {
    conn: Connection,
}

impl Store {
    /// Open (creating if needed) the database at `path` and run migrations.
    pub fn open(path: &str) -> Result<Store> {
        let conn = Connection::open(path)?;
        Self::migrate(&conn)?;
        Ok(Store { conn })
    }

    /// Open an in-memory database (used by tests).
    pub fn open_in_memory() -> Result<Store> {
        let conn = Connection::open_in_memory()?;
        Self::migrate(&conn)?;
        Ok(Store { conn })
    }

    fn migrate(conn: &Connection) -> Result<()> {
        conn.execute_batch(
            r#"
            PRAGMA foreign_keys = ON;
            CREATE TABLE IF NOT EXISTS feeds (
                id              INTEGER PRIMARY KEY AUTOINCREMENT,
                url             TEXT NOT NULL UNIQUE,
                title           TEXT,
                last_fetched_at TEXT,
                last_error      TEXT,
                etag            TEXT,
                last_modified   TEXT
            );
            CREATE TABLE IF NOT EXISTS entries (
                id           INTEGER PRIMARY KEY AUTOINCREMENT,
                feed_id      INTEGER NOT NULL REFERENCES feeds(id) ON DELETE CASCADE,
                guid         TEXT NOT NULL,
                title        TEXT NOT NULL,
                link         TEXT,
                summary      TEXT,
                published_at TEXT,
                published_ms INTEGER,
                fetched_at   TEXT NOT NULL,
                UNIQUE(feed_id, guid)
            );
            CREATE INDEX IF NOT EXISTS idx_entries_feed ON entries(feed_id);
            CREATE INDEX IF NOT EXISTS idx_entries_pub ON entries(published_ms);
            "#,
        )?;
        Ok(())
    }

    fn valid_http_url(url: &str) -> bool {
        let u = url.trim();
        (u.starts_with("http://") || u.starts_with("https://")) && u.len() > "https://".len()
    }

    /// Register a new feed URL. Errors: [`FeedError::InvalidUrl`],
    /// [`FeedError::Duplicate`].
    pub fn add_feed(&self, url: &str) -> Result<Feed> {
        let url = url.trim();
        if !Self::valid_http_url(url) {
            return Err(FeedError::InvalidUrl);
        }
        let existing: Option<i64> = self
            .conn
            .query_row("SELECT id FROM feeds WHERE url = ?1", params![url], |r| {
                r.get(0)
            })
            .optional()?;
        if existing.is_some() {
            return Err(FeedError::Duplicate);
        }
        self.conn
            .execute("INSERT INTO feeds (url) VALUES (?1)", params![url])?;
        let id = self.conn.last_insert_rowid();
        self.get_feed(id)?.ok_or(FeedError::NotFound)
    }

    /// Fetch one feed object (with `entry_count`) by id.
    pub fn get_feed(&self, id: i64) -> Result<Option<Feed>> {
        self.conn
            .query_row(
                r#"SELECT id, url, title, last_fetched_at, last_error,
                          (SELECT COUNT(*) FROM entries e WHERE e.feed_id = f.id)
                   FROM feeds f WHERE id = ?1"#,
                params![id],
                Self::row_to_feed,
            )
            .optional()
            .map_err(FeedError::from)
    }

    /// List all feeds ordered by id.
    pub fn list_feeds(&self) -> Result<Vec<Feed>> {
        let mut stmt = self.conn.prepare(
            r#"SELECT id, url, title, last_fetched_at, last_error,
                      (SELECT COUNT(*) FROM entries e WHERE e.feed_id = f.id)
               FROM feeds f ORDER BY id"#,
        )?;
        let rows = stmt.query_map([], Self::row_to_feed)?;
        let mut out = Vec::new();
        for r in rows {
            out.push(r?);
        }
        Ok(out)
    }

    fn row_to_feed(r: &rusqlite::Row<'_>) -> rusqlite::Result<Feed> {
        Ok(Feed {
            id: r.get(0)?,
            url: r.get(1)?,
            title: r.get(2)?,
            last_fetched_at: r.get(3)?,
            last_error: r.get(4)?,
            entry_count: r.get(5)?,
        })
    }

    /// Delete a feed and its entries. Returns `true` if a row was removed.
    pub fn delete_feed(&self, id: i64) -> Result<bool> {
        let n = self
            .conn
            .execute("DELETE FROM feeds WHERE id = ?1", params![id])?;
        Ok(n > 0)
    }

    /// Return the conditional-GET info for a feed, if it exists.
    pub fn fetch_info(&self, id: i64) -> Result<Option<FeedFetchInfo>> {
        self.conn
            .query_row(
                "SELECT url, etag, last_modified FROM feeds WHERE id = ?1",
                params![id],
                |r| {
                    Ok(FeedFetchInfo {
                        url: r.get(0)?,
                        etag: r.get(1)?,
                        last_modified: r.get(2)?,
                    })
                },
            )
            .optional()
            .map_err(FeedError::from)
    }

    /// All feed ids, ascending.
    pub fn feed_ids(&self) -> Result<Vec<i64>> {
        let mut stmt = self.conn.prepare("SELECT id FROM feeds ORDER BY id")?;
        let rows = stmt.query_map([], |r| r.get::<_, i64>(0))?;
        let mut out = Vec::new();
        for r in rows {
            out.push(r?);
        }
        Ok(out)
    }

    /// Record a successful fetch: update title/validators, clear `last_error`.
    pub fn record_success(
        &self,
        id: i64,
        title: Option<&str>,
        fetched_at: &str,
        etag: Option<&str>,
        last_modified: Option<&str>,
    ) -> Result<()> {
        // Only overwrite the title when the feed provided one.
        if let Some(t) = title {
            self.conn.execute(
                "UPDATE feeds SET title = ?2 WHERE id = ?1",
                params![id, t],
            )?;
        }
        self.conn.execute(
            "UPDATE feeds SET last_fetched_at = ?2, last_error = NULL, etag = ?3, last_modified = ?4 WHERE id = ?1",
            params![id, fetched_at, etag, last_modified],
        )?;
        Ok(())
    }

    /// Record a not-modified (304) fetch: success, validators/entries untouched.
    pub fn record_not_modified(&self, id: i64, fetched_at: &str) -> Result<()> {
        self.conn.execute(
            "UPDATE feeds SET last_fetched_at = ?2, last_error = NULL WHERE id = ?1",
            params![id, fetched_at],
        )?;
        Ok(())
    }

    /// Record a failed fetch/parse: set `last_error`, bump `last_fetched_at`.
    pub fn record_error(&self, id: i64, fetched_at: &str, error: &str) -> Result<()> {
        self.conn.execute(
            "UPDATE feeds SET last_fetched_at = ?2, last_error = ?3 WHERE id = ?1",
            params![id, fetched_at, error],
        )?;
        Ok(())
    }

    /// Insert or update an entry by (feed_id, guid). Returns `true` if the row
    /// was newly inserted. Updates keep the internal id and original
    /// `fetched_at`.
    pub fn upsert_entry(&self, feed_id: i64, item: &ParsedItem, fetched_at: &str) -> Result<bool> {
        let published_at = item.published_at.as_ref().map(dates::to_rfc3339_z);
        let published_ms = item.published_at.as_ref().map(|d| d.timestamp_millis());

        let existing: Option<i64> = self
            .conn
            .query_row(
                "SELECT id FROM entries WHERE feed_id = ?1 AND guid = ?2",
                params![feed_id, item.guid],
                |r| r.get(0),
            )
            .optional()?;

        if let Some(_id) = existing {
            self.conn.execute(
                "UPDATE entries SET title = ?3, link = ?4, summary = ?5,
                     published_at = ?6, published_ms = ?7
                 WHERE feed_id = ?1 AND guid = ?2",
                params![
                    feed_id,
                    item.guid,
                    item.title,
                    item.link,
                    item.summary,
                    published_at,
                    published_ms
                ],
            )?;
            Ok(false)
        } else {
            self.conn.execute(
                "INSERT INTO entries (feed_id, guid, title, link, summary, published_at, published_ms, fetched_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    feed_id,
                    item.guid,
                    item.title,
                    item.link,
                    item.summary,
                    published_at,
                    published_ms,
                    fetched_at
                ],
            )?;
            Ok(true)
        }
    }

    /// Query entries with filtering, ordering, and pagination.
    /// Returns `(total, items)` where `total` ignores limit/offset.
    pub fn query_entries(&self, q: &EntryQuery) -> Result<(i64, Vec<Entry>)> {
        let mut clauses: Vec<String> = Vec::new();
        let mut args: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

        if let Some(fid) = q.feed_id {
            clauses.push(format!("feed_id = ?{}", args.len() + 1));
            args.push(Box::new(fid));
        }
        // Date window: half-open [since, until), null published excluded when
        // either bound is present.
        if let Some(since) = &q.since {
            let ms = dates::parse_any(since)
                .ok_or_else(|| FeedError::Parse(format!("invalid 'since' date: {since}")))?
                .timestamp_millis();
            clauses.push(format!(
                "published_ms IS NOT NULL AND published_ms >= ?{}",
                args.len() + 1
            ));
            args.push(Box::new(ms));
        }
        if let Some(until) = &q.until {
            let ms = dates::parse_any(until)
                .ok_or_else(|| FeedError::Parse(format!("invalid 'until' date: {until}")))?
                .timestamp_millis();
            clauses.push(format!(
                "published_ms IS NOT NULL AND published_ms < ?{}",
                args.len() + 1
            ));
            args.push(Box::new(ms));
        }
        if let Some(text) = &q.q {
            clauses.push(format!(
                "instr(lower(title), lower(?{})) > 0",
                args.len() + 1
            ));
            args.push(Box::new(text.clone()));
        }

        let where_sql = if clauses.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", clauses.join(" AND "))
        };

        let total: i64 = {
            let sql = format!("SELECT COUNT(*) FROM entries {where_sql}");
            let params_ref: Vec<&dyn rusqlite::types::ToSql> =
                args.iter().map(|b| b.as_ref()).collect();
            self.conn
                .query_row(&sql, params_ref.as_slice(), |r| r.get(0))?
        };

        let limit = q.limit.clamp(0, 500);
        let offset = q.offset.max(0);
        let sql = format!(
            "SELECT id, feed_id, guid, title, link, summary, published_at, fetched_at
             FROM entries {where_sql}
             ORDER BY (published_ms IS NULL), published_ms DESC, id ASC
             LIMIT ?{} OFFSET ?{}",
            args.len() + 1,
            args.len() + 2
        );
        args.push(Box::new(limit));
        args.push(Box::new(offset));
        let params_ref: Vec<&dyn rusqlite::types::ToSql> =
            args.iter().map(|b| b.as_ref()).collect();

        let mut stmt = self.conn.prepare(&sql)?;
        let rows = stmt.query_map(params_ref.as_slice(), |r| {
            Ok(Entry {
                id: r.get(0)?,
                feed_id: r.get(1)?,
                guid: r.get(2)?,
                title: r.get(3)?,
                link: r.get(4)?,
                summary: r.get(5)?,
                published_at: r.get(6)?,
                fetched_at: r.get(7)?,
            })
        })?;
        let mut items = Vec::new();
        for row in rows {
            items.push(row?);
        }
        Ok((total, items))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};

    fn item(guid: &str, title: &str, secs: Option<i64>) -> ParsedItem {
        ParsedItem {
            guid: guid.to_string(),
            title: title.to_string(),
            link: Some(format!("http://e/{guid}")),
            summary: Some("s".into()),
            published_at: secs.map(|s| Utc.timestamp_opt(s, 0).unwrap()),
        }
    }

    #[test]
    fn add_and_duplicate_and_invalid() {
        let s = Store::open_in_memory().unwrap();
        let f = s.add_feed("http://example.com/feed").unwrap();
        assert_eq!(f.entry_count, 0);
        assert!(f.title.is_none());
        assert!(matches!(
            s.add_feed("http://example.com/feed"),
            Err(FeedError::Duplicate)
        ));
        assert!(matches!(s.add_feed("ftp://x"), Err(FeedError::InvalidUrl)));
        assert!(matches!(s.add_feed("notaurl"), Err(FeedError::InvalidUrl)));
    }

    #[test]
    fn dedupe_updates_in_place() {
        let s = Store::open_in_memory().unwrap();
        let f = s.add_feed("http://example.com/feed").unwrap();
        assert!(s.upsert_entry(f.id, &item("g1", "First", Some(100)), "T0").unwrap());
        // Second insert of same guid updates, not duplicates.
        assert!(!s.upsert_entry(f.id, &item("g1", "Updated", Some(200)), "T1").unwrap());
        let (total, items) = s.query_entries(&EntryQuery::new()).unwrap();
        assert_eq!(total, 1);
        assert_eq!(items[0].title, "Updated");
        assert_eq!(items[0].fetched_at, "T0"); // original fetched_at preserved
    }

    #[test]
    fn delete_cascades_entries() {
        let s = Store::open_in_memory().unwrap();
        let f = s.add_feed("http://example.com/feed").unwrap();
        s.upsert_entry(f.id, &item("g1", "x", Some(1)), "T0").unwrap();
        assert!(s.delete_feed(f.id).unwrap());
        let (total, _) = s.query_entries(&EntryQuery::new()).unwrap();
        assert_eq!(total, 0);
    }

    #[test]
    fn ordering_desc_nulls_last_tie_by_id() {
        let s = Store::open_in_memory().unwrap();
        let f = s.add_feed("http://example.com/feed").unwrap();
        s.upsert_entry(f.id, &item("a", "a", Some(300)), "T").unwrap();
        s.upsert_entry(f.id, &item("b", "b", None), "T").unwrap();
        s.upsert_entry(f.id, &item("c", "c", Some(300)), "T").unwrap();
        s.upsert_entry(f.id, &item("d", "d", Some(500)), "T").unwrap();
        let (_total, items) = s.query_entries(&EntryQuery::new()).unwrap();
        let guids: Vec<&str> = items.iter().map(|e| e.guid.as_str()).collect();
        // 500 first, then the two 300s tie-broken by id (a before c), null last.
        assert_eq!(guids, vec!["d", "a", "c", "b"]);
    }

    #[test]
    fn window_half_open_and_excludes_null() {
        let s = Store::open_in_memory().unwrap();
        let f = s.add_feed("http://example.com/feed").unwrap();
        // published at epoch 100, 200, 300 seconds.
        s.upsert_entry(f.id, &item("p100", "x", Some(100)), "T").unwrap();
        s.upsert_entry(f.id, &item("p200", "x", Some(200)), "T").unwrap();
        s.upsert_entry(f.id, &item("p300", "x", Some(300)), "T").unwrap();
        s.upsert_entry(f.id, &item("pnull", "x", None), "T").unwrap();
        let mut q = EntryQuery::new();
        q.since = Some(Utc.timestamp_opt(200, 0).unwrap().to_rfc3339());
        q.until = Some(Utc.timestamp_opt(300, 0).unwrap().to_rfc3339());
        let (total, items) = s.query_entries(&q).unwrap();
        assert_eq!(total, 1);
        assert_eq!(items[0].guid, "p200"); // 200 included, 300 excluded, null excluded
    }

    #[test]
    fn search_and_pagination() {
        let s = Store::open_in_memory().unwrap();
        let f = s.add_feed("http://example.com/feed").unwrap();
        s.upsert_entry(f.id, &item("g1", "Rust news", Some(10)), "T").unwrap();
        s.upsert_entry(f.id, &item("g2", "rust weekly", Some(20)), "T").unwrap();
        s.upsert_entry(f.id, &item("g3", "Cooking", Some(30)), "T").unwrap();
        let mut q = EntryQuery::new();
        q.q = Some("RUST".into());
        let (total, items) = s.query_entries(&q).unwrap();
        assert_eq!(total, 2);
        assert_eq!(items.len(), 2);

        let mut q2 = EntryQuery::new();
        q2.limit = 1;
        q2.offset = 1;
        let (total2, items2) = s.query_entries(&q2).unwrap();
        assert_eq!(total2, 3);
        assert_eq!(items2.len(), 1);
    }
}
