"""Tests for parser module"""


from loglens.parser import parse_line, parse_log_file


def test_parse_valid_line():
    """Test parsing a valid CLF log line"""
    line = (
        '203.0.113.7 - - [12/Jul/2026:06:25:24 +0000] '
        '"GET /index.html HTTP/1.1" 200 5413 '
        '"https://example.com/" "Mozilla/5.0"'
    )
    result = parse_line(line)
    assert result is not None
    assert result['ip'] == '203.0.113.7'
    assert result['status'] == 200
    assert result['bytes'] == 5413
    assert result['hour'] == 6


def test_parse_line_with_user():
    """Test parsing line with authenticated user"""
    line = (
        '198.51.100.22 - alice [12/Jul/2026:07:01:02 +0000] '
        '"POST /api/orders HTTP/1.1" 201 512 "-" "curl/8.5.0"'
    )
    result = parse_line(line)
    assert result is not None
    assert result['user'] == 'alice'
    assert result['status'] == 201


def test_parse_invalid_line():
    """Test parsing an invalid CLF log line"""
    line = 'This is not a valid log line'
    result = parse_line(line)
    assert result is None


def test_parse_file(tmp_path):
    """Test parsing a log file"""
    log_file = tmp_path / "test.log"
    log_file.write_text(
        '203.0.113.7 - - [12/Jul/2026:06:25:24 +0000] '
        '"GET /index.html HTTP/1.1" 200 5413 "-" "Mozilla/5.0"\n'
        'MALFORMED\n'
        '198.51.100.22 - - [12/Jul/2026:07:01:02 +0000] '
        '"POST /api/orders HTTP/1.1" 201 512 "-" "curl/8.5.0"\n'
    )
    lines, malformed = parse_log_file(str(log_file))
    assert len(lines) == 2
    assert malformed == 1


def test_parse_missing_file():
    """Test parsing a missing file"""
    lines, malformed = parse_log_file("/nonexistent/file.log")
    assert lines == []
    assert malformed == -1
