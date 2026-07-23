"""JSON output formatters."""

import json

from loglens.analyzers.errors import ErrorsAnalyzer
from loglens.analyzers.hourly import HourlyAnalyzer
from loglens.analyzers.summary import SummaryAnalyzer
from loglens.analyzers.top import TopAnalyzer


def format_summary_json(analyzer: SummaryAnalyzer) -> str:
    """Format summary as JSON."""
    return json.dumps(analyzer.to_dict(), indent=2)


def format_top_json(analyzer: TopAnalyzer, n: int = 10) -> str:
    """Format top items as JSON."""
    items = analyzer.get_top(n)
    data = {
        "by": analyzer.by,
        "items": [{"value": k, "count": v} for k, v in items],
    }
    return json.dumps(data, indent=2)


def format_errors_json(analyzer: ErrorsAnalyzer) -> str:
    """Format errors as JSON."""
    return json.dumps(analyzer.to_dict(), indent=2)


def format_hourly_json(analyzer: HourlyAnalyzer) -> str:
    """Format hourly distribution as JSON."""
    return json.dumps(analyzer.to_dict(), indent=2)
