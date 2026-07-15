"""Rendering of analysis results as text or JSON."""

from __future__ import annotations

import json
from typing import Any

from .analyze import Summary

BAR_WIDTH = 40


def _iso(value: Any) -> str:
    return value.isoformat()


def summary_text(summary: Summary) -> str:
    return "\n".join(
        [
            f"Total requests:  {summary.total}",
            f"Unique IPs:      {summary.unique_ips}",
            f"First timestamp: {_iso(summary.first)}",
            f"Last timestamp:  {_iso(summary.last)}",
            f"Error rate:      {summary.error_rate:.2f}% ({summary.errors} of {summary.total})",
        ]
    )


def summary_json(summary: Summary) -> dict[str, Any]:
    return {
        "total_requests": summary.total,
        "unique_ips": summary.unique_ips,
        "first_timestamp": _iso(summary.first),
        "last_timestamp": _iso(summary.last),
        "errors": summary.errors,
        "error_rate": round(summary.error_rate, 4),
    }


def top_text(rows: list[tuple[str, int]], by: str) -> str:
    if not rows:
        return f"No {by} values found."
    width = max(len(by), *(len(value) for value, _ in rows))
    lines = [f"{by.upper():<{width}}  COUNT"]
    lines += [f"{value:<{width}}  {count}" for value, count in rows]
    return "\n".join(lines)


def top_json(rows: list[tuple[str, int]], by: str) -> dict[str, Any]:
    return {
        "by": by,
        "results": [{"value": value, "count": count} for value, count in rows],
    }


def errors_text(rows: list[tuple[int, str, int]]) -> str:
    if not rows:
        return "No error responses found."
    width = max(len("PATH"), *(len(path) for _, path, _ in rows))
    lines = [f"STATUS  {'PATH':<{width}}  COUNT"]
    lines += [f"{status:<6}  {path:<{width}}  {count}" for status, path, count in rows]
    return "\n".join(lines)


def errors_json(rows: list[tuple[int, str, int]]) -> dict[str, Any]:
    return {
        "results": [
            {"status": status, "path": path, "count": count} for status, path, count in rows
        ]
    }


def hourly_text(buckets: list[int]) -> str:
    peak = max(buckets)
    width = max(len(str(count)) for count in buckets)
    lines = []
    for hour, count in enumerate(buckets):
        filled = 0 if peak == 0 else round(BAR_WIDTH * count / peak)
        lines.append(f"{hour:02d}  {count:>{width}}  {'#' * filled}".rstrip())
    return "\n".join(lines)


def hourly_json(buckets: list[int]) -> dict[str, Any]:
    return {"hourly": [{"hour": hour, "count": count} for hour, count in enumerate(buckets)]}


def dump_json(payload: dict[str, Any]) -> str:
    return json.dumps(payload, indent=2)
