"""Reader tests: file errors, stats accounting, and encoding."""

from __future__ import annotations

import os
from pathlib import Path

import pytest

from loglens.source import LogFileError, read_entries


class TestFileErrors:
    def test_missing_file(self, tmp_path: Path) -> None:
        with pytest.raises(LogFileError):
            read_entries(tmp_path / "nope.log")

    def test_directory(self, tmp_path: Path) -> None:
        with pytest.raises(LogFileError, match="directory"):
            read_entries(tmp_path)

    @pytest.mark.skipif(os.geteuid() == 0, reason="root can read mode-000 files")
    def test_permission_denied(self, tmp_path: Path) -> None:
        path = tmp_path / "secret.log"
        path.write_text("x\n")
        path.chmod(0o000)
        try:
            with pytest.raises(LogFileError):
                read_entries(path)
        finally:
            path.chmod(0o644)


class TestStats:
    def test_counts_add_up(self, sample_log: str) -> None:
        entries, stats = read_entries(sample_log)
        assert stats.valid == len(entries)
        assert stats.valid + stats.malformed == stats.total_lines

    def test_sample_has_the_two_malformed_lines_the_spec_requires(self, sample_log: str) -> None:
        _, stats = read_entries(sample_log)
        assert stats.malformed == 2

    def test_first_error_is_recorded_with_a_line_number(self, sample_log: str) -> None:
        _, stats = read_entries(sample_log)
        assert stats.first_error is not None
        assert stats.first_error.lineno == 10
        assert stats.first_error.reason

    def test_empty_file(self, empty_log: str) -> None:
        entries, stats = read_entries(empty_log)
        assert entries == []
        assert stats.valid == 0
        assert stats.total_lines == 0

    def test_blank_lines_are_not_counted_as_malformed(self, tmp_path: Path) -> None:
        path = tmp_path / "gappy.log"
        path.write_text(
            '203.0.113.7 - - [12/Jul/2026:06:25:24 +0000] "GET /a HTTP/1.1" 200 1 "-" "-"\n\n   \n'
        )
        entries, stats = read_entries(path)
        assert len(entries) == 1
        assert stats.malformed == 0


class TestEncoding:
    def test_invalid_utf8_is_replaced_not_raised(self, hostile_log: str) -> None:
        """Bare open() would use the locale's encoding and raise mid-iteration."""
        entries, _ = read_entries(hostile_log)
        assert any(e.ip == "203.0.113.25" for e in entries)

    def test_crlf_line_parses(self, hostile_log: str) -> None:
        entries, _ = read_entries(hostile_log)
        assert any(e.ip == "203.0.113.26" for e in entries)

    def test_truncated_final_line_is_malformed_not_a_crash(self, hostile_log: str) -> None:
        """The log-rotation race: the writer was mid-line when we read."""
        _, stats = read_entries(hostile_log)
        assert stats.malformed >= 1
