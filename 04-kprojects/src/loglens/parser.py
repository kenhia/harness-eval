"""Parsing of Combined Log Format (CLF) access-log lines."""

from __future__ import annotations

import re
from collections.abc import Iterable, Iterator
from dataclasses import dataclass
from datetime import datetime

# Combined Log Format:
# host ident authuser [timestamp] "request" status size "referer" "user-agent"
_LINE_RE = re.compile(
    r'(?P<host>\S+) '
    r'(?P<ident>\S+) '
    r'(?P<authuser>\S+) '
    r'\[(?P<timestamp>[^\]]+)\] '
    r'"(?P<request>[^"]*)" '
    r'(?P<status>\d{3}) '
    r'(?P<size>-|\d+)'
    r'(?: "(?P<referer>[^"]*)" "(?P<useragent>[^"]*)")?'
    r'\s*$'
)

_TS_FORMAT = "%d/%b/%Y:%H:%M:%S %z"


@dataclass(frozen=True)
class LogRecord:
    """A single successfully parsed access-log entry."""

    host: str
    ident: str
    authuser: str
    timestamp: datetime
    method: str
    path: str
    protocol: str
    status: int
    size: int | None
    referer: str
    useragent: str


def parse_line(line: str) -> LogRecord | None:
    """Parse one CLF line into a :class:`LogRecord`, or ``None`` if malformed."""
    match = _LINE_RE.match(line.rstrip("\n"))
    if match is None:
        return None

    try:
        timestamp = datetime.strptime(match["timestamp"], _TS_FORMAT)
    except ValueError:
        return None

    request = match["request"]
    parts = request.split()
    if len(parts) == 3:
        method, path, protocol = parts
    else:
        # Malformed or empty request line — cannot classify by path reliably.
        return None

    size_raw = match["size"]
    size = None if size_raw == "-" else int(size_raw)

    return LogRecord(
        host=match["host"],
        ident=match["ident"],
        authuser=match["authuser"],
        timestamp=timestamp,
        method=method,
        path=path,
        protocol=protocol,
        status=int(match["status"]),
        size=size,
        referer=match["referer"] or "",
        useragent=match["useragent"] or "",
    )


@dataclass
class ParseResult:
    """Outcome of parsing a whole log: valid records plus a malformed count."""

    records: list[LogRecord]
    malformed: int


def parse_lines(lines: Iterable[str]) -> ParseResult:
    """Parse an iterable of raw lines, skipping and counting malformed ones.

    Blank lines are ignored entirely (neither valid nor malformed).
    """
    records: list[LogRecord] = []
    malformed = 0
    for line in lines:
        if not line.strip():
            continue
        record = parse_line(line)
        if record is None:
            malformed += 1
        else:
            records.append(record)
    return ParseResult(records=records, malformed=malformed)


def iter_records(lines: Iterable[str]) -> Iterator[LogRecord]:
    """Yield only the valid records from raw lines (malformed lines skipped)."""
    for line in lines:
        if not line.strip():
            continue
        record = parse_line(line)
        if record is not None:
            yield record
