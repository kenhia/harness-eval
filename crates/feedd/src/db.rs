//! SQLite storage.
//!
//! Instants are stored as fixed-width RFC 3339 UTC strings (see
//! `feedhub_core::dates`), which makes lexicographic comparison in SQL the same
//! as chronological comparison — so the `since`/`until` window and the
//! `published_at` ordering are both plain SQL.

use std::path::Path;

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use feedhub_core::api::{Entry, EntriesPage, Feed};
use feedhub_core::{ParsedFeed, format_utc};
use rusqlite::types::Value;
use rusqlite::{Connection, OptionalExtension, params, params_from_iter};

/// Why [`insert_feed`] did not add the feed.
#[derive(Debug)]
pub enum InsertFeedError {
    /// The URL is already registered.
    Duplicate,
    Db(anyhow::Error),
}

impl From<rusqlite::Error> for InsertFeedError {
    fn from(e: rusqlite::Error) -> Self {
        InsertFeedError::Db(e.into())
    }
}

/// Open (creating if needed) the database at `path` and bring the schema up to
/// date.
pub fn open(path: &Path) -> Result<Connection> {
    let conn = Connection::open(path)
        .with_context(|| format!("cannot open database {}", path.display()))?;

    // WAL keeps the background poller's writes from blocking API reads.
    let _: String = conn
        .query_row("PRAGMA journal_mode = WAL", [], |row| row.get(0))
        .context("cannot enable WAL")?;
    conn.pragma_update(None, "foreign_keys", "ON")?;
    conn.pragma_update(None, "busy_timeout", 5_000)?;
    migrate(&conn)?;
    Ok(conn)
}

fn migrate(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS feeds (
            id              INTEGER PRIMARY KEY AUTOINCREMENT,
            url             TEXT NOT NULL UNIQUE,
            title           TEXT,
            last_fetched_at TEXT,
            last_error      TEXT,
            etag            TEXT,
            last_modified   TEXT,
            created_at      TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS entries (
            id           INTEGER PRIMARY KEY AUTOINCREMENT,
            feed_id      INTEGER NOT NULL REFERENCES feeds(id) ON DELETE CASCADE,
            guid         TEXT NOT NULL,
            title        TEXT NOT NULL,
            link         TEXT,
            summary      TEXT,
            published_at TEXT,
            fetched_at   TEXT NOT NULL,
            UNIQUE (feed_id, guid)
        );

        CREATE INDEX IF NOT EXISTS entries_feed_idx ON entries (feed_id);
        CREATE INDEX IF NOT EXISTS entries_published_idx ON entries (published_at);
        "#,
    )
    .context("cannot create schema")?;
    Ok(())
}

const FEED_COLUMNS: &str = "f.id, f.url, f.title, f.last_fetched_at, f.last_error,
     (SELECT COUNT(*) FROM entries e WHERE e.feed_id = f.id) AS entry_count";

fn feed_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Feed> {
    Ok(Feed {
        id: row.get(0)?,
        url: row.get(1)?,
        title: row.get(2)?,
        last_fetched_at: row.get(3)?,
        last_error: row.get(4)?,
        entry_count: row.get(5)?,
    })
}

/// Register a new feed.
pub fn insert_feed(
    conn: &Connection,
    url: &str,
    now: DateTime<Utc>,
) -> Result<Feed, InsertFeedError> {
    // The UNIQUE index on url is what actually decides this, so a concurrent
    // add of the same URL loses the race rather than inserting a duplicate.
    let inserted = conn.execute(
        "INSERT OR IGNORE INTO feeds (url, created_at) VALUES (?1, ?2)",
        params![url, format_utc(now)],
    )?;
    if inserted == 0 {
        return Err(InsertFeedError::Duplicate);
    }

    let id = conn.last_insert_rowid();
    get_feed(conn, id)
        .map_err(InsertFeedError::Db)?
        .ok_or_else(|| InsertFeedError::Db(anyhow::anyhow!("feed {id} vanished after insert")))
}

pub fn list_feeds(conn: &Connection) -> Result<Vec<Feed>> {
    let sql = format!("SELECT {FEED_COLUMNS} FROM feeds f ORDER BY f.id");
    let mut stmt = conn.prepare(&sql)?;
    let feeds = stmt
        .query_map([], feed_from_row)?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(feeds)
}

pub fn get_feed(conn: &Connection, id: i64) -> Result<Option<Feed>> {
    let sql = format!("SELECT {FEED_COLUMNS} FROM feeds f WHERE f.id = ?1");
    let feed = conn
        .query_row(&sql, params![id], feed_from_row)
        .optional()?;
    Ok(feed)
}

/// Delete a feed and, by way of the foreign key, its entries. Returns whether
/// the feed existed.
pub fn delete_feed(conn: &Connection, id: i64) -> Result<bool> {
    let deleted = conn.execute("DELETE FROM feeds WHERE id = ?1", params![id])?;
    Ok(deleted > 0)
}

pub fn feed_ids(conn: &Connection) -> Result<Vec<i64>> {
    let mut stmt = conn.prepare("SELECT id FROM feeds ORDER BY id")?;
    let ids = stmt
        .query_map([], |row| row.get(0))?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(ids)
}

/// What a refresh needs to know before going to the network.
pub struct FetchState {
    pub url: String,
    pub etag: Option<String>,
    pub last_modified: Option<String>,
}

pub fn fetch_state(conn: &Connection, id: i64) -> Result<Option<FetchState>> {
    let state = conn
        .query_row(
            "SELECT url, etag, last_modified FROM feeds WHERE id = ?1",
            params![id],
            |row| {
                Ok(FetchState {
                    url: row.get(0)?,
                    etag: row.get(1)?,
                    last_modified: row.get(2)?,
                })
            },
        )
        .optional()?;
    Ok(state)
}

