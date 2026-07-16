# loglens

Analyze web server access logs in Combined Log Format (CLF) from your terminal.
Four questions, four subcommands, text or JSON. No dependencies outside the standard
library.

```
203.0.113.7 - - [12/Jul/2026:06:25:24 +0000] "GET /index.html HTTP/1.1" 200 5413 "https://example.com/" "Mozilla/5.0"
```

That is the format loglens reads. If your log lines look like that, you are ready. Both
Combined (with referer and user-agent) and Common (without) are accepted.

## Install

With [uv](https://docs.astral.sh/uv/):

```bash
uv tool install git+https://github.com/ken/loglens
loglens --version
```

From a clone, for development:

```bash
git clone https://github.com/ken/loglens && cd loglens
uv sync
uv run loglens summary tests/fixtures/sample.log
```

Requires Python 3.12+.

## Usage

Every subcommand takes a LOGFILE path. `--format {text,json}` works either before or
after the subcommand.

### `summary` — the shape of the traffic

```console
$ loglens summary access.log
Total requests:  34
Unique IPs:      4
First request:   2026-07-12T06:25:24+00:00
Last request:    2026-07-12T14:44:29+00:00
Error rate:      29.41%
```

Error rate is the percentage of valid requests answered with 4xx or 5xx. `First` and
`last` are the earliest and latest timestamps, not the first and last lines — servers
log request *start* time but write the line at *completion*, so a busy log is
near-sorted rather than sorted.

### `top` — what is most requested

```console
$ loglens top access.log --by path -n 5
  COUNT  PATH
      9  /api/orders
      8  /index.html
      3  /api/health
      3  /style.css
      2  /about.html
```

`--by` takes `ip`, `path`, or `status`. `-n` defaults to 10. Results are ordered by
count descending, and ties are broken by value ascending, so the output is stable no
matter how the file is ordered.

```console
$ loglens top access.log --by status -n 3
  COUNT  STATUS
     18  200
      5  404
      3  304
```

### `errors` — what is failing, and where

```console
$ loglens errors access.log --since 2026-07-12T08:00:00Z
  COUNT  STATUS  PATH
      2     404  /wp-login.php
      2     500  /api/orders
      1     403  /admin
      1     404  /.env
      1     404  /missing
      1     502  /api/orders

Total error requests: 8
```

`--since` and `--until` accept ISO8601. Bounds are **half-open** (`since <= t < until`),
so consecutive windows chain without double-counting the request on the seam. A value
without a timezone (`2026-07-12`, `2026-07-12T08:00`) is read as **UTC**; add `Z` or an
offset to be explicit. Comparison is by absolute instant, so a `-0700` log and a UTC
`--since` line up correctly.

### `hourly` — when the traffic arrives

```console
$ loglens hourly access.log
HOUR   COUNT
00         0
01         0
...
06         4  ################################
07         5  ########################################
08         5  ########################################
09         5  ########################################
10         4  ################################
11         4  ################################
12         3  ########################
13         2  ################
14         2  ################
15         0
...
23         0

Total requests: 34
```

All 24 hours are always shown, including zeros — a histogram that omits empty hours
lies about the shape of the day. Requests are bucketed by the **wall-clock hour written
in the log line's own timezone offset**, which is what "our traffic peaks at 14:00"
means to whoever runs the server. A file spanning a DST fall-back legitimately shows
hour 01 twice; that is the log, not a bug.

### JSON

```console
$ loglens --format json summary access.log
{
  "total_requests": 34,
  "unique_ips": 4,
  "first": "2026-07-12T06:25:24+00:00",
  "last": "2026-07-12T14:44:29+00:00",
  "error_rate": 29.41
}
```

```console
$ loglens summary access.log --format json | jq .error_rate
29.41
```

On exit 0, stdout is a single valid JSON document and nothing else. Diagnostics go to
stderr, so `| jq` is always safe. **Check the exit code before parsing** — on exit 1 and
2, stdout is empty.

## Malformed lines

Lines that are not valid CLF are skipped, counted, and reported on **stderr**, never
stdout:

```console
$ loglens summary access.log > report.txt
loglens: skipped 2 of 36 malformed lines (first at line 10: does not match Combined Log Format: this line is not a log line at all)
```

Nothing is printed when every line parses, so a nightly cron job stays quiet.

"Malformed" means the CLF *structure* is invalid. It deliberately does not mean:

| Line | Treatment | Why |
|---|---|---|
| `"GET /a HTTP/1.1" 304 -` | Valid, `bytes = null` | `-` is the standard placeholder for "no body sent". Rejecting it would drop most 304s and HEADs and inflate the error rate. |
| `"\x16\x03\x01..."` (TLS bytes) | Valid, method `null`, path is the raw request | A well-formed record with a real status. This is scanner and probe traffic — exactly what you opened the tool to find. |
| `"-"`, `"GET /a b HTTP/1.1"` | Same | Same. |
| `"...says \"hi\""` | Valid, quote unescaped | Escaped quotes are parsed properly rather than shifting later fields. |

A status that is not a three-digit integer *does* make a line malformed.

## Notes and limits

- **Paths exclude the query string.** `/search?q=hello` counts as `/search`. That is
  what RFC 3986 calls a path, and it keeps `top --by path` from being dominated by
  whatever a scanner enumerated.
- **Terminal escapes are neutralized.** Paths and user agents are attacker-controlled;
  control characters are escaped on output so a logged `\x1b[2J` cannot clear your
  scrollback.
- **`%h` is the client that reached your server.** Behind a CDN or proxy that is the
  edge, not the end user, so `unique_ips` can read surprisingly low. CLF does not carry
  `X-Forwarded-For`.
- **Memory scales with distinct values**, not file size. The file streams, but
  `top --by path` holds one counter per distinct path and `summary` one per distinct IP.
- IP addresses are personal data in some jurisdictions, and `--format json` exports them.
- Not supported yet: stdin/pipes, multiple files, and gzipped logs. See `TODOS.md`.

## Exit codes

| Code | Meaning |
|---|---|
| 0 | Success. A filter that matches nothing is still a success. |
| 1 | The file contains no valid log lines (empty, or nothing parsed). stdout is empty. |
| 2 | Input error: LOGFILE missing, unreadable, or a directory — or a usage error. |
| 70 | Internal error. Distinct from the above so a crash never reads as a result. |

```bash
loglens summary access.log || case $? in
  1) echo "log is empty or unparseable" ;;
  2) echo "bad arguments or unreadable file" ;;
esac
```

## Development

```bash
uv sync          # install dev dependencies
just check       # lint + format check + tests + JSON smoke test
```

`just check` is the single command that runs everything. Without `just`:

```bash
uv run ruff check . && uv run ruff format --check . && uv run pytest
```

Individual targets: `just lint`, `just fmt`, `just test`, `just smoke`, `just format`
(applies fixes).

### Layout

```
src/loglens/
  cli.py      argparse wiring, exit-code policy, stderr reporting
  source.py   file reading: encoding, line numbering, parse stats
  parse.py    CLF line -> LogEntry | ParseError
  analyze.py  entries -> plain result dicts
  render.py   result dicts -> text or JSON
```

Analysis returns plain data and rendering is the only layer that knows about output
format, so `--format json` is not a second code path with its own bugs.

`tests/fixtures/sample.log` is a well-behaved log. `tests/fixtures/hostile.log` is the
one that matters: escaped quotes, binary request-lines, mixed timezone offsets, invalid
UTF-8, CRLF, ANSI escapes, and a truncated final line.
