"""Command line interface: argument parsing, exit-code policy, stderr reporting.

Exit codes:
    0   success (including a filter that matched nothing)
    1   the file contains no valid log lines
    2   input error -- LOGFILE missing/unreadable, or a usage error
    70  internal error (never confusable with a result)
"""

from __future__ import annotations

import argparse
import contextlib
import os
import sys
from typing import Any

from loglens import __version__, analyze, render
from loglens.source import LogFileError, ParseStats, read_entries

EXIT_OK = 0
EXIT_NO_VALID_LINES = 1
EXIT_INPUT_ERROR = 2
EXIT_INTERNAL_ERROR = 70

_EPILOG = """\
exit codes:
  0  success (a filter matching nothing is still a success)
  1  the file contains no valid log lines
  2  LOGFILE missing or unreadable, or a usage error
"""


def _iso8601(value: str) -> Any:
    """argparse type for --since/--until. Fails at the door, not mid-analysis."""
    try:
        return analyze.parse_bound(value)
    except ValueError as exc:
        raise argparse.ArgumentTypeError(str(exc)) from exc


def _positive_int(value: str) -> int:
    """argparse type for -n. Rejects negatives, which would otherwise return everything."""
    try:
        n = int(value)
    except ValueError as exc:
        raise argparse.ArgumentTypeError(f"expected an integer, got {value!r}") from exc
    if n < 0:
        raise argparse.ArgumentTypeError(f"must be zero or greater, got {n}")
    return n


def build_parser() -> argparse.ArgumentParser:
    """Build the argument parser."""
    # A shared parent carries --format onto both the top-level parser and every
    # subparser, so `loglens --format json summary f.log` and
    # `loglens summary --format json f.log` both work. Declaring it only at the top
    # level makes the second form -- the one most people type, by muscle memory from
    # `kubectl -o json` and `gh --json` -- fail with an error that blames the user's
    # filename. SUPPRESS keeps a subparser default from clobbering an explicit global.
    common = argparse.ArgumentParser(add_help=False)
    common.add_argument(
        "--format",
        choices=["text", "json"],
        default=argparse.SUPPRESS,
        help="output format (default: text)",
    )

    parser = argparse.ArgumentParser(
        prog="loglens",
        description="Analyze web server access logs in Combined Log Format.",
        parents=[common],
        epilog=_EPILOG,
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )
    parser.add_argument("--version", action="version", version=f"%(prog)s {__version__}")
    sub = parser.add_subparsers(dest="command", required=True, metavar="COMMAND")

    def add(name: str, help_text: str) -> argparse.ArgumentParser:
        p = sub.add_parser(name, help=help_text, description=help_text, parents=[common])
        p.add_argument("logfile", metavar="LOGFILE", help="path to a CLF access log")
        return p

    add("summary", "Total requests, unique IPs, time range, and error rate.")

    p_top = add("top", "Top values by request count.")
    p_top.add_argument(
        "--by",
        choices=list(analyze.TOP_DIMENSIONS),
        required=True,
        help="dimension to rank",
    )
    p_top.add_argument(
        "-n",
        type=_positive_int,
        default=10,
        metavar="N",
        help="number of values to show (default: 10)",
    )

    p_err = add("errors", "4xx/5xx requests grouped by (status, path).")
    p_err.add_argument(
        "--since",
        type=_iso8601,
        metavar="ISO8601",
        help="only requests at or after this time (naive values are read as UTC)",
    )
    p_err.add_argument(
        "--until",
        type=_iso8601,
        metavar="ISO8601",
        help="only requests strictly before this time (naive values are read as UTC)",
    )

    add("hourly", "Request count per hour of day, as a histogram.")

    return parser


def _report_malformed(stats: ParseStats) -> None:
    """Report the malformed-line count on stderr. Never stdout."""
    if stats.malformed == 0:
        return
    message = f"loglens: skipped {stats.malformed} of {stats.total_lines} malformed lines"
    if stats.first_error is not None:
        err = stats.first_error
        message += f" (first at line {err.lineno}: {err.reason}: {err.excerpt})"
    print(message, file=sys.stderr)


def _dispatch(args: argparse.Namespace, entries: list[Any]) -> dict[str, Any]:
    match args.command:
        case "summary":
            return analyze.summary(entries)
        case "top":
            return analyze.top(entries, by=args.by, n=args.n)
        case "errors":
            return analyze.errors(analyze.filter_entries(entries, args.since, args.until))
        case "hourly":
            return analyze.hourly(entries)
    raise AssertionError(f"unhandled command: {args.command}")  # pragma: no cover


def run(argv: list[str] | None = None) -> int:
    """Run the CLI. Returns an exit code."""
    parser = build_parser()
    args = parser.parse_args(argv)
    fmt = getattr(args, "format", "text")

    if args.command == "errors" and args.since and args.until and args.since >= args.until:
        parser.error("--since must be earlier than --until")

    try:
        entries, stats = read_entries(args.logfile)
    except LogFileError as exc:
        print(f"loglens: {exc}", file=sys.stderr)
        return EXIT_INPUT_ERROR

    _report_malformed(stats)

    # Exit 1 is a verdict on parsing, computed before any filtering. A valid 10k-line
    # file whose --since window matches nothing is a successful answer, not an error.
    if stats.valid == 0:
        detail = (
            f"loglens: no valid log lines in {args.logfile} ({stats.total_lines} lines read)"
            if stats.total_lines
            else f"loglens: {args.logfile} is empty"
        )
        print(detail, file=sys.stderr)
        # stdout stays empty, so the JSON invariant is "on exit 0, stdout is a single
        # valid JSON document". Consumers check the exit code before parsing.
        return EXIT_NO_VALID_LINES

    # Rendered in full before writing, so a late failure cannot leave partial output.
    print(render.render(_dispatch(args, entries), args.command, fmt))
    return EXIT_OK


def main(argv: list[str] | None = None) -> int:
    """Console entry point with top-level error handling."""
    try:
        return run(argv)
    except BrokenPipeError:
        # `loglens top big.log | head -3` is a normal invocation, not a crash.
        devnull = os.open(os.devnull, os.O_WRONLY)
        os.dup2(devnull, sys.stdout.fileno())
        sys.exit(EXIT_OK)
    except KeyboardInterrupt:
        print("loglens: interrupted", file=sys.stderr)
        return 130
    except Exception as exc:
        print(f"loglens: internal error: {exc}", file=sys.stderr)
        return EXIT_INTERNAL_ERROR
    finally:
        # Flush here so a broken pipe surfaces above rather than as an ignored
        # exception message at interpreter shutdown.
        with contextlib.suppress(BrokenPipeError, ValueError):
            sys.stdout.flush()


if __name__ == "__main__":  # pragma: no cover
    raise SystemExit(main())
