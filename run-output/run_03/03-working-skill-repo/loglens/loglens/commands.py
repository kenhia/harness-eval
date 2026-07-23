"""Subcommand implementations for loglens."""

from collections import Counter, defaultdict
from datetime import datetime
from typing import Any, Optional

from .parser import LogEntry


def summary(entries: list[LogEntry]) -> dict[str, Any]:
    """Generate summary statistics from log entries."""
    if not entries:
        return {}

    unique_ips = set(entry.ip for entry in entries)
    error_count = sum(1 for entry in entries if entry.status >= 400)
    error_rate = (error_count / len(entries) * 100) if entries else 0

    return {
        "total_requests": len(entries),
        "unique_ips": len(unique_ips),
        "first_timestamp": entries[0].timestamp.isoformat(),
        "last_timestamp": entries[-1].timestamp.isoformat(),
        "error_rate": round(error_rate, 2),
    }


def top(
    entries: list[LogEntry], by: str = "ip", n: int = 10
) -> list[dict[str, Any]]:
    """Get top N values by request count."""
    if by == "ip":
        counter = Counter(entry.ip for entry in entries)
    elif by == "path":
        counter = Counter(entry.path for entry in entries)
    elif by == "status":
        counter = Counter(entry.status for entry in entries)
    else:
        raise ValueError(f"Invalid by parameter: {by}")

    # Sort by count (descending), then by value (ascending) for ties
    sorted_items = sorted(counter.items(), key=lambda x: (-x[1], x[0]))
    result = []
    for value, count in sorted_items[:n]:
        result.append({by: value, "count": count})

    return result


def errors(
    entries: list[LogEntry],
    since: Optional[datetime] = None,
    until: Optional[datetime] = None,
) -> list[dict[str, Any]]:
    """Get 4xx/5xx errors grouped by (status, path)."""
    # Filter by status
    error_entries = [entry for entry in entries if entry.status >= 400]

    # Filter by date range
    if since:
        error_entries = [e for e in error_entries if e.timestamp >= since]
    if until:
        error_entries = [e for e in error_entries if e.timestamp <= until]

    # Group by (status, path)
    groups = defaultdict(int)
    for entry in error_entries:
        key = (entry.status, entry.path)
        groups[key] += 1

    # Sort by count (descending)
    result = []
    for (status, path), count in sorted(groups.items(), key=lambda x: -x[1]):
        result.append({"status": status, "path": path, "count": count})

    return result


def hourly(entries: list[LogEntry]) -> list[dict[str, Any]]:
    """Get request count per hour of day (0-23)."""
    hourly_counts = [0] * 24
    for entry in entries:
        hour = entry.timestamp.hour
        hourly_counts[hour] += 1

    result = []
    for hour, count in enumerate(hourly_counts):
        result.append({"hour": hour, "count": count})

    return result


def format_text_summary(data: dict[str, Any]) -> str:
    """Format summary data as text."""
    if not data:
        return "No valid log entries found."
    lines = [
        f"Total Requests: {data['total_requests']}",
        f"Unique IPs: {data['unique_ips']}",
        f"First Timestamp: {data['first_timestamp']}",
        f"Last Timestamp: {data['last_timestamp']}",
        f"Error Rate: {data['error_rate']}%",
    ]
    return "\n".join(lines)


def format_text_top(data: list[dict[str, Any]], by: str) -> str:
    """Format top data as text."""
    if not data:
        return f"No data to display for '{by}'."
    lines = [f"Top {by}:"]
    for i, item in enumerate(data, 1):
        lines.append(f"  {i}. {item[by]}: {item['count']}")
    return "\n".join(lines)


def format_text_errors(data: list[dict[str, Any]]) -> str:
    """Format error data as text."""
    if not data:
        return "No errors found."
    lines = ["Errors (status, path):"]
    for item in data:
        lines.append(f"  {item['status']} {item['path']}: {item['count']}")
    return "\n".join(lines)


def format_text_hourly(data: list[dict[str, Any]]) -> str:
    """Format hourly data as text histogram."""
    if not data:
        return "No data available."

    max_count = max((item["count"] for item in data), default=0)
    if max_count == 0:
        max_count = 1

    lines = ["Requests per Hour:"]
    for item in data:
        hour = item["hour"]
        count = item["count"]
        bar_width = int((count / max_count) * 40) if max_count > 0 else 0
        bar = "█" * bar_width
        lines.append(f"  {hour:02d}: {bar} ({count})")

    return "\n".join(lines)
