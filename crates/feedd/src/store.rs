//! SQLite storage.
//!
//! # Concurrency
//!
//! The connection lives behind a [`std::sync::Mutex`], and every method here is
//! **synchronous**. That is deliberate, and `tokio::sync::Mutex` must not be
//! substituted:
//!
//! - `std::sync::MutexGuard` is `!Send`, so holding one across an `.await` in an
//!   axum handler is a *compile error*, not a lint. That statically guarantees
//!   the invariant this module depends on — network I/O never happens while the
//!   database lock is held.
//! - `tokio::sync::Mutex` is designed to be held across awaits, so swapping it
//!   in would silently delete that guarantee (and clippy's `await_holding_lock`
//!   deliberately ignores it, so nothing would flag the regression).
//!
//! The cost is that handlers do blocking SQLite I/O on a runtime worker while
//! holding a global lock. At feed-aggregator scale — a handful of feeds, small
//! documents, microsecond queries — that is the right trade for a design whose
//! central invariant the compiler enforces for free.

use std::path::Path;
use std::sync::Mutex;

use chrono::{DateTime, Utc};
use feedhub_core::datetime::{format_rfc3339, to_millis};
use feedhub_core::model::{Entry, EntryPage, Feed, ParsedEntry};
use rusqlite::{params, Connection, OptionalExtension, TransactionBehavior};

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("a feed with that URL is already registered")]
    DuplicateUrl,
    /// The feed disappeared between being read and being written — it was
    /// deleted while its refresh was in flight.
    #[error("the feed no longer exists")]
    FeedGone,
    #[error(transparent)]
    Sqlite(#[from] rusqlite::Error),
}

pub type Result<T> = std::result::Result<T, StoreError>;

/// What a successful (200) fetch produced.
#[derive(Debug, Clone)]
pub struct FetchSuccess {
    pub title: Option<String>,
    pub entries: Vec<ParsedEntry>,
    pub etag: Option<String>,
    pub last_modified: Option<String>,
}

/// How many entries a fetch inserted versus updated in place.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ApplyCounts {
    pub new: usize,
    pub updated: usize,
}

/// The conditional-GET validators stored for a feed.
#[derive(Debug, Clone)]
pub struct FetchState {
    pub id: i64,
    pub url: String,
    pub etag: Option<String>,
    pub last_modified: Option<String>,
}

/// Filters for `GET /api/entries`. Instants are epoch milliseconds.
#[derive(Debug, Clone, Default)]
pub struct EntryQuery {
    pub feed_id: Option<i64>,
    /// Inclusive lower bound.
    pub since: Option<i64>,
    /// Exclusive upper bound.
    pub until: Option<i64>,
    /// Case-insensitive substring of the title.
    pub q: Option<String>,
    pub limit: i64,
    pub offset: i64,
}

const SCHEMA: &str = r#"
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
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    feed_id         INTEGER NOT NULL REFERENCES feeds(id) ON DELETE CASCADE,
    guid            TEXT NOT NULL,
    title           TEXT NOT NULL,
    link            TEXT,
    summary         TEXT,
    -- RFC 3339 with Z; what the API returns.
    published_at    TEXT,
    -- Epoch milliseconds; the ordering and range key. See feedhub_core::datetime.
    published_at_ms INTEGER,
    fetched_at      TEXT NOT NULL,
    UNIQUE(feed_id, guid)
);

CREATE INDEX IF NOT EXISTS entries_feed_idx  ON entries(feed_id);
CREATE INDEX IF NOT EXISTS entries_order_idx ON entries(published_at_ms DESC, id ASC);
"#;

/// The pinned ordering: newest first, nulls last, ties broken by entry id.
///
/// `(published_at_ms IS NULL)` yields 0/1 in SQLite, so ascending on it puts
/// non-null dates first. Written out rather than using `NULLS LAST` so the
/// intent is legible at the call site.
const ORDER_BY: &str = "ORDER BY (published_at_ms IS NULL) ASC, published_at_ms DESC, id ASC";

const FEED_COLUMNS: &str = "SELECT f.id, f.url, f.title, f.last_fetched_at, f.last_error, \
     (SELECT COUNT(*) FROM entries e WHERE e.feed_id = f.id) AS entry_count FROM feeds f";

pub struct Store {
    conn: Mutex<Connection>,
}

fn row_to_feed(row: &rusqlite::Row<'_>) -> rusqlite::Result<Feed> {
    Ok(Feed {
        id: row.get(0)?,
        url: row.get(1)?,
        title: row.get(2)?,
        last_fetched_at: row.get(3)?,
        last_error: row.get(4)?,
        entry_count: row.get(5)?,
    })
}

fn row_to_entry(row: &rusqlite::Row<'_>) -> rusqlite::Result<Entry> {
    Ok(Entry {
        id: row.get(0)?,
        feed_id: row.get(1)?,
        guid: row.get(2)?,
        title: row.get(3)?,
        link: row.get(4)?,
        summary: row.get(5)?,
        published_at: row.get(6)?,
        fetched_at: row.get(7)?,
    })
}

