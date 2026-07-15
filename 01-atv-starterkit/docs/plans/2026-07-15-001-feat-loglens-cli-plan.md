---
title: "feat: Build loglens web-access-log analysis CLI"
type: feat
status: completed
date: 2026-07-15
---

# feat: Build loglens web-access-log analysis CLI

## Overview

Build `loglens`, a Python 3.12+ CLI that parses web server access logs in Combined
Log Format (CLF) and reports aggregate statistics through four subcommands:
`summary`, `top`, `errors`, and `hourly`. The project is managed with `uv`,
linted with `ruff`, and tested with `pytest`. Output can be rendered as human
text or a single JSON document via a global `--format` option.

## Problem Frame

Operators need quick, dependency-light insight into web access logs without
standing up a full log pipeline. `loglens` gives them a single installable
console tool that answers common questions (traffic volume, top talkers, error
breakdown, time-of-day distribution) directly from a CLF log file.

## Requirements Trace

- R1. Python 3.12+, `uv`-managed (`pyproject.toml`), `ruff check` clean, `pytest` passing.
- R2. Installable console entry point `loglens`.
- R3. `summary LOGFILE` — total requests, unique client IPs, first/last timestamp, error rate (% of 4xx+5xx).
- R4. `top LOGFILE --by {ip,path,status} [-n N]` — top N (default 10) by count desc; ties broken by value ascending.
- R5. `errors LOGFILE [--since ISO8601] [--until ISO8601]` — 4xx/5xx grouped by (status, path) with counts, most frequent first.
- R6. `hourly LOGFILE` — request count per hour of day (00–23) as a text histogram.
- R7. Global `--format {text,json}` (default `text`); JSON output is a single valid JSON document on stdout.
- R8. Malformed lines skipped and counted; count reported on stderr, never stdout.
- R9. Exit codes: 0 success; 1 file has no valid log lines; 2 LOGFILE missing/unreadable.
- R10. `tests/fixtures/sample.log` ≥30 lines: multiple IPs/paths/statuses (2xx/3xx/4xx/5xx), several hours, ≥2 malformed lines.
- R11. `README.md` with `uv` install + usage examples for every subcommand.
- R12. Single command to run all checks (`just check` or documented equivalent); meaningful git commits.

## Scope Boundaries

- Only Combined Log Format is parsed. Other formats (e.g. JSON logs, W3C) are non-goals.
- No streaming/tail mode; the whole file is read per invocation.
- No network, geolocation, or reverse-DNS lookups.
- Timestamps parsed with their CLF timezone offset; `--since`/`--until` compared as timezone-aware datetimes.

## Context & Research

### Relevant Code and Patterns

- Greenfield project; no existing source. Follow standard `src/` layout package
  conventions with `argparse` from the stdlib to avoid extra dependencies.
- CLF line shape (reference):
  `IP - user [dd/Mon/yyyy:HH:MM:SS +0000] "METHOD path proto" status size "referer" "ua"`

### Institutional Learnings

- None on file (`docs/solutions/` empty for this topic).

### External References

- Combined Log Format is a well-established, stable spec; no external research needed.

## Key Technical Decisions

- **Stdlib-only runtime (`argparse`, `re`, `datetime`, `json`, `collections`):** keeps the tool dependency-free and easy to install; the CLF grammar is simple enough for a single compiled regex.
- **Single regex parser returning a typed record (dataclass):** centralizes parsing; a line that fails the regex OR fails field coercion (status/size int, timestamp parse) counts as malformed.
- **`src/loglens/` package layout with `pyproject.toml` `[project.scripts]` entry point:** satisfies R2 and standard `uv` packaging.
- **Separation of concerns:** `parser.py` (line → record), `analyze.py` (records → stat structures), `cli.py` (arg parsing, dispatch, rendering, exit codes). Renderers produce text or a JSON-serializable dict per command.
- **Exit-code precedence:** file access checked first (exit 2), then "no valid lines" (exit 1), else 0. Malformed count is orthogonal and always goes to stderr.
- **`hourly` fills all 24 hours** (zero counts included) so the histogram is always 00–23.
- **`top --by status`** sorts ties by value ascending as strings of the status code (spec says "value ascending"; statuses compared numerically for stable intuitive order, documented in code).

## Open Questions

### Resolved During Planning

- JSON shape per command: each command emits an object keyed by its domain (e.g. `summary` → `{total_requests, unique_ips, first_timestamp, last_timestamp, error_rate}`). `malformed_lines` count is NOT in JSON stdout (goes to stderr per R8).
- Timestamp serialization in JSON: ISO 8601 strings.
- `--since`/`--until` only apply to `errors` per spec; parsed as ISO 8601, timezone-aware (naive input assumed UTC).

### Deferred to Implementation

- Exact histogram bar scaling (fixed max width, scaled to peak hour).

## Implementation Units

- [x] **Unit 1: Project scaffold and packaging**

**Goal:** Create `uv`-managed project with entry point, ruff/pytest config, and a `justfile`.

**Requirements:** R1, R2, R12

**Files:**
- Create: `pyproject.toml`, `src/loglens/__init__.py`, `justfile`, `.gitignore`

**Approach:**
- `pyproject.toml` with `requires-python = ">=3.12"`, `[project.scripts] loglens = "loglens.cli:main"`, dev deps `pytest`, `ruff`; configure ruff and pytest (`testpaths = ["tests"]`).
- `justfile` with a `check` recipe running `ruff check .` then `pytest`.

**Test expectation:** none — scaffolding; exercised by later units.

**Verification:** `uv sync` succeeds; `uv run loglens --help` runs after CLI exists.

