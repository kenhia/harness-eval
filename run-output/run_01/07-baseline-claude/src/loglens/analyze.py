"""Analyses computed over parsed log entries."""

from __future__ import annotations

from collections import Counter
from collections.abc import Sequence
from dataclasses import dataclass
from datetime import UTC, datetime

from .parser import Entry


@dataclass(frozen=True, slots=True)
class Summary:
    total: int
    unique_ips: int
    first: datetime
    last: datetime
    errors: int

    @property
    def error_rate(self) -> float:
        """Percentage of requests that returned 4xx or 5xx."""
        return 100.0 * self.errors / self.total


def summarize(entries: Sequence[Entry]) -> Summary:
    """Compute overall statistics. Requires at least one entry."""
    times = [e.time for e in entries]
    return Summary(
        total=len(entries),
        unique_ips=len({e.ip for e in entries}),
        first=min(times),
        last=max(times),
        errors=sum(1 for e in entries if e.is_error),
    )


def _key_of(entry: Entry, by: str) -> str:
    match by:
        case "ip":
            return entry.ip
        case "path":
            return entry.path
        case "status":
            return str(entry.status)
    raise ValueError(f"unknown grouping: {by}")


def top(entries: Sequence[Entry], by: str, n: int = 10) -> list[tuple[str, int]]:
    """Return the n most frequent values of `by`, descending by count.

    Ties are broken by value ascending.
    """
    counts = Counter(_key_of(e, by) for e in entries)
    ordered = sorted(counts.items(), key=lambda item: (-item[1], item[0]))
    return ordered[:n]


def normalize_bound(when: datetime) -> datetime:
    """Treat a naive bound as UTC so it can be compared with log timestamps."""
    return when.replace(tzinfo=UTC) if when.tzinfo is None else when


def filter_time(
    entries: Sequence[Entry],
    since: datetime | None = None,
    until: datetime | None = None,
) -> list[Entry]:
    """Keep entries within [since, until], both bounds inclusive."""
    lo = normalize_bound(since) if since else None
    hi = normalize_bound(until) if until else None
    return [
        e
        for e in entries
        if (lo is None or e.time >= lo) and (hi is None or e.time <= hi)
    ]


def errors(
    entries: Sequence[Entry],
    since: datetime | None = None,
    until: datetime | None = None,
) -> list[tuple[int, str, int]]:
    """Group 4xx/5xx requests by (status, path).

    Returns (status, path, count) triples, most frequent first; ties broken by
    status ascending, then path ascending.
    """
    window = filter_time(entries, since, until)
    counts = Counter((e.status, e.path) for e in window if e.is_error)
    ordered = sorted(counts.items(), key=lambda item: (-item[1], item[0][0], item[0][1]))
    return [(status, path, count) for (status, path), count in ordered]


def hourly(entries: Sequence[Entry]) -> list[int]:
    """Return request counts indexed by hour of day (0-23)."""
    buckets = [0] * 24
    for entry in entries:
        buckets[entry.time.hour] += 1
    return buckets
