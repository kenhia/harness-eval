"""Top analyzer: rank by frequency (IP, path, or status)."""

from collections import Counter
from typing import Literal

from loglens.parser import LogEntry


class TopAnalyzer:
    """Analyze access logs to find top N items by frequency."""

    def __init__(self, by: Literal["ip", "path", "status"] = "ip"):
        self.by = by
        self.counter: Counter[str] = Counter()

    def process(self, entry: LogEntry) -> None:
        """Add an entry to the counter."""
        if self.by == "ip":
            key = entry.ip
        elif self.by == "path":
            key = entry.path
        elif self.by == "status":
            key = str(entry.status)
        else:
            raise ValueError(f"Unknown 'by' value: {self.by}")

        self.counter[key] += 1

    def get_top(self, n: int = 10) -> list[tuple[str, int]]:
        """Return top N items by count, ties broken by value ascending."""
        # Sort by count descending, then by value ascending
        sorted_items = sorted(self.counter.items(), key=lambda x: (-x[1], x[0]))
        return sorted_items[:n]

    def to_dict(self) -> dict:
        """Return top items as dictionary."""
        return {
            "by": self.by,
            "items": [{"value": k, "count": v} for k, v in self.get_top(10)],
        }
