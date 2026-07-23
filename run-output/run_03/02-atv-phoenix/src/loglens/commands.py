"""loglens subcommands"""

import json
from collections import Counter, defaultdict
from datetime import datetime

from .parser import parse_log_file


def summary(filepath: str, format_type: str = "text") -> tuple[int, str]:
    """Summary: total requests, unique IPs, first/last timestamp, error rate.

    Returns (exit_code, output)
    """
    lines, malformed = parse_log_file(filepath)
    if malformed == -1:
        return 2, ""  # File not found

    if not lines:
        return 1, ""  # No valid lines

    ips = set(line['ip'] for line in lines)
    errors = sum(1 for line in lines if line['status'] >= 400)
    error_rate = (errors / len(lines)) * 100 if lines else 0

    first_ts = min(line['timestamp'] for line in lines)
    last_ts = max(line['timestamp'] for line in lines)

    if format_type == "json":
        result = {
            "total_requests": len(lines),
            "unique_ips": len(ips),
            "first_timestamp": first_ts.isoformat(),
            "last_timestamp": last_ts.isoformat(),
            "error_rate": round(error_rate, 2),
        }
        output = json.dumps(result)
    else:
        output = (
            f"Total requests: {len(lines)}\n"
            f"Unique IPs: {len(ips)}\n"
            f"First timestamp: {first_ts}\n"
            f"Last timestamp: {last_ts}\n"
            f"Error rate: {error_rate:.2f}%"
        )

    return 0, output


def top(filepath: str, by: str = "ip", n: int = 10, format_type: str = "text") -> tuple[int, str]:
    """Top N values by request count (ip, path, or status).

    Returns (exit_code, output)
    """
    lines, malformed = parse_log_file(filepath)
    if malformed == -1:
        return 2, ""  # File not found

    if not lines:
        return 1, ""  # No valid lines

    if by == "ip":
        counter = Counter(line['ip'] for line in lines)
    elif by == "path":
        counter = Counter(
            line['request'].split()[1] if len(line['request'].split()) > 1 else "?"
            for line in lines
        )
    elif by == "status":
        counter = Counter(line['status'] for line in lines)
    else:
        return 2, ""

    # Sort by count descending, then by key ascending
    sorted_items = sorted(counter.items(), key=lambda x: (-x[1], str(x[0])))[:n]

    if format_type == "json":
        result = {by: [{"value": str(k), "count": v} for k, v in sorted_items]}
        output = json.dumps(result)
    else:
        lines_out = []
        for value, count in sorted_items:
            lines_out.append(f"{value}: {count}")
        output = "\n".join(lines_out)

    return 0, output


def errors(
    filepath: str,
    since: str = None,
    until: str = None,
    format_type: str = "text",
) -> tuple[int, str]:
    """4xx/5xx errors grouped by (status, path).

    Returns (exit_code, output)
    """
    lines, malformed = parse_log_file(filepath)
    if malformed == -1:
        return 2, ""  # File not found

    if not lines:
        return 1, ""  # No valid lines

    # Filter by timestamp if provided
    filtered_lines = lines
    if since:
        try:
            since_dt = datetime.fromisoformat(since)
            filtered_lines = [
                line for line in filtered_lines if line['timestamp'] >= since_dt
            ]
        except ValueError:
            pass

    if until:
        try:
            until_dt = datetime.fromisoformat(until)
            filtered_lines = [
                line for line in filtered_lines if line['timestamp'] <= until_dt
            ]
        except ValueError:
            pass

    # Group 4xx/5xx by (status, path)
    error_counts: dict = defaultdict(int)
    for line in filtered_lines:
        if line['status'] >= 400:
            path = line['request'].split()[1] if len(line['request'].split()) > 1 else "?"
            key = (line['status'], path)
            error_counts[key] += 1

    if not error_counts:
        if format_type == "json":
            output = json.dumps({"errors": []})
        else:
            output = ""
        return 0, output

    # Sort by count descending
    sorted_errors = sorted(error_counts.items(), key=lambda x: -x[1])

    if format_type == "json":
        result = {
            "errors": [
                {"status": k[0], "path": k[1], "count": v}
                for k, v in sorted_errors
            ]
        }
        output = json.dumps(result)
    else:
        lines_out = []
        for (status, path), count in sorted_errors:
            lines_out.append(f"{status} {path}: {count}")
        output = "\n".join(lines_out)

    return 0, output


def hourly(filepath: str, format_type: str = "text") -> tuple[int, str]:
    """Hourly request distribution (00-23).

    Returns (exit_code, output)
    """
    lines, malformed = parse_log_file(filepath)
    if malformed == -1:
        return 2, ""  # File not found

    if not lines:
        return 1, ""  # No valid lines

    hourly_counts = defaultdict(int)
    for line in lines:
        hourly_counts[line['hour']] += 1

    if format_type == "json":
        result = {"hourly": [{"hour": h, "count": hourly_counts[h]} for h in range(24)]}
        output = json.dumps(result)
    else:
        # Text histogram
        lines_out = []
        max_count = max(hourly_counts.values()) if hourly_counts else 1
        for h in range(24):
            count = hourly_counts[h]
            bar_width = int((count / max_count) * 40) if max_count > 0 else 0
            bar = "█" * bar_width
            lines_out.append(f"{h:02d}: {bar} ({count})")
        output = "\n".join(lines_out)

    return 0, output
