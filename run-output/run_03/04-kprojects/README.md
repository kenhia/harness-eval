# loglens — Web Access Log Analysis CLI

`loglens` is a Python command-line tool for analyzing web server access logs in Combined Log Format (CLF). It provides fast, flexible analysis with support for multiple output formats.

## Installation

### With uv (recommended)

```bash
uv pip install -e .
```

### With pip

```bash
pip install -e .
```

After installation, the `loglens` command will be available.

## Usage

### General syntax

```
loglens [--format {text,json}] <subcommand> <logfile> [options]
```

Global options:
- `--format {text,json}` — Output format (default: `text`)

### Subcommands

#### summary

Get overall statistics about the log file.

```bash
loglens summary access.log
loglens --format json summary access.log
```

Output includes:
- Total number of requests
- Number of unique client IPs
- First and last timestamps in the log
- Error rate (percentage of 4xx and 5xx responses)

#### top

Find the most frequently occurring values in the log.

```bash
loglens top access.log --by ip -n 20
loglens top access.log --by path
loglens top access.log --by status
loglens --format json top access.log --by ip -n 5
```

Options:
- `--by {ip,path,status}` — What to count (default: `ip`)
- `-n N` — Show top N results (default: 10)

Results are sorted by count (descending), with ties broken alphabetically by value.

#### errors

Analyze HTTP errors (4xx and 5xx status codes).

```bash
loglens errors access.log
loglens errors access.log --since 2026-07-12T10:00:00
loglens errors access.log --until 2026-07-12T20:00:00
loglens errors access.log --since 2026-07-12T10:00:00 --until 2026-07-12T20:00:00
loglens --format json errors access.log
```

Options:
- `--since ISO8601` — Only include errors after this timestamp
- `--until ISO8601` — Only include errors before this timestamp

Results show errors grouped by (status code, path) with counts, ordered by frequency.

#### hourly

Analyze request distribution by hour of day.

```bash
loglens hourly access.log
loglens --format json hourly access.log
```

Output shows request counts for each hour (00:00 through 23:00) as a text histogram or JSON.

## Examples

### Typical workflows

```bash
# Quick overview of the log
loglens summary access.log

# What are the most common paths?
loglens top access.log --by path -n 20

# Which IPs are generating errors?
loglens errors access.log

# When does traffic spike?
loglens hourly access.log

# Export errors as JSON for scripting
loglens --format json errors access.log > errors.json

# Check errors during off-hours
loglens errors access.log --since 2026-07-12T18:00:00 --until 2026-07-13T06:00:00
```

## Exit Codes

- `0` — Success
- `1` — No valid log entries found
- `2` — Log file not found or not readable

## Log Format

`loglens` expects logs in Combined Log Format (CLF), the standard format for most web servers:

```
203.0.113.7 - - [12/Jul/2026:06:25:24 +0000] "GET /index.html HTTP/1.1" 200 5413 "https://example.com/" "Mozilla/5.0"
```

Fields:
- Client IP address
- Remote user (or `-`)
- Authenticated user (or `-`)
- Request timestamp
- HTTP request (method, path, protocol)
- Status code
- Bytes sent
- Referrer
- User agent

## Error Handling

Malformed log lines are silently skipped, and a count of skipped lines is printed to stderr (not stdout, so it won't pollute JSON output).

## Testing

Run the test suite:

```bash
uv run pytest
```

With coverage:

```bash
uv run pytest --cov=loglens
```

## Development

Code quality checks:

```bash
just check      # Run all checks (lint + tests)
just lint       # Lint with ruff
just fmt        # Format code
just test       # Run tests
```

The project uses:
- **linting**: ruff
- **testing**: pytest
- **package management**: uv
