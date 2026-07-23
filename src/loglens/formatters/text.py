"""Text output formatters."""

from loglens.analyzers.errors import ErrorsAnalyzer
from loglens.analyzers.hourly import HourlyAnalyzer
from loglens.analyzers.summary import SummaryAnalyzer
from loglens.analyzers.top import TopAnalyzer


def format_summary_text(analyzer: SummaryAnalyzer) -> str:
    """Format summary as human-readable text."""
    data = analyzer.to_dict()
    lines = [
        f"Total Requests: {data['total_requests']}",
        f"Unique IPs: {data['unique_ips']}",
        f"First Timestamp: {data['first_timestamp']}",
        f"Last Timestamp: {data['last_timestamp']}",
        f"Error Rate: {data['error_rate_percent']}%",
    ]
    return "\n".join(lines)


def format_top_text(analyzer: TopAnalyzer, n: int = 10) -> str:
    """Format top items as human-readable text."""
    items = analyzer.get_top(n)
    if not items:
        return "No data found"

    lines = [f"Top {analyzer.by.upper()}:"]
    for i, (value, count) in enumerate(items, 1):
        lines.append(f"{i:2d}. {value:40s} {count}")

    return "\n".join(lines)


def format_errors_text(analyzer: ErrorsAnalyzer) -> str:
    """Format errors as human-readable text."""
    items = analyzer.get_sorted()
    if not items:
        return "No errors found"

    lines = ["STATUS  PATH                                 COUNT"]
    lines.append("-" * 50)
    for (status, path), count in items:
        lines.append(f"{status}     {path:37s} {count}")

    return "\n".join(lines)


def format_hourly_text(analyzer: HourlyAnalyzer) -> str:
    """Format hourly distribution as histogram."""
    return analyzer.to_histogram()
