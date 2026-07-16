# feedhub — design record

What was built and why, including the decisions that changed under review.
The README is the user-facing doc; this is the reasoning behind it.

## Premises

1. **The spec pins the hard parts.** Dates, text, dedupe identity, window
   semantics, and ordering are specified exactly. The engineering question was
   never *what* to build but *how to make the pins provable*.
2. **Determinism beats background machinery.** Refresh endpoints fetch on
   demand; tests drive those. The poll loop reuses the same path and is ticked
   explicitly by its test rather than raced against a timer.
3. **Hand-rolled parsing beats a general feed crate.** A general crate has its
   own opinions about identity and dates — notably substituting fetch time for a
   missing date, which the spec forbids outright. Owning ~350 lines is cheaper
   than fighting a dependency's semantics. This is the deliberate exception to
   "don't reinvent".
4. **No network beyond feedgen.** Everything binds `127.0.0.1:0`.

## Architecture

```
crates/
├── feedhub-core/   model, datetime, parser              (lib)
├── feedd/          store, fetch, api, poll, cli         (lib + bin)
├── feedctl/        cli                                  (lib + bin)
└── feedgen/        fixtures, serve, cli                 (lib + bin)

feedd    ──> feedhub-core            feedgen ──> feedhub-core (RFC 822 reuse)
feedctl  ──> feedhub-core
feedd    ┄┄> feedgen   (dev-only, for e2e)
feedctl  ┄┄> feedd, feedgen  (dev-only, for the CLI tests)
```

Each binary crate carries a lib target and a `main.rs` that is a pure shim
(`fn main() { run(Cli::parse()) }`). If `main.rs` declared its own modules, the
tree would compile twice into two disjoint type universes and the tests would be
exercising a different build than the binary.

`serve` takes an already-bound `TcpListener` so callers read `local_addr()`
before the server task spawns. That is why the suite contains no sleeps and no
readiness polling.

## Crate choices

| need | pick | why |
|---|---|---|
| HTTP server | `axum` | standard, clean extractors |
| HTTP client | `reqwest` (rustls) | no OpenSSL system dep |
| SQLite | `rusqlite` (`bundled`) | no system sqlite; trades it for a C toolchain, noted in the README |
| XML | `quick-xml` | exposes Text vs CData separately — exactly what the pinned text rule needs |
| dates | `chrono` + hand-rolled RFC 822 | see D5 |
| CLI | `clap` (derive) | exits 2 on usage errors, which the spec wants anyway |

## Decisions

Decisions marked **revised** were wrong in the first draft and changed under
review. Both revisions were on pinned semantics, and both were caught by
independent reviewers rather than by me.

### D1 — storage keeps an integer sort key beside the RFC 3339 text *(revised)*

`published_at` TEXT is what the API returns; `published_at_ms` INTEGER is the
ordering and range key. Instants normalize to millisecond precision so the two
columns can never disagree.

*First draft:* store only fixed-width `%Y-%m-%dT%H:%M:%SZ` text and rely on
lexicographic compare matching chronological compare.

*Why it changed:* the property holds only for 4-digit years and only if
precision is uniform — `'.'` (0x2E) sorts before `'Z'` (0x5A), so
`00:00:00.5Z` compares *less than* `00:00:00Z` despite being later. It also
composed with D2 into a real correctness hole (below). An integer key makes the
whole class of bug unreachable and lets the text form stay faithful to the
source. Covered by `millis_key_orders_sub_second_instants_that_text_would_misorder`.

### D2 — query bounds compare as true instants, with no ceiling *(revised — deleted)*

*First draft:* truncate fractional seconds on ingest, then ceil sub-second
`since`/`until` bounds to the next whole second.

*Why it changed:* the ceiling logic is correct *in isolation* — but its proof
assumes the stored value **is** the true instant, and truncation makes that
false. An entry published `12:00:00.9Z` stores as `12:00:00`; a query
`since=12:00:00.5Z` ceils to `12:00:01` and excludes it, though `.9 >= .5`. The
spec says "compared as UTC instants," so filtering against a lossy value breaks
the pin. Two individually-defensible decisions, jointly wrong.

D1's integer key dissolves this: bounds convert to epoch-ms and compare
directly. `since <= t < until`, exactly. Covered by
`a_sub_second_bound_compares_as_a_true_instant`.

### D3 — ordering is `(published_at_ms IS NULL) ASC, published_at_ms DESC, id ASC`

Written out rather than `NULLS LAST` for legibility at the call site. (The
first draft justified this as version portability, which was void — `bundled`
pins the SQLite version at compile time. Right decision, wrong reason.)

### D4 — the connection sits behind `std::sync::Mutex`, and `tokio::sync::Mutex` is banned

