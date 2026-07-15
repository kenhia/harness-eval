"""Render result dicts as text or JSON. The only format-aware layer."""

from __future__ import annotations

import json
from typing import Any

_HISTOGRAM_WIDTH = 40


def sanitize(value: Any) -> Any:
    """Escape C0/C1 control characters in attacker-derived strings.

    Paths and user agents are written by anonymous internet clients and printed to an
    operator's terminal. A path containing \x1b[2J clears the scrollback of the attack
    that was just logged. Escaping happens on the JSON path too: JSON quotes control
    characters structurally, but a consumer that echoes the value would re-introduce
    them.
    """
    if not isinstance(value, str):
        return value
    return "".join(ch if ch.isprintable() or ch == " " else f"\\x{ord(ch):02x}" for ch in value)


def render(data: dict[str, Any], command: str, fmt: str) -> str:
    """Render a result dict for the given command in the given format."""
    if fmt == "json":
        return json.dumps(_sanitize_tree(data), indent=2)
    return _TEXT_RENDERERS[command](data)


def _sanitize_tree(node: Any) -> Any:
    if isinstance(node, dict):
        return {k: _sanitize_tree(v) for k, v in node.items()}
    if isinstance(node, list):
        return [_sanitize_tree(v) for v in node]
    return sanitize(node)


def _summary_text(data: dict[str, Any]) -> str:
    lines = [
        f"Total requests:  {data['total_requests']}",
        f"Unique IPs:      {data['unique_ips']}",
        f"First request:   {data['first'] or '-'}",
        f"Last request:    {data['last'] or '-'}",
        f"Error rate:      {data['error_rate']:.2f}%",
    ]
    return "\n".join(lines)


def _top_text(data: dict[str, Any]) -> str:
    items = data["items"]
    if not items:
        return f"No {data['by']} values found."
    # The value is the last column, so it is never padded: trailing whitespace would
    # show up in diffs and break `| grep -c '$'`-style checks for no benefit.
    lines = [f"{'COUNT':>7}  {data['by'].upper()}"]
    lines += [f"{i['count']:>7}  {sanitize(i['value'])}" for i in items]
    return "\n".join(lines)


def _errors_text(data: dict[str, Any]) -> str:
    groups = data["groups"]
    if not groups:
        return "No 4xx/5xx requests found."
    lines = [f"{'COUNT':>7}  {'STATUS':>6}  PATH"]
    lines += [f"{g['count']:>7}  {g['status']:>6}  {sanitize(g['path'])}" for g in groups]
    lines.append(f"\nTotal error requests: {data['total_errors']}")
    return "\n".join(lines)


def _hourly_text(data: dict[str, Any]) -> str:
    hours = data["hours"]
    peak = max((h["count"] for h in hours), default=0)
    lines = [f"{'HOUR':>4}  {'COUNT':>6}"]
    for h in hours:
        # Fixed scale rather than terminal width: output must not differ between a tty
        # and a captured pipe.
        filled = 0 if peak == 0 else round(h["count"] / peak * _HISTOGRAM_WIDTH)
        lines.append(f"{h['hour']:02d}    {h['count']:>6}  {'#' * filled}".rstrip())
    lines.append(f"\nTotal requests: {data['total_requests']}")
    return "\n".join(lines)


_TEXT_RENDERERS = {
    "summary": _summary_text,
    "top": _top_text,
    "errors": _errors_text,
    "hourly": _hourly_text,
}
