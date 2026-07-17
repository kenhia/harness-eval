from loglens.render import BAR_WIDTH, errors_text, hourly_text, top_text


def _bar(line: str) -> str:
    return line.split("  ")[-1]


def test_small_nonzero_hour_still_gets_a_bar():
    """A count far below the peak must not round down to an empty bar."""
    buckets = [0] * 24
    buckets[8] = 1000
    buckets[7] = 10  # 40 * 10/1000 rounds to 0
    lines = hourly_text(buckets).splitlines()
    assert "#" in lines[7]
    assert "#" not in lines[6]  # a genuinely empty hour stays empty


def test_peak_hour_uses_full_width():
    buckets = [0] * 24
    buckets[3] = 50
    assert _bar(hourly_text(buckets).splitlines()[3]) == "#" * BAR_WIDTH


def test_hourly_bars_are_monotonic_in_count():
    buckets = list(range(24))
    bars = [len(_bar(line)) if "#" in line else 0 for line in hourly_text(buckets).splitlines()]
    assert bars == sorted(bars)


def test_hourly_all_zero_renders_without_bars():
    """peak == 0 must not raise ZeroDivisionError."""
    out = hourly_text([0] * 24)
    assert "#" not in out
    assert len(out.splitlines()) == 24


def test_top_text_header_wider_than_values():
    out = top_text([("200", 15)], "status")
    header, row = out.splitlines()
    assert header.startswith("STATUS")
    assert row.startswith("200")


def test_top_text_empty_rows():
    assert top_text([], "ip") == "No ip values found."


def test_errors_text_empty_rows():
    assert errors_text([]) == "No error responses found."


def test_errors_text_aligns_columns():
    lines = errors_text([(404, "/missing", 4), (500, "/a", 1)]).splitlines()
    assert lines[0].startswith("STATUS  PATH")
    assert all(line == line.rstrip() for line in lines)
