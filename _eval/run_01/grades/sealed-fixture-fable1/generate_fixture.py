#!/usr/bin/env python3
"""Sealed fixture generator — grader fable1, harness-eval POC run 1.

Deterministic by construction: the records are an explicit hand-designed
table (no randomness needed; the design plays the role of the fixed seed).
Emitted in chronological order with malformed lines interleaved at fixed
positions. Design goals (per acceptance.md):

- 40 lines: 37 valid + 3 malformed
- 6 unique client IPs, 6 paths, statuses 200/201/301/404/503/500
- exact tie in path counts (/about and /contact both 6) for the A4 tie-break
- errors on BOTH sides of the window 07:00:00 .. 14:00:00 UTC (nothing
  exactly at a boundary instant, so inclusive/exclusive semantics don't
  change the result)
- hours used: 05, 06, 07, 08, 13, 22

Also writes malformed-only.log (for the A8 exit-code-1 check).
"""

from pathlib import Path

IP1 = "203.0.113.7"
IP2 = "198.51.100.22"
IP3 = "192.0.2.9"
IP4 = "203.0.113.50"
IP5 = "198.51.100.99"
IP6 = "192.0.2.101"

# (ip, user, HH:MM:SS, method, path, status, size) — all on 14/Jul/2026 +0000
RECORDS = [
    (IP1, "-",     "05:02:11", "GET",  "/index.html", 200, 5413),
    (IP5, "-",     "05:10:40", "GET",  "/admin",      500, 512),
    (IP2, "-",     "05:15:30", "GET",  "/index.html", 200, 5413),
    (IP2, "alice", "05:20:00", "POST", "/api/orders", 201, 512),
    (IP3, "-",     "05:45:00", "GET",  "/about",      200, 2048),
    (IP4, "-",     "05:55:12", "GET",  "/contact",    301, 178),
    (IP3, "-",     "06:03:22", "GET",  "/index.html", 200, 5413),
    (IP5, "alice", "06:10:15", "POST", "/api/orders", 201, 512),
    (IP4, "-",     "06:20:45", "GET",  "/index.html", 200, 5413),
    (IP6, "-",     "06:30:00", "GET",  "/contact",    301, 178),
    # M1 inserted here
    (IP1, "-",     "06:45:00", "GET",  "/missing",    404, 153),
    (IP1, "-",     "06:50:31", "GET",  "/about",      200, 2048),
    (IP1, "-",     "07:05:10", "GET",  "/index.html", 200, 5413),
    (IP2, "-",     "07:10:05", "GET",  "/about",      200, 2048),
    (IP2, "-",     "07:15:44", "GET",  "/missing",    404, 153),
    (IP6, "alice", "07:22:33", "POST", "/api/orders", 201, 512),
    (IP5, "-",     "07:30:00", "GET",  "/index.html", 200, 5413),
    (IP6, "-",     "07:40:09", "GET",  "/admin",      503, 299),
    (IP3, "-",     "07:48:59", "GET",  "/contact",    301, 178),
    (IP4, "alice", "07:55:01", "POST", "/api/orders", 201, 512),
    (IP6, "-",     "08:05:47", "GET",  "/about",      200, 2048),
    (IP2, "-",     "08:12:34", "GET",  "/index.html", 200, 5413),
    # M2 inserted here
    (IP5, "-",     "08:25:16", "GET",  "/contact",    301, 178),
    (IP3, "-",     "08:30:29", "GET",  "/missing",    404, 153),
    (IP1, "alice", "08:40:12", "POST", "/api/orders", 201, 512),
    (IP6, "-",     "13:01:59", "GET",  "/index.html", 200, 5413),
    (IP3, "alice", "13:15:00", "POST", "/api/orders", 201, 512),
    (IP1, "-",     "13:20:18", "GET",  "/admin",      500, 512),
    (IP4, "-",     "13:30:44", "GET",  "/about",      200, 2048),
    (IP5, "alice", "13:45:27", "POST", "/api/orders", 201, 512),
    (IP2, "-",     "13:50:33", "GET",  "/contact",    301, 178),
    (IP6, "-",     "22:05:55", "GET",  "/missing",    404, 153),
    (IP1, "-",     "22:10:00", "GET",  "/index.html", 200, 5413),
    (IP5, "-",     "22:20:19", "GET",  "/about",      200, 2048),
    (IP6, "alice", "22:30:08", "POST", "/api/orders", 201, 512),
    (IP3, "-",     "22:45:51", "GET",  "/index.html", 200, 5413),
    (IP4, "-",     "22:55:07", "GET",  "/contact",    301, 178),
    # M3 appended at end
]

MALFORMED = {
    10: "this line is complete garbage and matches no log format at all",
    22: '203.0.113.7 - - [14/Jul/2026:09:00:00 +0000] "GET /truncated',
}
MALFORMED_TAIL = "- - - - -"


def fmt(rec):
    ip, user, hms, method, path, status, size = rec
    referer = "https://example.com/" if path == "/index.html" else "-"
    ua = "curl/8.5.0" if method == "POST" else "Mozilla/5.0"
    return (
        f'{ip} - {user} [14/Jul/2026:{hms} +0000] '
        f'"{method} {path} HTTP/1.1" {status} {size} "{referer}" "{ua}"'
    )


def main():
    out = Path(__file__).parent
    lines = []
    for i, rec in enumerate(RECORDS):
        if i in MALFORMED:
            lines.append(MALFORMED[i])
        lines.append(fmt(rec))
    lines.append(MALFORMED_TAIL)
    (out / "sealed.log").write_text("\n".join(lines) + "\n")
    (out / "malformed-only.log").write_text(
        "not a log line\n"
        "still not a log line\n"
        '1.2.3.4 - - [garbage] "GET\n'
    )
    print(f"wrote {len(lines)} lines to sealed.log "
          f"({len(RECORDS)} valid, {len(MALFORMED) + 1} malformed)")


if __name__ == "__main__":
    main()
