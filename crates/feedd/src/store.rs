//! SQLite-backed storage for feeds and entries.
//!
//! Times are stored as UTC epoch seconds (nullable for `published_at`). The
//! connection is shared behind a mutex so the background poller and the HTTP
//! handlers can both use it; critical sections are kept short and never span a
//! network fetch.

use std::sync::{Arc, Mutex};

use chrono::{TimeZone, Utc};
use feedlib::model::{Feed, ParsedEntry};
use rusqlite::{params, Connection, OptionalExtension};
use serde_json::{json, Value};

/// Shared handle to the feed database.
#[derive(Clone)]
pub struct Store {
    conn: Arc<Mutex<Connection>>,
}

/// Result of attempting to register a feed URL.
pub enum AddOutcome {
    Created(Feed),
    Duplicate,
}

/// Filters for an entries query.
#[derive(Default)]
pub struct EntryQuery {
    pub feed_id: Option<i64>,
    pub since: Option<i64>,
    pub until: Option<i64>,
    pub q: Option<String>,
    pub limit: i64,
    pub offset: i64,
    /// True when a time bound was supplied (null published_at then excluded).
    pub has_time_bound: bool,
}

fn epoch_to_rfc3339(secs: Option<i64>) -> Value {
    match secs.and_then(|s| Utc.timestamp_opt(s, 0).single()) {
        Some(dt) => Value::String(feedlib::date::to_rfc3339_z(&dt)),
        None => Value::Null,
    }
}

