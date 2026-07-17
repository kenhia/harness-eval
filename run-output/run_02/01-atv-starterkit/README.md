# feedhub

A small feed-aggregation system written in Rust. It consists of three
binaries built from a single Cargo workspace:

- **`feedd`** — a long-running server that fetches and stores RSS/Atom feeds
  in SQLite and exposes a JSON REST API.
- **`feedctl`** — a command-line client that drives the `feedd` API.
- **`feedgen`** — a fixture tool that serves test feeds over local HTTP and
  generates a fixture corpus, so everything can be developed and tested
  without touching the real internet.

A shared library crate, **`feedcore`**, holds the feed parsing, date handling
and common types.

```
crates/
  feedcore/   # parsing (RSS 2.0 + Atom), date normalization, shared types
  feedd/      # server binary + SQLite storage + HTTP fetcher
  feedctl/    # CLI client binary
  feedgen/    # fixture server + corpus generator (also a library for tests)
```

## Requirements

- Rust stable (edition 2021). SQLite is bundled via `rusqlite`, so no system
  SQLite is required.

## Build & install

```sh
# Build all three binaries (optimized)
cargo build --release
# Binaries land in target/release/{feedd,feedctl,feedgen}

# Optionally install them onto your PATH
cargo install --path crates/feedd
cargo install --path crates/feedctl
cargo install --path crates/feedgen
```

## Quality gate

A single command runs formatting checks, clippy, the release build and the
full test suite (unit tests plus an end-to-end test that drives `feedd`
against `feedgen` over local HTTP):

```sh
just check
```

If you don't have [`just`](https://github.com/casey/just), run the equivalent
commands directly:

```sh
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
cargo build --release
cargo test
```

## Quick start

```sh
# 1. Generate a fixture corpus and serve it locally.
feedgen make-fixtures ./fixtures
feedgen serve --dir ./fixtures --listen 127.0.0.1:8700 &

# 2. Start the server (no background polling; refresh on demand).
feedd --db ./feedhub.db --listen 127.0.0.1:8600 --poll-interval 0 &

# 3. Drive it with the client.
feedctl add http://127.0.0.1:8700/rss.xml
feedctl add http://127.0.0.1:8700/atom.xml
feedctl refresh
feedctl list
feedctl entries --search rust --limit 20
```

---

## `feedd` — the server

```
feedd --db PATH [--listen ADDR:PORT] [--poll-interval SECS]
```

| flag | default | meaning |
|------|---------|---------|
| `--db` | *(required)* | Path to the SQLite database file (created if missing). |
| `--listen` | `127.0.0.1:8600` | Address/port to bind the API. |
| `--poll-interval` | `300` | Background refresh interval in seconds; `0` disables polling. |

Regardless of background polling, the refresh endpoints always fetch on demand
so tests and scripts stay deterministic.

### Behavior notes

- **Formats:** RSS 2.0 and Atom (RFC 4287), UTF-8 with a tolerated leading BOM.
  - RSS `<item>`: identity = `guid` (falling back to `link`); `title`, `link`,
    `description` → summary, `pubDate`.
  - Atom `<entry>`: identity = `id`; `title`, the `rel="alternate"` link (or
    the first `<link>`), `summary` or `content` → summary, `published` falling
    back to `updated`.
- **Dates:** `published_at` is normalized to UTC and serialized as RFC 3339
  with `Z`. RFC 822 dates accept numeric offsets (`+0000`, `-0500`) and the
  zone names `GMT UT Z EST EDT CST CDT MST MDT PST PDT`; RFC 3339 dates accept
  any numeric offset and optional fractional seconds. A missing or unparseable
  date stores `published_at = null` — the fetch time is never substituted, and
  the entry is still stored.
- **Text:** titles and summaries are stored after XML entity unescaping
  (`&amp;` → `&`); CDATA is taken verbatim; a missing title becomes `(untitled)`.
- **Dedupe/update:** the identity key is `(feed, guid-or-id)`. Re-seeing a key
  updates title/link/summary/published_at in place, keeping the entry's
  internal id and original `fetched_at`.
- **Conditional GET:** each feed's `ETag`/`Last-Modified` are stored and sent
  as `If-None-Match`/`If-Modified-Since` on refetch. A `304` leaves entries
  untouched, counts as a successful fetch, and reports `new_entries: 0`.
- **Failure isolation:** a fetch/parse failure is recorded on that feed's
  `last_error` and never affects other feeds or crashes the server.

### REST API

All responses are JSON. Errors are `{"error": "<message>"}` with a 4xx/5xx
status.

#### `GET /api/health`

Liveness check.

```json
{ "status": "ok", "version": "0.1.0" }
```

#### `POST /api/feeds`

Register a feed. Body: `{"url": "https://..."}`.

- `201` + the feed object on success.
- `409` if the URL is already registered.
- `422` if the URL is not a valid `http(s)` URL or the body is invalid.

#### `GET /api/feeds`

