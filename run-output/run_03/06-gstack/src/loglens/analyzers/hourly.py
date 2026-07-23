"""Hourly analyzer: request distribution across hours (0-23)."""

from loglens.parser import LogEntry


class HourlyAnalyzer:
    """Analyze access logs for hourly distribution."""

    def __init__(self):
        self.hourly_counts = [0] * 24

    def process(self, entry: LogEntry) -> None:
        """Add an entry to hourly counts."""
        hour = entry.hour
        if 0 <= hour < 24:
            self.hourly_counts[hour] += 1

    def to_dict(self) -> dict:
        """Return hourly counts as dictionary."""
        return {
            "hourly": [
                {"hour": hour, "count": count} for hour, count in enumerate(self.hourly_counts)
            ]
        }

    def to_histogram(self) -> str:
        """Return hourly distribution as ASCII histogram."""
        if not any(self.hourly_counts):
            return "No requests found"

        max_count = max(self.hourly_counts)
        if max_count == 0:
            max_count = 1

        lines = []
        for hour, count in enumerate(self.hourly_counts):
            bar_width = int((count / max_count) * 50)  # Scale to 50 chars
            bar = "█" * bar_width
            lines.append(f"{hour:02d}:00 │ {bar} {count}")

        return "\n".join(lines)
