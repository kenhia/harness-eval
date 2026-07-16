Fix the reported bug described below.

## Bug report: malformed feed XML is silently treated as a successful empty fetch

Work in the current repository — this is the feedhub codebase you are
maintaining. A user has filed the following bug.

### Summary

When `feedd` refreshes a feed whose body is not well-formed XML (for
example, a truncated document cut off mid-element by a broken upstream
server), the refresh reports success with zero new entries instead of
recording a fetch failure. The feed looks healthy in `feedctl list` even
though its content was garbage.

### Reproduction

1. Serve a file with this content from any local HTTP server (e.g.
   `feedgen serve`):

   ```
   <?xml version="1.0" encoding="UTF-8"?>
   <rss version="2.0"><channel><title>Nightly</title>
   <item><guid>n-1</guid><title>Release no
   ```

   (Note: the document ends mid-tag — the upstream server truncated the
   response.)

2. `feedctl add <url>` then `feedctl refresh <id>`.

### Observed

- `POST /api/feeds/{id}/refresh` returns `{"status": "ok",
  "new_entries": 0, ...}`.
- `GET /api/feeds/{id}` shows `last_error: null`.

### Expected (per the project spec)

> A fetch/parse failure (connection refused, HTTP error, malformed XML)
> is recorded on that feed (`last_error`) and never affects other feeds
> or crashes the server.

The refresh result should report `status: "error"` and the feed's
`last_error` should describe the parse failure. Behavior for feeds that
fetch and parse correctly must not change.

### Operating rules

- Work autonomously start to finish; do not stop to ask questions unless
  truly blocked.
- Use whatever workflow, planning, and review machinery your environment
  provides.
- Commit your work when done.
