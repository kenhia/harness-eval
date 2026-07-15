"""Parsing of Combined Log Format access-log lines."""

from __future__ import annotations

import re
from collections.abc import Iterable, Iterator
from dataclasses import dataclass
from datetime import datetime

# 203.0.113.7 - - [12/Jul/2026:06:25:24 +0000] "GET /index.html HTTP/1.1" 200 5413 "ref" "ua"
_LINE_RE = re.compile(
    r"""^
    (?P<ip>\S+)\s+
    (?P<ident>\S+)\s+
    (?P<user>\S+)\s+
    \[(?P<time>[^\]]+)\]\s+
    "(?P<request>(?:[^"\\]|\\.)*)"\s+
    (?P<status>\d{3})\s+
    (?P<size>\d+|-)\s+
    "(?P<referer>(?:[^"\\]|\\.)*)"\s+
    "(?P<agent>(?:[^"\\]|\\.)*)"
    \s*$""",
    re.VERBOSE,
)

_TIME_FMT = "%d/%b/%Y:%H:%M:%S %z"


@dataclass(frozen=True, slots=True)
class Entry:
    """A single successfully parsed access-log record."""

    ip: str
    user: str | None
    time: datetime
    method: str
    path: str
    protocol: str
    status: int
    size: int
    referer: str | None
    agent: str | None

    @property
    def is_error(self) -> bool:
        """True for 4xx and 5xx responses."""
        return self.status >= 400


def _dash_to_none(value: str) -> str | None:
    return None if value == "-" else value


def parse_line(line: str) -> Entry | None:
    """Parse one log line, returning None if it is malformed."""
    match = _LINE_RE.match(line.strip())
    if match is None:
        return None

    request = match["request"].split()
    if len(request) != 3:
        return None
    method, path, protocol = request

    try:
        time = datetime.strptime(match["time"], _TIME_FMT)
    except ValueError:
        return None

    size = match["size"]
    return Entry(
        ip=match["ip"],
        user=_dash_to_none(match["user"]),
        time=time,
        method=method,
        path=path,
        protocol=protocol,
        status=int(match["status"]),
        size=0 if size == "-" else int(size),
        referer=_dash_to_none(match["referer"]),
        agent=_dash_to_none(match["agent"]),
    )


@dataclass(slots=True)
class ParseResult:
    """Parsed entries plus a count of lines that could not be parsed."""

    entries: list[Entry]
    malformed: int


def parse_lines(lines: Iterable[str]) -> ParseResult:
    """Parse an iterable of lines, skipping and counting malformed ones.

    Blank lines are ignored entirely and are not counted as malformed.
    """
    entries: list[Entry] = []
    malformed = 0
    for line in lines:
        if not line.strip():
            continue
        entry = parse_line(line)
        if entry is None:
            malformed += 1
        else:
            entries.append(entry)
    return ParseResult(entries=entries, malformed=malformed)


def iter_file(path: str) -> Iterator[str]:
    """Yield lines from a log file, tolerating undecodable bytes."""
    with open(path, encoding="utf-8", errors="replace") as handle:
        yield from handle
