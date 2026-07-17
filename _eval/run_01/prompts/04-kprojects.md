Build the loglens CLI described below, start to finish, following this repository's project conventions.

## Project: loglens — a web-access-log analysis CLI

Build this project start to finish in the current repository: working code,
tests, documentation, all committed to git.

### What it is

`loglens` is a Python command-line tool that analyzes web server access logs
in Combined Log Format (CLF). Example lines for format reference:

```
203.0.113.7 - - [12/Jul/2026:06:25:24 +0000] "GET /index.html HTTP/1.1" 200 5413 "https://example.com/" "Mozilla/5.0"
198.51.100.22 - alice [12/Jul/2026:07:01:02 +0000] "POST /api/orders HTTP/1.1" 201 512 "-" "curl/8.5.0"
192.0.2.9 - - [12/Jul/2026:07:15:44 +0000] "GET /missing HTTP/1.1" 404 153 "-" "Mozilla/5.0"
```

### Requirements

- Python 3.12+, project managed with `uv` (`pyproject.toml`); lint-clean under
  `ruff check`; tests with `pytest`.
- Installable console entry point named `loglens`.

Subcommands (each takes a LOGFILE path):

1. `loglens summary LOGFILE` — total requests, unique client IPs, first and
   last timestamp, error rate (percentage of 4xx+5xx responses).
2. `loglens top LOGFILE --by {ip,path,status} [-n N]` — top N (default 10)
   values by request count, descending; ties broken by value ascending.
3. `loglens errors LOGFILE [--since ISO8601] [--until ISO8601]` — 4xx/5xx
   requests grouped by (status, path) with counts, most frequent first.
4. `loglens hourly LOGFILE` — request count per hour of day (00–23) rendered
   as a text histogram.

Global option `--format {text,json}` (default `text`). JSON output must be a
single valid JSON document on stdout.

Error handling:

- Malformed lines are skipped and counted; the count is reported on stderr,
  never on stdout.
- Exit codes: 0 success; 1 the file contains no valid log lines; 2 LOGFILE is
  missing or unreadable.

Sample data: create `tests/fixtures/sample.log` with at least 30 lines
covering multiple IPs, paths, and statuses (2xx/3xx/4xx/5xx), several
different hours, and at least 2 malformed lines.

### Definition of done

- All four subcommands implemented per spec; `pytest` passes; `ruff check` is
  clean.
- `README.md` covering install (with `uv`) and usage examples for every
  subcommand.
- Meaningful git commits as you go; final state committed.
- A single command to run all checks (`just check` or a documented
  equivalent).

### Operating rules

- Work autonomously start to finish; do not stop to ask questions unless
  truly blocked.
- Use whatever workflow, planning, and review machinery your environment
  provides.
