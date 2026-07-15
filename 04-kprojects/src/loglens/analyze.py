"""Aggregation and analysis over parsed log records."""

from __future__ import annotations

from collections import Counter
from collections.abc import Sequence
from datetime import UTC, datetime

from .parser import LogRecord


def is_error(record: LogRecord) -> bool:
    """Return ``True`` when the record's status is 4xx or 5xx."""
    return 400 <= record.status < 600


def summary(records: Sequence[LogRecord]) -> dict:
    """Compute overall summary statistics for a set of records.

    Assumes ``records`` is non-empty (callers handle the empty case).
    """
    total = len(records)
    unique_ips = len({r.host for r in records})
    timestamps = [r.timestamp for r in records]
    first = min(timestamps)
    last = max(timestamps)
    errors = sum(1 for r in records if is_error(r))
    error_rate = (errors / total) * 100 if total else 0.0
    return {
        "total_requests": total,
        "unique_ips": unique_ips,
        "first_timestamp": first,
        "last_timestamp": last,
        "error_count": errors,
        "error_rate": error_rate,
    }


def _key_for(record: LogRecord, by: str) -> str:
    if by == "ip":
        return record.host
    if by == "path":
        return record.path
    if by == "status":
        return str(record.status)
    raise ValueError(f"unknown --by value: {by!r}")


def top(records: Sequence[LogRecord], by: str, n: int = 10) -> list[tuple[str, int]]:
    """Return the top ``n`` values by request count.

    Sorted by count descending; ties broken by value ascending.
    """
    counts = Counter(_key_for(r, by) for r in records)
    ordered = sorted(counts.items(), key=lambda item: (-item[1], item[0]))
    return ordered[:n]


def errors(
    records: Sequence[LogRecord],
    since: datetime | None = None,
    until: datetime | None = None,
) -> list[tuple[int, str, int]]:
    """Group 4xx/5xx requests by ``(status, path)`` with counts.

    Optionally filter by a ``since``/``until`` timestamp window (inclusive).
    Returns ``(status, path, count)`` tuples, most frequent first; ties broken
    by status ascending then path ascending.
    """
    counts: Counter[tuple[int, str]] = Counter()
    for r in records:
        if not is_error(r):
            continue
        ts = _as_aware(r.timestamp)
        if since is not None and ts < _as_aware(since):
            continue
        if until is not None and ts > _as_aware(until):
            continue
        counts[(r.status, r.path)] += 1
    ordered = sorted(counts.items(), key=lambda item: (-item[1], item[0][0], item[0][1]))
    return [(status, path, count) for (status, path), count in ordered]


def hourly(records: Sequence[LogRecord]) -> list[int]:
    """Return request counts indexed by hour of day (0..23)."""
    buckets = [0] * 24
    for r in records:
        buckets[r.timestamp.hour] += 1
    return buckets


def _as_aware(dt: datetime) -> datetime:
    """Normalize a datetime to an aware UTC value for safe comparison.

    Naive datetimes (e.g. an ISO8601 ``--since`` without offset) are assumed to
    be UTC so they can be compared against tz-aware log timestamps.
    """
    if dt.tzinfo is None:
        return dt.replace(tzinfo=UTC)
    return dt.astimezone(UTC)
