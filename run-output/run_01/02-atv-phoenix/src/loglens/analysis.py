"""Analysis functions over parsed :class:`~loglens.parser.LogRecord` sequences.

Each function is pure: it takes an iterable of records (and options) and returns
plain data structures, leaving formatting and I/O to the CLI layer.
"""

from __future__ import annotations

from collections import Counter
from collections.abc import Iterable, Sequence
from dataclasses import dataclass
from datetime import datetime

from .parser import LogRecord


@dataclass(frozen=True)
class Summary:
    total_requests: int
    unique_ips: int
    first_timestamp: datetime | None
    last_timestamp: datetime | None
    error_rate: float  # percentage 0..100 of 4xx+5xx responses


def summarize(records: Sequence[LogRecord]) -> Summary:
    """Compute the overall summary statistics."""
    total = len(records)
    unique_ips = len({r.host for r in records})
    if total:
        timestamps = [r.timestamp for r in records]
        first_ts: datetime | None = min(timestamps)
        last_ts: datetime | None = max(timestamps)
        errors = sum(1 for r in records if r.is_error)
        error_rate = errors / total * 100
    else:
        first_ts = last_ts = None
        error_rate = 0.0
    return Summary(
        total_requests=total,
        unique_ips=unique_ips,
        first_timestamp=first_ts,
        last_timestamp=last_ts,
        error_rate=error_rate,
    )


_TOP_KEYS = {
    "ip": lambda r: r.host,
    "path": lambda r: r.path,
    "status": lambda r: str(r.status),
}


def top(records: Iterable[LogRecord], by: str, n: int = 10) -> list[tuple[str, int]]:
    """Return the top ``n`` ``(value, count)`` pairs for dimension ``by``.

    Ordered by count descending, ties broken by value ascending.
    ``by`` is one of ``"ip"``, ``"path"`` or ``"status"``.
    """
    if by not in _TOP_KEYS:
        raise ValueError(f"invalid 'by' value: {by!r}; expected one of {sorted(_TOP_KEYS)}")
    key = _TOP_KEYS[by]
    counts = Counter(key(r) for r in records)
    ordered = sorted(counts.items(), key=lambda kv: (-kv[1], kv[0]))
    if n is not None and n >= 0:
        return ordered[:n]
    return ordered


def _in_window(
    ts: datetime, since: datetime | None, until: datetime | None
) -> bool:
    if since is not None and ts < since:
        return False
    if until is not None and ts > until:
        return False
    return True


def errors(
    records: Iterable[LogRecord],
    since: datetime | None = None,
    until: datetime | None = None,
) -> list[tuple[int, str, int]]:
    """Group 4xx/5xx requests by ``(status, path)`` with counts.

    Optionally restrict to the ``[since, until]`` time window (inclusive).
    Ordered by count descending, ties broken by ``(status, path)`` ascending.
    """
    counts: Counter[tuple[int, str]] = Counter()
    for r in records:
        if not r.is_error:
            continue
        if not _in_window(r.timestamp, since, until):
            continue
        counts[(r.status, r.path)] += 1
    ordered = sorted(counts.items(), key=lambda kv: (-kv[1], kv[0][0], kv[0][1]))
    return [(status, path, count) for (status, path), count in ordered]


def hourly(records: Iterable[LogRecord]) -> list[int]:
    """Return a 24-element list: request counts indexed by hour of day (0..23)."""
    buckets = [0] * 24
    for r in records:
        buckets[r.timestamp.hour] += 1
    return buckets
