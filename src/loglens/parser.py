"""Parsing of Combined Log Format (CLF) access-log lines."""

from __future__ import annotations

import re
from collections.abc import Iterable
from dataclasses import dataclass
from datetime import datetime

# Combined Log Format:
# host - user [10/Oct/2000:13:55:36 -0700] "GET /path HTTP/1.0" 200 2326 "referer" "ua"
_LINE_RE = re.compile(
    r"^(?P<ip>\S+)\s+"
    r"(?P<ident>\S+)\s+"
    r"(?P<user>\S+)\s+"
    r"\[(?P<time>[^\]]+)\]\s+"
    r'"(?P<request>[^"]*)"\s+'
    r"(?P<status>\d{3})\s+"
    r"(?P<size>-|\d+)"
    r'(?:\s+"(?P<referer>(?:[^"\\]|\\.)*)")?'
    r'(?:\s+"(?P<agent>(?:[^"\\]|\\.)*)")?'
    r"\s*$"
)

_TIME_FMT = "%d/%b/%Y:%H:%M:%S %z"


@dataclass(frozen=True)
class LogRecord:
    """A single parsed access-log entry."""

    ip: str
    user: str
    timestamp: datetime
    method: str
    path: str
    protocol: str
    status: int
    size: int
    referer: str
    user_agent: str


def parse_line(line: str) -> LogRecord | None:
    """Parse one CLF line into a :class:`LogRecord`.

    Returns ``None`` if the line does not match CLF or a field cannot be coerced.
    """
    match = _LINE_RE.match(line.rstrip("\n"))
    if match is None:
        return None

    try:
        timestamp = datetime.strptime(match["time"], _TIME_FMT)
    except ValueError:
        return None

    request = match["request"].split()
    if len(request) == 3:
        method, path, protocol = request
    elif len(request) == 2:
        method, path, protocol = request[0], request[1], ""
    else:
        return None

    size_raw = match["size"]
    size = 0 if size_raw == "-" else int(size_raw)

    user = match["user"]
    return LogRecord(
        ip=match["ip"],
        user="" if user == "-" else user,
        timestamp=timestamp,
        method=method,
        path=path,
        protocol=protocol,
        status=int(match["status"]),
        size=size,
        referer=match["referer"] or "",
        user_agent=match["agent"] or "",
    )


def parse_lines(lines: Iterable[str]) -> tuple[list[LogRecord], int]:
    """Parse an iterable of lines.

    Returns a tuple of ``(records, malformed_count)``. Blank/whitespace-only
    lines are ignored entirely and are not counted as malformed.
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
    return records, malformed
