"""Parse Combined Log Format (CLF) web server access logs."""

import re
from datetime import datetime
from dataclasses import dataclass
from typing import Optional


@dataclass
class LogEntry:
    """Represents a single log entry from Combined Log Format."""

    ip: str
    ident: str
    user: Optional[str]
    timestamp: datetime
    method: str
    path: str
    protocol: str
    status: int
    bytes_sent: int
    referrer: Optional[str]
    user_agent: Optional[str]


# Combined Log Format regex
# Format: remote_addr - remote_user [timestamp] "request" status bytes "referrer" "user_agent"
CLF_PATTERN = re.compile(
    r'(?P<ip>[^\s]+)'
    r'\s+-'
    r'\s+(?P<user>[^\s]+)'
    r'\s+\[(?P<timestamp>[^\]]+)\]'
    r'\s+"(?P<request>[^"]+)"'
    r'\s+(?P<status>\d+)'
    r'\s+(?P<bytes>-|\d+)'
    r'\s+"(?P<referrer>[^"]*)"'
    r'\s+"(?P<user_agent>[^"]*)"'
)

# Timestamp format: 12/Jul/2026:06:25:24 +0000
TIMESTAMP_FORMAT = "%d/%b/%Y:%H:%M:%S %z"


def parse_log_line(line: str) -> Optional[LogEntry]:
    """Parse a single log line in Combined Log Format.

    Returns None if the line is malformed.
    """
    match = CLF_PATTERN.match(line)
    if not match:
        return None

    try:
        request = match.group("request")
        parts = request.split(" ")
        if len(parts) != 3:
            return None

        method, path, protocol = parts
        timestamp_str = match.group("timestamp")
        timestamp = datetime.strptime(timestamp_str, TIMESTAMP_FORMAT)
        bytes_sent_str = match.group("bytes")
        bytes_sent = 0 if bytes_sent_str == "-" else int(bytes_sent_str)

        user = match.group("user")
        if user == "-":
            user = None

        referrer = match.group("referrer")
        if referrer == "-":
            referrer = None

        user_agent = match.group("user_agent")
        if user_agent == "-":
            user_agent = None

        return LogEntry(
            ip=match.group("ip"),
            ident="-",
            user=user,
            timestamp=timestamp,
            method=method,
            path=path,
            protocol=protocol,
            status=int(match.group("status")),
            bytes_sent=bytes_sent,
            referrer=referrer,
            user_agent=user_agent,
        )
    except (ValueError, AttributeError):
        return None


def parse_log_file(filepath: str) -> tuple[list[LogEntry], int]:
    """Parse a log file and return (list of entries, count of malformed lines).

    Raises FileNotFoundError if file doesn't exist or isn't readable.
    """
    entries = []
    malformed_count = 0

    try:
        with open(filepath, "r") as f:
            for line in f:
                line = line.rstrip("\n")
                if not line.strip():
                    continue

                entry = parse_log_line(line)
                if entry is None:
                    malformed_count += 1
                else:
                    entries.append(entry)
    except (FileNotFoundError, IOError, OSError) as e:
        raise FileNotFoundError(f"Cannot read file: {filepath}") from e

    return entries, malformed_count
