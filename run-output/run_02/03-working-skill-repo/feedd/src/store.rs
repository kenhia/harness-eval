//! SQLite storage for feeds and entries.
//!
//! A single [`Store`] wraps one connection behind a mutex. Network fetches
//! happen outside the lock; only short read/write transactions hold it.
//! `published_at` is stored as an RFC 3339 `Z` string for the API and as an
//! integer millisecond column (`published_ms`) for correct ordering and
//! half-open window comparisons independent of string precision.

use std::sync::Mutex;

use feedcore::api::{EntryDto, FeedDto};
use feedcore::dates::to_rfc3339_z;
use feedcore::ParsedFeed;
use rusqlite::types::Value;
use rusqlite::{params, params_from_iter, Connection, OptionalExtension};

pub struct Store {
    conn: Mutex<Connection>,
}

#[derive(Debug)]
pub enum AddError {
    Duplicate,
    Db(rusqlite::Error),
}

impl From<rusqlite::Error> for AddError {
    fn from(e: rusqlite::Error) -> Self {
        AddError::Db(e)
    }
}

/// Conditional-GET validators stored per feed.
#[derive(Debug, Clone, Default)]
pub struct FetchInfo {
    pub url: String,
    pub etag: Option<String>,
    pub last_modified: Option<String>,
}

#[derive(Debug, Default)]
pub struct EntryFilter {
    pub feed_id: Option<i64>,
    pub since_ms: Option<i64>,
    pub until_ms: Option<i64>,
    pub q: Option<String>,
    pub limit: i64,
    pub offset: i64,
}

fn now() -> String {
    to_rfc3339_z(chrono::Utc::now())
}

