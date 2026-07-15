#!/usr/bin/env python3
"""Independent ground-truth computation for sealed.log — grader fable1.

Parses sealed.log with its own strict CLF regex (shares no code with any
project under test) and prints every value the acceptance checks compare
against. expected.md is transcribed from this output.
"""

import json
import re
from collections import Counter
from datetime import datetime, timezone
from pathlib import Path

CLF = re.compile(
    r'^(\S+) (\S+) (\S+) \[([^\]]+)\] "([A-Z]+) (\S+) [^"]*" (\d{3}) (\S+) "[^"]*" "[^"]*"$'
)

def main():
    here = Path(__file__).parent
    valid, malformed = [], 0
    for line in (here / "sealed.log").read_text().splitlines():
        m = CLF.match(line)
        if not m:
            malformed += 1
            continue
        ip, _, _, ts_raw, method, path, status, _size = m.groups()
        ts = datetime.strptime(ts_raw, "%d/%b/%Y:%H:%M:%S %z")
        valid.append((ip, ts, method, path, int(status)))

    total = len(valid)
    ips = sorted({v[0] for v in valid})
    times = sorted(v[1] for v in valid)
    errors = [v for v in valid if v[4] >= 400]

    print(f"total_valid_requests: {total}")
    print(f"malformed_lines: {malformed}")
    print(f"unique_ips: {len(ips)}  ({', '.join(ips)})")
    print(f"first_ts: {times[0].isoformat()}")
    print(f"last_ts:  {times[-1].isoformat()}")
    print(f"errors_4xx5xx: {len(errors)}")
    print(f"error_rate: {len(errors)}/{total} = {len(errors)/total*100:.4f}%")

    for key, idx in (("path", 3), ("ip", 0), ("status", 4)):
        c = Counter(v[idx] for v in valid)
        ranked = sorted(c.items(), key=lambda kv: (-kv[1], str(kv[0])))
        print(f"top_by_{key}: {ranked}")

    grouped = Counter((v[4], v[3]) for v in errors)
    print("errors_all:", sorted(grouped.items(), key=lambda kv: -kv[1]))

    since = datetime(2026, 7, 14, 7, 0, 0, tzinfo=timezone.utc)
    until = datetime(2026, 7, 14, 14, 0, 0, tzinfo=timezone.utc)
    win = [e for e in errors if since <= e[1] <= until]
    gwin = Counter((v[4], v[3]) for v in win)
    print(f"errors_window {since.isoformat()}..{until.isoformat()}:",
          sorted(gwin.items(), key=lambda kv: -kv[1]))

    hourly = Counter(v[1].hour for v in valid)
    print("hourly:", json.dumps({f"{h:02d}": hourly.get(h, 0) for h in range(24)}))

if __name__ == "__main__":
    main()
