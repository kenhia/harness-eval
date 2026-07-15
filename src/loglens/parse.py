"""Parse Combined Log Format lines.

A line is *malformed* only when its CLF structure is invalid. A well-formed record
whose request-line is junk (scanner probes, TLS bytes sent to a plaintext port) is a
valid entry with ``method=None`` and the raw request preserved as ``path`` -- skipping
those would drop exactly the traffic an operator opens this tool to investigate.
"""

from __future__ import annotations

import re
from dataclasses import dataclass
from datetime import datetime, timedelta, timezone

# One pattern for every quoted field. `[^"]*` would terminate early on the `\"` that
# Apache writes for a client-supplied quote, shifting every later field left and
# yielding a line that parses "successfully" with the wrong status and path.
_QUOTED = r'"((?:[^"\\]|\\.)*)"'

_LINE_RE = re.compile(
    r"\A"
    r"(\S+)"  # host
    r"\s+(\S+)"  # ident
    r"\s+(\S+)"  # authuser
    r"\s+\[([^\]]+)\]"  # [timestamp]
    r"\s+"
    + _QUOTED  # "request"
    + r"\s+(\d{3})"  # status
    r"\s+(-|\d+)"  # bytes
    r"(?:\s+"
    + _QUOTED  # "referer"
    + r"\s+"
    + _QUOTED
    + r")?"  # "user-agent"
    r"\s*\Z"
)

# `%b` in strptime resolves month names through locale.LC_TIME: under de_DE it expects
# "Okt" and every line becomes malformed. An explicit map is locale-proof and faster.
_MONTHS = {
    "Jan": 1, "Feb": 2, "Mar": 3, "Apr": 4, "May": 5, "Jun": 6,
    "Jul": 7, "Aug": 8, "Sep": 9, "Oct": 10, "Nov": 11, "Dec": 12,
}  # fmt: skip

_TS_RE = re.compile(r"\A(\d{2})/(\w{3})/(\d{4}):(\d{2}):(\d{2}):(\d{2})\s+([+-]\d{4})\Z")

_ESCAPE_RE = re.compile(r"\\(x[0-9A-Fa-f]{2}|.)")


@dataclass(frozen=True, slots=True)
class LogEntry:
    """One valid CLF record."""

    ip: str
    ident: str | None
    authuser: str | None
    time: datetime  # timezone-aware, in the offset written in the log line
    method: str | None  # None when the request-line was not METHOD PATH PROTO
    path: str  # query string stripped; raw request field if unparseable
    protocol: str | None
    status: int
    size: int | None  # None for the `-` placeholder (304s, most HEADs)
    referer: str | None
    user_agent: str | None

    @property
    def is_error(self) -> bool:
        """True for 4xx and 5xx responses."""
        return self.status >= 400


@dataclass(frozen=True, slots=True)
class ParseError:
    """A line whose CLF structure is invalid. Carries the diagnosis, not just a None."""

    lineno: int
    reason: str
    raw: str

    @property
    def excerpt(self) -> str:
        """The offending line, truncated for a one-line error report."""
        raw = self.raw.strip()
        return raw if len(raw) <= 80 else raw[:77] + "..."


def _unescape(value: str) -> str:
    r"""Undo Apache's backslash escaping (``\"`` -> ``"``, ``\xHH`` -> byte)."""

    def repl(m: re.Match[str]) -> str:
        token = m.group(1)
        if len(token) == 3 and token[0] in "xX":
            return chr(int(token[1:], 16))
        return {"n": "\n", "t": "\t", "r": "\r"}.get(token, token)

    return _ESCAPE_RE.sub(repl, value)


def parse_timestamp(raw: str) -> datetime | None:
    """Parse ``10/Oct/2000:13:55:36 -0700`` into an aware datetime, or None."""
    m = _TS_RE.match(raw)
    if m is None:
        return None
    day, mon, year, hour, minute, second, offset = m.groups()
    month = _MONTHS.get(mon)
    if month is None:
        return None
    sign = 1 if offset[0] == "+" else -1
    delta = timedelta(hours=int(offset[1:3]), minutes=int(offset[3:5])) * sign
    try:
        return datetime(
            int(year), month, int(day), int(hour), int(minute), int(second),
            tzinfo=timezone(delta),
        )  # fmt: skip
    except ValueError:
        return None  # e.g. 31/Feb


def split_request(request: str) -> tuple[str | None, str, str | None]:
    """Split a request-line into (method, path, protocol).

    Junk request-lines keep the raw field as the path so the record stays countable.
    The query string is stripped: RFC 3986 defines a URL path as excluding the query,
    and it keeps `top --by path` cardinality bounded by routes rather than by whatever
    a scanner enumerated.
    """
    parts = request.split(" ")
    if len(parts) == 3 and parts[0].isalpha():
        return parts[0], parts[1].split("?", 1)[0], parts[2]
    return None, request, None


def _dash_to_none(value: str) -> str | None:
    return None if value == "-" else value


def parse_line(line: str, lineno: int = 0) -> LogEntry | ParseError:
    """Parse one CLF line into a LogEntry, or a ParseError explaining the rejection."""
    text = line.rstrip("\r\n")
    if not text.strip():
        return ParseError(lineno, "blank line", line)

    m = _LINE_RE.match(text)
    if m is None:
        return ParseError(lineno, "does not match Combined Log Format", line)

    host, ident, authuser, ts_raw, request, status, size, referer, agent = m.groups()

    time = parse_timestamp(ts_raw)
    if time is None:
        return ParseError(lineno, f"unparsable timestamp: [{ts_raw}]", line)

    method, path, protocol = split_request(_unescape(request))

    return LogEntry(
        ip=host,
        ident=_dash_to_none(ident),
        authuser=_dash_to_none(authuser),
        time=time,
        method=method,
        path=path,
        protocol=protocol,
        status=int(status),
        size=None if size == "-" else int(size),
        referer=_unescape(referer) if referer and referer != "-" else None,
        user_agent=_unescape(agent) if agent and agent != "-" else None,
    )
