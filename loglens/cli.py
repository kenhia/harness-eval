from __future__ import annotations

import json
import sys
from collections import Counter, defaultdict
from datetime import datetime
from pathlib import Path

import click

from loglens.parser import LogParser


def load_logs(filepath: str) -> tuple[list, int, bool]:
    """Load logs from file. Returns (entries, malformed_count, is_read_error)."""
    if not Path(filepath).exists():
        return [], 0, True

    entries, malformed_count = LogParser.parse_file(filepath)
    return entries, malformed_count, False


def report_malformed(count: int) -> None:
    """Report malformed lines to stderr."""
    if count > 0:
        click.echo(f"Warning: {count} malformed log line(s) skipped", err=True)


@click.group()
def cli() -> None:
    """loglens - web access log analysis CLI."""
    pass


@cli.command()
@click.argument("logfile", type=str)
@click.option("--format", "output_format", type=click.Choice(["text", "json"]), default="text")
def summary(logfile: str, output_format: str) -> None:
    """Display summary statistics for a log file."""
    entries, malformed_count, is_read_error = load_logs(logfile)

    if is_read_error:
        click.echo(f"Error: Cannot read file '{logfile}'", err=True)
        sys.exit(2)

    if not entries:
        click.echo("Error: No valid log lines found", err=True)
        sys.exit(1)

    report_malformed(malformed_count)

    unique_ips = len(set(entry.ip for entry in entries))
    error_count = sum(1 for entry in entries if entry.is_error)
    error_rate = (error_count / len(entries) * 100) if entries else 0
    first_ts = min(entry.timestamp for entry in entries)
    last_ts = max(entry.timestamp for entry in entries)

    data = {
        "total_requests": len(entries),
        "unique_ips": unique_ips,
        "first_timestamp": first_ts.isoformat(),
        "last_timestamp": last_ts.isoformat(),
        "error_rate": f"{error_rate:.2f}%",
    }

    if output_format == "json":
        click.echo(json.dumps(data))
    else:
        click.echo(f"Total Requests: {data['total_requests']}")
        click.echo(f"Unique IPs: {data['unique_ips']}")
        click.echo(f"First Timestamp: {data['first_timestamp']}")
        click.echo(f"Last Timestamp: {data['last_timestamp']}")
        click.echo(f"Error Rate: {data['error_rate']}")


@cli.command()
@click.argument("logfile", type=str)
@click.option("--by", "group_by", type=click.Choice(["ip", "path", "status"]), default="ip")
@click.option("-n", "limit", type=int, default=10)
@click.option("--format", "output_format", type=click.Choice(["text", "json"]), default="text")
def top(logfile: str, group_by: str, limit: int, output_format: str) -> None:
    """Show top N values by request count."""
    entries, malformed_count, is_read_error = load_logs(logfile)

    if is_read_error:
        click.echo(f"Error: Cannot read file '{logfile}'", err=True)
        sys.exit(2)

    if not entries:
        click.echo("Error: No valid log lines found", err=True)
        sys.exit(1)

    report_malformed(malformed_count)

    if group_by == "ip":
        counter = Counter(entry.ip for entry in entries)
    elif group_by == "path":
        counter = Counter(entry.path for entry in entries)
    else:  # status
        counter = Counter(entry.status for entry in entries)

    # Sort by count descending, then by value ascending (for ties)
    sorted_items = sorted(counter.items(), key=lambda x: (-x[1], str(x[0])))
    top_items = sorted_items[:limit]

    data = [{"value": str(key), "count": count} for key, count in top_items]

    if output_format == "json":
        click.echo(json.dumps(data))
    else:
        for key, count in top_items:
            click.echo(f"{key}: {count}")


@cli.command()
@click.argument("logfile", type=str)
@click.option("--since", type=str, default=None)
@click.option("--until", type=str, default=None)
@click.option("--format", "output_format", type=click.Choice(["text", "json"]), default="text")
def errors(logfile: str, since: str | None, until: str | None, output_format: str) -> None:
    """Show 4xx/5xx errors grouped by (status, path)."""
    entries, malformed_count, is_read_error = load_logs(logfile)

    if is_read_error:
        click.echo(f"Error: Cannot read file '{logfile}'", err=True)
        sys.exit(2)

    if not entries:
        click.echo("Error: No valid log lines found", err=True)
        sys.exit(1)

    report_malformed(malformed_count)

    # Parse since/until timestamps
    since_dt: datetime | None = None
    until_dt: datetime | None = None

    if since:
        try:
            since_dt = datetime.fromisoformat(since)
        except ValueError:
            click.echo(f"Error: Invalid ISO8601 timestamp: {since}", err=True)
            sys.exit(2)

    if until:
        try:
            until_dt = datetime.fromisoformat(until)
        except ValueError:
            click.echo(f"Error: Invalid ISO8601 timestamp: {until}", err=True)
            sys.exit(2)

    # Filter errors by timestamp and error status
    error_entries = [
        e
        for e in entries
        if e.is_error
        and (since_dt is None or e.timestamp >= since_dt)
        and (until_dt is None or e.timestamp <= until_dt)
    ]

    # Group by (status, path) and count
    error_groups: dict[tuple[int, str], int] = defaultdict(int)
    for entry in error_entries:
        error_groups[(entry.status, entry.path)] += 1

    # Sort by count descending, then by status/path ascending
    sorted_errors = sorted(
        error_groups.items(),
        key=lambda x: (-x[1], x[0][0], x[0][1]),
    )

    data = [
        {"status": status, "path": path, "count": count}
        for (status, path), count in sorted_errors
    ]

    if output_format == "json":
        click.echo(json.dumps(data))
    else:
        for (status, path), count in sorted_errors:
            click.echo(f"{status} {path}: {count}")


@cli.command()
@click.argument("logfile", type=str)
@click.option("--format", "output_format", type=click.Choice(["text", "json"]), default="text")
def hourly(logfile: str, output_format: str) -> None:
    """Show request count per hour of day as histogram."""
    entries, malformed_count, is_read_error = load_logs(logfile)

    if is_read_error:
        click.echo(f"Error: Cannot read file '{logfile}'", err=True)
        sys.exit(2)

    if not entries:
        click.echo("Error: No valid log lines found", err=True)
        sys.exit(1)

    report_malformed(malformed_count)

    # Count requests per hour
    hourly_counts = [0] * 24
    for entry in entries:
        hourly_counts[entry.hour] += 1

    data = [{"hour": hour, "count": count} for hour, count in enumerate(hourly_counts)]

    if output_format == "json":
        click.echo(json.dumps(data))
    else:
        max_count = max(hourly_counts) if hourly_counts else 1
        max_width = 40
        scale = max_width / max_count if max_count > 0 else 1

        for hour, count in enumerate(hourly_counts):
            bar_width = int(count * scale)
            bar = "█" * bar_width
            click.echo(f"{hour:02d}:00 {bar} {count}")


def main() -> None:
    """Entry point for the CLI."""
    cli()


if __name__ == "__main__":
    main()
