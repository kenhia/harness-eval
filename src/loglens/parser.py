"""Log parser for Combined Log Format (CLF)."""

import re
from dataclasses import dataclass
from datetime import datetime
from typing import Generator, Optional


@dataclass
class LogEntry:
    """Represents a single access log entry."""

    ip: str
    timestamp: datetime
    method: str
    path: str
    status: int
    bytes_sent: int
    referer: str
    user_agent: str

    @property
    def hour(self) -> int:
        """Extract hour (0-23) from timestamp."""
        return self.timestamp.hour

    @property
    def is_error(self) -> bool:
        """True if status is 4xx or 5xx."""
        return self.status >= 400


# Combined Log Format regex
# Format: IP ident user [timestamp] "METHOD path HTTP/version" status bytes "referer" "user-agent"
CLF_PATTERN = re.compile(
    r'(?P<ip>[\d.]+) '
    r'(?P<ident>\S+) '
    r'(?P<user>\S+) '
    r'\[(?P<timestamp>[^\]]+)\] '
    r'"(?P<method>\w+) (?P<path>\S+) \S+" '
    r'(?P<status>\d{3}) '
    r'(?P<bytes>[-\d]+) '
    r'"(?P<referer>[^"]*)" '
    r'"(?P<user_agent>[^"]*)"'
)


def parse_timestamp(ts_str: str) -> Optional[datetime]:
    """Parse CLF timestamp format: 12/Jul/2026:06:25:24 +0000."""
    try:
        return datetime.strptime(ts_str, "%d/%b/%Y:%H:%M:%S %z")
    except ValueError:
        return None


def parse_log_file(filepath: str) -> tuple[list[LogEntry], int]:
    """
    Parse a CLF log file and return (entries, malformed_count).

    Malformed lines are skipped silently; count is tracked separately.
    """
    entries = []
    malformed_count = 0

    try:
        with open(filepath, "r", encoding="utf-8", errors="replace") as f:
            for line in f:
                line = line.rstrip("\n\r")
                if not line:
                    continue

                match = CLF_PATTERN.match(line)
                if not match:
                    malformed_count += 1
                    continue

                ts = parse_timestamp(match.group("timestamp"))
                if not ts:
                    malformed_count += 1
                    continue

                try:
                    bytes_sent = int(match.group("bytes"))
                except ValueError:
                    bytes_sent = 0

                entry = LogEntry(
                    ip=match.group("ip"),
                    timestamp=ts,
                    method=match.group("method"),
                    path=match.group("path"),
                    status=int(match.group("status")),
                    bytes_sent=bytes_sent,
                    referer=match.group("referer"),
                    user_agent=match.group("user_agent"),
                )
                entries.append(entry)

    except FileNotFoundError:
        return [], -1  # Sentinel: file not found

    return entries, malformed_count


def stream_log_file(filepath: str) -> Generator[tuple[LogEntry, bool], None, None]:
    """
    Stream log entries from a file.

    Yields (entry, is_valid) tuples. Malformed lines have is_valid=False.
    """
    try:
        with open(filepath, "r", encoding="utf-8", errors="replace") as f:
            for line in f:
                line = line.rstrip("\n\r")
                if not line:
                    continue

                match = CLF_PATTERN.match(line)
                if not match:
                    yield (None, False)
                    continue

                ts = parse_timestamp(match.group("timestamp"))
                if not ts:
                    yield (None, False)
                    continue

                try:
                    bytes_sent = int(match.group("bytes"))
                except ValueError:
                    bytes_sent = 0

                entry = LogEntry(
                    ip=match.group("ip"),
                    timestamp=ts,
                    method=match.group("method"),
                    path=match.group("path"),
                    status=int(match.group("status")),
                    bytes_sent=bytes_sent,
                    referer=match.group("referer"),
                    user_agent=match.group("user_agent"),
                )
                yield (entry, True)

    except FileNotFoundError:
        yield (None, False)  # File not found
