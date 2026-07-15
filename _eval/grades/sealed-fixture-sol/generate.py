#!/usr/bin/env python3
from __future__ import annotations

import random
from datetime import datetime
from pathlib import Path


SEED = 20260715
IPS = [
    "192.0.2.10",
    "192.0.2.11",
    "198.51.100.20",
    "198.51.100.21",
    "203.0.113.30",
    "203.0.113.31",
    "203.0.113.32",
]

RECORDS = [
    ("2026-07-12T00:05:00+00:00", "/alpha", 200),
    ("2026-07-12T00:15:00+00:00", "/beta", 301),
    ("2026-07-12T00:45:00+00:00", "/alpha", 404),
    ("2026-07-12T01:00:00+00:00", "/charlie", 200),
    ("2026-07-12T01:30:00+00:00", "/delta", 500),
    ("2026-07-12T06:10:00+00:00", "/alpha", 200),
    ("2026-07-12T06:20:00+00:00", "/beta", 201),
    ("2026-07-12T06:30:00+00:00", "/epsilon", 302),
    ("2026-07-12T06:40:00+00:00", "/zeta", 404),
    ("2026-07-12T09:00:00+00:00", "/alpha", 200),
    ("2026-07-12T09:30:00+00:00", "/beta", 403),
    ("2026-07-12T09:59:59+00:00", "/charlie", 500),
    ("2026-07-12T10:00:00+00:00", "/alpha", 404),
    ("2026-07-12T10:05:00+00:00", "/beta", 404),
    ("2026-07-12T10:10:00+00:00", "/charlie", 500),
    ("2026-07-12T10:15:00+00:00", "/delta", 500),
    ("2026-07-12T10:20:00+00:00", "/alpha", 404),
    ("2026-07-12T10:25:00+00:00", "/beta", 404),
    ("2026-07-12T10:30:00+00:00", "/epsilon", 503),
    ("2026-07-12T10:40:00+00:00", "/zeta", 200),
    ("2026-07-12T10:50:00+00:00", "/charlie", 500),
    ("2026-07-12T11:00:00+00:00", "/alpha", 404),
    ("2026-07-12T11:00:01+00:00", "/delta", 500),
    ("2026-07-12T11:15:00+00:00", "/beta", 200),
    ("2026-07-12T11:45:00+00:00", "/epsilon", 200),
    ("2026-07-12T12:00:00+00:00", "/alpha", 200),
    ("2026-07-12T12:10:00+00:00", "/charlie", 301),
    ("2026-07-12T12:20:00+00:00", "/delta", 200),
    ("2026-07-12T12:30:00+00:00", "/beta", 404),
    ("2026-07-12T12:40:00+00:00", "/epsilon", 500),
    ("2026-07-12T15:00:00+00:00", "/delta", 200),
    ("2026-07-12T15:10:00+00:00", "/charlie", 200),
    ("2026-07-12T15:20:00+00:00", "/delta", 302),
    ("2026-07-12T15:30:00+00:00", "/beta", 200),
    ("2026-07-12T15:40:00+00:00", "/zeta", 200),
    ("2026-07-12T23:00:00+00:00", "/alpha", 500),
    ("2026-07-12T23:10:00+00:00", "/charlie", 404),
    ("2026-07-12T23:20:00+00:00", "/delta", 200),
    ("2026-07-12T23:30:00+00:00", "/epsilon", 200),
    ("2026-07-12T23:59:59+00:00", "/zeta", 503),
]

MALFORMED = [
    "this is not a log line",
    '192.0.2.99 - - [bad timestamp] "GET /broken HTTP/1.1" 200 12 "-" "bad"',
    '203.0.113.99 - - [12/Jul/2026:13:00:00 +0000] "GET /missing-fields HTTP/1.1"',
]


def clf_line(index: int, timestamp: str, path: str, status: int) -> str:
    parsed = datetime.fromisoformat(timestamp)
    rendered = parsed.strftime("%d/%b/%Y:%H:%M:%S %z")
    method = "POST" if index % 6 == 0 else "GET"
    return (
        f'{IPS[index % len(IPS)]} - - [{rendered}] '
        f'"{method} {path} HTTP/1.1" {status} {100 + index} '
        f'"https://example.test/" "sealed-fixture/{index}"'
    )


def main() -> None:
    lines = [
        clf_line(index, timestamp, path, status)
        for index, (timestamp, path, status) in enumerate(RECORDS)
    ]
    lines.extend(MALFORMED)
    random.Random(SEED).shuffle(lines)
    output = Path(__file__).with_name("sealed.log")
    output.write_text("\n".join(lines) + "\n", encoding="utf-8")


if __name__ == "__main__":
    main()
