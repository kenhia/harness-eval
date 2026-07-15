"""Command-line interface for loglens."""

from __future__ import annotations

import argparse
import json
import sys
from collections.abc import Sequence
from datetime import UTC, datetime

from . import analyze
from .parser import LogRecord, parse_lines


def _parse_iso(value: str) -> datetime:
    """Parse an ISO 8601 timestamp, assuming UTC when no offset is given."""
    dt = datetime.fromisoformat(value)
    if dt.tzinfo is None:
        dt = dt.replace(tzinfo=UTC)
    return dt


def _iso(dt: datetime | None) -> str | None:
    return dt.isoformat() if dt is not None else None


def _build_parser() -> argparse.ArgumentParser:
    common = argparse.ArgumentParser(add_help=False)
    common.add_argument(
        "--format",
        choices=["text", "json"],
        default=argparse.SUPPRESS,
        help="output format (default: text)",
    )

    parser = argparse.ArgumentParser(
        prog="loglens",
        description="Analyze web server access logs in Combined Log Format (CLF).",
    )
    parser.add_argument("--format", choices=["text", "json"], default="text")

    sub = parser.add_subparsers(dest="command", required=True)

    p_summary = sub.add_parser("summary", parents=[common], help="overall statistics")
    p_summary.add_argument("logfile")

    p_top = sub.add_parser("top", parents=[common], help="top values by request count")
    p_top.add_argument("logfile")
    p_top.add_argument("--by", choices=["ip", "path", "status"], required=True)
    p_top.add_argument("-n", type=int, default=10, help="number of results (default: 10)")

    p_errors = sub.add_parser("errors", parents=[common], help="4xx/5xx grouped by status+path")
    p_errors.add_argument("logfile")
    p_errors.add_argument("--since", help="ISO 8601 lower bound (inclusive)")
    p_errors.add_argument("--until", help="ISO 8601 upper bound (exclusive)")

    p_hourly = sub.add_parser("hourly", parents=[common], help="requests per hour histogram")
    p_hourly.add_argument("logfile")

    return parser


def _load_records(path: str) -> tuple[list[LogRecord], int]:
    with open(path, encoding="utf-8", errors="replace") as fh:
        return parse_lines(fh)


def _render_summary(records: Sequence[LogRecord], fmt: str) -> str:
    stats = analyze.summarize(records)
    if fmt == "json":
        return json.dumps(
            {
                "total_requests": stats["total_requests"],
                "unique_ips": stats["unique_ips"],
                "first_timestamp": _iso(stats["first_timestamp"]),
                "last_timestamp": _iso(stats["last_timestamp"]),
                "error_rate": round(stats["error_rate"], 2),
            }
        )
    return "\n".join(
        [
            f"Total requests:    {stats['total_requests']}",
            f"Unique client IPs: {stats['unique_ips']}",
            f"First timestamp:   {_iso(stats['first_timestamp'])}",
            f"Last timestamp:    {_iso(stats['last_timestamp'])}",
            f"Error rate:        {stats['error_rate']:.2f}%",
        ]
    )


def _render_top(records: Sequence[LogRecord], by: str, n: int, fmt: str) -> str:
    rows = analyze.top(records, by, n)
    if fmt == "json":
        return json.dumps(
            {"by": by, "results": [{"value": str(value), "count": count} for value, count in rows]}
        )
    if not rows:
        return "(no data)"
    width = max(len(str(count)) for _, count in rows)
    return "\n".join(f"{count:>{width}}  {value}" for value, count in rows)


def _render_errors(
    records: Sequence[LogRecord],
    since: datetime | None,
    until: datetime | None,
    fmt: str,
) -> str:
    rows = analyze.errors(records, since, until)
    if fmt == "json":
        return json.dumps(
            {
                "results": [
                    {"status": status, "path": path, "count": count}
                    for (status, path), count in rows
                ]
            }
        )
    if not rows:
        return "(no errors)"
    width = max(len(str(count)) for _, count in rows)
    return "\n".join(
        f"{count:>{width}}  {status}  {path}" for (status, path), count in rows
    )


def _render_hourly(records: Sequence[LogRecord], fmt: str) -> str:
    buckets = analyze.hourly(records)
    if fmt == "json":
        return json.dumps({"hourly": {f"{hour:02d}": count for hour, count in enumerate(buckets)}})
    peak = max(buckets) if buckets else 0
    max_bar = 40
    lines = []
    for hour, count in enumerate(buckets):
        bar_len = round(count / peak * max_bar) if peak else 0
        lines.append(f"{hour:02d}  {'#' * bar_len:<{max_bar}} {count}")
    return "\n".join(lines)


def main(argv: Sequence[str] | None = None) -> int:
    """Entry point. Returns a process exit code."""
    parser = _build_parser()
    args = parser.parse_args(argv)
    fmt = getattr(args, "format", "text")

    try:
        records, malformed = _load_records(args.logfile)
    except OSError as exc:
        print(f"loglens: cannot read {args.logfile!r}: {exc.strerror or exc}", file=sys.stderr)
        return 2

    if malformed:
        plural = "s" if malformed != 1 else ""
        print(f"loglens: skipped {malformed} malformed line{plural}", file=sys.stderr)

    if not records:
        print(f"loglens: no valid log lines in {args.logfile!r}", file=sys.stderr)
        return 1

    if args.command == "summary":
        output = _render_summary(records, fmt)
    elif args.command == "top":
        output = _render_top(records, args.by, args.n, fmt)
    elif args.command == "errors":
        since = _parse_iso(args.since) if args.since else None
        until = _parse_iso(args.until) if args.until else None
        output = _render_errors(records, since, until, fmt)
    elif args.command == "hourly":
        output = _render_hourly(records, fmt)
    else:  # pragma: no cover - argparse enforces a valid command
        parser.error(f"unknown command {args.command!r}")

    print(output)
    return 0


if __name__ == "__main__":  # pragma: no cover
    raise SystemExit(main())