Returns an array of feed objects, ordered by id.

#### `GET /api/feeds/{id}`

Returns the feed object, or `404` if unknown.

#### `DELETE /api/feeds/{id}`

Deletes the feed and all of its entries. Returns `204`, or `404` if unknown.

#### `POST /api/feeds/{id}/refresh`

Fetches the feed immediately. Returns a result object:

```json
{ "id": 1, "status": "ok", "new_entries": 3 }
```

On a conditional `304` the result also carries `"not_modified": true`; on
failure `status` is `"error"` and an `"error"` message is included. Returns
`404` if the feed is unknown.

#### `POST /api/refresh`

Refreshes every feed. Returns an array of the same per-feed result objects,
each including the feed `id`.

#### `GET /api/entries`

Query stored entries. All parameters are optional:

| param | default | meaning |
|-------|---------|---------|
| `feed_id` | — | Restrict to one feed. |
| `since` | — | RFC 3339 instant (any offset); lower bound, inclusive. |
| `until` | — | RFC 3339 instant (any offset); upper bound, exclusive. |
| `q` | — | Case-insensitive substring match on the title. |
| `limit` | `50` | Max items returned (capped at `500`). |
| `offset` | `0` | Number of items to skip. |

Window semantics are half-open and compared as UTC instants:
`since <= published_at < until`. When either bound is given, entries with a
null `published_at` are excluded.

Response:

```json
{ "total": 42, "items": [ /* entry objects */ ] }
```

`total` is the match count ignoring `limit`/`offset`. Items are ordered by
`published_at` descending with nulls last, ties broken by internal entry id
ascending.

### Object shapes

Feed object:

```json
{
  "id": 1,
  "url": "http://127.0.0.1:8700/rss.xml",
  "title": "Example RSS Feed",
  "last_fetched_at": "2021-01-04T08:00:00Z",
  "last_error": null,
  "entry_count": 2
}
```

`title` is `null` until the first successful fetch and updates on refresh;
`last_error` is `null` when the last fetch succeeded.

Entry object:

```json
{
  "id": 5,
  "feed_id": 1,
  "guid": "https://example.com/posts/1",
  "title": "First post",
  "link": "https://example.com/posts/1",
  "summary": "Hello from the first post.",
  "published_at": "2021-01-04T08:00:00Z",
  "fetched_at": "2021-01-04T09:12:00Z"
}
```

---

## `feedctl` — the client

```
feedctl [--server URL] [--format text|json] <command>
```

- `--server` defaults to `http://127.0.0.1:8600`.
- `--format` defaults to `text` (human-readable); `json` prints the raw API
  response as a single valid JSON document on stdout.

| command | description |
|---------|-------------|
| `add URL` | Register a new feed. |
| `list` | List all feeds. |
| `show ID` | Show one feed. |
| `remove ID` | Delete a feed and its entries. |
| `refresh [ID]` | Refresh one feed, or all feeds when no id is given. |
| `entries [--feed ID] [--since T] [--until T] [--search Q] [--limit N] [--offset N]` | Query entries. |

Exit codes:

- `0` — success.
- `1` — the server answered with an error (message printed to stderr).
- `2` — the server is unreachable, or usage was invalid.

Examples:

```sh
feedctl add http://127.0.0.1:8700/atom.xml
feedctl --format json list
feedctl refresh 1
feedctl entries --feed 1 --since 2021-01-01T00:00:00Z --until 2021-02-01T00:00:00Z
feedctl entries --search rust --limit 10 --offset 20
```

---

## `feedgen` — the fixture tool

```
feedgen serve --dir DIR [--listen ADDR:PORT]
feedgen make-fixtures DIR
```

- **`serve`** serves the files in `DIR` over HTTP with a correct
  `Content-Type`, a content-derived `ETag` and a `Last-Modified` header, and
  honors `If-None-Match` / `If-Modified-Since` with `304`. `--listen` defaults
  to `127.0.0.1:8700`.
- **`make-fixtures`** writes a fixture corpus into `DIR`:

  | file | purpose |
  |------|---------|
  | `rss.xml` | Valid RSS 2.0 feed (GUID + link identity, RFC 822 dates). |
  | `atom.xml` | Valid Atom feed (`alternate` link, `summary` vs `content`, `published`/`updated`). |
  | `dates.xml` | Edge-case dates: zone names, a missing date and an unparseable date. |
  | `cdata.xml` | CDATA verbatim and XML entity unescaping. |
  | `malformed.xml` | Malformed XML for failure-isolation testing. |
  | `README.md` | Documents the corpus. |

---

## Testing

```sh
cargo test
```

Unit tests in `feedcore` cover date parsing and RSS/Atom parsing. The
end-to-end test in `crates/feedd/tests/e2e.rs` starts an in-process `feedgen`
server, launches the real `feedd` binary, and exercises the full API over
local HTTP — registration, refresh, conditional GET, entry querying, window
semantics, search and cascade delete. No test touches the real internet.

## License

MIT.