impl Store {
    /// Open (creating if needed) the database at `path` and ensure the schema.
    pub fn open(path: &str) -> rusqlite::Result<Store> {
        let conn = Connection::open(path)?;
        conn.pragma_update(None, "foreign_keys", "ON")?;
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS feeds (
                id              INTEGER PRIMARY KEY,
                url             TEXT NOT NULL UNIQUE,
                title           TEXT,
                last_fetched_at INTEGER,
                last_error      TEXT,
                etag            TEXT,
                last_modified   TEXT
            );
            CREATE TABLE IF NOT EXISTS entries (
                id           INTEGER PRIMARY KEY,
                feed_id      INTEGER NOT NULL REFERENCES feeds(id) ON DELETE CASCADE,
                guid         TEXT NOT NULL,
                title        TEXT NOT NULL,
                link         TEXT,
                summary      TEXT,
                published_at INTEGER,
                fetched_at   INTEGER NOT NULL,
                UNIQUE(feed_id, guid)
            );
            CREATE INDEX IF NOT EXISTS idx_entries_feed ON entries(feed_id);
            CREATE INDEX IF NOT EXISTS idx_entries_published ON entries(published_at);
            "#,
        )?;
        Ok(Store {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    fn row_to_feed(row: &rusqlite::Row) -> rusqlite::Result<Feed> {
        let last_fetched_at: Option<i64> = row.get("last_fetched_at")?;
        Ok(Feed {
            id: row.get("id")?,
            url: row.get("url")?,
            title: row.get("title")?,
            last_fetched_at: last_fetched_at.and_then(|s| Utc.timestamp_opt(s, 0).single()),
            last_error: row.get("last_error")?,
            etag: row.get("etag")?,
            last_modified: row.get("last_modified")?,
        })
    }

    /// Register a new feed URL. Returns `Duplicate` if it already exists.
    pub fn add_feed(&self, url: &str) -> rusqlite::Result<AddOutcome> {
        let conn = self.conn.lock().unwrap();
        let existing: Option<i64> = conn
            .query_row("SELECT id FROM feeds WHERE url = ?1", params![url], |r| {
                r.get(0)
            })
            .optional()?;
        if existing.is_some() {
            return Ok(AddOutcome::Duplicate);
        }
        conn.execute("INSERT INTO feeds (url) VALUES (?1)", params![url])?;
        let id = conn.last_insert_rowid();
        let feed = conn.query_row("SELECT * FROM feeds WHERE id = ?1", params![id], |r| {
            Self::row_to_feed(r)
        })?;
        Ok(AddOutcome::Created(feed))
    }

    /// List all registered feeds ordered by id.
    pub fn list_feeds(&self) -> rusqlite::Result<Vec<Feed>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT * FROM feeds ORDER BY id")?;
        let feeds = stmt
            .query_map([], |r| Self::row_to_feed(r))?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(feeds)
    }

    /// Fetch a single feed by id.
    pub fn get_feed(&self, id: i64) -> rusqlite::Result<Option<Feed>> {
        let conn = self.conn.lock().unwrap();
        conn.query_row("SELECT * FROM feeds WHERE id = ?1", params![id], |r| {
            Self::row_to_feed(r)
        })
        .optional()
    }

    /// Delete a feed (and, via cascade, its entries). Returns whether a row
    /// was removed.
    pub fn delete_feed(&self, id: i64) -> rusqlite::Result<bool> {
        let conn = self.conn.lock().unwrap();
        let n = conn.execute("DELETE FROM feeds WHERE id = ?1", params![id])?;
        Ok(n > 0)
    }

    /// Count of entries for a feed.
    pub fn entry_count(&self, feed_id: i64) -> rusqlite::Result<i64> {
        let conn = self.conn.lock().unwrap();
        conn.query_row(
            "SELECT COUNT(*) FROM entries WHERE feed_id = ?1",
            params![feed_id],
            |r| r.get(0),
        )
    }

    /// Serialize a feed to its JSON representation (includes `entry_count`).
    pub fn feed_json(&self, feed: &Feed) -> rusqlite::Result<Value> {
        let count = self.entry_count(feed.id)?;
        Ok(json!({
            "id": feed.id,
            "url": feed.url,
            "title": feed.title,
            "last_fetched_at": feed.last_fetched_at.map(|d| feedlib::date::to_rfc3339_z(&d)),
            "last_error": feed.last_error,
            "entry_count": count,
        }))
    }

    /// Insert or update an entry by (feed, guid). Returns `true` when the row
    /// was newly inserted. Existing rows keep their id and `fetched_at`.
    pub fn upsert_entry(
        &self,
        feed_id: i64,
        entry: &ParsedEntry,
        fetched_at: i64,
    ) -> rusqlite::Result<bool> {
        let conn = self.conn.lock().unwrap();
        let published = entry.published_at.map(|d| d.timestamp());
        let existing: Option<i64> = conn
            .query_row(
                "SELECT id FROM entries WHERE feed_id = ?1 AND guid = ?2",
                params![feed_id, entry.guid],
                |r| r.get(0),
            )
            .optional()?;
        match existing {
            Some(id) => {
                conn.execute(
                    "UPDATE entries SET title = ?1, link = ?2, summary = ?3, published_at = ?4 \
                     WHERE id = ?5",
                    params![entry.title, entry.link, entry.summary, published, id],
                )?;
                Ok(false)
            }
            None => {
                conn.execute(
                    "INSERT INTO entries (feed_id, guid, title, link, summary, published_at, fetched_at) \
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                    params![
                        feed_id,
                        entry.guid,
                        entry.title,
                        entry.link,
                        entry.summary,
                        published,
                        fetched_at
                    ],
                )?;
                Ok(true)
            }
        }
    }

    /// Update feed metadata after a successful (200) fetch.
    pub fn record_success(
        &self,
        feed_id: i64,
        title: Option<&str>,
        etag: Option<&str>,
        last_modified: Option<&str>,
        fetched_at: i64,
    ) -> rusqlite::Result<()> {
        let conn = self.conn.lock().unwrap();
        // Title updates only when the feed provided one; otherwise keep prior.
        conn.execute(
            "UPDATE feeds SET \
                title = COALESCE(?1, title), \
                etag = ?2, \
                last_modified = ?3, \
                last_fetched_at = ?4, \
                last_error = NULL \
             WHERE id = ?5",
            params![title, etag, last_modified, fetched_at, feed_id],
        )?;
        Ok(())
    }

