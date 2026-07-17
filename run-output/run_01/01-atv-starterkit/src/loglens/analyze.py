"""Aggregate analysis over parsed log records."""

from __future__ import annotations

from collections import Counter
from collections.abc import Sequence
from datetime import datetime

from .parser import LogRecord


def is_error(status: int) -> bool:
    """Return True for 4xx and 5xx responses."""
    return 400 <= status < 600


def summarize(records: Sequence[LogRecord]) -> dict:
    """Compute total requests, unique IPs, time span, and error rate."""
    total = len(records)
    unique_ips = len({r.ip for r in records})
    timestamps = [r.timestamp for r in records]
    first_ts = min(timestamps) if timestamps else None
    last_ts = max(timestamps) if timestamps else None
    error_count = sum(1 for r in records if is_error(r.status))
    error_rate = (error_count / total * 100) if total else 0.0
    return {
        "total_requests": total,
        "unique_ips": unique_ips,
        "first_timestamp": first_ts,
        "last_timestamp": last_ts,
        "error_rate": error_rate,
    }


def _value_getter(by: str):
    if by == "ip":
        return lambda r: r.ip
    if by == "path":
        return lambda r: r.path
    if by == "status":
        return lambda r: r.status
    raise ValueError(f"invalid --by value: {by!r}")


def top(records: Sequence[LogRecord], by: str, n: int = 10) -> list[tuple]:
    """Return the top ``n`` values by request count.

    Sorted by count descending; ties broken by value ascending.
    """
    getter = _value_getter(by)
    counts = Counter(getter(r) for r in records)
    ordered = sorted(counts.items(), key=lambda kv: (-kv[1], kv[0]))
    return ordered[:n]


def errors(
    records: Sequence[LogRecord],
    since: datetime | None = None,
    until: datetime | None = None,
) -> list[tuple[tuple[int, str], int]]:
    """Return 4xx/5xx requests grouped by (status, path), most frequent first.

    ``since`` is inclusive and ``until`` is exclusive. Ties in count are broken
    by (status ascending, path ascending) for deterministic output.
    """
    counts: Counter[tuple[int, str]] = Counter()
    for r in records:
        if not is_error(r.status):
            continue
        if since is not None and r.timestamp < since:
            continue
        if until is not None and r.timestamp >= until:
            continue
        counts[(r.status, r.path)] += 1
    return sorted(counts.items(), key=lambda kv: (-kv[1], kv[0][0], kv[0][1]))


def hourly(records: Sequence[LogRecord]) -> list[int]:
    """Return a 24-element list of request counts indexed by hour of day."""
    buckets = [0] * 24
    for r in records:
        buckets[r.timestamp.hour] += 1
    return buckets
