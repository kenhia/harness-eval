"""Analysis tests: tie-breaks, timezone semantics, and filter boundaries."""

from __future__ import annotations

from datetime import UTC, datetime, timedelta, timezone

import pytest

from loglens import analyze
from loglens.parse import LogEntry, parse_line
from loglens.source import read_entries


def entry(
    ip: str = "10.0.0.1",
    path: str = "/a",
    status: int = 200,
    hour: int = 6,
    offset: int = 0,
) -> LogEntry:
    """Build a LogEntry directly, so analysis tests don't depend on parser behavior."""
    tz = timezone(timedelta(hours=offset))
    return LogEntry(
        ip=ip,
        ident=None,
        authuser=None,
        time=datetime(2026, 7, 12, hour, 0, 0, tzinfo=tz),
        method="GET",
        path=path,
        protocol="HTTP/1.1",
        status=status,
        size=100,
        referer=None,
        user_agent=None,
    )


class TestSummary:
    def test_sample(self, sample_log: str) -> None:
        entries, stats = read_entries(sample_log)
        result = analyze.summary(entries)
        assert result["total_requests"] == stats.valid
        assert result["unique_ips"] == 4
        assert result["first"] == "2026-07-12T06:25:24+00:00"
        assert result["last"] == "2026-07-12T14:44:29+00:00"
        assert 0 < result["error_rate"] < 100

    def test_error_rate_is_a_percentage_float(self) -> None:
        entries = [entry(status=200), entry(status=404), entry(status=500), entry(status=200)]
        result = analyze.summary(entries)
        assert result["error_rate"] == 50.0
        assert isinstance(result["error_rate"], float)

    def test_error_rate_denominator_is_valid_lines(self) -> None:
        entries = [entry(status=200), entry(status=404), entry(status=301)]
        assert analyze.summary(entries)["error_rate"] == pytest.approx(33.33)

    def test_first_last_are_min_max_not_file_order(self) -> None:
        """Servers log request start but write at completion: lines are not sorted."""
        entries = [entry(hour=10), entry(hour=6), entry(hour=8)]
        result = analyze.summary(entries)
        assert result["first"].endswith("06:00:00+00:00")
        assert result["last"].endswith("10:00:00+00:00")

    def test_empty(self) -> None:
        result = analyze.summary([])
        assert result == {
            "total_requests": 0,
            "unique_ips": 0,
            "first": None,
            "last": None,
            "error_rate": 0.0,
        }


class TestTop:
    def test_orders_by_count_descending(self, sample_log: str) -> None:
        entries, _ = read_entries(sample_log)
        items = analyze.top(entries, by="path")["items"]
        counts = [i["count"] for i in items]
        assert counts == sorted(counts, reverse=True)
        # 9 includes the `?page=2` request, folded in by query-stripping.
        assert (items[0]["value"], items[0]["count"]) == ("/api/orders", 9)
        assert (items[1]["value"], items[1]["count"]) == ("/index.html", 8)

    def test_ties_break_by_value_ascending(self) -> None:
        """Counter.most_common breaks ties by *insertion order* -- a spec violation.

        The values below arrive in reverse of the expected output, so an
        insertion-ordered implementation fails here and only here.
        """
        entries = [entry(path=p) for p in ("/zebra", "/mango", "/apple")]
        items = analyze.top(entries, by="path")["items"]
        assert [i["count"] for i in items] == [1, 1, 1]
        assert [i["value"] for i in items] == ["/apple", "/mango", "/zebra"]

    def test_ties_break_by_value_with_mixed_counts(self) -> None:
        entries = [entry(path="/zebra")] * 2 + [entry(path="/beta"), entry(path="/alpha")]
        items = analyze.top(entries, by="path")["items"]
        assert [(i["value"], i["count"]) for i in items] == [
            ("/zebra", 2),
            ("/alpha", 1),
            ("/beta", 1),
        ]

    def test_status_ties_sort_numerically_not_lexically(self) -> None:
        """As strings, "1000" < "404". Keeping status an int keeps the order sane."""
        entries = [entry(status=s) for s in (500, 404, 200)]
        items = analyze.top(entries, by="status")["items"]
        assert [i["value"] for i in items] == [200, 404, 500]
        assert all(isinstance(i["value"], int) for i in items)

    def test_ip_ties_break_lexicographically(self) -> None:
        entries = [entry(ip=ip) for ip in ("9.9.9.9", "10.0.0.2")]
        items = analyze.top(entries, by="ip")["items"]
        assert [i["value"] for i in items] == ["10.0.0.2", "9.9.9.9"]

    def test_n_limits(self, sample_log: str) -> None:
        entries, _ = read_entries(sample_log)
        assert len(analyze.top(entries, by="path", n=2)["items"]) == 2

    def test_n_zero_yields_nothing(self, sample_log: str) -> None:
        entries, _ = read_entries(sample_log)
        assert analyze.top(entries, by="path", n=0)["items"] == []

    def test_n_larger_than_cardinality(self, sample_log: str) -> None:
        entries, _ = read_entries(sample_log)
        result = analyze.top(entries, by="ip", n=1000)
        assert len(result["items"]) == 4

    def test_unknown_dimension(self) -> None:
        with pytest.raises(ValueError, match="unknown dimension"):
            analyze.top([], by="frobnicate")


