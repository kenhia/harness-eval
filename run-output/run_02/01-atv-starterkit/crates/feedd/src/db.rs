//! SQLite-backed storage for feeds and entries.

use chrono::{DateTime, SecondsFormat, Utc};
use feedcore::ParsedFeed;
use rusqlite::{params, Connection, OptionalExtension};
use serde_json::{json, Value};

pub struct Store {
    conn: Connection,
}

/// Format an instant in the fixed-width RFC 3339 `Z` form used for storage,
/// so lexicographic string comparison matches chronological order.
pub fn fmt_utc(dt: &DateTime<Utc>) -> String {
    dt.to_rfc3339_opts(SecondsFormat::Secs, true)
}

pub fn now_utc() -> String {
    fmt_utc(&Utc::now())
}

#[derive(Debug, Default)]
pub struct EntryQuery {
    pub feed_id: Option<i64>,
    pub since: Option<String>,
    pub until: Option<String>,
    pub q: Option<String>,
    pub limit: i64,
    pub offset: i64,
}

impl Store {
    pub fn open(path: &str) -> rusqlite::Result<Self> {
        let conn = Connection::open(path)?;
        conn.pragma_update(None, "foreign_keys", "ON")?;
        conn.pragma_update(None, "journal_mode", "WAL")?;
        let store = Store { conn };
        store.init_schema()?;
        Ok(store)
    }

