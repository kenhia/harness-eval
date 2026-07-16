"""Read a log file into entries, owning encoding, line numbering, and parse stats.

Separated from `cli.py` so the file-error paths are unit-testable without going through
argparse, and so `parse_line` stays a pure per-line function with no shared counter to
mutate.
"""

from __future__ import annotations

from collections.abc import Iterator
from dataclasses import dataclass, field
from pathlib import Path

from loglens.parse import LogEntry, ParseError, parse_line


class LogFileError(Exception):
    """LOGFILE is missing, unreadable, or not a regular file. Maps to exit code 2."""


@dataclass
class ParseStats:
    """Counts accumulated while reading. Owned by the reader, not the parser."""

    total_lines: int = 0
    valid: int = 0
    malformed: int = 0
    first_error: ParseError | None = field(default=None)

    def record_error(self, error: ParseError) -> None:
        self.malformed += 1
        if self.first_error is None:
            self.first_error = error


def read_entries(path: str | Path) -> tuple[list[LogEntry], ParseStats]:
    """Read and parse a log file.

    Raises LogFileError when the path cannot be read at all (missing, a directory,
    permission denied). Individual bad lines are counted in stats, never raised.
    """
    stats = ParseStats()
    entries = list(_iter_entries(path, stats))
    return entries, stats


def _iter_entries(path: str | Path, stats: ParseStats) -> Iterator[LogEntry]:
    """Stream entries, accumulating stats as a side effect on the caller's object."""
    p = Path(path)
    if p.is_dir():
        raise LogFileError(f"{p} is a directory, not a log file")
    try:
        # Encoding is pinned rather than left to the locale, and errors are replaced
        # rather than raised: real access logs carry latin-1 user agents and raw TLS
        # bytes, and a mid-stream UnicodeDecodeError would surface as a traceback after
        # output had already been written.
        handle = p.open("r", encoding="utf-8", errors="replace", newline="")
    except OSError as exc:
        raise LogFileError(f"{p}: {exc.strerror or exc}") from exc

    with handle:
        try:
            for lineno, line in enumerate(handle, start=1):
                if not line.strip():
                    continue
                stats.total_lines += 1
                result = parse_line(line, lineno)
                if isinstance(result, ParseError):
                    stats.record_error(result)
                else:
                    stats.valid += 1
                    yield result
        except OSError as exc:
            raise LogFileError(f"{p}: {exc.strerror or exc}") from exc
