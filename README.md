# loglens

A Python command-line tool for analyzing web server access logs in Combined Log Format (CLF).

## Installation

Install with `uv`:

```bash
uv pip install -e .
```

Or install directly:

```bash
uv sync
```

The `loglens` command will be available after installation.

## Usage

### summary

Display overall statistics about the log file:

```bash
loglens summary /path/to/access.log
```

Output (text):
```
Total requests: 8
Unique client IPs: 6
First timestamp: 2026-07-12T06:25:24
Last timestamp: 2026-07-12T10:20:44
Error rate: 37.5%
```

JSON output:
```bash
loglens --format json summary /path/to/access.log
```

### top

Show the most frequent values by category:

```bash
# Top 10 IPs (default)
loglens top /path/to/access.log --by ip

# Top 5 paths
loglens top /path/to/access.log --by path -n 5

# Top 20 status codes
loglens top /path/to/access.log --by status -n 20
```

Output (text):
```
Top 10 by ip:
  203.0.113.7: 2
  198.51.100.22: 2
  192.0.2.9: 1
```

JSON output:
```bash
loglens --format json top /path/to/access.log --by ip
```

### errors

Show 4xx and 5xx errors grouped by status and path:

```bash
# All errors
loglens errors /path/to/access.log

# Errors in a time range
loglens errors /path/to/access.log --since 2026-07-12T08:00:00 --until 2026-07-12T10:00:00
```

Output (text):
```
Errors (status, path):
  404 /missing: 1
  401 /api/login: 1
  500 /api/products: 1
```

JSON output:
```bash
loglens --format json errors /path/to/access.log
```

### hourly

Show request count by hour of day as a histogram:

```bash
loglens hourly /path/to/access.log
```

Output (text):
```
00: 0
01: 0
02: 0
03: 0
04: 0
05: 0
06: ████ 1
07: ███████ 2
08: ███████ 2
09: ████ 1
10: ██ 1
...
```

JSON output:
```bash
loglens --format json hourly /path/to/access.log
```

## Error Handling

- Malformed log lines are skipped and a count is reported to stderr
- Exit codes:
  - `0`: Success
  - `1`: No valid log lines found
  - `2`: File not found or unreadable

## Development

### Run tests

```bash
pytest
```

### Lint

```bash
ruff check
```

### Run all checks

```bash
just check
```

## Log Format

loglens expects the Combined Log Format (CLF):

```
IP - - [timestamp] "METHOD /path HTTP/version" status size "referer" "user_agent"
```

Example:
```
203.0.113.7 - - [12/Jul/2026:06:25:24 +0000] "GET /index.html HTTP/1.1" 200 5413 "https://example.com/" "Mozilla/5.0"
```