`std::sync::MutexGuard` is `!Send`, so holding one across an `.await` in an axum
handler is a **compile error**. That statically guarantees the invariant the
store depends on: network I/O never happens while the DB lock is held.

The subtlety worth writing down: clippy's `await_holding_lock` deliberately does
*not* fire for `tokio::sync::Mutex`, which is built to be held across awaits. So
"fixing" the `!Send` error by swapping mutex types would silently delete both
guardrails at once. The type choice is the enforcement; the lint is not.

Cost: handlers do blocking SQLite on a runtime worker under a global lock. At
this scale that is the right trade for a compiler-enforced invariant, and it is
a deliberate choice rather than an oversight.

### D5 — the RFC 822 parser is hand-rolled

RFC 2822 §4.3 declares the obsolete US zone abbreviations meaningless and says
to read them as `-0000`. The spec pins `EST`/`PDT`/etc. to real offsets. Rather
than depend on chrono's interpretation of an ambiguous clause, we resolve the
pinned table ourselves. ~110 lines, table-driven tests over every pinned zone,
plus optional seconds, optional day-of-week, single-digit days, 2/3/4-digit
years, and a garbage-rejection set.

### D6 — items with no identity are skipped **and counted**

An RSS item with neither `guid` nor `link`, or an Atom entry with no `id`, has
no stable dedupe key and would duplicate on every refresh. Skipping is the
honest failure — but skipping *silently* is not: a feed of identity-less items
would be indistinguishable from an empty one. `ParseOutcome` carries
`skipped_without_identity`, and the fetcher logs it at WARN.

### D7 — `new_entries` uses `INSERT OR IGNORE` + `UPDATE` *(revised)*

*First draft:* `INSERT ... ON CONFLICT DO UPDATE`, discriminating insert from
update via `changes()`.

*Why it changed:* **that mechanism does not exist.** SQLite reports 1 row
changed for *both* branches of an upsert. `new_entries` would have equaled the
feed's entire item count on every refresh, forever.

`INSERT OR IGNORE` reports `changes() == 1` on a true insert and `0` on
conflict — *that* discriminates. An unconditional `UPDATE` follows, with `id`
and `fetched_at` absent from the SET list, so the entry keeps both per the pin.
It also handles a guid repeated within one document: first occurrence inserts,
second updates, count stays 1.

Verified by mutation: reinstating the `changes()` version makes
`refetching_identical_content_without_a_304_still_adds_nothing` report 3
instead of 0.

### D8 — feed `title` is `COALESCE`d; entry fields are clobbered

The spec pins entry updates as overwriting title/link/summary/published_at, so
that is unconditional. It says nothing about a feed whose `<title>` disappears,
where blanking a known title would be strictly worse than keeping it. So the
feed title only advances on a title-bearing success, and a failure never touches
it.

### D9 — `q` uses `instr(lower(title), lower(?))`, not `LIKE '%..%'`

`LIKE` would treat `%` and `_` *in the search term* as wildcards, so
`q=100%` would match everything. `instr` has no metacharacters, and SQLite's
`lower()` is ASCII-only — which is exactly the pinned folding rule.

### D10 — `limit` clamps at 500; bad values are 422

The spec pins a max, not a rejection, so clamping is the friendlier read.
Unparseable or negative values are 422 in the pinned error envelope. Query
parsing is hand-rolled from a string map because a typed `Query<T>` would emit
axum's default rejection body instead.

## Test strategy

149 tests. The ones that carry weight:

| test | holds the line on |
|---|---|
| `refetching_identical_content_without_a_304_still_adds_nothing` | D7. Requires feedgen's `--no-conditional`; a 304 never reaches the upsert, so the 304 test *cannot* catch this |
| `a_second_refresh_sends_if_none_match_and_gets_a_304` | that feedd **sends** the validators. Without asserting the request log, the etag column could be write-only and everything else would still pass |
| `a_sub_second_bound_compares_as_a_true_instant` | D1+D2 |
| `an_upstream_edit_updates_the_entry_in_place` | id and fetched_at preserved across a real HTTP round trip |
| `a_malformed_feed_records_last_error_and_spares_its_neighbours` | failure isolation |
| `a_poll_tick_refreshes_every_feed_and_isolates_failures` | the poll loop, ticked explicitly |
| `feed_list_columns_line_up_with_the_header` | a bug `contains()` could not see |
| `rfc822_all_pinned_zone_names` | every zone in the pinned table |

## NOT in scope

- Auth, rate limiting, cursor pagination (offset paging is pinned)
- Atom `content type="xhtml"` tree reconstruction (text extraction suffices)
- Feed discovery, OPML
- Non-UTF-8 feeds. The spec pins UTF-8; other encodings are a parse error
  recorded to `last_error`. `bytes()` (not `text()`) is used precisely so
  reqwest cannot guess an encoding out from under the parser.
- Real-network fetching in tests (explicitly forbidden)
