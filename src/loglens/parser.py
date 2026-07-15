"""Parsing of Combined Log Format (CLF) access-log lines.

A Combined Log Format line looks like (wrapped here for width)::

    203.0.113.7 - - [12/Jul/2026:06:25:24 +0000]
    "GET /index.html HTTP/1.1" 200 5413 "https://example.com/" "Mozilla/5.0"

Fields, in order:

    remote_host - remote_user [timestamp] "request line" status size "referer" "user agent"
"""

from __future__ import annotations

import re
from collections.abc import Iterable, Iterator
from dataclasses import dataclass
from datetime import datetime

# Apache/nginx CLF timestamp, e.g. 12/Jul/2026:06:25:24 +0000
_CLF_TIME_FORMAT = "%d/%b/%Y:%H:%M:%S %z"

_LINE_RE = re.compile(
    r"^(?P<host>\S+)"
    r"\s+(?P<ident>\S+)"
    r"\s+(?P<user>\S+)"
    r"\s+\[(?P<time>[^\]]+)\]"
    r'\s+"(?P<request>[^"]*)"'
    r"\s+(?P<status>\d{3})"
    r"\s+(?P<size>-|\d+)"
    r'\s+"(?P<referer>(?:[^"\\]|\\.)*)"'
    r'\s+"(?P<agent>(?:[^"\\]|\\.)*)"'
    r"\s*$"
)


@dataclass(frozen=True)
class LogRecord:
    """A single successfully-parsed access-log entry."""

    host: str
    ident: str
    user: str
    timestamp: datetime
    method: str
    path: str
    protocol: str
    status: int
    size: int  # bytes sent; 0 when the log recorded "-"
    referer: str
    user_agent: str

    @property
    def is_error(self) -> bool:
        """True for 4xx and 5xx responses."""
        return 400 <= self.status <= 599


def parse_line(line: str) -> LogRecord | None:
    """Parse a single CLF line into a :class:`LogRecord`.

    Returns ``None`` if the line is blank or malformed.
    """
    if not line or not line.strip():
        return None

    match = _LINE_RE.match(line.strip())
    if match is None:
        return None

    fields = match.groupdict()

    try:
        timestamp = datetime.strptime(fields["time"], _CLF_TIME_FORMAT)
    except ValueError:
        return None

    request = fields["request"]
    parts = request.split()
    if len(parts) != 3:
        # A well-formed CLF request is "METHOD PATH PROTOCOL".
        return None
    method, path, protocol = parts

    status = int(fields["status"])

    raw_size = fields["size"]
    size = 0 if raw_size == "-" else int(raw_size)

    return LogRecord(
        host=fields["host"],
        ident=fields["ident"],
        user=fields["user"],
        timestamp=timestamp,
        method=method,
        path=path,
        protocol=protocol,
        status=status,
        size=size,
        referer=fields["referer"],
        user_agent=fields["agent"],
    )


def parse_lines(lines: Iterable[str]) -> tuple[list[LogRecord], int]:
    """Parse many lines, returning ``(records, malformed_count)``.

    Malformed (and blank) lines are skipped and counted rather than raising.
    """
    records: list[LogRecord] = []
    malformed = 0
    for line in lines:
        if not line.strip():
            # Blank / whitespace-only lines carry no data; ignore silently.
            continue
        record = parse_line(line)
        if record is None:
            malformed += 1
        else:
            records.append(record)
    return records, malformed


def iter_records(lines: Iterable[str]) -> Iterator[LogRecord]:
    """Yield only the successfully-parsed records from ``lines``."""
    for line in lines:
        record = parse_line(line)
        if record is not None:
            yield record
