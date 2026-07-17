//! SQLite storage for feedd (rusqlite).
//!
//! A single [`Connection`] is shared behind a mutex in the app state; all
//! query helpers here take `&Connection` and are synchronous.

use anyhow::Result;
use chrono::{DateTime, Utc};
use feedcore::date::to_rfc3339_z;
use feedcore::types::ParsedItem;
use rusqlite::{params, params_from_iter, types::Value, Connection, OptionalExtension};

use crate::models::{EntryDto, FeedDto, FeedRow};

/// Open (creating if needed) the database at `path`, apply pragmas, and
/// ensure the schema exists.
pub fn open(path: &str) -> Result<Connection> {
    let conn = Connection::open(path)?;
    conn.pragma_update(None, "journal_mode", "WAL")?;
    conn.pragma_update(None, "foreign_keys", "ON")?;
    init_schema(&conn)?;
    Ok(conn)
}

/// Open an in-memory database (used in tests).
#[cfg(test)]
pub fn open_memory() -> Result<Connection> {
    let conn = Connection::open_in_memory()?;
    conn.pragma_update(None, "foreign_keys", "ON")?;
    init_schema(&conn)?;
    Ok(conn)
}

fn init_schema(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS feeds (
            id              INTEGER PRIMARY KEY AUTOINCREMENT,
            url             TEXT NOT NULL UNIQUE,
            title           TEXT,
            etag            TEXT,
            last_modified   TEXT,
            last_fetched_at TEXT,
            last_error      TEXT
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
            UNIQUE(feed_id, guid)
        );
        CREATE INDEX IF NOT EXISTS idx_entries_feed ON entries(feed_id);
        CREATE INDEX IF NOT EXISTS idx_entries_published ON entries(published_at);
        "#,
    )?;
    Ok(())
}

/// Outcome of attempting to register a feed URL.
pub enum InsertOutcome {
    Created(FeedDto),
    Conflict,
}

/// Register a new feed URL. Returns [`InsertOutcome::Conflict`] if already
/// present.
pub fn insert_feed(conn: &Connection, url: &str) -> Result<InsertOutcome> {
    let existing: Option<i64> = conn
        .query_row("SELECT id FROM feeds WHERE url = ?1", params![url], |r| {
            r.get(0)
        })
        .optional()?;
    if existing.is_some() {
        return Ok(InsertOutcome::Conflict);
    }
    conn.execute("INSERT INTO feeds (url) VALUES (?1)", params![url])?;
    let id = conn.last_insert_rowid();
    Ok(InsertOutcome::Created(
        get_feed(conn, id)?.expect("just inserted"),
    ))
}

/// All feeds, ordered by id.
pub fn list_feeds(conn: &Connection) -> Result<Vec<FeedDto>> {
    let mut stmt = conn.prepare(
        "SELECT f.id, f.url, f.title, f.last_fetched_at, f.last_error,
                (SELECT COUNT(*) FROM entries e WHERE e.feed_id = f.id)
         FROM feeds f ORDER BY f.id",
    )?;
    let rows = stmt.query_map([], row_to_feed_dto)?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

/// A single feed DTO by id.
pub fn get_feed(conn: &Connection, id: i64) -> Result<Option<FeedDto>> {
    let feed = conn
        .query_row(
            "SELECT f.id, f.url, f.title, f.last_fetched_at, f.last_error,
                    (SELECT COUNT(*) FROM entries e WHERE e.feed_id = f.id)
             FROM feeds f WHERE f.id = ?1",
            params![id],
            row_to_feed_dto,
        )
        .optional()?;
    Ok(feed)
}

/// The internal feed row (with conditional-GET headers) by id.
pub fn get_feed_row(conn: &Connection, id: i64) -> Result<Option<FeedRow>> {
    let row = conn
        .query_row(
            "SELECT id, url, etag, last_modified FROM feeds WHERE id = ?1",
            params![id],
            |r| {
                Ok(FeedRow {
                    id: r.get(0)?,
                    url: r.get(1)?,
                    etag: r.get(2)?,
                    last_modified: r.get(3)?,
                })
            },
        )
        .optional()?;
    Ok(row)
}

/// Delete a feed (and, by cascade, its entries). Returns true if a row was
/// removed.
pub fn delete_feed(conn: &Connection, id: i64) -> Result<bool> {
    let n = conn.execute("DELETE FROM feeds WHERE id = ?1", params![id])?;
    Ok(n > 0)
}

/// All feed ids, ordered.
pub fn all_feed_ids(conn: &Connection) -> Result<Vec<i64>> {
    let mut stmt = conn.prepare("SELECT id FROM feeds ORDER BY id")?;
    let ids = stmt.query_map([], |r| r.get(0))?;
    Ok(ids.collect::<rusqlite::Result<Vec<_>>>()?)
}

fn row_to_feed_dto(r: &rusqlite::Row) -> rusqlite::Result<FeedDto> {
    Ok(FeedDto {
        id: r.get(0)?,
        url: r.get(1)?,
        title: r.get(2)?,
        last_fetched_at: r.get(3)?,
        last_error: r.get(4)?,
        entry_count: r.get(5)?,
    })
}

/// Record a successful fetch: update title (if present), conditional-GET
/// headers, last_fetched_at, and clear last_error.
#[allow(clippy::too_many_arguments)]
pub fn mark_success(
    conn: &Connection,
    id: i64,
    title: Option<&str>,
    etag: Option<&str>,
    last_modified: Option<&str>,
    fetched_at: DateTime<Utc>,
) -> Result<()> {
    // COALESCE keeps the previous title when a refetch omits one.
    conn.execute(
        "UPDATE feeds SET title = COALESCE(?2, title), etag = ?3, last_modified = ?4,
                last_fetched_at = ?5, last_error = NULL WHERE id = ?1",
        params![id, title, etag, last_modified, to_rfc3339_z(&fetched_at)],
    )?;
    Ok(())
}

/// Record a `304 Not Modified`: successful fetch, entries untouched.
pub fn mark_not_modified(conn: &Connection, id: i64, fetched_at: DateTime<Utc>) -> Result<()> {
    conn.execute(
        "UPDATE feeds SET last_fetched_at = ?2, last_error = NULL WHERE id = ?1",
        params![id, to_rfc3339_z(&fetched_at)],
    )?;
    Ok(())
}

/// Record a fetch/parse failure on the feed (leaves entries and
/// last_fetched_at untouched).
pub fn mark_error(conn: &Connection, id: i64, error: &str) -> Result<()> {
    conn.execute(
        "UPDATE feeds SET last_error = ?2 WHERE id = ?1",
        params![id, error],
    )?;
    Ok(())
}

/// Insert or update an item by (feed, guid). Returns true when the entry was
/// newly inserted (updates keep the internal id and original fetched_at).
pub fn upsert_entry(
    conn: &Connection,
    feed_id: i64,
    item: &ParsedItem,
    now: DateTime<Utc>,
) -> Result<bool> {
    let published = item.published_at.as_ref().map(to_rfc3339_z);
    let existing: Option<i64> = conn
        .query_row(
            "SELECT id FROM entries WHERE feed_id = ?1 AND guid = ?2",
            params![feed_id, item.guid],
            |r| r.get(0),
        )
        .optional()?;
    if existing.is_some() {
        conn.execute(
            "UPDATE entries SET title = ?3, link = ?4, summary = ?5, published_at = ?6
             WHERE feed_id = ?1 AND guid = ?2",
            params![
                feed_id,
                item.guid,
                item.title,
                item.link,
                item.summary,
                published
            ],
        )?;
        Ok(false)
    } else {
        conn.execute(
            "INSERT INTO entries (feed_id, guid, title, link, summary, published_at, fetched_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                feed_id,
                item.guid,
                item.title,
                item.link,
                item.summary,
                published,
                to_rfc3339_z(&now)
            ],
        )?;
        Ok(true)
    }
}

