# 001 — build loglens CLI

**Goal:** ship `loglens`, a Python CLI that analyzes Combined Log Format (CLF)
web access logs, with four subcommands, text/JSON output, tests, and docs — as
tracked by korg WI #445 under project `loglens` (id 29).

## What shipped

- `uv`-managed project (`pyproject.toml`, Python 3.12+, hatchling build) with a
  `loglens` console entry point.
- Package under `src/loglens/`:
  - `parser.py` — regex CLF parser → `LogRecord`; `parse_lines` returns valid
    records plus a malformed count; blank lines ignored.
  - `analyze.py` — `summary`, `top`, `errors` (with `--since`/`--until`
    windowing), `hourly` aggregations.
  - `cli.py` — argparse CLI, global `--format {text,json}`, spec'd exit codes.
- `tests/fixtures/sample.log` — 34 lines (32 valid, 2 malformed) spanning many
  IPs, paths, statuses (2xx/3xx/4xx/5xx), and hours.
- 28 pytest tests across parser, analysis, and end-to-end CLI behavior.
- `README.md` (install + usage per subcommand), `justfile` (`just check`).

## Decisions

- **CLF parsing via a single regex.** Simple, dependency-free, and robust
  enough for well-formed CLF; anything that fails the shape, has a bad
  timestamp, or lacks a 3-part request line is counted malformed.
- **Malformed count → stderr only.** Keeps stdout a single clean document so
  `--format json` is always safe to pipe into `jq`.
- **Exit codes:** 0 ok, 1 no valid lines, 2 missing/unreadable file (also used
  for bad `--since`/`--until` values and argparse errors).
- **Timezone handling:** log timestamps are tz-aware (`+0000`); a naive
  `--since`/`--until` is assumed UTC so comparisons never raise.
- **Ties:** `top` breaks count ties by value ascending; `errors` orders by
  count desc then status then path.

## Checks

`just check` runs `ruff check .` (clean) and `pytest` (28 passing).

## Follow-ups

- Optional: gzip input support, configurable histogram width, `ty` typecheck in
  CI once the astral typechecker stabilizes.
