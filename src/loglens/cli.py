"""Command-line interface for loglens."""

from __future__ import annotations

import argparse
import sys
from collections.abc import Sequence
from datetime import datetime

from . import analyze, render
from .parser import ParseResult, iter_file, parse_lines

EXIT_OK = 0
EXIT_NO_VALID_LINES = 1
EXIT_BAD_FILE = 2


def _iso8601(value: str) -> datetime:
    try:
        return datetime.fromisoformat(value)
    except ValueError as exc:
        raise argparse.ArgumentTypeError(f"not a valid ISO 8601 timestamp: {value!r}") from exc


def build_parser() -> argparse.ArgumentParser:
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

    common = argparse.ArgumentParser(add_help=False)
    common.add_argument("logfile", metavar="LOGFILE", help="path to the access log")
    common.add_argument(
        "--format",
        choices=("text", "json"),
        default=argparse.SUPPRESS,
        help="output format (default: text)",
    )

    subs = parser.add_subparsers(dest="command", required=True)

    subs.add_parser(
        "summary",
        parents=[common],
        help="overall request statistics",
    )

    top = subs.add_parser("top", parents=[common], help="most frequent values by count")
    top.add_argument(
        "--by",
        choices=("ip", "path", "status"),
        required=True,
        help="field to rank",
    )
    top.add_argument("-n", type=int, default=10, help="number of results (default: 10)")

    errors = subs.add_parser("errors", parents=[common], help="4xx/5xx grouped by status and path")
    errors.add_argument(
        "--since", type=_iso8601, help="only entries at or after this ISO 8601 time"
    )
    errors.add_argument(
        "--until", type=_iso8601, help="only entries at or before this ISO 8601 time"
    )

    subs.add_parser("hourly", parents=[common], help="request histogram by hour of day")

    return parser


def _load(path: str) -> ParseResult | None:
    """Read and parse a log file, returning None if it cannot be read."""
    try:
        return parse_lines(iter_file(path))
    except OSError as exc:
        print(f"loglens: cannot read {path}: {exc.strerror or exc}", file=sys.stderr)
        return None


def _render(args: argparse.Namespace, entries: Sequence) -> str:
    as_json = args.format == "json"
    match args.command:
        case "summary":
            summary = analyze.summarize(entries)
            return (
                render.dump_json(render.summary_json(summary))
                if as_json
                else render.summary_text(summary)
            )
        case "top":
            rows = analyze.top(entries, args.by, args.n)
            return (
                render.dump_json(render.top_json(rows, args.by))
                if as_json
                else render.top_text(rows, args.by)
            )
        case "errors":
            rows = analyze.errors(entries, args.since, args.until)
            return (
                render.dump_json(render.errors_json(rows)) if as_json else render.errors_text(rows)
            )
        case "hourly":
            buckets = analyze.hourly(entries)
            return (
                render.dump_json(render.hourly_json(buckets))
                if as_json
                else render.hourly_text(buckets)
            )
    raise AssertionError(f"unhandled command: {args.command}")


def main(argv: list[str] | None = None) -> int:
    parser = build_parser()
    args = parser.parse_args(argv)

    if getattr(args, "n", 1) < 1:
        parser.error("-n must be at least 1")

    result = _load(args.logfile)
    if result is None:
        return EXIT_BAD_FILE

    if result.malformed:
        plural = "s" if result.malformed != 1 else ""
        print(f"loglens: skipped {result.malformed} malformed line{plural}", file=sys.stderr)

    if not result.entries:
        print(f"loglens: no valid log lines in {args.logfile}", file=sys.stderr)
        return EXIT_NO_VALID_LINES

    print(_render(args, result.entries))
    return EXIT_OK


if __name__ == "__main__":
    sys.exit(main())
