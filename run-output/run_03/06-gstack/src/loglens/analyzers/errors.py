"""Errors analyzer: group 4xx/5xx by status and path."""

from collections import defaultdict
from datetime import datetime
from typing import Optional

from loglens.parser import LogEntry


class ErrorsAnalyzer:
    """Analyze access logs for error requests (4xx, 5xx)."""

    def __init__(self, since: Optional[datetime] = None, until: Optional[datetime] = None):
        self.since = since
        self.until = until
        self.errors: dict[tuple[int, str], int] = defaultdict(int)

    def process(self, entry: LogEntry) -> None:
        """Add an entry if it's an error within the date range."""
        if not entry.is_error:
            return

        if self.since and entry.timestamp < self.since:
            return
        if self.until and entry.timestamp > self.until:
            return

        key = (entry.status, entry.path)
        self.errors[key] += 1

    def get_sorted(self) -> list[tuple[tuple[int, str], int]]:
        """Return errors sorted by count descending."""
        return sorted(self.errors.items(), key=lambda x: (-x[1], x[0][0], x[0][1]))

    def to_dict(self) -> dict:
        """Return errors as dictionary."""
        return {
            "errors": [
                {"status": status, "path": path, "count": count}
                for (status, path), count in self.get_sorted()
            ]
        }
