from pathlib import Path

import pytest

FIXTURES = Path(__file__).parent / "fixtures"


@pytest.fixture
def sample_log() -> str:
    """Path to the checked-in sample access log (32 valid lines, 2 malformed)."""
    return str(FIXTURES / "sample.log")


@pytest.fixture
def sample_entries():
    from loglens.parser import iter_file, parse_lines

    return parse_lines(iter_file(str(FIXTURES / "sample.log"))).entries