class TestErrors:
    def test_groups_only_4xx_5xx(self, sample_log: str) -> None:
        entries, _ = read_entries(sample_log)
        result = analyze.errors(entries)
        assert all(g["status"] >= 400 for g in result["groups"])
        assert result["total_errors"] == sum(g["count"] for g in result["groups"])

    def test_most_frequent_first(self, sample_log: str) -> None:
        entries, _ = read_entries(sample_log)
        counts = [g["count"] for g in analyze.errors(entries)["groups"]]
        assert counts == sorted(counts, reverse=True)

    def test_groups_by_status_and_path(self) -> None:
        entries = [entry(path="/x", status=404), entry(path="/x", status=500)]
        groups = analyze.errors(entries)["groups"]
        assert len(groups) == 2, "same path, different status must not merge"

    def test_tie_break_is_status_then_path(self) -> None:
        entries = [entry(path="/z", status=500), entry(path="/a", status=404)]
        groups = analyze.errors(entries)["groups"]
        assert [(g["status"], g["path"]) for g in groups] == [(404, "/a"), (500, "/z")]

    def test_no_errors(self) -> None:
        result = analyze.errors([entry(status=200)])
        assert result == {"total_errors": 0, "groups": []}


class TestHourly:
    def test_all_24_hours_present(self, sample_log: str) -> None:
        entries, _ = read_entries(sample_log)
        result = analyze.hourly(entries)
        assert [h["hour"] for h in result["hours"]] == list(range(24))
        assert result["total_requests"] == len(entries)

    def test_counts_match_total(self, sample_log: str) -> None:
        entries, _ = read_entries(sample_log)
        result = analyze.hourly(entries)
        assert sum(h["count"] for h in result["hours"]) == len(entries)

    def test_buckets_by_wall_clock_hour_in_the_logs_own_offset(self) -> None:
        """Same instant, two offsets -> two different buckets, as documented."""
        utc = parse_line('1.1.1.1 - - [12/Jul/2026:06:00:00 +0000] "GET /a HTTP/1.1" 200 1 "-" "-"')
        pacific = parse_line(
            '1.1.1.2 - - [11/Jul/2026:23:00:00 -0700] "GET /a HTTP/1.1" 200 1 "-" "-"'
        )
        assert isinstance(utc, LogEntry) and isinstance(pacific, LogEntry)
        assert utc.time == pacific.time, "these are the same instant"
        hours = {h["hour"]: h["count"] for h in analyze.hourly([utc, pacific])["hours"]}
        assert hours[6] == 1
        assert hours[23] == 1


class TestFilter:
    def test_bounds_are_half_open(self) -> None:
        entries = [entry(hour=6), entry(hour=7), entry(hour=8)]
        since = datetime(2026, 7, 12, 7, tzinfo=UTC)
        until = datetime(2026, 7, 12, 8, tzinfo=UTC)
        result = analyze.filter_entries(entries, since, until)
        assert len(result) == 1, "since is inclusive, until is exclusive"
        assert result[0].time.hour == 7

    def test_no_bounds_passes_everything(self) -> None:
        entries = [entry(hour=6), entry(hour=7)]
        assert len(analyze.filter_entries(entries)) == 2

    def test_since_only(self) -> None:
        entries = [entry(hour=6), entry(hour=9)]
        since = datetime(2026, 7, 12, 8, tzinfo=UTC)
        assert len(analyze.filter_entries(entries, since=since)) == 1

    def test_compares_across_offsets_as_instants(self) -> None:
        # 23:00 -0700 is 06:00 UTC, so a 05:00-07:00 UTC window must include it.
        pacific = parse_line(
            '1.1.1.2 - - [11/Jul/2026:23:00:00 -0700] "GET /a HTTP/1.1" 200 1 "-" "-"'
        )
        assert isinstance(pacific, LogEntry)
        since = datetime(2026, 7, 12, 5, tzinfo=UTC)
        until = datetime(2026, 7, 12, 7, tzinfo=UTC)
        assert len(analyze.filter_entries([pacific], since, until)) == 1


class TestParseBound:
    def test_naive_input_is_read_as_utc(self) -> None:
        """`--since 2026-07-12` is what people type, and it must not crash."""
        assert analyze.parse_bound("2026-07-12") == datetime(2026, 7, 12, tzinfo=UTC)
        assert analyze.parse_bound("2026-07-12T06:00:00") == datetime(2026, 7, 12, 6, tzinfo=UTC)

    def test_aware_input_is_converted_to_utc(self) -> None:
        assert analyze.parse_bound("2026-07-12T06:00:00Z") == datetime(2026, 7, 12, 6, tzinfo=UTC)
        assert analyze.parse_bound("2026-07-11T23:00:00-07:00") == datetime(
            2026, 7, 12, 6, tzinfo=UTC
        )

    def test_result_is_always_aware(self) -> None:
        # The whole point: comparing naive to aware raises TypeError.
        assert analyze.parse_bound("2026-07-12").tzinfo is not None

    @pytest.mark.parametrize("value", ["yesterday", "1 hour ago", "", "2026-13-45"])
    def test_junk_raises_valueerror_with_an_example(self, value: str) -> None:
        with pytest.raises(ValueError, match="invalid ISO8601"):
            analyze.parse_bound(value)
