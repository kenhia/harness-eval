"""Analysis functions over parsed log records."""

from __future__ import annotations

from collections import Counter
from datetime import datetime

from .parser import LogRecord


def is_error(status: int) -> bool:
    """Return True for 4xx and 5xx status codes."""
    return 400 <= status <= 599


def summary(records: list[LogRecord]) -> dict:
    """Compute overall statistics for a set of records."""
    total = len(records)
    unique_ips = len({r.ip for r in records})
    timestamps = [r.timestamp for r in records]
    first = min(timestamps)
    last = max(timestamps)
    errors = sum(1 for r in records if is_error(r.status))
    error_rate = (errors / total * 100) if total else 0.0
    return {
        "total_requests": total,
        "unique_ips": unique_ips,
        "first_timestamp": first.isoformat(),
        "last_timestamp": last.isoformat(),
        "error_count": errors,
        "error_rate": round(error_rate, 2),
    }


def _key_for(record: LogRecord, by: str) -> str:
    if by == "ip":
        return record.ip
    if by == "path":
        return record.path
    if by == "status":
        return str(record.status)
    raise ValueError(f"invalid --by value: {by}")


def top(records: list[LogRecord], by: str, n: int) -> list[dict]:
    """Return the top ``n`` values by request count.

    Sorted by count descending, ties broken by value ascending.
    """
    counts = Counter(_key_for(r, by) for r in records)
    ordered = sorted(counts.items(), key=lambda kv: (-kv[1], kv[0]))
    return [{"value": value, "count": count} for value, count in ordered[:n]]


def errors(
    records: list[LogRecord],
    since: datetime | None = None,
    until: datetime | None = None,
) -> list[dict]:
    """Group 4xx/5xx requests by (status, path), most frequent first.

    Ties broken by status ascending, then path ascending.
    """
    counts: Counter[tuple[int, str]] = Counter()
    for r in records:
        if not is_error(r.status):
            continue
        if since is not None and r.timestamp < since:
            continue
        if until is not None and r.timestamp > until:
            continue
        counts[(r.status, r.path)] += 1
    ordered = sorted(counts.items(), key=lambda kv: (-kv[1], kv[0][0], kv[0][1]))
    return [
        {"status": status, "path": path, "count": count}
        for (status, path), count in ordered
    ]


def hourly(records: list[LogRecord]) -> list[int]:
    """Return a 24-element list of request counts indexed by hour (0-23)."""
    buckets = [0] * 24
    for r in records:
        buckets[r.timestamp.hour] += 1
    return buckets
