# Roadmap

> The general plan for this project. Keep it current; detail lives in the
> sprint records.

feedhub is a Rust feed-aggregation service: `feedd` (server), `feedctl`
(client), `feedgen` (fixtures), and the `feedcore` shared library.

## Now

- Sprint 001 — end-to-end build. Shipped: workspace, feedcore parser +
  date/text rules, feedd (SQLite + REST + conditional-GET fetch + polling),
  feedgen (serve + fixtures), feedctl, e2e test, docs, CI gate.

## Next

- Concurrent fan-out for `POST /api/refresh`.
- SIGTERM handling alongside the existing Ctrl-C graceful shutdown.

## Later / Ideas

- Store full entry content; richer `feedctl` table output.
- Per-feed poll intervals / backoff on repeated failures.
- Pagination cursors for `GET /api/entries`.
