"""Parse Combined Log Format (CLF) web server access logs."""

import re
from dataclasses import dataclass
from datetime import datetime


@dataclass
class LogEntry:
    """A parsed log entry."""
    ip: str
    ident: str
    user: str
    timestamp: datetime
    method: str
    path: str
    protocol: str
    status: int
    size: int
    referer: str
    user_agent: str

    @property
    def hour(self) -> int:
        """Return the hour of the day (0-23)."""
        return self.timestamp.hour


# Combined Log Format pattern
# IP - - [timestamp] "METHOD /path HTTP/version" status size "referer" "user_agent"
CLF_PATTERN = re.compile(
    r'(\S+) (\S+) (\S+) \[([^\]]+)\] "(\S+) (\S+) (\S+)" (\d+) (\d+|-) "([^"]*)" "([^"]*)"'
)


def parse_timestamp(timestamp_str: str) -> datetime:
    """Parse CLF timestamp format: 12/Jul/2026:06:25:24 +0000."""
    # Remove timezone (we'll just parse the datetime part)
    dt_part = timestamp_str.split(' ')[0]
    return datetime.strptime(dt_part, '%d/%b/%Y:%H:%M:%S')


def parse_line(line: str) -> LogEntry | None:
    """Parse a single log line. Returns None if the line is malformed."""
    match = CLF_PATTERN.match(line)
    if not match:
        return None

    (
        ip,
        ident,
        user,
        timestamp_str,
        method,
        path,
        protocol,
        status,
        size,
        referer,
        user_agent,
    ) = match.groups()

    try:
        timestamp = parse_timestamp(timestamp_str)
        size_int = int(size) if size != '-' else 0
        status_int = int(status)

        return LogEntry(
            ip=ip,
            ident=ident,
            user=user,
            timestamp=timestamp,
            method=method,
            path=path,
            protocol=protocol,
            status=status_int,
            size=size_int,
            referer=referer,
            user_agent=user_agent,
        )
    except (ValueError, AttributeError):
        return None


def load_log_file(logfile_path: str) -> tuple[list[LogEntry], int]:
    """
    Load and parse a log file.
    Returns (list of parsed entries, count of malformed lines).
    """
    entries = []
    malformed_count = 0

    try:
        with open(logfile_path, encoding='utf-8', errors='ignore') as f:
            for line in f:
                line = line.strip()
                if not line:
                    continue

                entry = parse_line(line)
                if entry:
                    entries.append(entry)
                else:
                    malformed_count += 1
    except FileNotFoundError:
        raise FileNotFoundError(f"Log file not found: {logfile_path}")
    except OSError as e:
        raise OSError(f"Cannot read log file: {logfile_path}: {e}")

    return entries, malformed_count
