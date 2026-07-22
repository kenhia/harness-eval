"""Click CLI for loglens."""

import json
import sys
from datetime import datetime

import click

from .parser import parse_log_file
from .commands import (
    summary,
    errors as errors_cmd,
    hourly as hourly_cmd,
    top as top_cmd,
    format_text_summary,
    format_text_top,
    format_text_errors,
    format_text_hourly,
)


@click.group()
@click.option(
    "--format",
    type=click.Choice(["text", "json"]),
    default="text",
    help="Output format",
)
@click.pass_context
def cli(ctx, format):
    """loglens - Web access log analyzer."""
    ctx.ensure_object(dict)
    ctx.obj["format"] = format


@cli.command()
@click.argument("logfile", type=click.Path(exists=False))
@click.pass_context
def summary_cmd(ctx, logfile):
    """Analyze log summary: requests, IPs, timestamps, error rate."""
    try:
        entries, malformed = parse_log_file(logfile)
    except FileNotFoundError:
        click.echo(f"Error: File not found or unreadable: {logfile}", err=True)
        sys.exit(2)

    if malformed > 0:
        click.echo(f"Skipped {malformed} malformed lines", err=True)

    if not entries:
        click.echo("Error: No valid log lines found", err=True)
        sys.exit(1)

    data = summary(entries)
    output = json.dumps(data) if ctx.obj["format"] == "json" else format_text_summary(data)
    click.echo(output)


@cli.command()
@click.argument("logfile", type=click.Path(exists=False))
@click.option("--by", type=click.Choice(["ip", "path", "status"]), default="ip", help="Group by")
@click.option("-n", "--count", type=int, default=10, help="Number of results")
@click.pass_context
def top(ctx, logfile, by, count):
    """Show top N values by request count."""
    try:
        entries, malformed = parse_log_file(logfile)
    except FileNotFoundError:
        click.echo(f"Error: File not found or unreadable: {logfile}", err=True)
        sys.exit(2)

    if malformed > 0:
        click.echo(f"Skipped {malformed} malformed lines", err=True)

    if not entries:
        click.echo("Error: No valid log lines found", err=True)
        sys.exit(1)

    data = top_cmd(entries, by=by, n=count)
    output = json.dumps(data) if ctx.obj["format"] == "json" else format_text_top(data, by)
    click.echo(output)


@cli.command()
@click.argument("logfile", type=click.Path(exists=False))
@click.option("--since", type=str, default=None, help="ISO8601 start date")
@click.option("--until", type=str, default=None, help="ISO8601 end date")
@click.pass_context
def errors(ctx, logfile, since, until):
    """Show 4xx/5xx errors grouped by status and path."""
    try:
        entries, malformed = parse_log_file(logfile)
    except FileNotFoundError:
        click.echo(f"Error: File not found or unreadable: {logfile}", err=True)
        sys.exit(2)

    if malformed > 0:
        click.echo(f"Skipped {malformed} malformed lines", err=True)

    if not entries:
        click.echo("Error: No valid log lines found", err=True)
        sys.exit(1)

    since_dt = None
    until_dt = None

    try:
        if since:
            since_dt = datetime.fromisoformat(since)
        if until:
            until_dt = datetime.fromisoformat(until)
    except ValueError as e:
        click.echo(f"Error: Invalid ISO8601 date format: {e}", err=True)
        sys.exit(1)

    data = errors_cmd(entries, since=since_dt, until=until_dt)
    output = json.dumps(data) if ctx.obj["format"] == "json" else format_text_errors(data)
    click.echo(output)


@cli.command()
@click.argument("logfile", type=click.Path(exists=False))
@click.pass_context
def hourly(ctx, logfile):
    """Show request count per hour of day as histogram."""
    try:
        entries, malformed = parse_log_file(logfile)
    except FileNotFoundError:
        click.echo(f"Error: File not found or unreadable: {logfile}", err=True)
        sys.exit(2)

    if malformed > 0:
        click.echo(f"Skipped {malformed} malformed lines", err=True)

    if not entries:
        click.echo("Error: No valid log lines found", err=True)
        sys.exit(1)

    data = hourly_cmd(entries)
    output = json.dumps(data) if ctx.obj["format"] == "json" else format_text_hourly(data)
    click.echo(output)


def main():
    """Main entry point."""
    cli(obj={})


if __name__ == "__main__":
    main()
