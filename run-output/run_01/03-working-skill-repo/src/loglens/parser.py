"""Parsing of Combined Log Format (CLF) access-log lines."""

from __future__ import annotations

import re
from collections.abc import Iterable, Iterator
from dataclasses import dataclass
from datetime import datetime

# Combined Log Format:
#   host ident authuser [date] "request" status bytes "referer" "user-agent"
_LINE_RE = re.compile(
    r"""^
    (?P<ip>\S+)\s+
    (?P<ident>\S+)\s+
    (?P<user>\S+)\s+
    \[(?P<time>[^\]]+)\]\s+
    "(?P<request>[^"]*)"\s+
    (?P<status>\d{3})\s+
    (?P<size>-|\d+)
    (?:\s+"(?P<referer>[^"]*)"\s+"(?P<agent>[^"]*)")?
    \s*$
    """,
    re.VERBOSE,
)

_TIME_FORMAT = "%d/%b/%Y:%H:%M:%S %z"


@dataclass(frozen=True)
class LogRecord:
    """A single successfully-parsed access-log entry."""

    ip: str
    user: str
    timestamp: datetime
    method: str
    path: str
    protocol: str
    status: int
    size: int
    referer: str
    agent: str


@dataclass
class ParseResult:
    """Outcome of parsing a log source."""

    records: list[LogRecord]
    malformed: int


def parse_line(line: str) -> LogRecord | None:
    """Parse a single CLF line, returning ``None`` if it is malformed."""
    match = _LINE_RE.match(line.strip())
    if match is None:
        return None

    request = match.group("request").split()
    if len(request) != 3:
        return None
    method, path, protocol = request

    try:
        timestamp = datetime.strptime(match.group("time"), _TIME_FORMAT)
    except ValueError:
        return None

    raw_size = match.group("size")
    size = 0 if raw_size == "-" else int(raw_size)

    return LogRecord(
        ip=match.group("ip"),
        user=match.group("user"),
        timestamp=timestamp,
        method=method,
        path=path,
        protocol=protocol,
        status=int(match.group("status")),
        size=size,
        referer=match.group("referer") or "",
        agent=match.group("agent") or "",
    )


def parse_lines(lines: Iterable[str]) -> ParseResult:
    """Parse an iterable of lines, skipping and counting malformed ones.

    Blank lines are ignored entirely and do not count as malformed.
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


def iter_file_lines(path: str) -> Iterator[str]:
    """Yield lines from ``path`` (raises OSError if unreadable)."""
    with open(path, encoding="utf-8", errors="replace") as handle:
        yield from handle
