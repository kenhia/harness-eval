"""Command-line interface for loglens."""

from __future__ import annotations

import argparse
import json
import sys
from datetime import datetime

from . import __version__, analyze
from .parser import LogRecord, parse_lines

EXIT_OK = 0
EXIT_NO_VALID_LINES = 1
EXIT_FILE_ERROR = 2


def _parse_iso(value: str) -> datetime:
    try:
        return datetime.fromisoformat(value)
    except ValueError as exc:  # pragma: no cover - argparse wraps the message
        raise argparse.ArgumentTypeError(f"invalid ISO8601 timestamp: {value!r}") from exc


def build_parser() -> argparse.ArgumentParser:
    """Construct the top-level argument parser."""
    parser = argparse.ArgumentParser(
        prog="loglens",
        description="Analyze web server access logs in Combined Log Format (CLF).",
    )
    parser.add_argument("--version", action="version", version=f"%(prog)s {__version__}")
    parser.add_argument(
        "--format",
        choices=["text", "json"],
        default="text",
        help="output format (default: text)",
    )

    sub = parser.add_subparsers(dest="command", required=True)

    p_summary = sub.add_parser("summary", help="overall request statistics")
    p_summary.add_argument("logfile")

    p_top = sub.add_parser("top", help="top N values by request count")
    p_top.add_argument("logfile")
    p_top.add_argument("--by", choices=["ip", "path", "status"], required=True)
    p_top.add_argument("-n", type=int, default=10, help="number of results (default: 10)")

    p_errors = sub.add_parser("errors", help="4xx/5xx requests grouped by (status, path)")
    p_errors.add_argument("logfile")
    p_errors.add_argument(
        "--since", type=_parse_iso, help="only errors at or after this ISO8601 time"
    )
    p_errors.add_argument(
        "--until", type=_parse_iso, help="only errors at or before this ISO8601 time"
    )

    p_hourly = sub.add_parser("hourly", help="request count per hour of day")
    p_hourly.add_argument("logfile")

    return parser


def _load(logfile: str) -> tuple[list[LogRecord], int]:
    with open(logfile, encoding="utf-8", errors="replace") as fh:
        return parse_lines(fh)


def _print_json(payload: object) -> None:
    json.dump(payload, sys.stdout, indent=2)
    sys.stdout.write("\n")


def _render_summary(data: dict, fmt: str) -> None:
    if fmt == "json":
        _print_json(data)
        return
    print(f"Total requests: {data['total_requests']}")
    print(f"Unique client IPs: {data['unique_ips']}")
    print(f"First timestamp: {data['first_timestamp']}")
    print(f"Last timestamp: {data['last_timestamp']}")
    print(f"Error rate: {data['error_rate']:.2f}%")


def _render_top(data: list[dict], by: str, fmt: str) -> None:
    if fmt == "json":
        _print_json(data)
        return
    if not data:
        print("(no data)")
        return
    width = max(len(str(row["value"])) for row in data)
    for row in data:
        print(f"{str(row['value']):<{width}}  {row['count']}")


def _render_errors(data: list[dict], fmt: str) -> None:
    if fmt == "json":
        _print_json(data)
        return
    if not data:
        print("(no errors)")
        return
    for row in data:
        print(f"{row['status']}  {row['path']}  {row['count']}")


def _render_hourly(buckets: list[int], fmt: str) -> None:
    if fmt == "json":
        _print_json({f"{hour:02d}": count for hour, count in enumerate(buckets)})
        return
    peak = max(buckets) if buckets else 0
    scale = 50
    for hour, count in enumerate(buckets):
        bar_len = 0 if peak == 0 else round(count / peak * scale)
        bar = "#" * bar_len
        print(f"{hour:02d}  {bar} {count}")


def main(argv: list[str] | None = None) -> int:
    """Program entry point. Returns a process exit code."""
    args = build_parser().parse_args(argv)

    try:
        records, malformed = _load(args.logfile)
    except (FileNotFoundError, IsADirectoryError, PermissionError) as exc:
        print(f"loglens: cannot read {args.logfile!r}: {exc.strerror}", file=sys.stderr)
        return EXIT_FILE_ERROR
    except OSError as exc:
        print(f"loglens: cannot read {args.logfile!r}: {exc}", file=sys.stderr)
        return EXIT_FILE_ERROR

    if malformed:
        plural = "s" if malformed != 1 else ""
        print(f"loglens: skipped {malformed} malformed line{plural}", file=sys.stderr)

    if not records:
        print(f"loglens: no valid log lines in {args.logfile!r}", file=sys.stderr)
        return EXIT_NO_VALID_LINES

    if args.command == "summary":
        _render_summary(analyze.summary(records), args.format)
    elif args.command == "top":
        _render_top(analyze.top(records, args.by, args.n), args.by, args.format)
    elif args.command == "errors":
        _render_errors(analyze.errors(records, args.since, args.until), args.format)
    elif args.command == "hourly":
        _render_hourly(analyze.hourly(records), args.format)

    return EXIT_OK


if __name__ == "__main__":  # pragma: no cover
    sys.exit(main())
