"""Command-line interface for loglens.

Exit codes:
    0  success
    1  the file contains no valid log lines
    2  LOGFILE is missing or unreadable
"""

from __future__ import annotations

import argparse
import json
import sys
from collections.abc import Sequence
from datetime import datetime

from . import __version__
from .analysis import Summary, errors, hourly, summarize, top
from .parser import LogRecord, parse_lines

EXIT_OK = 0
EXIT_NO_VALID_LINES = 1
EXIT_FILE_ERROR = 2


def _iso(ts: datetime | None) -> str | None:
    return ts.isoformat() if ts is not None else None


def _parse_iso(value: str, option: str) -> datetime:
    try:
        return datetime.fromisoformat(value)
    except ValueError:
        raise SystemExit(f"loglens: invalid ISO 8601 value for {option}: {value!r}") from None


def _read_records(path: str) -> tuple[list[LogRecord], int]:
    """Read and parse the log file.

    Exits with code 2 if the file is missing or unreadable.
    """
    try:
        with open(path, encoding="utf-8", errors="replace") as fh:
            lines = fh.read().splitlines()
    except (FileNotFoundError, IsADirectoryError, PermissionError, OSError) as exc:
        print(f"loglens: cannot read {path}: {exc}", file=sys.stderr)
        raise SystemExit(EXIT_FILE_ERROR) from None
    return parse_lines(lines)


# --------------------------------------------------------------------------- #
# Rendering
# --------------------------------------------------------------------------- #


def _emit(payload: dict, text: str, fmt: str, out) -> None:
    if fmt == "json":
        json.dump(payload, out)
        out.write("\n")
    else:
        out.write(text)
        if not text.endswith("\n"):
            out.write("\n")


def _render_summary(s: Summary, fmt: str, out) -> None:
    payload = {
        "total_requests": s.total_requests,
        "unique_ips": s.unique_ips,
        "first_timestamp": _iso(s.first_timestamp),
        "last_timestamp": _iso(s.last_timestamp),
        "error_rate": round(s.error_rate, 2),
    }
    lines = [
        f"Total requests:  {s.total_requests}",
        f"Unique IPs:      {s.unique_ips}",
        f"First timestamp: {_iso(s.first_timestamp) or '-'}",
        f"Last timestamp:  {_iso(s.last_timestamp) or '-'}",
        f"Error rate:      {s.error_rate:.2f}%",
    ]
    _emit(payload, "\n".join(lines), fmt, out)


def _render_top(rows: list[tuple[str, int]], by: str, fmt: str, out) -> None:
    payload = {"by": by, "results": [{"value": v, "count": c} for v, c in rows]}
    if rows:
        width = max(len(v) for v, _ in rows)
        text = "\n".join(f"{v:<{width}}  {c}" for v, c in rows)
    else:
        text = "(no data)"
    _emit(payload, text, fmt, out)


def _render_errors(rows: list[tuple[int, str, int]], fmt: str, out) -> None:
    payload = {
        "results": [{"status": s, "path": p, "count": c} for s, p, c in rows]
    }
    if rows:
        text = "\n".join(f"{s} {p}  {c}" for s, p, c in rows)
    else:
        text = "(no errors)"
    _emit(payload, text, fmt, out)


def _render_hourly(buckets: list[int], fmt: str, out) -> None:
    payload = {f"{h:02d}": buckets[h] for h in range(24)}
    peak = max(buckets) if buckets else 0
    bar_width = 40
    lines = []
    for h in range(24):
        count = buckets[h]
        bar_len = round(count / peak * bar_width) if peak else 0
        bar = "#" * bar_len
        lines.append(f"{h:02d} {bar:<{bar_width}} {count}")
    _emit(payload, "\n".join(lines), fmt, out)


# --------------------------------------------------------------------------- #
# Subcommand handlers
# --------------------------------------------------------------------------- #


def _cmd_summary(args, records, out) -> int:
    _render_summary(summarize(records), args.format, out)
    return EXIT_OK


def _cmd_top(args, records, out) -> int:
    rows = top(records, by=args.by, n=args.number)
    _render_top(rows, args.by, args.format, out)
    return EXIT_OK


def _cmd_errors(args, records, out) -> int:
    since = _parse_iso(args.since, "--since") if args.since else None
    until = _parse_iso(args.until, "--until") if args.until else None
    rows = errors(records, since=since, until=until)
    _render_errors(rows, args.format, out)
    return EXIT_OK


def _cmd_hourly(args, records, out) -> int:
    _render_hourly(hourly(records), args.format, out)
    return EXIT_OK


_HANDLERS = {
    "summary": _cmd_summary,
    "top": _cmd_top,
    "errors": _cmd_errors,
    "hourly": _cmd_hourly,
}


# --------------------------------------------------------------------------- #
# Argument parsing
# --------------------------------------------------------------------------- #


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        prog="loglens",
        description="Analyze web server access logs in Combined Log Format (CLF).",
    )
    parser.add_argument("--version", action="version", version=f"loglens {__version__}")
    parser.add_argument(
        "--format",
        choices=["text", "json"],
        default="text",
        help="output format (default: text)",
    )
    sub = parser.add_subparsers(dest="command", required=True, metavar="COMMAND")

    p_summary = sub.add_parser("summary", help="overall request summary")
    p_summary.add_argument("logfile", metavar="LOGFILE")

    p_top = sub.add_parser("top", help="top values by request count")
    p_top.add_argument("logfile", metavar="LOGFILE")
    p_top.add_argument(
        "--by", choices=["ip", "path", "status"], required=True, help="dimension to rank"
    )
    p_top.add_argument(
        "-n", "--number", type=int, default=10, help="number of rows (default: 10)"
    )

    p_errors = sub.add_parser("errors", help="4xx/5xx requests grouped by (status, path)")
    p_errors.add_argument("logfile", metavar="LOGFILE")
    p_errors.add_argument("--since", help="only include entries at/after this ISO 8601 time")
    p_errors.add_argument("--until", help="only include entries at/before this ISO 8601 time")

    p_hourly = sub.add_parser("hourly", help="request count per hour of day as a histogram")
    p_hourly.add_argument("logfile", metavar="LOGFILE")

    return parser


def main(argv: Sequence[str] | None = None) -> int:
    parser = build_parser()
    args = parser.parse_args(argv)

    records, malformed = _read_records(args.logfile)

    if malformed:
        noun = "line" if malformed == 1 else "lines"
        print(f"loglens: skipped {malformed} malformed {noun}", file=sys.stderr)

    if not records:
        print(f"loglens: no valid log lines found in {args.logfile}", file=sys.stderr)
        return EXIT_NO_VALID_LINES

    handler = _HANDLERS[args.command]
    return handler(args, records, sys.stdout)


if __name__ == "__main__":  # pragma: no cover
    raise SystemExit(main())
