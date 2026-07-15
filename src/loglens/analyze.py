"""Turn log entries into plain result dicts.

Analyzers return data, never formatted output. Rendering is the only format-aware layer,
which is what keeps `--format json` from being a second code path with its own bugs.
"""

from __future__ import annotations

from collections import Counter
from collections.abc import Iterable, Sequence
from datetime import UTC, datetime
from typing import Any

from loglens.parse import LogEntry

TOP_DIMENSIONS = ("ip", "path", "status")


def filter_entries(
    entries: Iterable[LogEntry],
    since: datetime | None = None,
    until: datetime | None = None,
) -> list[LogEntry]:
    """Filter to `since <= t < until`.

    Bounds are half-open so consecutive windows chain without double-counting the
    instant on the seam. Both sides are compared as absolute instants, so a log in
    -0800 and a `--since` in UTC line up correctly.
    """
    result = []
    for entry in entries:
        t = entry.time
        if since is not None and t < since:
            continue
        if until is not None and t >= until:
            continue
        result.append(entry)
    return result


def _rank(counts: Counter[Any], limit: int | None = None) -> list[tuple[Any, int]]:
    """Rank by count descending, ties broken by value ascending.

    Not `Counter.most_common`: that breaks ties by insertion order, which is a spec
    violation that happens to pass any fixture where tied values arrive already sorted.
    """
    ranked = sorted(counts.items(), key=lambda kv: (-kv[1], kv[0]))
    return ranked if limit is None else ranked[:limit]


def summary(entries: Sequence[LogEntry]) -> dict[str, Any]:
    """Total requests, unique IPs, first/last timestamp, error rate."""
    total = len(entries)
    if total == 0:
        return {
            "total_requests": 0,
            "unique_ips": 0,
            "first": None,
            "last": None,
            "error_rate": 0.0,
        }

    # min/max, not first-line/last-line: servers log request start time but write at
    # completion, so lines are near-sorted, not sorted. The shortcut is wrong only
    # under load, which is exactly when it matters.
    times = [e.time for e in entries]
    errors = sum(1 for e in entries if e.is_error)

    return {
        "total_requests": total,
        "unique_ips": len({e.ip for e in entries}),
        "first": min(times).isoformat(),
        "last": max(times).isoformat(),
        "error_rate": round(errors / total * 100, 2),
    }


def top(entries: Sequence[LogEntry], by: str, n: int = 10) -> dict[str, Any]:
    """Top n values for a dimension, count descending, ties by value ascending."""
    if by not in TOP_DIMENSIONS:
        raise ValueError(f"unknown dimension: {by}")

    # status stays an int so the tie-break sorts numerically and JSON emits 404, not
    # "404". ip/path are strings and sort lexicographically.
    counts: Counter[Any] = Counter(getattr(e, "status" if by == "status" else by) for e in entries)

    return {
        "by": by,
        "count": len(counts),
        "items": [{"value": value, "count": count} for value, count in _rank(counts, n)],
    }


def errors(entries: Sequence[LogEntry]) -> dict[str, Any]:
    """4xx/5xx requests grouped by (status, path), most frequent first."""
    counts: Counter[tuple[int, str]] = Counter((e.status, e.path) for e in entries if e.is_error)
    # The spec leaves the tie-break unstated; (-count, status, path) keeps output stable
    # across re-sorted input and matches `top`'s rule.
    ranked = sorted(counts.items(), key=lambda kv: (-kv[1], kv[0][0], kv[0][1]))

    return {
        "total_errors": sum(counts.values()),
        "groups": [
            {"status": status, "path": path, "count": count} for (status, path), count in ranked
        ],
    }


def hourly(entries: Sequence[LogEntry]) -> dict[str, Any]:
    """Request count per hour of day (00-23).

    Buckets by the wall-clock hour as written in the log line's own offset. That is what
    an operator means by "traffic peaks at 14:00", and it is deterministic. UTC would
    shift the histogram away from the timestamps printed in the file; machine-local time
    would give different answers on different laptops.
    """
    counts = Counter(e.time.hour for e in entries)
    # All 24 buckets, including zeros: a histogram that omits empty hours lies.
    return {
        "total_requests": len(entries),
        "hours": [{"hour": h, "count": counts.get(h, 0)} for h in range(24)],
    }


def parse_bound(value: str) -> datetime:
    """Parse an ISO8601 --since/--until value into an aware UTC datetime.

    Naive input (`2026-07-15`, `2026-07-15 10:00` -- what people actually type) is
    interpreted as UTC. Comparing a naive value against an aware log timestamp raises
    TypeError, so coercion happens here, at the argument boundary, where a bad value is
    a usage error rather than a crash deep in the analysis.
    """
    try:
        parsed = datetime.fromisoformat(value)
    except ValueError as exc:
        raise ValueError(
            f"invalid ISO8601 timestamp: {value!r} (try 2026-07-12T06:00:00Z)"
        ) from exc
    if parsed.tzinfo is None:
        return parsed.replace(tzinfo=UTC)
    return parsed.astimezone(UTC)
