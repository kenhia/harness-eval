# feedhub

A small feed-aggregation system in Rust:

| binary | what it does |
|---|---|
| `feedd` | long-running server: fetches RSS/Atom feeds into SQLite, serves a REST API |
| `feedctl` | command-line client for that API |
| `feedgen` | serves fixture feeds over local HTTP, so everything can be developed and tested without touching the real internet |

`feedhub-core` is the shared library the three binaries agree through: the feed
parser, the date handling, and the JSON types of the API.

## Build

Rust stable (2024 edition; developed against 1.96).

```sh
cargo build --release        # all three binaries, into target/release/
```

The binaries are self-contained — SQLite is compiled in, so there is nothing to
install alongside them. To put them on your `PATH`:

```sh
cargo install --path crates/feedd
cargo install --path crates/feedctl
cargo install --path crates/feedgen
```

## Checks

```sh
just check      # fmt-check + clippy + test
```

That is the whole gate, and it is what CI would run. Without `just`:

```sh
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
cargo test
```

`just --list` shows the rest (`build`, `serve-fixtures`, `run-server`, `clean`).

## Try it

Two terminals, no internet required:

```sh
just serve-fixtures                  # writes ./fixtures, serves them on :8601
feedd --db feedhub.db --poll-interval 0
```

```sh
feedctl add http://127.0.0.1:8601/rss-basic.xml
feedctl refresh
feedctl entries --search notes
```

```
2024-03-04T09:00:00Z  Rust release notes
                      http://feedgen.invalid/rss/1
2024-03-03T17:00:00Z  Filesystem notes
                      http://feedgen.invalid/rss/2

Showing 2 of 3 entries.
```

## `feedd`

```
feedd --db PATH [--listen ADDR:PORT] [--poll-interval SECS]
```

| flag | default | meaning |
|---|---|---|
| `--db PATH` | required | SQLite database file; created if missing |
| `--listen ADDR:PORT` | `127.0.0.1:8600` | where to serve the API |
| `--poll-interval SECS` | `300` | seconds between background refreshes of every feed; `0` disables polling |

Background polling is independent of the refresh endpoints: those always fetch
on demand, which is what makes tests deterministic. `RUST_LOG=feedd=debug`
turns up logging.

### What it understands

RSS 2.0 and Atom (RFC 4287), UTF-8, with a leading BOM tolerated.

| | RSS `<item>` | Atom `<entry>` |
|---|---|---|
| identity | `guid`, falling back to `link` | `id` |
| title | `title` | `title` |
| link | `link` | the `rel="alternate"` link, falling back to the first `<link>` |
| summary | `description` | `summary`, falling back to `content` |
| date | `pubDate` (RFC 822) | `published`, falling back to `updated` (RFC 3339) |

An item with neither identity nor link is skipped: without an identity key there
is nothing to dedupe it against, so it would be duplicated on every refresh.

### Semantics worth knowing

**Dates.** `published_at` is stored normalized to UTC and serialized RFC 3339
with `Z` at second precision (`2024-03-01T12:30:00Z`). RFC 822 dates accept
4-digit years, numeric offsets (`+0000`, `-0500`), and the zone names
`GMT UT Z EST EDT CST CDT MST MDT PST PDT`; RFC 3339 dates accept any numeric
offset and optional fractional seconds. A missing or unparseable date stores
`published_at: null` — never the fetch time — and the entry is still stored.

**Text.** Titles and summaries are stored with XML entities unescaped
(`&amp;` → `&`). CDATA is stored verbatim, inner whitespace and markup
included. A missing title is stored as `(untitled)`. An entity nothing defines
(`&nbsp;`) is kept as written rather than failing the feed.

**Dedupe.** Identity is `(feed, guid-or-id)`. Seeing a known key again updates
its title, link, summary and date in place: no duplicate row, and the entry
keeps its internal id and its original `fetched_at`.

**Conditional GET.** Each feed's `ETag` and `Last-Modified` are stored and sent
back as `If-None-Match` / `If-Modified-Since`. A `304` leaves entries untouched,
counts as a successful fetch, and reports `new_entries: 0`.

**Failure isolation.** A refused connection, an HTTP error or a malformed
document is recorded in that feed's `last_error` and returned in its refresh
result. It never affects another feed and never brings down the server. The next
successful fetch clears the error.

## `feedctl`

```
feedctl [--server URL] [--format text|json] <command>
```

`--server` defaults to `http://127.0.0.1:8600`; `--format` defaults to `text`.
With `--format json`, the raw API response is printed as a single JSON document
on stdout, so it pipes into `jq`.

| command | what it does |
|---|---|
| `add URL` | register a feed |
| `list` | table of every feed |
| `show ID` | one feed in full |
| `remove ID` | delete a feed and its entries |
| `refresh [ID]` | fetch one feed now, or every feed if no id is given |
| `entries [--feed ID] [--since T] [--until T] [--search Q] [--limit N] [--offset N]` | query stored entries |

### Exit codes

| code | when |
|---|---|
| `0` | success |
| `1` | the server answered with an error; the message is on stderr |
| `2` | the server is unreachable, or the usage is invalid |

A `refresh` whose feed failed to fetch exits `0`: the API call itself succeeded,
and the failure is reported in the output (`feed 3: error: ...`). Exit `1` is
for the server answering with `{"error": ...}` and a 4xx/5xx status.

## `feedgen`

```
feedgen serve --dir DIR [--listen ADDR:PORT]     # default 127.0.0.1:8601
feedgen make-fixtures DIR
```

