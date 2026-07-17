"""Command-line interface for loglens."""

from __future__ import annotations

import argparse
import json
import sys
from datetime import datetime

from . import analyze
from .parser import LogRecord, ParseResult, iter_file_lines, parse_lines

EXIT_OK = 0
EXIT_NO_VALID_LINES = 1
EXIT_FILE_ERROR = 2


def _parse_iso(value: str) -> datetime:
    """Parse an ISO-8601 timestamp for --since/--until options."""
    try:
        return datetime.fromisoformat(value)
    except ValueError as exc:
        raise argparse.ArgumentTypeError(f"invalid ISO-8601 timestamp: {value}") from exc


def build_parser() -> argparse.ArgumentParser:
    """Construct the argument parser."""
    common = argparse.ArgumentParser(add_help=False)
    common.add_argument(
        "--format",
        choices=("text", "json"),
        default=argparse.SUPPRESS,
        help="output format (default: text)",
    )

    parser = argparse.ArgumentParser(
        prog="loglens",
        description="Analyze web server access logs in Combined Log Format.",
    )
    parser.add_argument(
        "--format",
        choices=("text", "json"),
        default="text",
        help="output format (default: text)",
    )
    subparsers = parser.add_subparsers(dest="command", required=True)

    p_summary = subparsers.add_parser(
        "summary", parents=[common], help="overall request statistics"
    )
    p_summary.add_argument("logfile")

    p_top = subparsers.add_parser(
        "top", parents=[common], help="top values by request count"
    )
    p_top.add_argument("logfile")
    p_top.add_argument("--by", choices=("ip", "path", "status"), required=True)
    p_top.add_argument("-n", type=int, default=10, help="number of results (default: 10)")

    p_errors = subparsers.add_parser(
        "errors", parents=[common], help="4xx/5xx requests grouped by status and path"
    )
    p_errors.add_argument("logfile")
    p_errors.add_argument("--since", type=_parse_iso, help="ISO-8601 lower bound (inclusive)")
    p_errors.add_argument("--until", type=_parse_iso, help="ISO-8601 upper bound (inclusive)")

    p_hourly = subparsers.add_parser(
        "hourly", parents=[common], help="request count per hour of day"
    )
    p_hourly.add_argument("logfile")

    return parser


def _load(logfile: str) -> ParseResult | None:
    """Read and parse a logfile, returning None on file errors."""
    try:
        lines = list(iter_file_lines(logfile))
    except OSError as exc:
        print(f"loglens: cannot read '{logfile}': {exc.strerror or exc}", file=sys.stderr)
        return None
    return parse_lines(lines)


def _report_malformed(malformed: int) -> None:
    if malformed:
        noun = "line" if malformed == 1 else "lines"
        print(f"loglens: skipped {malformed} malformed {noun}", file=sys.stderr)


def _emit(data: object, fmt: str, text_fn) -> None:
    if fmt == "json":
        json.dump(data, sys.stdout, indent=2)
        sys.stdout.write("\n")
    else:
        text_fn()


def _cmd_summary(records: list[LogRecord], fmt: str) -> None:
    data = analyze.summary(records)

    def as_text() -> None:
        print(f"Total requests:  {data['total_requests']}")
        print(f"Unique IPs:      {data['unique_ips']}")
        print(f"First timestamp: {data['first_timestamp']}")
        print(f"Last timestamp:  {data['last_timestamp']}")
        print(f"Error rate:      {data['error_rate']:.2f}% ({data['error_count']} errors)")

    _emit(data, fmt, as_text)


def _cmd_top(records: list[LogRecord], by: str, n: int, fmt: str) -> None:
    rows = analyze.top(records, by, n)
    data = {"by": by, "results": rows}

    def as_text() -> None:
        if not rows:
            print("(no data)")
            return
        width = max(len(row["value"]) for row in rows)
        for row in rows:
            print(f"{row['value']:<{width}}  {row['count']}")

    _emit(data, fmt, as_text)


def _cmd_errors(
    records: list[LogRecord],
    since: datetime | None,
    until: datetime | None,
    fmt: str,
) -> None:
    rows = analyze.errors(records, since, until)

    def as_text() -> None:
        if not rows:
            print("(no errors)")
            return
        for row in rows:
            print(f"{row['status']} {row['path']}  {row['count']}")

    _emit(rows, fmt, as_text)


def _cmd_hourly(records: list[LogRecord], fmt: str) -> None:
    buckets = analyze.hourly(records)
    data = {f"{hour:02d}": count for hour, count in enumerate(buckets)}

    def as_text() -> None:
        peak = max(buckets) if buckets else 0
        max_bar = 40
        for hour, count in enumerate(buckets):
            bar_len = round(count / peak * max_bar) if peak else 0
            bar = "#" * bar_len
            print(f"{hour:02d} {bar:<{max_bar}} {count}")

    _emit(data, fmt, as_text)


def main(argv: list[str] | None = None) -> int:
    """Program entry point. Returns the process exit code."""
    parser = build_parser()
    args = parser.parse_args(argv)
    fmt = getattr(args, "format", "text")

    result = _load(args.logfile)
    if result is None:
        return EXIT_FILE_ERROR

    _report_malformed(result.malformed)

    if not result.records:
        print(f"loglens: no valid log lines in '{args.logfile}'", file=sys.stderr)
        return EXIT_NO_VALID_LINES

    if args.command == "summary":
        _cmd_summary(result.records, fmt)
    elif args.command == "top":
        _cmd_top(result.records, args.by, args.n, fmt)
    elif args.command == "errors":
        _cmd_errors(result.records, args.since, args.until, fmt)
    elif args.command == "hourly":
        _cmd_hourly(result.records, fmt)

    return EXIT_OK


if __name__ == "__main__":
    sys.exit(main())
