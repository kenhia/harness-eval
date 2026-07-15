"""Render tests: format dispatch and control-character sanitization."""

from __future__ import annotations

import json

import pytest

from loglens.render import render, sanitize


class TestSanitize:
    @pytest.mark.parametrize(
        ("raw", "expected"),
        [
            ("/normal/path", "/normal/path"),
            ("/with space", "/with space"),
            ("/clear\x1b[2Jscreen", "/clear\\x1b[2Jscreen"),
            ("/bell\x07", "/bell\\x07"),
            ("/null\x00", "/null\\x00"),
            ("/newline\n", "/newline\\x0a"),
        ],
    )
    def test_control_chars_escaped(self, raw: str, expected: str) -> None:
        assert sanitize(raw) == expected

    def test_non_strings_pass_through(self) -> None:
        assert sanitize(404) == 404
        assert sanitize(None) is None
        assert sanitize(1.5) == 1.5

    def test_unicode_is_preserved(self) -> None:
        # Printable non-ASCII is not a terminal hazard; don't mangle it.
        assert sanitize("/café") == "/café"


class TestRender:
    def test_json_is_parseable(self) -> None:
        data = {
            "total_requests": 1,
            "unique_ips": 1,
            "first": None,
            "last": None,
            "error_rate": 0.0,
        }
        assert json.loads(render(data, "summary", "json")) == data

    def test_json_sanitizes_nested_values(self) -> None:
        data = {"items": [{"value": "/x\x1b[2J", "count": 1}], "by": "path", "count": 1}
        out = json.loads(render(data, "top", "json"))
        assert out["items"][0]["value"] == "/x\\x1b[2J"

    def test_text_top_empty(self) -> None:
        out = render({"by": "path", "count": 0, "items": []}, "top", "text")
        assert "No path values found." in out

    def test_text_errors_empty(self) -> None:
        out = render({"total_errors": 0, "groups": []}, "errors", "text")
        assert "No 4xx/5xx requests found." in out

    def test_hourly_with_no_requests_draws_no_bars(self) -> None:
        data = {"total_requests": 0, "hours": [{"hour": h, "count": 0} for h in range(24)]}
        out = render(data, "hourly", "text")
        assert "#" not in out
        assert "Total requests: 0" in out

    def test_hourly_scale_is_fixed_not_terminal_width(self) -> None:
        """Output must be identical between a tty and a captured pipe."""
        data = {
            "total_requests": 10,
            "hours": [{"hour": h, "count": 10 if h == 0 else 0} for h in range(24)],
        }
        out = render(data, "hourly", "text")
        assert "#" * 40 in out


class TestNoTrailingWhitespace:
    """Trailing spaces show up in diffs and in redirected output for no benefit."""

    @pytest.mark.parametrize(
        ("data", "command"),
        [
            ({"by": "path", "count": 1, "items": [{"value": "/a", "count": 9}]}, "top"),
            (
                {"total_errors": 1, "groups": [{"status": 404, "path": "/a", "count": 1}]},
                "errors",
            ),
            (
                {
                    "total_requests": 1,
                    "hours": [{"hour": h, "count": 1 if h == 6 else 0} for h in range(24)],
                },
                "hourly",
            ),
        ],
    )
    def test_no_line_ends_in_whitespace(self, data: dict, command: str) -> None:
        for line in render(data, command, "text").splitlines():
            assert line == line.rstrip(), f"trailing whitespace: {line!r}"