`serve` serves `DIR` over HTTP with a content-type derived from what each
document actually is, a content-derived `ETag`, and a `Last-Modified` from the
file's mtime, and answers `304` to a matching `If-None-Match` or
`If-Modified-Since`. Because the ETag comes from the content rather than the
mtime, editing a fixture invalidates it immediately — even within the same
second, which one-second-resolution HTTP dates cannot see. `GET /` lists the
corpus.

`make-fixtures` writes the corpus below, plus a `README.md` describing it. Every
date in it is a constant, so tests can assert exact `published_at` values
without depending on the wall clock.

| fixture | covers |
|---|---|
| `rss-basic.xml` | valid RSS 2.0: three items with guid, link, description, RFC 822 dates |
| `atom-basic.xml` | valid Atom: `rel="alternate"` selection, summary-over-content, RFC 3339 dates |
| `dates-edge.xml` | every accepted zone name, both offset directions, a missing date, an unparseable one |
| `atom-dates-edge.xml` | fractional seconds, non-UTC offsets, `updated` without `published`, junk |
| `cdata-entities.xml` | CDATA kept verbatim, entities unescaped, an item with no title |
| `malformed.xml` | not well-formed XML — fetching it must record `last_error` |

## API reference

JSON throughout. Errors are `{"error": "<message>"}` with a 4xx/5xx status,
including for unknown routes and unparseable bodies.

### `GET /api/health`

`200` — `{"status": "ok", "version": "0.1.0"}`.

### `POST /api/feeds`

Body `{"url": "https://example.com/feed.xml"}`.

| status | when |
|---|---|
| `201` | created; the body is the feed object |
| `409` | that URL is already registered |
| `422` | the URL is not a valid absolute http(s) URL, or the body has no `url` |

### `GET /api/feeds`

`200` — an array of feed objects, by id.

### `GET /api/feeds/{id}`

`200` with the feed object, or `404`.

### `DELETE /api/feeds/{id}`

`204`, no body. The feed's entries are deleted with it. `404` if it does not
exist.

### `POST /api/feeds/{id}/refresh`

Fetches now. `200` with a refresh result, or `404` if the feed does not exist.
A fetch that fails still answers `200`, with `status: "error"`.

### `POST /api/refresh`

Refreshes every feed, in id order, one at a time. `200` with an array of refresh
results, each carrying its `feed_id`.

### `GET /api/entries`

All parameters are optional.

| parameter | default | meaning |
|---|---|---|
| `feed_id` | — | restrict to one feed |
| `since` | — | RFC 3339 instant, any offset; inclusive lower bound |
| `until` | — | RFC 3339 instant, any offset; exclusive upper bound |
| `q` | — | case-insensitive substring of the title (ASCII case folding) |
| `limit` | `50` | maximum items; values above `500` are capped at `500` |
| `offset` | `0` | items to skip |

The window is half-open, `since <= published_at < until`, compared as UTC
instants regardless of the offsets you write them with. When either bound is
given, entries with a null `published_at` are excluded.

Ordering is `published_at` descending, entries with no date last, ties broken by
internal entry id ascending.

`200` — `{"total": N, "items": [...]}`, where `total` is the number of matches
ignoring `limit`/`offset`. A malformed parameter is `422`.

### Objects

Feed:

```json
{
  "id": 1,
  "url": "http://127.0.0.1:8601/rss-basic.xml",
  "title": "Basic RSS Feed",
  "last_fetched_at": "2024-03-06T10:00:00Z",
  "last_error": null,
  "entry_count": 3
}
```

`title` is null until the first successful fetch, and is refreshed from the
document on every fetch. `last_fetched_at` is the last *successful* fetch, and
is null until there has been one. `last_error` is null when the last fetch
succeeded.

Entry:

```json
{
  "id": 1,
  "feed_id": 1,
  "guid": "rss-1",
  "title": "Rust release notes",
  "link": "http://feedgen.invalid/rss/1",
  "summary": "What shipped in the latest release.",
  "published_at": "2024-03-04T09:00:00Z",
  "fetched_at": "2024-03-06T10:00:00Z"
}
```

Refresh result:

```json
{
  "feed_id": 1,
  "status": "ok",
  "new_entries": 3,
  "updated_entries": 0,
  "not_modified": false,
  "error": null
}
```

`status` is `"ok"` or `"error"`; `error` carries the message when it is
`"error"`. `not_modified` is true when the origin answered `304`, in which case
`new_entries` is `0`.

## Layout

```
crates/feedhub-core   parser, date handling, API types
crates/feedd          server: storage, HTTP API, fetching, polling
crates/feedctl        CLI client
crates/feedgen        fixture corpus and fixture server
```

### Storage

One SQLite file, two tables. `feeds` holds the URL, the last-known title, the
fetch state (`last_fetched_at`, `last_error`) and the conditional-GET validators
(`etag`, `last_modified`). `entries` holds the entries, with
`UNIQUE (feed_id, guid)` as the identity that dedupe relies on and
`ON DELETE CASCADE` so deleting a feed takes its entries with it.

Timestamps are stored as fixed-width RFC 3339 UTC strings, which makes
lexicographic comparison in SQL identical to chronological comparison — so the
`since`/`until` window and the `published_at` ordering are both plain SQL rather
than something reimplemented in Rust.

### Tests

`cargo test` runs everything, offline. It covers:

- the parser and date handling as unit tests, against the pinned rules;
- `feedgen`'s HTTP behavior, including 304s and stale validators;
- `feedctl`'s rendering, and the real binary's exit codes;
- end to end: a real `feedd` fetching from a real `feedgen`, both over local
  HTTP on port 0, with polling disabled so that every fetch is one a test asked
  for.

No test reaches the network beyond `127.0.0.1`.
