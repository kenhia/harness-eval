"""Output formatting for different output formats."""

import json


def format_summary_text(summary: dict) -> str:
    """Format summary statistics as text."""
    return (
        f"Total Requests: {summary['total_requests']}\n"
        f"Unique Client IPs: {summary['unique_ips']}\n"
        f"First Timestamp: {summary['first_timestamp']}\n"
        f"Last Timestamp: {summary['last_timestamp']}\n"
        f"Error Rate: {summary['error_rate']}%"
    )


def format_summary_json(summary: dict) -> str:
    """Format summary statistics as JSON."""
    return json.dumps(summary, indent=2)


def format_top_text(top_values: list[tuple], by: str) -> str:
    """Format top values as text."""
    lines = []
    for value, count in top_values:
        lines.append(f"{value}: {count}")
    return "\n".join(lines)


def format_top_json(top_values: list[tuple], by: str) -> str:
    """Format top values as JSON."""
    data = [{"value": str(value), "count": count} for value, count in top_values]
    return json.dumps(data, indent=2)


def format_errors_text(errors: list[tuple]) -> str:
    """Format errors as text."""
    lines = []
    for (status, path), count in errors:
        lines.append(f"{status} {path}: {count}")
    return "\n".join(lines)


def format_errors_json(errors: list[tuple]) -> str:
    """Format errors as JSON."""
    data = [{"status": status, "path": path, "count": count} for (status, path), count in errors]
    return json.dumps(data, indent=2)


def format_hourly_text(hourly: dict[int, int]) -> str:
    """Format hourly distribution as a text histogram."""
    lines = []
    max_count = max(hourly.values()) if hourly.values() else 0

    for hour in range(24):
        count = hourly[hour]
        # Scale to max 50 chars for visualization
        if max_count > 0:
            bar_length = int((count / max_count) * 50)
        else:
            bar_length = 0
        bar = "█" * bar_length
        lines.append(f"{hour:02d}:00 {bar} {count}")

    return "\n".join(lines)


def format_hourly_json(hourly: dict[int, int]) -> str:
    """Format hourly distribution as JSON."""
    data = [{"hour": hour, "count": count} for hour, count in sorted(hourly.items())]
    return json.dumps(data, indent=2)