/// Record a failed fetch or parse against one feed. `last_fetched_at` is left
/// alone: it means "last *successful* fetch".
pub fn record_error(conn: &Connection, id: i64, error: &str) -> Result<()> {
    conn.execute(
        "UPDATE feeds SET last_error = ?2 WHERE id = ?1",
        params![id, error],
    )?;
    Ok(())
}

/// Record a fetch that succeeded without new content (HTTP 304). Validators are
/// left in place and entries are untouched.
pub fn record_not_modified(conn: &Connection, id: i64, now: DateTime<Utc>) -> Result<()> {
    conn.execute(
        "UPDATE feeds SET last_fetched_at = ?2, last_error = NULL WHERE id = ?1",
        params![id, format_utc(now)],
    )?;
    Ok(())
}

/// How many entries a successful fetch created and how many it updated.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ApplyCounts {
    pub new_entries: i64,
    pub updated_entries: i64,
}

/// Store a successfully parsed feed: refresh the feed's title and validators,
/// clear its error, and reconcile its entries.
pub fn apply_fetch(
    conn: &mut Connection,
    id: i64,
    parsed: &ParsedFeed,
    etag: Option<&str>,
    last_modified: Option<&str>,
    now: DateTime<Utc>,
) -> Result<ApplyCounts> {
    let tx = conn.transaction()?;

    tx.execute(
        "UPDATE feeds
            SET title = COALESCE(?2, title),
                etag = ?3,
                last_modified = ?4,
                last_fetched_at = ?5,
                last_error = NULL
          WHERE id = ?1",
        params![id, parsed.title, etag, last_modified, format_utc(now)],
    )?;

    let mut counts = ApplyCounts::default();
    for entry in &parsed.entries {
        let published_at = entry.published_at.map(format_utc);
        // Identity is (feed, guid). A known key updates in place so the entry
        // keeps its id and its original fetched_at.
        let existing: Option<i64> = tx
            .query_row(
                "SELECT id FROM entries WHERE feed_id = ?1 AND guid = ?2",
                params![id, entry.guid],
                |row| row.get(0),
            )
            .optional()?;

        match existing {
            Some(entry_id) => {
                tx.execute(
                    "UPDATE entries
                        SET title = ?2, link = ?3, summary = ?4, published_at = ?5
                      WHERE id = ?1",
                    params![entry_id, entry.title, entry.link, entry.summary, published_at],
                )?;
                counts.updated_entries += 1;
            }
            None => {
                tx.execute(
                    "INSERT INTO entries (feed_id, guid, title, link, summary, published_at, fetched_at)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                    params![
                        id,
                        entry.guid,
                        entry.title,
                        entry.link,
                        entry.summary,
                        published_at,
                        format_utc(now)
                    ],
                )?;
                counts.new_entries += 1;
            }
        }
    }

    tx.commit()?;
    Ok(counts)
}

/// A parsed, validated `GET /api/entries` query.
#[derive(Debug, Clone, Default)]
pub struct EntryQuery {
    pub feed_id: Option<i64>,
    /// Inclusive lower bound, already normalized to the storage format.
    pub since: Option<String>,
    /// Exclusive upper bound, already normalized to the storage format.
    pub until: Option<String>,
    /// Case-insensitive substring of the title.
    pub search: Option<String>,
    pub limit: i64,
    pub offset: i64,
}

pub fn query_entries(conn: &Connection, query: &EntryQuery) -> Result<EntriesPage> {
    let mut conditions: Vec<&str> = Vec::new();
    let mut args: Vec<Value> = Vec::new();

    if let Some(feed_id) = query.feed_id {
        conditions.push("feed_id = ?");
        args.push(Value::Integer(feed_id));
    }
    // Half-open window, `since <= published_at < until`. Entries with no date
    // fall outside any window, so a NULL comparison excluding them is exactly
    // the wanted behavior.
    if let Some(since) = &query.since {
        conditions.push("published_at >= ?");
        args.push(Value::Text(since.clone()));
    }
    if let Some(until) = &query.until {
        conditions.push("published_at < ?");
        args.push(Value::Text(until.clone()));
    }
    if let Some(search) = &query.search {
        // SQLite's lower() folds ASCII only, which the spec allows. instr()
        // avoids LIKE's wildcard escaping entirely.
        conditions.push("instr(lower(title), lower(?)) > 0");
        args.push(Value::Text(search.clone()));
    }

    let where_clause = if conditions.is_empty() {
        String::new()
    } else {
        format!(" WHERE {}", conditions.join(" AND "))
    };

    let total: i64 = conn.query_row(
        &format!("SELECT COUNT(*) FROM entries{where_clause}"),
        params_from_iter(args.iter()),
        |row| row.get(0),
    )?;

    // Ordering is pinned: newest first, entries without a date last, ties broken
    // by insertion order.
    let sql = format!(
        "SELECT id, feed_id, guid, title, link, summary, published_at, fetched_at
           FROM entries{where_clause}
          ORDER BY published_at IS NULL ASC, published_at DESC, id ASC
          LIMIT ? OFFSET ?"
    );
    let mut page_args = args;
    page_args.push(Value::Integer(query.limit));
    page_args.push(Value::Integer(query.offset));

    let mut stmt = conn.prepare(&sql)?;
    let items = stmt
        .query_map(params_from_iter(page_args.iter()), |row| {
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
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;

    Ok(EntriesPage { total, items })
}
