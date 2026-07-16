# feedhub

A small RSS/Atom feed-aggregation system in Rust. It has three binaries and a
shared library, organized as a Cargo workspace:

| Crate      | Kind    | Role                                                            |
|------------|---------|-----------------------------------------------------------------|
| `feedcore` | library | Feed parsing (RSS 2.0 + Atom), date handling, shared API types. |
| `feedd`    | binary  | Long-running server: fetches/stores feeds, exposes a REST API.  |
| `feedctl`  | binary  | Command-line client that drives the `feedd` API.                |
| `feedgen`  | binary  | Fixture tool: serves test feeds over local HTTP for dev/test.   |

Storage is a single SQLite database file. At runtime and in tests, `feedhub`
only contacts the feed URLs you explicitly register (tests use `feedgen`), so
it never touches the real internet on its own.

## Requirements

- Rust stable (1.74+). SQLite is compiled in via `rusqlite`'s bundled feature —
  no system SQLite needed.
- [`just`](https://github.com/casey/just) (optional) for the check runner.

## Build & install

```sh
# Build all three binaries (debug)
cargo build

# Release binaries land in target/release/{feedd,feedctl,feedgen}
cargo build --release

# Install into ~/.cargo/bin
cargo install --path feedd
cargo install --path feedctl
cargo install --path feedgen
```

## Run all checks

A single command runs formatting, linting, tests, and the release build:

```sh
just check
```

Equivalent without `just`:

```sh
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo build --release
```

## Quick start

```sh
# 1. Generate a fixture corpus and serve it locally.
feedgen make-fixtures ./fixtures
feedgen serve --dir ./fixtures --listen 127.0.0.1:8700 &

# 2. Start the server (no background polling; refresh on demand).
feedd --db feedhub.db --listen 127.0.0.1:8600 --poll-interval 0 &

# 3. Drive it with the client.
feedctl add http://127.0.0.1:8700/rss.xml
feedctl refresh
feedctl list
feedctl entries --search post
```

## `feedd` — the server

```
feedd --db PATH [--listen ADDR:PORT] [--poll-interval SECS]
```

- `--db PATH` — SQLite database file (created if missing). Required.
- `--listen ADDR:PORT` — default `127.0.0.1:8600`.
- `--poll-interval SECS` — background refresh interval; default `300`.
  `0` disables background polling. Refresh endpoints always fetch on demand,
  independent of polling.

### Behavior highlights

- **Formats:** RSS 2.0 and Atom (RFC 4287), UTF-8, tolerating a leading BOM.
- **Identity / dedupe:** an entry's identity is `(feed, guid)` for RSS
  (falling back to `link`) and `(feed, id)` for Atom. Re-seeing a key updates
  title/link/summary/published_at in place, keeping the entry's internal id and
  original `fetched_at` — never a duplicate row.
- **Dates:** `published_at` is normalized to UTC and serialized RFC 3339 with a
  trailing `Z`. RSS `pubDate` (RFC 822) accepts 4-digit years, numeric offsets
  (`+0000`, `-0500`), and the zone names `GMT UT Z EST EDT CST CDT MST MDT PST
  PDT`. Atom dates (RFC 3339) accept any numeric offset and optional fractional
  seconds. A missing/unparseable date stores `published_at = null` (fetch time
  is never substituted); the entry is still stored.
- **Text:** titles and summaries are stored after XML entity unescaping; CDATA
  is kept verbatim; a missing title is stored as `(untitled)`.
- **Conditional GET:** each feed's `ETag`/`Last-Modified` are stored and sent
  back as `If-None-Match`/`If-Modified-Since`. A `304` counts as a successful
  fetch, leaves entries untouched, and reports `new_entries: 0`.
- **Failure isolation:** a connection/HTTP/parse failure is recorded on that
  feed's `last_error` and never affects other feeds or crashes the server.

## `feedctl` — the client

```
feedctl [--server URL] [--format text|json] <command>
```

- `--server URL` — default `http://127.0.0.1:8600`.
- `--format text|json` — default `text`. `json` prints the raw API response as a
  single JSON document on stdout.

| Command | Description |
|---|---|
| `add URL` | Register a feed. |
| `list` | List all feeds. |
| `show ID` | Show one feed. |
| `remove ID` | Delete a feed and its entries. |
| `refresh [ID]` | Refresh one feed, or all feeds when `ID` is omitted. |
| `entries [--feed ID] [--since T] [--until T] [--search Q] [--limit N] [--offset N]` | Query entries. |

**Exit codes:** `0` success; `1` the server answered with an error (message on
stderr); `2` the server is unreachable or the usage is invalid.

## `feedgen` — the fixture tool

```
feedgen serve --dir DIR [--listen ADDR:PORT]
feedgen make-fixtures DIR
```

- `serve` — serves files in `DIR` over HTTP with the correct `Content-Type`, a
  content-derived `ETag`, and `Last-Modified`, honoring
  `If-None-Match`/`If-Modified-Since` with `304`. Default listen
  `127.0.0.1:8700`.
- `make-fixtures` — writes a documented fixture corpus into `DIR`:

  | File          | Purpose |
  |---------------|---------|
  | `rss.xml`     | Valid RSS 2.0 feed (numeric-offset dates, an entity). |
  | `atom.xml`    | Valid Atom feed (alternate link, summary/content fallback). |
  | `dates.xml`   | Edge-case dates: `GMT` and `EST` zone names, plus a missing date. |
  | `cdata.xml`   | CDATA (verbatim) and XML entity unescaping. |
  | `malformed.xml` | Not well-formed XML; exercises parse-failure isolation. |
  | `README.md`   | Describes the corpus. |

## API reference

All responses are JSON. Errors are `{"error": "<message>"}` with a 4xx/5xx
status.

### `GET /api/health`

Returns `{"status":"ok"}` (extra fields allowed). Always `200`.

### `POST /api/feeds`

Body: `{"url": "http://..."}`.

- `201` + feed object on success.
- `409` if the URL is already registered.
- `422` if the URL is not a valid `http(s)` URL.

### `GET /api/feeds`

`200` + an array of feed objects.

### `GET /api/feeds/{id}`

`200` + the feed object, or `404`.

### `DELETE /api/feeds/{id}`

`204` on success (the feed's entries are deleted too), or `404`.

### `POST /api/feeds/{id}/refresh`

Fetches the feed now. `200` + `{"status":"ok"|"error", "new_entries": N, ...}`,
or `404` if the feed does not exist. On a fetch/parse failure the result is
`{"status":"error", "new_entries":0, "error":"<message>"}` and the feed's
`last_error` is set.

### `POST /api/refresh`

Refreshes every feed. `200` + an array of the same result objects, each
including the feed's `id` (`feed_id`).

### `GET /api/entries`

Query parameters (all optional):

- `feed_id` — restrict to one feed.
- `since`, `until` — RFC 3339 instants (any offset). Window is **half-open**:
  `since <= published_at < until`, compared as UTC instants. When either bound
  is given, entries with a null `published_at` are excluded.
- `q` — case-insensitive (ASCII) substring match on the title.
- `limit` — default `50`, max `500`.
- `offset` — default `0`.

Response: `{"total": N, "items": [...]}`. `total` is the match count ignoring
`limit`/`offset`. Ordering is `published_at` descending, **nulls last**, ties
broken by internal entry id ascending.

### Object shapes

Feed object:

```json
{
  "id": 1,
  "url": "http://example.com/rss.xml",
  "title": "Example RSS Feed",
  "last_fetched_at": "2026-01-01T00:00:00Z",
  "last_error": null,
  "entry_count": 2
}
```

`title` is `null` until the first successful fetch and updates on refresh.
`last_error` is `null` when the last fetch succeeded.

Entry object:

```json
{
  "id": 1,
  "feed_id": 1,
  "guid": "http://example.com/posts/1",
  "title": "First Post",
  "link": "http://example.com/posts/1",
  "summary": "Hello from the first post.",
  "published_at": "2021-09-06T16:20:00Z",
  "fetched_at": "2026-01-01T00:00:00Z"
}
```

## Testing

`cargo test` runs unit tests plus an end-to-end test (`feedd/tests/e2e.rs`) that
launches the real `feedd` and `feedgen` binaries and drives the API over local
HTTP — covering registration, conditional-GET (`304`), dedupe, date
normalization, window filtering, failure isolation, and cascade delete.

## License

MIT — see [LICENSE](LICENSE).
