"""Command-line interface for loglens."""

import argparse
import json
import sys
from collections import Counter
from datetime import datetime

from loglens.parser import load_log_file


def format_output(data, fmt: str = "text") -> str | dict:
    """Format output for display."""
    if fmt == "json":
        return data
    return data


def cmd_summary(args, entries, malformed_count):
    """Handle summary subcommand."""
    if not entries:
        print("Error: no valid log lines found", file=sys.stderr)
        return 1

    unique_ips = len(set(e.ip for e in entries))
    error_count = sum(1 for e in entries if e.status >= 400)
    error_rate = (error_count / len(entries) * 100) if entries else 0

    first_timestamp = min(e.timestamp for e in entries)
    last_timestamp = max(e.timestamp for e in entries)

    result = {
        "total_requests": len(entries),
        "unique_client_ips": unique_ips,
        "first_timestamp": first_timestamp.isoformat(),
        "last_timestamp": last_timestamp.isoformat(),
        "error_rate": round(error_rate, 2),
    }

    if malformed_count > 0:
        print(f"Skipped {malformed_count} malformed lines", file=sys.stderr)

    if args.format == "json":
        print(json.dumps(result))
    else:
        print(f"Total requests: {result['total_requests']}")
        print(f"Unique client IPs: {result['unique_client_ips']}")
        print(f"First timestamp: {result['first_timestamp']}")
        print(f"Last timestamp: {result['last_timestamp']}")
        print(f"Error rate: {result['error_rate']}%")

    return 0


def cmd_top(args, entries, malformed_count):
    """Handle top subcommand."""
    if not entries:
        print("Error: no valid log lines found", file=sys.stderr)
        return 1

    if args.by == "ip":
        counter = Counter(e.ip for e in entries)
    elif args.by == "path":
        counter = Counter(e.path for e in entries)
    elif args.by == "status":
        counter = Counter(e.status for e in entries)
    else:
        print(f"Error: unknown --by option: {args.by}", file=sys.stderr)
        return 2

    # Sort by count (descending), then by value (ascending) for tie-breaking
    sorted_items = sorted(counter.items(), key=lambda x: (-x[1], str(x[0])))[:args.n]

    result = [{"value": str(value), "count": count} for value, count in sorted_items]

    if malformed_count > 0:
        print(f"Skipped {malformed_count} malformed lines", file=sys.stderr)

    if args.format == "json":
        print(json.dumps(result))
    else:
        print(f"Top {args.n} by {args.by}:")
        for item in result:
            print(f"  {item['value']}: {item['count']}")

    return 0


def cmd_errors(args, entries, malformed_count):
    """Handle errors subcommand."""
    if not entries:
        print("Error: no valid log lines found", file=sys.stderr)
        return 1

    # Filter by status code (4xx, 5xx)
    errors = [e for e in entries if e.status >= 400]

    # Apply timestamp filters
    if args.since:
        try:
            since_dt = datetime.fromisoformat(args.since)
            errors = [e for e in errors if e.timestamp >= since_dt]
        except ValueError:
            print(f"Error: invalid --since timestamp: {args.since}", file=sys.stderr)
            return 2

    if args.until:
        try:
            until_dt = datetime.fromisoformat(args.until)
            errors = [e for e in errors if e.timestamp <= until_dt]
        except ValueError:
            print(f"Error: invalid --until timestamp: {args.until}", file=sys.stderr)
            return 2

    # Group by (status, path)
    error_groups = {}
    for e in errors:
        key = (e.status, e.path)
        error_groups[key] = error_groups.get(key, 0) + 1

    # Sort by count (descending), then by status and path (ascending)
    sorted_errors = sorted(error_groups.items(), key=lambda x: (-x[1], x[0]))

    result = [{"status": k[0], "path": k[1], "count": v} for k, v in sorted_errors]

    if malformed_count > 0:
        print(f"Skipped {malformed_count} malformed lines", file=sys.stderr)

    if args.format == "json":
        print(json.dumps(result))
    else:
        if result:
            print("Errors (status, path):")
            for item in result:
                print(f"  {item['status']} {item['path']}: {item['count']}")
        else:
            print("No errors found")

    return 0


def cmd_hourly(args, entries, malformed_count):
    """Handle hourly subcommand."""
    if not entries:
        print("Error: no valid log lines found", file=sys.stderr)
        return 1

    # Count requests per hour
    hourly_counts = Counter(e.hour for e in entries)

    # Create histogram
    result = {}
    for hour in range(24):
        count = hourly_counts.get(hour, 0)
        result[f"{hour:02d}"] = count

    if malformed_count > 0:
        print(f"Skipped {malformed_count} malformed lines", file=sys.stderr)

    if args.format == "json":
        print(json.dumps(result))
    else:
        # Text histogram
        max_count = max(result.values()) if result.values() else 1
        if max_count == 0:
            max_count = 1

        for hour, count in result.items():
            bar_width = int(count / max_count * 40) if max_count > 0 else 0
            bar = "█" * bar_width
            print(f"{hour}: {bar} {count}")

    return 0


def main():
    """Main entry point."""
    parser = argparse.ArgumentParser(description="Analyze web server access logs")
    parser.add_argument("--format", choices=["text", "json"], default="text", help="Output format")

    subparsers = parser.add_subparsers(dest="command", required=True)

    # summary subcommand
    summary_parser = subparsers.add_parser("summary", help="Summarize log statistics")
    summary_parser.add_argument("logfile", help="Path to log file")

    # top subcommand
    top_parser = subparsers.add_parser("top", help="Show top values by category")
    top_parser.add_argument("logfile", help="Path to log file")
    top_parser.add_argument("--by", choices=["ip", "path", "status"], default="ip", help="Group by")
    top_parser.add_argument("-n", type=int, default=10, help="Number of results")

    # errors subcommand
    errors_parser = subparsers.add_parser("errors", help="Show error responses")
    errors_parser.add_argument("logfile", help="Path to log file")
    errors_parser.add_argument("--since", help="Start timestamp (ISO8601)")
    errors_parser.add_argument("--until", help="End timestamp (ISO8601)")

    # hourly subcommand
    hourly_parser = subparsers.add_parser("hourly", help="Show requests per hour")
    hourly_parser.add_argument("logfile", help="Path to log file")

    args = parser.parse_args()

    # Load log file
    try:
        entries, malformed_count = load_log_file(args.logfile)
    except FileNotFoundError:
        print(f"Error: file not found: {args.logfile}", file=sys.stderr)
        return 2
    except OSError as e:
        print(f"Error: {e}", file=sys.stderr)
        return 2

    # Check if we have any valid entries
    if not entries:
        print("Error: no valid log lines found", file=sys.stderr)
        return 1

    # Dispatch to appropriate command handler
    if args.command == "summary":
        return cmd_summary(args, entries, malformed_count)
    elif args.command == "top":
        return cmd_top(args, entries, malformed_count)
    elif args.command == "errors":
        return cmd_errors(args, entries, malformed_count)
    elif args.command == "hourly":
        return cmd_hourly(args, entries, malformed_count)

    return 0


if __name__ == "__main__":
    sys.exit(main())