fn is_unique_violation(e: &rusqlite::Error) -> bool {
    matches!(
        e,
        rusqlite::Error::SqliteFailure(inner, _)
            if inner.code == rusqlite::ErrorCode::ConstraintViolation
    )
}

impl Store {
    /// Open (creating if needed) the database at `path`.
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let conn = Connection::open(path)?;
        Self::init(conn)
    }

    /// An in-memory store, for tests.
    pub fn open_in_memory() -> Result<Self> {
        Self::init(Connection::open_in_memory()?)
    }

    fn init(conn: Connection) -> Result<Self> {
        // SQLite defaults foreign_keys OFF, which would make ON DELETE CASCADE
        // silently do nothing and orphan every entry of a deleted feed.
        conn.pragma_update(None, "foreign_keys", "ON")?;
        conn.busy_timeout(std::time::Duration::from_secs(5))?;
        conn.execute_batch(SCHEMA)?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Connection> {
        self.conn.lock().expect("store mutex poisoned")
    }

    /// Register a feed. Fails with [`StoreError::DuplicateUrl`] if the URL is
    /// already registered.
    pub fn add_feed(&self, url: &str) -> Result<Feed> {
        let conn = self.lock();
        conn.execute("INSERT INTO feeds (url) VALUES (?1)", params![url])
            .map_err(|e| {
                if is_unique_violation(&e) {
                    StoreError::DuplicateUrl
                } else {
                    StoreError::Sqlite(e)
                }
            })?;
        let id = conn.last_insert_rowid();
        let sql = format!("{FEED_COLUMNS} WHERE f.id = ?1");
        Ok(conn.query_row(&sql, params![id], row_to_feed)?)
    }

    pub fn list_feeds(&self) -> Result<Vec<Feed>> {
        let conn = self.lock();
        let sql = format!("{FEED_COLUMNS} ORDER BY f.id ASC");
        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map([], row_to_feed)?;
        Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
    }

    pub fn get_feed(&self, id: i64) -> Result<Option<Feed>> {
        let conn = self.lock();
        let sql = format!("{FEED_COLUMNS} WHERE f.id = ?1");
        Ok(conn.query_row(&sql, params![id], row_to_feed).optional()?)
    }

    /// Delete a feed. Its entries go with it, via ON DELETE CASCADE.
    /// Returns false if no such feed existed.
    pub fn delete_feed(&self, id: i64) -> Result<bool> {
        let conn = self.lock();
        Ok(conn.execute("DELETE FROM feeds WHERE id = ?1", params![id])? > 0)
    }

    /// The URL and conditional-GET validators for one feed.
    pub fn fetch_state(&self, id: i64) -> Result<Option<FetchState>> {
        let conn = self.lock();
        Ok(conn
            .query_row(
                "SELECT id, url, etag, last_modified FROM feeds WHERE id = ?1",
                params![id],
                |row| {
                    Ok(FetchState {
                        id: row.get(0)?,
                        url: row.get(1)?,
                        etag: row.get(2)?,
                        last_modified: row.get(3)?,
                    })
                },
            )
            .optional()?)
    }

    /// Fetch state for every feed, in id order.
    pub fn all_fetch_states(&self) -> Result<Vec<FetchState>> {
        let conn = self.lock();
        let mut stmt =
            conn.prepare("SELECT id, url, etag, last_modified FROM feeds ORDER BY id ASC")?;
        let rows = stmt.query_map([], |row| {
            Ok(FetchState {
                id: row.get(0)?,
                url: row.get(1)?,
                etag: row.get(2)?,
                last_modified: row.get(3)?,
            })
        })?;
        Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
    }

    /// Apply a successful 200 fetch: upsert entries and refresh feed metadata,
    /// in one transaction.
    ///
    /// Returns how many entries were newly inserted versus updated in place.
    pub fn apply_success(
        &self,
        feed_id: i64,
        fetch: &FetchSuccess,
        fetched_at: DateTime<Utc>,
    ) -> Result<ApplyCounts> {
        let mut conn = self.lock();
        // One transaction per refresh: a failure partway through must not leave
        // half the entries committed with a count that claims otherwise.
        let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let stamp = format_rfc3339(fetched_at);
        let mut counts = ApplyCounts::default();

        for entry in &fetch.entries {
            let published_at = entry.published_at.map(format_rfc3339);
            let published_ms = entry.published_at.map(to_millis);

            // INSERT OR IGNORE is what discriminates a true insert from a
            // repeat: it reports 1 row changed on insert and 0 on conflict.
            //
            // The obvious-looking alternative — INSERT ... ON CONFLICT DO
            // UPDATE, then read changes() — cannot work: SQLite reports 1 row
            // changed for *both* branches, so new_entries would equal the
            // feed's whole item count on every single refresh, forever.
            let inserted = tx.execute(
                "INSERT OR IGNORE INTO entries
                   (feed_id, guid, title, link, summary, published_at, published_at_ms, fetched_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    feed_id,
                    entry.guid,
                    entry.title,
                    entry.link,
                    entry.summary,
                    published_at,
                    published_ms,
                    stamp,
                ],
            )?;

            if inserted == 1 {
                counts.new += 1;
                continue;
            }

            // Known identity: update in place. `id` and `fetched_at` are
            // deliberately absent from the SET list — the entry keeps both.
            let updated = tx.execute(
                "UPDATE entries
                    SET title = ?1, link = ?2, summary = ?3,
                        published_at = ?4, published_at_ms = ?5
                  WHERE feed_id = ?6 AND guid = ?7",
                params![
                    entry.title,
                    entry.link,
                    entry.summary,
                    published_at,
                    published_ms,
                    feed_id,
                    entry.guid,
                ],
            )?;
            counts.updated += updated;
        }

        // A feed that omits its title keeps the one we already had; a failed or
        // title-less fetch must not blank it.
        let touched = tx.execute(
            "UPDATE feeds
                SET title = COALESCE(?1, title), last_fetched_at = ?2,
                    last_error = NULL, etag = ?3, last_modified = ?4
              WHERE id = ?5",
            params![fetch.title, stamp, fetch.etag, fetch.last_modified, feed_id],
        )?;
        // No row means the feed was deleted mid-refresh. Reporting success for
        // a feed that no longer exists would be a lie — and with no entries in
        // the document, nothing above would have tripped the foreign key.
        if touched != 1 {
            return Err(StoreError::FeedGone);
        }

        tx.commit()?;
        Ok(counts)
    }

    /// Apply a 304: entries untouched, and it counts as a successful fetch.
    pub fn apply_not_modified(&self, feed_id: i64, fetched_at: DateTime<Utc>) -> Result<()> {
        let conn = self.lock();
        let touched = conn.execute(
            "UPDATE feeds SET last_fetched_at = ?1, last_error = NULL WHERE id = ?2",
            params![format_rfc3339(fetched_at), feed_id],
        )?;
        // A 304 never reaches the entries table, so the foreign key cannot
        // catch a mid-refresh delete here. This is the only thing that does.
        if touched != 1 {
            return Err(StoreError::FeedGone);
        }
        Ok(())
    }

    /// Record a fetch or parse failure against one feed.
    ///
    /// Only touches that feed's row: a broken feed never disturbs its
    /// neighbours. Title and entries are left intact.
    pub fn apply_error(
        &self,
        feed_id: i64,
        message: &str,
        fetched_at: DateTime<Utc>,
    ) -> Result<()> {
        let conn = self.lock();
        conn.execute(
            "UPDATE feeds SET last_fetched_at = ?1, last_error = ?2 WHERE id = ?3",
            params![format_rfc3339(fetched_at), message, feed_id],
        )?;
        Ok(())
    }

    /// Query entries. `total` counts matches ignoring limit/offset.
    pub fn query_entries(&self, query: &EntryQuery) -> Result<EntryPage> {
        let mut where_sql = String::from(" WHERE 1 = 1");
        let mut args: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        if let Some(feed_id) = query.feed_id {
            where_sql.push_str(" AND feed_id = ?");
            args.push(Box::new(feed_id));
        }
        // A NULL published_at_ms makes these comparisons NULL, never true, so
        // dated-window queries drop undated entries without a special case —
        // which is exactly the pinned rule.
        if let Some(since) = query.since {
            where_sql.push_str(" AND published_at_ms >= ?");
            args.push(Box::new(since));
        }
        if let Some(until) = query.until {
            where_sql.push_str(" AND published_at_ms < ?");
            args.push(Box::new(until));
        }
        if let Some(q) = &query.q {
            // instr/lower rather than LIKE '%..%': LIKE would treat '%' and '_'
            // in the search term as wildcards. SQLite's lower() is ASCII-only,
            // which is the pinned folding rule.
            where_sql.push_str(" AND instr(lower(title), lower(?)) > 0");
            args.push(Box::new(q.clone()));
        }

        let conn = self.lock();

        let count_sql = format!("SELECT COUNT(*) FROM entries{where_sql}");
        let arg_refs: Vec<&dyn rusqlite::ToSql> = args.iter().map(AsRef::as_ref).collect();
        let total: i64 = conn.query_row(&count_sql, arg_refs.as_slice(), |row| row.get(0))?;

        let items_sql = format!(
            "SELECT id, feed_id, guid, title, link, summary, published_at, fetched_at \
             FROM entries{where_sql} {ORDER_BY} LIMIT ? OFFSET ?"
        );
        let mut page_args = args;
        page_args.push(Box::new(query.limit));
        page_args.push(Box::new(query.offset));
        let page_refs: Vec<&dyn rusqlite::ToSql> = page_args.iter().map(AsRef::as_ref).collect();

        let mut stmt = conn.prepare(&items_sql)?;
        let rows = stmt.query_map(page_refs.as_slice(), row_to_entry)?;
        let items = rows.collect::<rusqlite::Result<Vec<_>>>()?;

        Ok(EntryPage { total, items })
    }
}
