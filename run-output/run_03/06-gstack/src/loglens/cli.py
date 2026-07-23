"""Click CLI for loglens."""

import sys
from datetime import datetime

import click

from loglens.analyzers.errors import ErrorsAnalyzer
from loglens.analyzers.hourly import HourlyAnalyzer
from loglens.analyzers.summary import SummaryAnalyzer
from loglens.analyzers.top import TopAnalyzer
from loglens.formatters.json import (
    format_errors_json,
    format_hourly_json,
    format_summary_json,
    format_top_json,
)
from loglens.formatters.text import (
    format_errors_text,
    format_hourly_text,
    format_summary_text,
    format_top_text,
)
from loglens.parser import parse_log_file


@click.group()
@click.option("--format", type=click.Choice(["text", "json"]), default="text")
@click.pass_context
def main(ctx, format):
    """loglens: Fast CLI tool for analyzing web server access logs."""
    ctx.ensure_object(dict)
    ctx.obj["format"] = format


@main.command()
@click.argument("logfile", type=click.Path(exists=False))
@click.pass_context
def summary(ctx, logfile):
    """Show summary statistics: total requests, unique IPs, error rate."""
    entries, malformed = parse_log_file(logfile)

    if malformed == -1:
        click.echo(f"Error: File '{logfile}' not found or unreadable", err=True)
        sys.exit(2)

    if not entries:
        click.echo("Error: No valid log lines found", err=True)
        sys.exit(1)

    fmt = ctx.obj["format"]
    if malformed > 0 and fmt == "text":
        sys.stderr.write(f"⚠ Skipped {malformed} malformed line(s)\n")
        sys.stderr.flush()

    analyzer = SummaryAnalyzer()
    for entry in entries:
        analyzer.process(entry)

    if fmt == "json":
        output = format_summary_json(analyzer)
    else:
        output = format_summary_text(analyzer)

    click.echo(output)


@main.command()
@click.argument("logfile", type=click.Path(exists=False))
@click.option("--by", type=click.Choice(["ip", "path", "status"]), default="ip")
@click.option("-n", type=int, default=10)
@click.pass_context
def top(ctx, logfile, by, n):
    """Show top N values by request count."""
    entries, malformed = parse_log_file(logfile)

    if malformed == -1:
        click.echo(f"Error: File '{logfile}' not found or unreadable", err=True)
        sys.exit(2)

    if not entries:
        click.echo("Error: No valid log lines found", err=True)
        sys.exit(1)

    fmt = ctx.obj["format"]
    if malformed > 0 and fmt == "text":
        sys.stderr.write(f"⚠ Skipped {malformed} malformed line(s)\n")
        sys.stderr.flush()

    analyzer = TopAnalyzer(by=by)
    for entry in entries:
        analyzer.process(entry)

    if fmt == "json":
        output = format_top_json(analyzer, n)
    else:
        output = format_top_text(analyzer, n)

    click.echo(output)


@main.command()
@click.argument("logfile", type=click.Path(exists=False))
@click.option("--since", type=str, default=None, help="ISO8601 timestamp")
@click.option("--until", type=str, default=None, help="ISO8601 timestamp")
@click.pass_context
def errors(ctx, logfile, since, until):
    """Show 4xx/5xx errors grouped by status and path."""
    entries, malformed = parse_log_file(logfile)

    if malformed == -1:
        click.echo(f"Error: File '{logfile}' not found or unreadable", err=True)
        sys.exit(2)

    if not entries:
        click.echo("Error: No valid log lines found", err=True)
        sys.exit(1)

    fmt = ctx.obj["format"]
    if malformed > 0 and fmt == "text":
        sys.stderr.write(f"⚠ Skipped {malformed} malformed line(s)\n")
        sys.stderr.flush()

    # Parse since/until
    since_dt = None
    until_dt = None
    try:
        if since:
            since_dt = datetime.fromisoformat(since)
        if until:
            until_dt = datetime.fromisoformat(until)
    except ValueError as e:
        click.echo(f"Error: Invalid ISO8601 timestamp: {e}", err=True)
        sys.exit(1)

    analyzer = ErrorsAnalyzer(since=since_dt, until=until_dt)
    for entry in entries:
        analyzer.process(entry)

    if fmt == "json":
        output = format_errors_json(analyzer)
    else:
        output = format_errors_text(analyzer)

    click.echo(output)


@main.command()
@click.argument("logfile", type=click.Path(exists=False))
@click.pass_context
def hourly(ctx, logfile):
    """Show request count per hour (0-23) as histogram."""
    entries, malformed = parse_log_file(logfile)

    if malformed == -1:
        click.echo(f"Error: File '{logfile}' not found or unreadable", err=True)
        sys.exit(2)

    if not entries:
        click.echo("Error: No valid log lines found", err=True)
        sys.exit(1)

    fmt = ctx.obj["format"]
    if malformed > 0 and fmt == "text":
        sys.stderr.write(f"⚠ Skipped {malformed} malformed line(s)\n")
        sys.stderr.flush()

    analyzer = HourlyAnalyzer()
    for entry in entries:
        analyzer.process(entry)

    if fmt == "json":
        output = format_hourly_json(analyzer)
    else:
        output = format_hourly_text(analyzer)

    click.echo(output)


if __name__ == "__main__":
    main(obj={})
