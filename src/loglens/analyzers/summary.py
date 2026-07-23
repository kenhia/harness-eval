"""Summary analyzer: total requests, unique IPs, date range, error rate."""

from datetime import datetime
from typing import Optional

from loglens.parser import LogEntry


class SummaryAnalyzer:
    """Analyze access logs for summary statistics."""

    def __init__(self):
        self.total_requests = 0
        self.unique_ips = set()
        self.first_timestamp: Optional[datetime] = None
        self.last_timestamp: Optional[datetime] = None
        self.error_count = 0
        self.success_count = 0

    def process(self, entry: LogEntry) -> None:
        """Add an entry to the summary."""
        self.total_requests += 1
        self.unique_ips.add(entry.ip)

        if self.first_timestamp is None or entry.timestamp < self.first_timestamp:
            self.first_timestamp = entry.timestamp
        if self.last_timestamp is None or entry.timestamp > self.last_timestamp:
            self.last_timestamp = entry.timestamp

        if entry.is_error:
            self.error_count += 1
        else:
            self.success_count += 1

    def to_dict(self) -> dict:
        """Return summary as dictionary."""
        error_rate = 0.0
        if self.total_requests > 0:
            error_rate = round((self.error_count / self.total_requests) * 100, 2)

        return {
            "total_requests": self.total_requests,
            "unique_ips": len(self.unique_ips),
            "first_timestamp": self.first_timestamp.isoformat() if self.first_timestamp else None,
            "last_timestamp": self.last_timestamp.isoformat() if self.last_timestamp else None,
            "error_rate_percent": error_rate,
        }