    /// Update feed metadata after a `304 Not Modified` (counts as success).
    pub fn record_not_modified(&self, feed_id: i64, fetched_at: i64) -> rusqlite::Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE feeds SET last_fetched_at = ?1, last_error = NULL WHERE id = ?2",
            params![fetched_at, feed_id],
        )?;
        Ok(())
    }

    /// Record a fetch/parse failure on a feed without touching its entries.
    pub fn record_error(&self, feed_id: i64, message: &str, fetched_at: i64) -> rusqlite::Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE feeds SET last_error = ?1, last_fetched_at = ?2 WHERE id = ?3",
            params![message, fetched_at, feed_id],
        )?;
        Ok(())
    }

    /// Query entries with filters, returning `(total, items)` where `total`
    /// ignores limit/offset.
    pub fn query_entries(&self, q: &EntryQuery) -> rusqlite::Result<(i64, Vec<Value>)> {
        let conn = self.conn.lock().unwrap();
        let mut where_clauses: Vec<String> = Vec::new();
        let mut args: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

        if let Some(fid) = q.feed_id {
            where_clauses.push(format!("feed_id = ?{}", args.len() + 1));
            args.push(Box::new(fid));
        }
        if q.has_time_bound {
            where_clauses.push("published_at IS NOT NULL".to_string());
        }
        if let Some(since) = q.since {
            where_clauses.push(format!("published_at >= ?{}", args.len() + 1));
            args.push(Box::new(since));
        }
        if let Some(until) = q.until {
            where_clauses.push(format!("published_at < ?{}", args.len() + 1));
            args.push(Box::new(until));
        }
        if let Some(text) = &q.q {
            where_clauses.push(format!(
                "instr(lower(title), lower(?{})) > 0",
                args.len() + 1
            ));
            args.push(Box::new(text.clone()));
        }

        let where_sql = if where_clauses.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", where_clauses.join(" AND "))
        };

        let count_sql = format!("SELECT COUNT(*) FROM entries {where_sql}");
        let arg_refs: Vec<&dyn rusqlite::types::ToSql> =
            args.iter().map(|b| b.as_ref()).collect();
        let total: i64 = conn.query_row(&count_sql, arg_refs.as_slice(), |r| r.get(0))?;

        // Ordering: published_at DESC nulls last, ties by id ASC.
        let list_sql = format!(
            "SELECT id, feed_id, guid, title, link, summary, published_at, fetched_at \
             FROM entries {where_sql} \
             ORDER BY (published_at IS NULL), published_at DESC, id ASC \
             LIMIT ?{} OFFSET ?{}",
            args.len() + 1,
            args.len() + 2
        );
        let mut list_args = args;
        list_args.push(Box::new(q.limit));
        list_args.push(Box::new(q.offset));
        let list_arg_refs: Vec<&dyn rusqlite::types::ToSql> =
            list_args.iter().map(|b| b.as_ref()).collect();

        let mut stmt = conn.prepare(&list_sql)?;
        let rows = stmt.query_map(list_arg_refs.as_slice(), |row| {
            let published_at: Option<i64> = row.get("published_at")?;
            let fetched_at: i64 = row.get("fetched_at")?;
            Ok(json!({
                "id": row.get::<_, i64>("id")?,
                "feed_id": row.get::<_, i64>("feed_id")?,
                "guid": row.get::<_, String>("guid")?,
                "title": row.get::<_, String>("title")?,
                "link": row.get::<_, Option<String>>("link")?,
                "summary": row.get::<_, Option<String>>("summary")?,
                "published_at": epoch_to_rfc3339(published_at),
                "fetched_at": epoch_to_rfc3339(Some(fetched_at)),
            }))
        })?;
        let items = rows.collect::<rusqlite::Result<Vec<_>>>()?;
        Ok((total, items))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(guid: &str, title: &str, published: Option<i64>) -> ParsedEntry {
        ParsedEntry {
            guid: guid.to_string(),
            title: title.to_string(),
            link: Some(format!("http://x/{guid}")),
            summary: Some("s".into()),
            published_at: published.and_then(|s| Utc.timestamp_opt(s, 0).single()),
        }
    }

    #[test]
    fn add_and_duplicate() {
        let s = Store::open(":memory:").unwrap();
        assert!(matches!(
            s.add_feed("http://x/f").unwrap(),
            AddOutcome::Created(_)
        ));
        assert!(matches!(
            s.add_feed("http://x/f").unwrap(),
            AddOutcome::Duplicate
        ));
    }

    #[test]
    fn upsert_dedupes_and_updates() {
        let s = Store::open(":memory:").unwrap();
        let AddOutcome::Created(f) = s.add_feed("http://x/f").unwrap() else {
            panic!()
        };
        assert!(s.upsert_entry(f.id, &entry("g1", "Old", Some(100)), 10).unwrap());
        // Re-seen: updates in place, not a new row.
        assert!(!s.upsert_entry(f.id, &entry("g1", "New", Some(200)), 99).unwrap());
        assert_eq!(s.entry_count(f.id).unwrap(), 1);
        let (_total, items) = s.query_entries(&EntryQuery { limit: 50, ..Default::default() }).unwrap();
        assert_eq!(items[0]["title"], "New");
        // fetched_at preserved from original insert.
        assert_eq!(items[0]["fetched_at"], epoch_to_rfc3339(Some(10)));
    }

    #[test]
    fn ordering_nulls_last_then_id() {
        let s = Store::open(":memory:").unwrap();
        let AddOutcome::Created(f) = s.add_feed("http://x/f").unwrap() else {
            panic!()
        };
        s.upsert_entry(f.id, &entry("a", "A", Some(100)), 1).unwrap();
        s.upsert_entry(f.id, &entry("b", "B", None), 1).unwrap();
        s.upsert_entry(f.id, &entry("c", "C", Some(300)), 1).unwrap();
        s.upsert_entry(f.id, &entry("d", "D", None), 1).unwrap();
        let (total, items) = s
            .query_entries(&EntryQuery { limit: 50, ..Default::default() })
            .unwrap();
        assert_eq!(total, 4);
        let titles: Vec<_> = items.iter().map(|i| i["title"].as_str().unwrap()).collect();
        // 300 desc, 100, then nulls last by id asc (b before d).
        assert_eq!(titles, vec!["C", "A", "B", "D"]);
    }

    #[test]
    fn window_half_open_excludes_null() {
        let s = Store::open(":memory:").unwrap();
        let AddOutcome::Created(f) = s.add_feed("http://x/f").unwrap() else {
            panic!()
        };
        s.upsert_entry(f.id, &entry("a", "A", Some(100)), 1).unwrap();
        s.upsert_entry(f.id, &entry("b", "B", Some(200)), 1).unwrap();
        s.upsert_entry(f.id, &entry("n", "N", None), 1).unwrap();
        // since=100 until=200 -> only A (200 excluded, null excluded).
        let (total, items) = s
            .query_entries(&EntryQuery {
                since: Some(100),
                until: Some(200),
                has_time_bound: true,
                limit: 50,
                ..Default::default()
            })
            .unwrap();
        assert_eq!(total, 1);
        assert_eq!(items[0]["title"], "A");
    }

    #[test]
    fn search_case_insensitive() {
        let s = Store::open(":memory:").unwrap();
        let AddOutcome::Created(f) = s.add_feed("http://x/f").unwrap() else {
            panic!()
        };
        s.upsert_entry(f.id, &entry("a", "Hello World", Some(1)), 1).unwrap();
        s.upsert_entry(f.id, &entry("b", "Goodbye", Some(2)), 1).unwrap();
        let (total, items) = s
            .query_entries(&EntryQuery {
                q: Some("hello".into()),
                limit: 50,
                ..Default::default()
            })
            .unwrap();
        assert_eq!(total, 1);
        assert_eq!(items[0]["title"], "Hello World");
    }

    #[test]
    fn delete_cascades_entries() {
        let s = Store::open(":memory:").unwrap();
        let AddOutcome::Created(f) = s.add_feed("http://x/f").unwrap() else {
            panic!()
        };
        s.upsert_entry(f.id, &entry("a", "A", Some(1)), 1).unwrap();
        assert!(s.delete_feed(f.id).unwrap());
        let (total, _) = s.query_entries(&EntryQuery { limit: 50, ..Default::default() }).unwrap();
        assert_eq!(total, 0);
    }
}