    fn init_schema(&self) -> rusqlite::Result<()> {
        self.conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS feeds (
                id              INTEGER PRIMARY KEY,
                url             TEXT NOT NULL UNIQUE,
                title           TEXT,
                last_fetched_at TEXT,
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
                published_at TEXT,
                fetched_at   TEXT NOT NULL,
                UNIQUE(feed_id, guid)
            );
            CREATE INDEX IF NOT EXISTS idx_entries_feed ON entries(feed_id);
            "#,
        )?;
        Ok(())
    }

    // ---- feed CRUD ---------------------------------------------------------

    pub fn add_feed(&self, url: &str) -> rusqlite::Result<i64> {
        self.conn
            .execute("INSERT INTO feeds (url) VALUES (?1)", params![url])?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn feed_id_for_url(&self, url: &str) -> rusqlite::Result<Option<i64>> {
        self.conn
            .query_row("SELECT id FROM feeds WHERE url = ?1", params![url], |r| {
                r.get(0)
            })
            .optional()
    }

    pub fn feed_exists(&self, id: i64) -> rusqlite::Result<bool> {
        Ok(self
            .conn
            .query_row("SELECT 1 FROM feeds WHERE id = ?1", params![id], |_| Ok(()))
            .optional()?
            .is_some())
    }

    pub fn delete_feed(&self, id: i64) -> rusqlite::Result<bool> {
        let n = self
            .conn
            .execute("DELETE FROM feeds WHERE id = ?1", params![id])?;
        Ok(n > 0)
    }

    pub fn feed_json(&self, id: i64) -> rusqlite::Result<Option<Value>> {
        self.conn
            .query_row(
                "SELECT id, url, title, last_fetched_at, last_error,
                        (SELECT COUNT(*) FROM entries e WHERE e.feed_id = feeds.id)
                 FROM feeds WHERE id = ?1",
                params![id],
                |r| {
                    Ok(json!({
                        "id": r.get::<_, i64>(0)?,
                        "url": r.get::<_, String>(1)?,
                        "title": r.get::<_, Option<String>>(2)?,
                        "last_fetched_at": r.get::<_, Option<String>>(3)?,
                        "last_error": r.get::<_, Option<String>>(4)?,
                        "entry_count": r.get::<_, i64>(5)?,
                    }))
                },
            )
            .optional()
    }

    pub fn all_feeds_json(&self) -> rusqlite::Result<Vec<Value>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, url, title, last_fetched_at, last_error,
                    (SELECT COUNT(*) FROM entries e WHERE e.feed_id = feeds.id)
             FROM feeds ORDER BY id ASC",
        )?;
        let rows = stmt.query_map([], |r| {
            Ok(json!({
                "id": r.get::<_, i64>(0)?,
                "url": r.get::<_, String>(1)?,
                "title": r.get::<_, Option<String>>(2)?,
                "last_fetched_at": r.get::<_, Option<String>>(3)?,
                "last_error": r.get::<_, Option<String>>(4)?,
                "entry_count": r.get::<_, i64>(5)?,
            }))
        })?;
        rows.collect()
    }

    pub fn all_feed_ids(&self) -> rusqlite::Result<Vec<i64>> {
        let mut stmt = self.conn.prepare("SELECT id FROM feeds ORDER BY id ASC")?;
        let rows = stmt.query_map([], |r| r.get::<_, i64>(0))?;
        rows.collect()
    }

    pub fn feed_url(&self, id: i64) -> rusqlite::Result<Option<String>> {
        self.conn
            .query_row("SELECT url FROM feeds WHERE id = ?1", params![id], |r| {
                r.get(0)
            })
            .optional()
    }

    pub fn conditional_headers(
        &self,
        id: i64,
    ) -> rusqlite::Result<(Option<String>, Option<String>)> {
        self.conn
            .query_row(
                "SELECT etag, last_modified FROM feeds WHERE id = ?1",
                params![id],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .optional()
            .map(|o| o.unwrap_or((None, None)))
    }

    // ---- refresh bookkeeping ----------------------------------------------

    /// Record a failed fetch: sets `last_error` and `last_fetched_at`.
    pub fn record_error(&self, id: i64, err: &str) -> rusqlite::Result<()> {
        self.conn.execute(
            "UPDATE feeds SET last_error = ?2, last_fetched_at = ?3 WHERE id = ?1",
            params![id, err, now_utc()],
        )?;
        Ok(())
    }

    /// Record a successful fetch that yielded no body change (HTTP 304).
    pub fn record_not_modified(&self, id: i64) -> rusqlite::Result<()> {
        self.conn.execute(
            "UPDATE feeds SET last_error = NULL, last_fetched_at = ?2 WHERE id = ?1",
            params![id, now_utc()],
        )?;
        Ok(())
    }

    /// Apply a freshly parsed feed: upsert entries, update feed metadata.
    /// Returns the number of newly inserted entries.
    pub fn apply_fetch(
        &mut self,
        id: i64,
        feed: &ParsedFeed,
        etag: Option<&str>,
        last_modified: Option<&str>,
    ) -> rusqlite::Result<usize> {
        let now = now_utc();
        let tx = self.conn.transaction()?;
        let mut new_entries = 0usize;
        for e in &feed.entries {
            let title = e.title.clone().unwrap_or_else(|| "(untitled)".to_string());
            let published = e.published_at.as_ref().map(fmt_utc);
            let existing: Option<i64> = tx
                .query_row(
                    "SELECT id FROM entries WHERE feed_id = ?1 AND guid = ?2",
                    params![id, e.guid],
                    |r| r.get(0),
                )
                .optional()?;
            match existing {
                Some(_) => {
                    tx.execute(
                        "UPDATE entries SET title = ?3, link = ?4, summary = ?5, published_at = ?6
                         WHERE feed_id = ?1 AND guid = ?2",
                        params![id, e.guid, title, e.link, e.summary, published],
                    )?;
                }
                None => {
                    tx.execute(
                        "INSERT INTO entries (feed_id, guid, title, link, summary, published_at, fetched_at)
                         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                        params![id, e.guid, title, e.link, e.summary, published, now],
                    )?;
                    new_entries += 1;
                }
            }
        }
        tx.execute(
            "UPDATE feeds SET title = COALESCE(?2, title), last_error = NULL,
                    last_fetched_at = ?3, etag = ?4, last_modified = ?5 WHERE id = ?1",
            params![id, feed.title, now, etag, last_modified],
        )?;
        tx.commit()?;
        Ok(new_entries)
    }

    // ---- entry query -------------------------------------------------------

    pub fn query_entries(&self, q: &EntryQuery) -> rusqlite::Result<Value> {
        let mut where_clauses: Vec<String> = Vec::new();
        let mut binds: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        if let Some(fid) = q.feed_id {
            where_clauses.push(format!("feed_id = ?{}", binds.len() + 1));
            binds.push(Box::new(fid));
        }
        if let Some(since) = &q.since {
            where_clauses.push(format!(
                "published_at IS NOT NULL AND published_at >= ?{}",
                binds.len() + 1
            ));
            binds.push(Box::new(since.clone()));
        }
        if let Some(until) = &q.until {
            where_clauses.push(format!(
                "published_at IS NOT NULL AND published_at < ?{}",
                binds.len() + 1
            ));
            binds.push(Box::new(until.clone()));
        }
        if let Some(text) = &q.q {
            where_clauses.push(format!(
                "instr(lower(title), lower(?{})) > 0",
                binds.len() + 1
            ));
            binds.push(Box::new(text.clone()));
        }

        let where_sql = if where_clauses.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", where_clauses.join(" AND "))
        };

        let bind_refs: Vec<&dyn rusqlite::ToSql> = binds.iter().map(|b| b.as_ref()).collect();

        let total: i64 = self.conn.query_row(
            &format!("SELECT COUNT(*) FROM entries {where_sql}"),
            bind_refs.as_slice(),
            |r| r.get(0),
        )?;

        let sql = format!(
            "SELECT id, feed_id, guid, title, link, summary, published_at, fetched_at
             FROM entries {where_sql}
             ORDER BY (published_at IS NULL) ASC, published_at DESC, id ASC
             LIMIT ?{} OFFSET ?{}",
            binds.len() + 1,
            binds.len() + 2
        );
        let mut all_binds = bind_refs;
        all_binds.push(&q.limit);
        all_binds.push(&q.offset);

        let mut stmt = self.conn.prepare(&sql)?;
        let items: Vec<Value> = stmt
            .query_map(all_binds.as_slice(), |r| {
                Ok(json!({
                    "id": r.get::<_, i64>(0)?,
                    "feed_id": r.get::<_, i64>(1)?,
                    "guid": r.get::<_, String>(2)?,
                    "title": r.get::<_, String>(3)?,
                    "link": r.get::<_, Option<String>>(4)?,
                    "summary": r.get::<_, Option<String>>(5)?,
                    "published_at": r.get::<_, Option<String>>(6)?,
                    "fetched_at": r.get::<_, String>(7)?,
                }))
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;

        Ok(json!({ "total": total, "items": items }))
    }
}
