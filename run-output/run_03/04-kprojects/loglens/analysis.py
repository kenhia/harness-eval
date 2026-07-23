"""Analysis functions for log data."""

from collections import Counter, defaultdict
from datetime import datetime

from loglens.parser import LogEntry


def calculate_summary(entries: list[LogEntry]) -> dict:
    """Calculate summary statistics.

    Returns a dict with keys: total_requests, unique_ips, first_timestamp,
    last_timestamp, error_rate.
    """
    if not entries:
        return {
            "total_requests": 0,
            "unique_ips": 0,
            "first_timestamp": None,
            "last_timestamp": None,
            "error_rate": 0.0,
        }

    unique_ips = len({e.ip for e in entries})
    total_requests = len(entries)
    error_count = sum(1 for e in entries if e.is_error)
    error_rate = (error_count / total_requests * 100) if total_requests > 0 else 0.0
    first_timestamp = min(e.timestamp for e in entries)
    last_timestamp = max(e.timestamp for e in entries)

    return {
        "total_requests": total_requests,
        "unique_ips": unique_ips,
        "first_timestamp": first_timestamp.isoformat(),
        "last_timestamp": last_timestamp.isoformat(),
        "error_rate": round(error_rate, 2),
    }


def get_top_values(entries: list[LogEntry], by: str, n: int = 10) -> list[tuple]:
    """Get top N values by request count.

    Returns a list of (value, count) tuples, ordered by count (descending)
    then by value (ascending).
    """
    if by == "ip":
        counter = Counter(e.ip for e in entries)
    elif by == "path":
        counter = Counter(e.path for e in entries)
    elif by == "status":
        counter = Counter(e.status for e in entries)
    else:
        raise ValueError(f"Unknown 'by' value: {by}")

    # Sort by count descending, then by value ascending
    sorted_items = sorted(counter.items(), key=lambda x: (-x[1], str(x[0])))
    return sorted_items[:n]


def get_errors(
    entries: list[LogEntry], since: datetime | None = None, until: datetime | None = None
) -> list[tuple]:
    """Get errors grouped by (status, path) with counts.

    Returns a list of ((status, path), count) tuples, ordered by count
    (descending) then by status and path (ascending).
    """
    error_entries = [
        e for e in entries
        if e.is_error
        and (since is None or e.timestamp >= since)
        and (until is None or e.timestamp <= until)
    ]

    counter = Counter((e.status, e.path) for e in error_entries)

    # Sort by count descending, then by (status, path) ascending
    sorted_items = sorted(counter.items(), key=lambda x: (-x[1], x[0]))
    return sorted_items


def get_hourly_distribution(entries: list[LogEntry]) -> dict[int, int]:
    """Get request count per hour (0-23).

    Returns a dict mapping hour to count.
    """
    hourly_counts: dict[int, int] = defaultdict(int)
    for entry in entries:
        hour = entry.timestamp.hour
        hourly_counts[hour] += 1

    # Fill in missing hours with 0
    result = {hour: hourly_counts.get(hour, 0) for hour in range(24)}
    return result
