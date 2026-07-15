"""Aggregation logic for the loglens subcommands."""

from __future__ import annotations

from collections import Counter
from datetime import datetime

from .parser import LogRecord


def _is_error(status: int) -> bool:
    return 400 <= status <= 599


def summary(records: list[LogRecord]) -> dict:
    """Compute overall statistics for a set of records."""
    times = [r.time for r in records]
    errors = sum(1 for r in records if _is_error(r.status))
    total = len(records)
    return {
        "total_requests": total,
        "unique_ips": len({r.ip for r in records}),
        "first_timestamp": min(times).isoformat() if times else None,
        "last_timestamp": max(times).isoformat() if times else None,
        "error_rate": round(errors / total * 100, 2) if total else 0.0,
    }


def top(records: list[LogRecord], by: str, n: int = 10) -> list[dict]:
    """Return the top ``n`` values by request count.

    Ties are broken by value ascending. ``by`` is one of ``ip``, ``path`` or
    ``status``.
    """
    if by == "ip":
        values = [r.ip for r in records]
    elif by == "path":
        values = [r.path for r in records]
    elif by == "status":
        values = [str(r.status) for r in records]
    else:  # pragma: no cover - guarded by argparse choices
        raise ValueError(f"invalid 'by' value: {by}")

    counts = Counter(values)
    ordered = sorted(counts.items(), key=lambda kv: (-kv[1], kv[0]))
    return [{"value": value, "count": count} for value, count in ordered[:n]]


def errors(
    records: list[LogRecord],
    since: datetime | None = None,
    until: datetime | None = None,
) -> list[dict]:
    """Return 4xx/5xx requests grouped by ``(status, path)``, most frequent first.

    Ties are broken by status ascending then path ascending.
    """
    counts: Counter[tuple[int, str]] = Counter()
    for r in records:
        if not _is_error(r.status):
            continue
        if since is not None and r.time < since:
            continue
        if until is not None and r.time > until:
            continue
        counts[(r.status, r.path)] += 1

    ordered = sorted(counts.items(), key=lambda kv: (-kv[1], kv[0][0], kv[0][1]))
    return [
        {"status": status, "path": path, "count": count}
        for (status, path), count in ordered
    ]


def hourly(records: list[LogRecord]) -> list[int]:
    """Return a 24-element list of request counts indexed by hour of day."""
    buckets = [0] * 24
    for r in records:
        buckets[r.time.hour] += 1
    return buckets
