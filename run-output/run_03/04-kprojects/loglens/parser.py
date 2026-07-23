"""Parse Combined Log Format (CLF) access logs."""

import re
from dataclasses import dataclass
from datetime import datetime


@dataclass
class LogEntry:
    """Represents a single log line in Combined Log Format."""

    ip: str
    remote_user: str
    timestamp: datetime
    method: str
    path: str
    status: int
    bytes_sent: int
    referrer: str
    user_agent: str

    @property
    def is_error(self) -> bool:
        """Return True if status is 4xx or 5xx."""
        return self.status >= 400


class LogParser:
    """Parser for Combined Log Format access logs."""

    # CLF regex pattern
    # Format: IP - user [timestamp] "method path protocol" status bytes "referrer" "user_agent"
    CLF_PATTERN = re.compile(
        r'(?P<ip>[^\s]+) - (?P<remote_user>[^\s]+) \[(?P<timestamp>[^\]]+)\] '
        r'"(?P<request>[^"]*)" (?P<status>\d+) (?P<bytes>[-\d]+) '
        r'"(?P<referrer>[^"]*)" "(?P<user_agent>[^"]*)"'
    )

    TIMESTAMP_FORMAT = "%d/%b/%Y:%H:%M:%S %z"

    def __init__(self) -> None:
        """Initialize the parser."""
        self.malformed_count = 0

    def parse_line(self, line: str) -> LogEntry | None:
        """Parse a single CLF log line.

        Returns None if the line is malformed.
        """
        match = self.CLF_PATTERN.match(line.strip())
        if not match:
            self.malformed_count += 1
            return None

        try:
            groups = match.groupdict()
            request_parts = groups["request"].split()
            if len(request_parts) < 3:
                self.malformed_count += 1
                return None

            method, path, _ = request_parts[0], request_parts[1], request_parts[2]
            status = int(groups["status"])
            bytes_sent = int(groups["bytes"]) if groups["bytes"] != "-" else 0

            # Parse timestamp
            timestamp_str = groups["timestamp"]
            timestamp = datetime.strptime(timestamp_str, self.TIMESTAMP_FORMAT)

            return LogEntry(
                ip=groups["ip"],
                remote_user=groups["remote_user"],
                timestamp=timestamp,
                method=method,
                path=path,
                status=status,
                bytes_sent=bytes_sent,
                referrer=groups["referrer"],
                user_agent=groups["user_agent"],
            )
        except (ValueError, IndexError):
            self.malformed_count += 1
            return None

    def parse_file(self, filepath: str) -> list[LogEntry]:
        """Parse all entries from a log file.

        Returns a list of valid LogEntry objects.
        """
        entries = []
        try:
            with open(filepath, encoding="utf-8") as f:
                for line in f:
                    entry = self.parse_line(line)
                    if entry is not None:
                        entries.append(entry)
        except (FileNotFoundError, OSError):
            pass
        return entries
