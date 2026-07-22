"""CLF log format parser"""

import re
from datetime import datetime
from typing import Optional

# CLF log format regex
# IP - user [timestamp] "request" status bytes "referrer" "user-agent"
CLF_REGEX = re.compile(
    r'(?P<ip>\S+)\s'
    r'(?P<ident>\S+)\s'
    r'(?P<user>\S+)\s'
    r'\[(?P<timestamp>[^\]]+)\]\s'
    r'"(?P<request>[^"]*)"\s'
    r'(?P<status>\d+)\s'
    r'(?P<bytes>\S+)\s'
    r'"(?P<referrer>[^"]*)"\s'
    r'"(?P<user_agent>[^"]*)"'
)


def parse_line(line: str) -> Optional[dict]:
    """Parse a single CLF log line.

    Returns a dict with parsed fields, or None if the line is malformed.
    """
    match = CLF_REGEX.match(line.strip())
    if not match:
        return None

    groups = match.groupdict()
    try:
        # Parse timestamp: 12/Jul/2026:06:25:24 +0000
        timestamp_str = groups['timestamp']
        dt = datetime.strptime(timestamp_str[:20], '%d/%b/%Y:%H:%M:%S')
        hour = dt.hour

        return {
            'ip': groups['ip'],
            'user': groups['user'] if groups['user'] != '-' else None,
            'timestamp': dt,
            'hour': hour,
            'request': groups['request'],
            'status': int(groups['status']),
            'bytes': int(groups['bytes']) if groups['bytes'] != '-' else 0,
            'referrer': groups['referrer'] if groups['referrer'] != '-' else None,
            'user_agent': groups['user_agent'],
        }
    except (ValueError, KeyError):
        return None


def parse_log_file(filepath: str) -> tuple[list[dict], int]:
    """Parse a log file and return (parsed_lines, malformed_count)."""
    lines = []
    malformed = 0

    try:
        with open(filepath, 'r') as f:
            for line in f:
                parsed = parse_line(line)
                if parsed:
                    lines.append(parsed)
                else:
                    malformed += 1
    except FileNotFoundError:
        return [], -1  # Signal file not found with -1

    return lines, malformed
