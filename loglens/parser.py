from __future__ import annotations

import re
from dataclasses import dataclass
from datetime import datetime


@dataclass
class LogEntry:
    """Represents a single Combined Log Format entry."""

    ip: str
    timestamp: datetime
    method: str
    path: str
    status: int
    size: int
    referrer: str
    user_agent: str

    @property
    def is_error(self) -> bool:
        """Check if response is a 4xx or 5xx error."""
        return 400 <= self.status < 600

    @property
    def hour(self) -> int:
        """Get hour of day (0-23)."""
        return self.timestamp.hour


class LogParser:
    """Parser for Combined Log Format."""

    # Combined Log Format regex
    # Format: ip - user [timestamp] "method path protocol" status size "referrer" "user_agent"
    CLF_PATTERN = re.compile(
        r'(\S+) - (\S+) \[([^\]]+)\] "(\S+) (\S+) (\S+)" (\d+) (-|\d+) "([^"]*)" "([^"]*)"'
    )

    @staticmethod
    def parse_timestamp(timestamp_str: str) -> datetime | None:
        """Parse CLF timestamp format: 12/Jul/2026:06:25:24 +0000"""
        try:
            return datetime.strptime(timestamp_str[:20], "%d/%b/%Y:%H:%M:%S")
        except ValueError:
            return None

    @staticmethod
    def parse_line(line: str) -> LogEntry | None:
        """Parse a single log line. Returns None if malformed."""
        match = LogParser.CLF_PATTERN.match(line.strip())
        if not match:
            return None

        try:
            ip, _, timestamp_str, method, path, _, status, size, referrer, user_agent = match.groups()
            timestamp = LogParser.parse_timestamp(timestamp_str)
            if timestamp is None:
                return None

            return LogEntry(
                ip=ip,
                timestamp=timestamp,
                method=method,
                path=path,
                status=int(status),
                size=int(size) if size != "-" else 0,
                referrer=referrer if referrer != "-" else "",
                user_agent=user_agent if user_agent != "-" else "",
            )
        except (ValueError, IndexError):
            return None

    @staticmethod
    def parse_file(filepath: str) -> tuple[list[LogEntry], int]:
        """
        Parse a log file.

        Returns:
            Tuple of (valid_entries, malformed_count)
        """
        entries = []
        malformed_count = 0

        try:
            with open(filepath, encoding="utf-8") as f:
                for line in f:
                    entry = LogParser.parse_line(line)
                    if entry is None:
                        malformed_count += 1
                    else:
                        entries.append(entry)
        except OSError:
            return [], 0

        return entries, malformed_count
