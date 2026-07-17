"""Command-line interface for loglens."""

from __future__ import annotations

import argparse
import json
import sys
from collections.abc import Sequence
from datetime import datetime

from . import __version__, analyze
from .parser import LogRecord, ParseResult, parse_lines

EXIT_OK = 0
EXIT_NO_VALID_LINES = 1
EXIT_FILE_ERROR = 2


class CLIError(Exception):
    """Raised for user-facing errors that map to a specific exit code."""

    def __init__(self, message: str, code: int):
        super().__init__(message)
        self.message = message
        self.code = code


def _read_log(path: str) -> ParseResult:
    try:
        with open(path, encoding="utf-8", errors="replace") as fh:
            return parse_lines(fh)
    except (FileNotFoundError, IsADirectoryError, PermissionError, OSError) as exc:
        raise CLIError(f"cannot read {path!r}: {exc}", EXIT_FILE_ERROR) from exc


def _parse_iso(value: str, flag: str) -> datetime:
    try:
        return datetime.fromisoformat(value)
    except ValueError as exc:
        raise CLIError(f"invalid {flag} timestamp: {value!r}", EXIT_FILE_ERROR) from exc


def _iso(dt: datetime) -> str:
    return dt.isoformat()


# --- summary ---------------------------------------------------------------


def _cmd_summary(records: Sequence[LogRecord], args: argparse.Namespace) -> str:
    data = analyze.summary(records)
    if args.format == "json":
        payload = {
            "total_requests": data["total_requests"],
            "unique_ips": data["unique_ips"],
            "first_timestamp": _iso(data["first_timestamp"]),
            "last_timestamp": _iso(data["last_timestamp"]),
            "error_count": data["error_count"],
            "error_rate": round(data["error_rate"], 2),
        }
        return json.dumps(payload, indent=2)
    lines = [
        f"Total requests:  {data['total_requests']}",
        f"Unique IPs:      {data['unique_ips']}",
        f"First timestamp: {_iso(data['first_timestamp'])}",
        f"Last timestamp:  {_iso(data['last_timestamp'])}",
        f"Error responses: {data['error_count']}",
        f"Error rate:      {data['error_rate']:.2f}%",
    ]
    return "\n".join(lines)


# --- top -------------------------------------------------------------------


def _cmd_top(records: Sequence[LogRecord], args: argparse.Namespace) -> str:
    rows = analyze.top(records, by=args.by, n=args.n)
    if args.format == "json":
        payload = {
            "by": args.by,
            "n": args.n,
            "results": [{"value": value, "count": count} for value, count in rows],
        }
        return json.dumps(payload, indent=2)
    if not rows:
        return "(no requests)"
    width = max(len(value) for value, _ in rows)
    return "\n".join(f"{value:<{width}}  {count}" for value, count in rows)


# --- errors ----------------------------------------------------------------


def _cmd_errors(records: Sequence[LogRecord], args: argparse.Namespace) -> str:
    since = _parse_iso(args.since, "--since") if args.since else None
    until = _parse_iso(args.until, "--until") if args.until else None
    rows = analyze.errors(records, since=since, until=until)
    if args.format == "json":
        payload = {
            "results": [
                {"status": status, "path": path, "count": count}
                for status, path, count in rows
            ]
        }
        return json.dumps(payload, indent=2)
    if not rows:
        return "(no errors)"
    path_width = max(len(path) for _, path, _ in rows)
    lines = [
        f"{status}  {path:<{path_width}}  {count}" for status, path, count in rows
    ]
    return "\n".join(lines)


# --- hourly ----------------------------------------------------------------


def _cmd_hourly(records: Sequence[LogRecord], args: argparse.Namespace) -> str:
    buckets = analyze.hourly(records)
    if args.format == "json":
        payload = {
            "hourly": [{"hour": hour, "count": count} for hour, count in enumerate(buckets)]
        }
        return json.dumps(payload, indent=2)
    peak = max(buckets) if buckets else 0
    bar_max = 40
    lines = []
    for hour, count in enumerate(buckets):
        bar_len = 0 if peak == 0 else round(count / peak * bar_max)
        bar = "#" * bar_len
        lines.append(f"{hour:02d}  {bar:<{bar_max}}  {count}")
    return "\n".join(lines)


_COMMANDS = {
    "summary": _cmd_summary,
    "top": _cmd_top,
    "errors": _cmd_errors,
    "hourly": _cmd_hourly,
}


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        prog="loglens",
        description="Analyze web server access logs in Combined Log Format.",
    )
    parser.add_argument("--version", action="version", version=f"%(prog)s {__version__}")
    parser.add_argument(
        "--format",
        choices=("text", "json"),
        default="text",
        help="output format (default: text)",
    )
    sub = parser.add_subparsers(dest="command", required=True)

    p_summary = sub.add_parser("summary", help="overall request statistics")
    p_summary.add_argument("logfile")

    p_top = sub.add_parser("top", help="top values by request count")
    p_top.add_argument("logfile")
    p_top.add_argument(
        "--by",
        choices=("ip", "path", "status"),
        required=True,
        help="dimension to rank by",
    )
    p_top.add_argument("-n", type=int, default=10, help="number of results (default: 10)")

    p_errors = sub.add_parser("errors", help="4xx/5xx requests grouped by status and path")
    p_errors.add_argument("logfile")
    p_errors.add_argument("--since", help="only include requests at/after this ISO8601 time")
    p_errors.add_argument("--until", help="only include requests at/before this ISO8601 time")

    p_hourly = sub.add_parser("hourly", help="request count per hour of day")
    p_hourly.add_argument("logfile")

    return parser


def main(argv: Sequence[str] | None = None) -> int:
    parser = build_parser()
    args = parser.parse_args(argv)

    if getattr(args, "command", None) == "top" and args.n < 0:
        parser.error("-n must be non-negative")

    try:
        result = _read_log(args.logfile)
    except CLIError as exc:
        print(f"loglens: {exc.message}", file=sys.stderr)
        return exc.code

    if result.malformed:
        plural = "s" if result.malformed != 1 else ""
        print(f"loglens: skipped {result.malformed} malformed line{plural}", file=sys.stderr)

    if not result.records:
        print(f"loglens: no valid log lines in {args.logfile!r}", file=sys.stderr)
        return EXIT_NO_VALID_LINES

    handler = _COMMANDS[args.command]
    try:
        output = handler(result.records, args)
    except CLIError as exc:
        print(f"loglens: {exc.message}", file=sys.stderr)
        return exc.code

    print(output)
    return EXIT_OK


if __name__ == "__main__":  # pragma: no cover
    raise SystemExit(main())