impl Store {
    pub fn open(path: &str) -> rusqlite::Result<Self> {
        let conn = Connection::open(path)?;
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "foreign_keys", "ON")?;
        conn.execute_batch(SCHEMA)?;
        Ok(Store {
            conn: Mutex::new(conn),
        })
    }

    /// Register a new feed URL. Returns the created feed or `AddError::Duplicate`.
    pub fn add_feed(&self, url: &str) -> Result<FeedDto, AddError> {
        let conn = self.conn.lock().unwrap();
        let existing: Option<i64> = conn
            .query_row("SELECT id FROM feeds WHERE url = ?1", params![url], |r| {
                r.get(0)
            })
            .optional()?;
        if existing.is_some() {
            return Err(AddError::Duplicate);
        }
        conn.execute("INSERT INTO feeds (url) VALUES (?1)", params![url])?;
        let id = conn.last_insert_rowid();
        Ok(feed_dto(&conn, id)?.expect("row just inserted"))
    }

    pub fn list_feeds(&self) -> rusqlite::Result<Vec<FeedDto>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(&format!("{FEED_SELECT} ORDER BY f.id ASC"))?;
        let rows = stmt.query_map([], map_feed)?;
        rows.collect()
    }

    pub fn get_feed(&self, id: i64) -> rusqlite::Result<Option<FeedDto>> {
        let conn = self.conn.lock().unwrap();
        feed_dto(&conn, id)
    }

    /// Delete a feed and its entries (via cascade). Returns whether it existed.
    pub fn delete_feed(&self, id: i64) -> rusqlite::Result<bool> {
        let conn = self.conn.lock().unwrap();
        let n = conn.execute("DELETE FROM feeds WHERE id = ?1", params![id])?;
        Ok(n > 0)
    }

    pub fn fetch_info(&self, id: i64) -> rusqlite::Result<Option<FetchInfo>> {
        let conn = self.conn.lock().unwrap();
        conn.query_row(
            "SELECT url, etag, last_modified FROM feeds WHERE id = ?1",
            params![id],
            |r| {
                Ok(FetchInfo {
                    url: r.get(0)?,
                    etag: r.get(1)?,
                    last_modified: r.get(2)?,
                })
            },
        )
        .optional()
    }

    pub fn all_feed_ids(&self) -> rusqlite::Result<Vec<i64>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id FROM feeds ORDER BY id ASC")?;
        let ids = stmt.query_map([], |r| r.get(0))?;
        ids.collect()
    }

    /// Record a failed fetch/parse without touching entries or validators.
    pub fn record_error(&self, id: i64, message: &str) -> rusqlite::Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE feeds SET last_error = ?2, last_fetched_at = ?3 WHERE id = ?1",
            params![id, message, now()],
        )?;
        Ok(())
    }

    /// Record a successful 304: counts as a fetch, clears error, keeps entries.
    pub fn record_not_modified(&self, id: i64) -> rusqlite::Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE feeds SET last_error = NULL, last_fetched_at = ?2 WHERE id = ?1",
            params![id, now()],
        )?;
        Ok(())
    }

    /// Apply a parsed feed: upsert entries by identity, update feed metadata and
    /// conditional-GET validators. Returns the count of newly inserted entries.
    pub fn apply_parsed(
        &self,
        id: i64,
        feed: &ParsedFeed,
        etag: Option<String>,
        last_modified: Option<String>,
    ) -> rusqlite::Result<i64> {
        let mut conn = self.conn.lock().unwrap();
        let tx = conn.transaction()?;
        let ts = now();
        let mut new_entries = 0i64;
        for entry in &feed.entries {
            let published_at = entry.published_at.map(to_rfc3339_z);
            let published_ms = entry.published_at.map(|d| d.timestamp_millis());
            let existing: Option<i64> = tx
                .query_row(
                    "SELECT id FROM entries WHERE feed_id = ?1 AND guid = ?2",
                    params![id, entry.guid],
                    |r| r.get(0),
                )
                .optional()?;
            if existing.is_some() {
                tx.execute(
                    "UPDATE entries SET title = ?3, link = ?4, summary = ?5, \
                     published_at = ?6, published_ms = ?7 WHERE feed_id = ?1 AND guid = ?2",
                    params![
                        id,
                        entry.guid,
                        entry.title,
                        entry.link,
                        entry.summary,
                        published_at,
                        published_ms
                    ],
                )?;
            } else {
                tx.execute(
                    "INSERT INTO entries \
                     (feed_id, guid, title, link, summary, published_at, published_ms, fetched_at) \
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                    params![
                        id,
                        entry.guid,
                        entry.title,
                        entry.link,
                        entry.summary,
                        published_at,
                        published_ms,
                        ts
                    ],
                )?;
                new_entries += 1;
            }
        }
        // Update feed title only when the parsed feed provides one.
        if let Some(title) = &feed.title {
            tx.execute(
                "UPDATE feeds SET title = ?2 WHERE id = ?1",
                params![id, title],
            )?;
        }
        tx.execute(
            "UPDATE feeds SET etag = ?2, last_modified = ?3, last_error = NULL, \
             last_fetched_at = ?4 WHERE id = ?1",
            params![id, etag, last_modified, ts],
        )?;
        tx.commit()?;
        Ok(new_entries)
    }

    /// Query entries with filters; returns `(total_ignoring_paging, page)`.
    pub fn query_entries(&self, filter: &EntryFilter) -> rusqlite::Result<(i64, Vec<EntryDto>)> {
        let conn = self.conn.lock().unwrap();
        let mut clauses: Vec<&str> = Vec::new();
        let mut args: Vec<Value> = Vec::new();
        if let Some(fid) = filter.feed_id {
            clauses.push("feed_id = ?");
            args.push(Value::Integer(fid));
        }
        if let Some(since) = filter.since_ms {
            clauses.push("published_ms IS NOT NULL AND published_ms >= ?");
            args.push(Value::Integer(since));
        }
        if let Some(until) = filter.until_ms {
            clauses.push("published_ms IS NOT NULL AND published_ms < ?");
            args.push(Value::Integer(until));
        }
        if let Some(q) = &filter.q {
            clauses.push("instr(lower(title), lower(?)) > 0");
            args.push(Value::Text(q.clone()));
        }
        let where_sql = if clauses.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", clauses.join(" AND "))
        };

        let total: i64 = conn.query_row(
            &format!("SELECT COUNT(*) FROM entries {where_sql}"),
            params_from_iter(args.iter()),
            |r| r.get(0),
        )?;

        let sql = format!(
            "SELECT id, feed_id, guid, title, link, summary, published_at, fetched_at \
             FROM entries {where_sql} \
             ORDER BY published_ms IS NULL, published_ms DESC, id ASC \
             LIMIT ? OFFSET ?"
        );
        let mut page_args = args.clone();
        page_args.push(Value::Integer(filter.limit));
        page_args.push(Value::Integer(filter.offset));
        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map(params_from_iter(page_args.iter()), map_entry)?;
        let items: rusqlite::Result<Vec<EntryDto>> = rows.collect();
        Ok((total, items?))
    }
}

const SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS feeds (
    id              INTEGER PRIMARY KEY,
    url             TEXT NOT NULL UNIQUE,
    title           TEXT,
    etag            TEXT,
    last_modified   TEXT,
    last_fetched_at TEXT,
    last_error      TEXT
);
CREATE TABLE IF NOT EXISTS entries (
    id           INTEGER PRIMARY KEY,
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
"#;

const FEED_SELECT: &str = "SELECT f.id, f.url, f.title, f.last_fetched_at, f.last_error, \
     (SELECT COUNT(*) FROM entries e WHERE e.feed_id = f.id) AS entry_count FROM feeds f";

fn map_feed(r: &rusqlite::Row) -> rusqlite::Result<FeedDto> {
    Ok(FeedDto {
        id: r.get(0)?,
        url: r.get(1)?,
        title: r.get(2)?,
        last_fetched_at: r.get(3)?,
        last_error: r.get(4)?,
        entry_count: r.get(5)?,
    })
}

fn map_entry(r: &rusqlite::Row) -> rusqlite::Result<EntryDto> {
    Ok(EntryDto {
        id: r.get(0)?,
        feed_id: r.get(1)?,
        guid: r.get(2)?,
        title: r.get(3)?,
        link: r.get(4)?,
        summary: r.get(5)?,
        published_at: r.get(6)?,
        fetched_at: r.get(7)?,
    })
}

fn feed_dto(conn: &Connection, id: i64) -> rusqlite::Result<Option<FeedDto>> {
    conn.query_row(
        &format!("{FEED_SELECT} WHERE f.id = ?1"),
        params![id],
        map_feed,
    )
    .optional()
}
