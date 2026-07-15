"""Shared fixtures."""

from __future__ import annotations

from pathlib import Path

import pytest

FIXTURES = Path(__file__).parent / "fixtures"


@pytest.fixture
def sample_log() -> str:
    """The spec fixture: 30+ lines, multiple IPs/paths/statuses, several hours."""
    return str(FIXTURES / "sample.log")


@pytest.fixture
def hostile_log() -> str:
    """Lines that are valid CLF but hostile: escapes, binary, mixed offsets, bad UTF-8."""
    return str(FIXTURES / "hostile.log")


@pytest.fixture
def empty_log(tmp_path: Path) -> str:
    return str(_write(tmp_path / "empty.log", ""))


@pytest.fixture
def garbage_log(tmp_path: Path) -> str:
    """A file with lines, none of which are CLF."""
    return str(_write(tmp_path / "garbage.log", "not a log\nstill not a log\n"))


def _write(path: Path, text: str) -> Path:
    path.write_text(text, encoding="utf-8")
    return path
