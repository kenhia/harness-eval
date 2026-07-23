"""Command-line interface for loglens."""

import sys
from datetime import datetime
from pathlib import Path

from loglens.analysis import (
    calculate_summary,
    get_errors,
    get_hourly_distribution,
    get_top_values,
)
from loglens.formatter import (
    format_errors_json,
    format_errors_text,
    format_hourly_json,
    format_hourly_text,
    format_summary_json,
    format_summary_text,
    format_top_json,
    format_top_text,
)
from loglens.parser import LogParser


def check_file_exists(logfile: str) -> tuple[bool, int]:
    """Check if file exists and is readable.

    Returns (is_valid, exit_code).
    """
    path = Path(logfile)
    if not path.exists() or not path.is_file():
        print(f"Error: File '{logfile}' not found or is not readable", file=sys.stderr)
        return False, 2
    return True, 0


def parse_iso8601(date_string: str) -> datetime | None:
    """Parse ISO8601 date string to datetime."""
    try:
        return datetime.fromisoformat(date_string)
    except ValueError:
        return None


def cmd_summary(logfile: str, output_format: str) -> int:
    """Handle summary subcommand."""
    is_valid, exit_code = check_file_exists(logfile)
    if not is_valid:
        return exit_code

    parser = LogParser()
    entries = parser.parse_file(logfile)

    if parser.malformed_count > 0:
        print(f"Warning: {parser.malformed_count} malformed lines skipped", file=sys.stderr)

    if not entries:
        print("Error: No valid log entries found", file=sys.stderr)
        return 1

    summary = calculate_summary(entries)

    if output_format == "json":
        print(format_summary_json(summary))
    else:
        print(format_summary_text(summary))

    return 0


def cmd_top(logfile: str, by: str, n: int, output_format: str) -> int:
    """Handle top subcommand."""
    is_valid, exit_code = check_file_exists(logfile)
    if not is_valid:
        return exit_code

    parser = LogParser()
    entries = parser.parse_file(logfile)

    if parser.malformed_count > 0:
        print(f"Warning: {parser.malformed_count} malformed lines skipped", file=sys.stderr)

    if not entries:
        print("Error: No valid log entries found", file=sys.stderr)
        return 1

    top_values = get_top_values(entries, by, n)

    if output_format == "json":
        print(format_top_json(top_values, by))
    else:
        print(format_top_text(top_values, by))

    return 0


def cmd_errors(logfile: str, since: str | None, until: str | None, output_format: str) -> int:
    """Handle errors subcommand."""
    is_valid, exit_code = check_file_exists(logfile)
    if not is_valid:
        return exit_code

    since_dt = None
    until_dt = None

    if since:
        since_dt = parse_iso8601(since)
        if since_dt is None:
            print(f"Error: Invalid ISO8601 date for --since: {since}", file=sys.stderr)
            return 1

    if until:
        until_dt = parse_iso8601(until)
        if until_dt is None:
            print(f"Error: Invalid ISO8601 date for --until: {until}", file=sys.stderr)
            return 1

    parser = LogParser()
    entries = parser.parse_file(logfile)

    if parser.malformed_count > 0:
        print(f"Warning: {parser.malformed_count} malformed lines skipped", file=sys.stderr)

    if not entries:
        print("Error: No valid log entries found", file=sys.stderr)
        return 1

    errors = get_errors(entries, since_dt, until_dt)

    if output_format == "json":
        print(format_errors_json(errors))
    else:
        print(format_errors_text(errors))

    return 0


def cmd_hourly(logfile: str, output_format: str) -> int:
    """Handle hourly subcommand."""
    is_valid, exit_code = check_file_exists(logfile)
    if not is_valid:
        return exit_code

    parser = LogParser()
    entries = parser.parse_file(logfile)

    if parser.malformed_count > 0:
        print(f"Warning: {parser.malformed_count} malformed lines skipped", file=sys.stderr)

    if not entries:
        print("Error: No valid log entries found", file=sys.stderr)
        return 1

    hourly = get_hourly_distribution(entries)

    if output_format == "json":
        print(format_hourly_json(hourly))
    else:
        print(format_hourly_text(hourly))

    return 0


def main() -> None:
    """Main entry point for the CLI."""
    if len(sys.argv) < 2:
        print("Usage: loglens [--format {text,json}] <subcommand> <logfile> [options]")
        print("Subcommands: summary, top, errors, hourly")
        sys.exit(1)

    # Parse global format option
    output_format = "text"
    args = sys.argv[1:]

    if "--format" in args:
        idx = args.index("--format")
        if idx + 1 < len(args):
            output_format = args[idx + 1]
            args = args[:idx] + args[idx + 2 :]
        else:
            print("Error: --format requires a value", file=sys.stderr)
            sys.exit(1)

    if len(args) < 2:
        print("Usage: loglens [--format {text,json}] <subcommand> <logfile> [options]")
        sys.exit(1)

    subcommand = args[0]
    logfile = args[1]
    remaining_args = args[2:]

    exit_code = 0

    if subcommand == "summary":
        exit_code = cmd_summary(logfile, output_format)
    elif subcommand == "top":
        by = "ip"
        n = 10
        i = 0
        while i < len(remaining_args):
            if remaining_args[i] == "--by" and i + 1 < len(remaining_args):
                by = remaining_args[i + 1]
                i += 2
            elif remaining_args[i] == "-n" and i + 1 < len(remaining_args):
                try:
                    n = int(remaining_args[i + 1])
                    i += 2
                except ValueError:
                    print("Error: -n requires an integer value", file=sys.stderr)
                    sys.exit(1)
            else:
                i += 1

        exit_code = cmd_top(logfile, by, n, output_format)
    elif subcommand == "errors":
        since = None
        until = None
        i = 0
        while i < len(remaining_args):
            if remaining_args[i] == "--since" and i + 1 < len(remaining_args):
                since = remaining_args[i + 1]
                i += 2
            elif remaining_args[i] == "--until" and i + 1 < len(remaining_args):
                until = remaining_args[i + 1]
                i += 2
            else:
                i += 1

        exit_code = cmd_errors(logfile, since, until, output_format)
    elif subcommand == "hourly":
        exit_code = cmd_hourly(logfile, output_format)
    else:
        print(f"Error: Unknown subcommand '{subcommand}'", file=sys.stderr)
        sys.exit(1)

    sys.exit(exit_code)


if __name__ == "__main__":
    main()
