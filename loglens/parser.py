"""Parsing for Combined Log Format (CLF) access-log lines."""

from __future__ import annotations

import re
from dataclasses import dataclass
from datetime import datetime

# Combined Log Format:
#   %h %l %u %t "%r" %>s %b "%{Referer}i" "%{User-agent}i"
# Example:
#   203.0.113.7 - - [12/Jul/2026:06:25:24 +0000] "GET /x HTTP/1.1" 200 5413 "-" "UA"
_LINE_RE = re.compile(
    r"^(?P<ip>\S+)\s+"
    r"(?P<ident>\S+)\s+"
    r"(?P<user>\S+)\s+"
    r"\[(?P<time>[^\]]+)\]\s+"
    r'"(?P<request>[^"]*)"\s+'
    r"(?P<status>\d{3})\s+"
    r"(?P<size>\d+|-)"
    r'(?:\s+"(?P<referer>[^"]*)"\s+"(?P<agent>[^"]*)")?'
    r"\s*$"
)

_TIME_FMT = "%d/%b/%Y:%H:%M:%S %z"


@dataclass(frozen=True)
class LogRecord:
    """A single parsed access-log entry."""

    ip: str
    user: str
    time: datetime
    method: str
    path: str
    protocol: str
    status: int
    size: int
    referer: str
    agent: str


def parse_line(line: str) -> LogRecord | None:
    """Parse one CLF line into a :class:`LogRecord`, or ``None`` if malformed."""
    match = _LINE_RE.match(line.strip())
    if match is None:
        return None

    try:
        time = datetime.strptime(match["time"], _TIME_FMT)
    except ValueError:
        return None

    request = match["request"].split()
    if len(request) != 3:
        return None
    method, path, protocol = request

    size_raw = match["size"]
    size = 0 if size_raw == "-" else int(size_raw)

    return LogRecord(
        ip=match["ip"],
        user=match["user"],
        time=time,
        method=method,
        path=path,
        protocol=protocol,
        status=int(match["status"]),
        size=size,
        referer=match["referer"] or "",
        agent=match["agent"] or "",
    )


def parse_lines(lines: object) -> tuple[list[LogRecord], int]:
    """Parse an iterable of raw lines.

    Returns a tuple of ``(records, malformed_count)``. Blank lines are ignored
    and do not count as malformed.
    """
    records: list[LogRecord] = []
    malformed = 0
    for raw in lines:  # type: ignore[assignment]
        if not raw.strip():
            continue
        record = parse_line(raw)
        if record is None:
            malformed += 1
        else:
            records.append(record)
    return records, malformed
