"""Sealed fixture corpus + ground truth for the loglens executable
acceptance suite (run_03, haiku tier — ports run_01's A1–A12).

Entries are structured data; CLF lines AND expected values derive from
the same source, so they cannot drift. SEALED — never shown to agents.
"""

from collections import Counter
from datetime import datetime, timezone

# (ip, path, status, size, hour, minute) — all on 2026-07-12, +0000
# unless noted. Chosen so:
#  - 4 unique IPs, several paths, 2xx/3xx/4xx/5xx spread, 6 distinct hours
#  - /alpha and /beta have EQUAL counts (A4 tie -> value-ascending)
#  - errors exist inside AND outside the 06:00-09:00 probe window
ENTRIES = [
    ("203.0.113.7",  "/alpha",    200, 5413, 5, 12),
    ("203.0.113.7",  "/alpha",    200, 5413, 5, 48),
    ("198.51.100.22","/beta",     200,  512, 5, 55),
    ("203.0.113.7",  "/gamma",    200, 1000, 6, 5),
    ("198.51.100.22","/alpha",    301,  180, 6, 15),
    ("192.0.2.9",    "/missing",  404,  153, 6, 30),
    ("192.0.2.9",    "/missing",  404,  153, 6, 45),
    ("198.51.100.22","/api/x",    500,  270, 7, 10),
    ("203.0.113.7",  "/beta",     200,  512, 7, 20),
    ("192.0.2.9",    "/alpha",    200, 5413, 7, 40),
    ("198.51.100.22","/api/x",    500,  270, 8, 5),
    ("192.0.2.9",    "/missing",  404,  153, 8, 25),
    ("203.0.113.7",  "/beta",     200,  512, 8, 50),
    ("198.51.100.22","/gamma",    200, 1000, 11, 10),
    ("192.0.2.9",    "/missing",  404,  153, 11, 30),
    ("203.0.113.10", "/alpha",    200, 5413, 11, 45),
    ("203.0.113.10", "/beta",     200,  512, 23, 5),
    ("198.51.100.22","/alpha",    200, 5413, 23, 30),
    ("192.0.2.9",    "/api/x",    500,  270, 23, 55),
    ("203.0.113.10", "/beta",     301,  180, 23, 59),
    ("192.0.2.9",    "/beta",     200,  512, 9, 15),   # -> /alpha & /beta TIE at 6 (A4)
]

MALFORMED = [
    "this is not a log line at all",
    '999.999.999.999 broken [not-a-date] "GET" oops',
]


def clf_line(ip, path, status, size, hour, minute, day=12, offset="+0000"):
    return (
        f'{ip} - - [{day:02d}/Jul/2026:{hour:02d}:{minute:02d}:00 {offset}] '
        f'"GET {path} HTTP/1.1" {status} {size} "-" "test/1.0"'
    )


def render(entries=ENTRIES, malformed=MALFORMED):
    lines = [clf_line(*e) for e in entries]
    # interleave malformed lines mid-file
    lines.insert(4, malformed[0])
    lines.insert(11, malformed[1])
    return "\n".join(lines) + "\n"


# ---------------------------------------------------------------- ground truth
TOTAL_VALID = len(ENTRIES)                                   # 20
UNIQUE_IPS = len({e[0] for e in ENTRIES})                    # 4
N_MALFORMED = len(MALFORMED)                                 # 2
ERROR_COUNT = sum(1 for e in ENTRIES if e[2] >= 400)         # 4xx+5xx
ERROR_RATE_PCT = ERROR_COUNT / TOTAL_VALID * 100             # e.g. 35.0
FIRST_TS = datetime(2026, 7, 12, 5, 12, tzinfo=timezone.utc)
LAST_TS = datetime(2026, 7, 12, 23, 59, tzinfo=timezone.utc)

PATH_COUNTS = Counter(e[1] for e in ENTRIES)
# top-3 by count desc, ties by value (path) ascending — the A4 rule
TOP3_PATHS = sorted(PATH_COUNTS.items(), key=lambda kv: (-kv[1], kv[0]))[:3]

HOURLY = Counter(e[4] for e in ENTRIES)                      # hour -> count

# errors grouped by (status, path), most frequent first
ERROR_GROUPS = Counter((e[2], e[1]) for e in ENTRIES if e[2] >= 400)
TOP_ERROR_GROUP = ERROR_GROUPS.most_common(1)[0]             # ((404,/missing),3)

# A5 window: 06:00 <= t < 09:00 (+0000) — no boundary records in core
WINDOW_SINCE = "2026-07-12T06:00:00+00:00"
WINDOW_UNTIL = "2026-07-12T09:00:00+00:00"
ERRORS_IN_WINDOW = Counter(
    (e[2], e[1]) for e in ENTRIES if e[2] >= 400 and 6 <= e[4] < 9
)  # {(404,'/missing'):3, (500,'/api/x'):2}

# ---------------------------------------------------------------- hard fixtures
# H1/H2 — mixed UTC offsets in one file (the run-1 killer: naive vs aware)
MIXED_TZ_LINES = [
    clf_line("203.0.113.7", "/alpha", 200, 100, 6, 0, offset="+0000"),
    clf_line("203.0.113.7", "/alpha", 404, 100, 3, 30, offset="-0500"),  # = 08:30Z
    clf_line("198.51.100.22", "/beta", 500, 100, 12, 15, offset="+0200"),  # = 10:15Z
]
MIXED_TZ = "\n".join(MIXED_TZ_LINES) + "\n"
# instants: 06:00Z, 08:30Z, 10:15Z — window 06:00Z..09:00Z holds the 404 only
#           (500 at 10:15Z outside; 200 at 06:00Z is not an error)

# H4 — boundary probes: errors exactly AT the since and until instants
BOUNDARY_LINES = [
    clf_line("192.0.2.9", "/at-since", 404, 100, 6, 0),    # exactly since
    clf_line("192.0.2.9", "/inside",   404, 100, 7, 0),
    clf_line("192.0.2.9", "/at-until", 404, 100, 9, 0),    # exactly until
]
BOUNDARY = "\n".join(BOUNDARY_LINES) + "\n"

# H7 — valid file, window that matches nothing
# (reuses the main fixture with a 1971 window)

# H8 — CLF dash-for-bytes: valid per CLF; spec is silent. Dual-accept:
# counted as valid OR as malformed, but never a crash.
DASH_BYTES = clf_line("203.0.113.7", "/alpha", 200, 0, 6, 0).replace(' 200 0 "', ' 200 - "') + "\n"
