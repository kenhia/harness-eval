"""CLI entry point for loglens"""

import argparse
import sys

from .commands import errors, hourly, summary, top


def main():
    """Main CLI entry point"""
    parser = argparse.ArgumentParser(
        prog="loglens",
        description="Analyze web server access logs in Combined Log Format",
    )
    parser.add_argument(
        "--format",
        choices=["text", "json"],
        default="text",
        help="Output format (default: text)",
    )

    subparsers = parser.add_subparsers(dest="command", required=True)

    # summary subcommand
    summary_parser = subparsers.add_parser("summary", help="Log summary statistics")
    summary_parser.add_argument("logfile", help="Path to log file")

    # top subcommand
    top_parser = subparsers.add_parser("top", help="Top N values")
    top_parser.add_argument("logfile", help="Path to log file")
    top_parser.add_argument(
        "--by",
        choices=["ip", "path", "status"],
        default="ip",
        help="Group by (default: ip)",
    )
    top_parser.add_argument("-n", type=int, default=10, help="Top N (default: 10)")

    # errors subcommand
    errors_parser = subparsers.add_parser(
        "errors", help="4xx/5xx errors by status and path"
    )
    errors_parser.add_argument("logfile", help="Path to log file")
    errors_parser.add_argument(
        "--since", help="Since ISO8601 timestamp (optional)"
    )
    errors_parser.add_argument("--until", help="Until ISO8601 timestamp (optional)")

    # hourly subcommand
    hourly_parser = subparsers.add_parser(
        "hourly", help="Hourly request distribution"
    )
    hourly_parser.add_argument("logfile", help="Path to log file")

    args = parser.parse_args()

    if args.command == "summary":
        exit_code, output = summary(args.logfile, args.format)
    elif args.command == "top":
        exit_code, output = top(args.logfile, args.by, args.n, args.format)
    elif args.command == "errors":
        exit_code, output = errors(
            args.logfile,
            args.since,
            args.until,
            args.format,
        )
    elif args.command == "hourly":
        exit_code, output = hourly(args.logfile, args.format)
    else:
        exit_code = 2
        output = ""

    if output:
        print(output)

    sys.exit(exit_code)


if __name__ == "__main__":
    main()