/// Filters for [`query_entries`].
#[derive(Default)]
pub struct EntryQuery {
    pub feed_id: Option<i64>,
    /// Normalized RFC 3339 Z lower bound (inclusive).
    pub since: Option<String>,
    /// Normalized RFC 3339 Z upper bound (exclusive).
    pub until: Option<String>,
    pub q: Option<String>,
    pub limit: i64,
    pub offset: i64,
}

/// Query entries with the pinned window/order/pagination semantics. Returns
/// `(total_matches, page_items)`.
pub fn query_entries(conn: &Connection, query: &EntryQuery) -> Result<(i64, Vec<EntryDto>)> {
    let mut where_sql = String::from(" WHERE 1=1");
    let mut args: Vec<Value> = Vec::new();

    if let Some(fid) = query.feed_id {
        where_sql.push_str(&format!(" AND feed_id = ?{}", args.len() + 1));
        args.push(Value::Integer(fid));
    }
    // When any bound is set, null published_at is excluded.
    if let Some(since) = &query.since {
        where_sql.push_str(&format!(
            " AND published_at IS NOT NULL AND published_at >= ?{}",
            args.len() + 1
        ));
        args.push(Value::Text(since.clone()));
    }
    if let Some(until) = &query.until {
        where_sql.push_str(&format!(
            " AND published_at IS NOT NULL AND published_at < ?{}",
            args.len() + 1
        ));
        args.push(Value::Text(until.clone()));
    }
    if let Some(q) = &query.q {
        where_sql.push_str(&format!(
            " AND INSTR(LOWER(title), LOWER(?{})) > 0",
            args.len() + 1
        ));
        args.push(Value::Text(q.clone()));
    }

    let count_sql = format!("SELECT COUNT(*) FROM entries{where_sql}");
    let total: i64 = conn.query_row(&count_sql, params_from_iter(args.iter()), |r| r.get(0))?;

    let mut page_args = args.clone();
    let limit_idx = page_args.len() + 1;
    let offset_idx = page_args.len() + 2;
    page_args.push(Value::Integer(query.limit));
    page_args.push(Value::Integer(query.offset));

    let list_sql = format!(
        "SELECT id, feed_id, guid, title, link, summary, published_at, fetched_at
         FROM entries{where_sql}
         ORDER BY published_at IS NULL, published_at DESC, id ASC
         LIMIT ?{limit_idx} OFFSET ?{offset_idx}"
    );
    let mut stmt = conn.prepare(&list_sql)?;
    let rows = stmt.query_map(params_from_iter(page_args.iter()), |r| {
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
    })?;
    let items = rows.collect::<rusqlite::Result<Vec<_>>>()?;
    Ok((total, items))
}