- [x] **Unit 2: CLF parser**

**Goal:** Parse a CLF line into a typed record; identify malformed lines.

**Requirements:** R8 (malformed detection)

**Files:**
- Create: `src/loglens/parser.py`
- Test: `tests/test_parser.py`

**Approach:**
- `LogRecord` dataclass: `ip`, `user`, `timestamp` (aware datetime), `method`, `path`, `protocol`, `status` (int), `size` (int or 0 for `-`), `referer`, `user_agent`.
- `parse_line(line) -> LogRecord | None`: compiled regex + field coercion; return `None` on any failure.
- `parse_lines(iterable) -> tuple[list[LogRecord], int]`: returns records and malformed count.

**Test scenarios:**
- Happy path: canonical GET line parses all fields; timestamp is timezone-aware with correct offset.
- Happy path: `size` of `-` coerces to 0; named user (`alice`) captured.
- Edge case: blank line and comment-like line → `None`.
- Error path: bad status (non-numeric), truncated line, bad date → `None`.
- Happy path: `parse_lines` over mixed input returns correct record list and malformed count.

**Verification:** `pytest tests/test_parser.py` passes.

- [x] **Unit 3: Analysis functions**

**Goal:** Pure functions computing each command's statistics from records.

**Requirements:** R3, R4, R5, R6

**Files:**
- Create: `src/loglens/analyze.py`
- Test: `tests/test_analyze.py`

**Approach:**
- `summarize(records)` → total, unique_ips, first_ts, last_ts, error_rate.
- `top(records, by, n)` → ordered list of `(value, count)`, count desc, value asc tiebreak.
- `errors(records, since, until)` → ordered list of `((status, path), count)` filtered by time window, count desc.
- `hourly(records)` → list of 24 counts indexed by hour.

**Test scenarios:**
- Happy path: `summarize` computes error_rate as (4xx+5xx)/total*100; first/last from min/max timestamp.
- Edge case: `summarize` on records with zero errors → 0.0 rate.
- Happy path: `top by=ip/path/status` returns descending counts; ties ordered by value ascending.
- Edge case: `top` with `n` larger than distinct values returns all.
- Happy path: `errors` groups by (status, path), filters `since`/`until` inclusive/exclusive per decision, orders by count desc.
- Happy path: `hourly` returns length-24 list with correct per-hour counts and zeros for empty hours.

**Verification:** `pytest tests/test_analyze.py` passes.

- [x] **Unit 4: CLI, rendering, exit codes**

**Goal:** Wire argparse subcommands, text/JSON renderers, error handling, and exit codes.

**Requirements:** R2, R7, R8, R9

**Files:**
- Create: `src/loglens/cli.py`
- Test: `tests/test_cli.py`

**Approach:**
- `main(argv=None) -> int`. Global `--format`. Subparsers: `summary`, `top` (`--by`, `-n`), `errors` (`--since`, `--until`), `hourly`.
- Read file: `OSError`/missing → stderr message, return 2. Parse; if no valid records → stderr message, return 1.
- Print malformed count to stderr when >0.
- Renderers: `render_text_*` and `render_json` (single `json.dumps` to stdout). `hourly` text = histogram bars scaled to peak.
- `top --by status` renders status codes as strings.

**Test scenarios:**
- Happy path (via `capsys`/`tmp_path`): each subcommand text output contains expected values from a small fixture.
- Happy path: `--format json` produces a single parseable JSON document (`json.loads` succeeds) with expected keys.
- Error path: missing file → exit 2, message on stderr, stdout empty.
- Error path: file with only malformed lines → exit 1, malformed count on stderr.
- Edge case: malformed count reported on stderr, never stdout (assert stdout has no count).
- Happy path: `errors --since/--until` filters output.
- Happy path: `top -n 2` limits results.

**Verification:** `pytest tests/test_cli.py` passes; manual `uv run loglens summary tests/fixtures/sample.log`.

- [x] **Unit 5: Sample fixture and README**

**Goal:** Provide realistic sample data and user documentation.

**Requirements:** R10, R11

**Files:**
- Create: `tests/fixtures/sample.log`, `README.md`

**Approach:**
- `sample.log`: ≥30 lines, multiple IPs/paths, statuses spanning 2xx/3xx/4xx/5xx, timestamps across several hours, ≥2 malformed lines.
- `README.md`: `uv` install/run, and a usage example + sample output for each subcommand, plus `--format json` and `just check`.

**Test expectation:** none for README; `sample.log` is consumed by CLI tests to assert stable outputs.

**Verification:** CLI tests referencing the fixture pass; README examples match actual output.

## System-Wide Impact

- **Interaction graph:** `cli.main` → `parser.parse_lines` → `analyze.*` → renderers. No shared mutable state.
- **Error propagation:** file errors and empty-input handled in `cli`; parser never raises on bad lines (returns `None`).
- **API surface parity:** all four subcommands must support both `--format` values.

## Risks & Dependencies

| Risk | Mitigation |
|------|------------|
| Regex too strict, over-counting malformed lines | Base regex on canonical CLF; cover with parser tests including named-user and `-` size variants. |
| JSON contaminated by malformed-count or logs on stdout | Route all diagnostics to stderr; test asserts stdout is pure JSON. |
| Timezone handling for `--since/--until` | Parse to aware datetimes; assume UTC for naive input; document behavior. |

## Documentation / Operational Notes

- README is the primary doc; `just check` is the single all-checks command (documented equivalent: `uv run ruff check . && uv run pytest`).

## Sources & References

- Combined Log Format (Apache mod_log_config standard).
