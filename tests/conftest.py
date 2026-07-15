"""Shared pytest fixtures."""

from __future__ import annotations

from pathlib import Path

import pytest

SAMPLE_LOG = Path(__file__).parent / "fixtures" / "sample.log"


@pytest.fixture
def sample_log() -> Path:
    return SAMPLE_LOG
