# loglens

A command-line tool for analyzing web server access logs in
[Combined Log Format](https://httpd.apache.org/docs/current/logs.html#combined) (CLF).

```
203.0.113.7 - - [12/Jul/2026:06:25:24 +0000] "GET /index.html HTTP/1.1" 200 5413 "https://example.com/" "Mozilla/5.0"
```

## Install

loglens is managed with [uv](https://docs.astral.sh/uv/) and needs Python 3.12+.

```sh
git clone <this-repo>
cd loglens
uv sync
```

`uv sync` creates a virtualenv in `.venv` with the `loglens` entry point installed:

```sh
uv run loglens --help
```

To install it onto your PATH instead:

```sh
uv tool install .
loglens --help
```

## Usage

Every subcommand takes a `LOGFILE` path. The global `--format` option selects
`text` (default) or `json` output, and may be given before or after the
subcommand.

### `summary` — overall statistics

Total requests, unique client IPs, the first and last timestamp, and the error
rate (the percentage of responses with a 4xx or 5xx status).

```sh
$ loglens summary tests/fixtures/sample.log
Total requests:  32
Unique IPs:      5
First timestamp: 2026-07-12T06:25:24+00:00
Last timestamp:  2026-07-12T23:58:47+00:00
Error rate:      31.25% (10 of 32)
```

### `top` — most frequent values

Ranks IPs, paths, or statuses by request count, descending. Ties are broken by
value ascending. `-n` sets how many rows to show (default 10).

```sh
$ loglens top tests/fixtures/sample.log --by path -n 3
PATH         COUNT
/api/orders  8
/index.html  7
/missing     4
```

```sh
$ loglens top tests/fixtures/sample.log --by status -n 3
STATUS  COUNT
200     15
404     4
201     3
```

### `errors` — 4xx/5xx breakdown

Groups error responses by (status, path), most frequent first. `--since` and
`--until` take ISO 8601 timestamps and are both inclusive; a timestamp without a
timezone is interpreted as UTC.

```sh
$ loglens errors tests/fixtures/sample.log
STATUS  PATH         COUNT
404     /missing     4
403     /admin       2
500     /api/orders  2
401     /api/orders  1
503     /health      1
```

```sh
$ loglens errors tests/fixtures/sample.log --since 2026-07-12T12:00:00 --until 2026-07-12T23:59:59
STATUS  PATH         COUNT
401     /api/orders  1
403     /admin       1
404     /missing     1
500     /api/orders  1
503     /health      1
```

### `hourly` — requests per hour of day

A text histogram of request counts bucketed by hour (00–23). Bars are scaled to
the busiest hour.

```sh
$ loglens hourly tests/fixtures/sample.log
00  0
01  0
02  0
03  0
04  0
05  0
06  2  ################
07  5  ########################################
08  5  ########################################
09  5  ########################################
10  0
11  0
12  4  ################################
13  4  ################################
14  3  ########################
15  0
16  0
17  0
18  0
19  0
20  0
21  0
22  0
23  4  ################################
```

### JSON output

`--format json` prints a single JSON document on stdout, suitable for piping
into `jq`:

```sh
$ loglens --format json summary tests/fixtures/sample.log
{
  "total_requests": 32,
  "unique_ips": 5,
  "first_timestamp": "2026-07-12T06:25:24+00:00",
  "last_timestamp": "2026-07-12T23:58:47+00:00",
  "errors": 10,
  "error_rate": 31.25
}

$ loglens --format json top tests/fixtures/sample.log --by ip -n 2 | jq '.results[0].value'
"203.0.113.7"
```

## Behavior notes

- **Malformed lines** are skipped and counted. The count is reported on stderr,
  so it never contaminates `text` or `json` output on stdout. Blank lines are
  ignored and not counted as malformed.
- **Timestamps** keep the timezone offset recorded in the log.
- **JSON types** are stable across subcommands: an HTTP status is always a
  number (`"value": 200` from `top --by status`, `"status": 404` from
  `errors`), while IPs and paths are strings.
- **Histogram bars** are scaled to the busiest hour, and any hour with at least
  one request always gets at least one block, so a quiet hour is never rendered
  as an empty one.

### Exit codes

| Code | Meaning                                  |
| ---- | ---------------------------------------- |
| 0    | Success                                  |
| 1    | The file contains no valid log lines     |
| 2    | LOGFILE is missing or unreadable, or the command line was invalid |

## Development

Run the full check suite — lint and tests:

```sh
just check
```

Without [just](https://just.systems), the equivalent is:

```sh
uv run ruff check .
uv run pytest
```

Other recipes: `just test`, `just lint`, `just fmt`.
