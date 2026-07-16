# Sealed fixture ground truth — grader: sol

These values were derived directly from the fixture design, independently of
all contender implementations.

- Generator seed: `20260715`
- Valid lines: 40
- Malformed lines: 3
- Unique IPs: 7
- First timestamp: `2026-07-12T00:05:00+00:00`
- Last timestamp: `2026-07-12T23:59:59+00:00`
- Error responses (4xx/5xx): 20
- Error rate: 50%

## Path counts

| path | count |
|---|---:|
| `/alpha` | 9 |
| `/beta` | 8 |
| `/charlie` | 7 |
| `/delta` | 7 |
| `/epsilon` | 5 |
| `/zeta` | 4 |

For `top --by path -n 3`, the expected order is `/alpha`, `/beta`,
`/charlie`. `/charlie` and `/delta` are tied at 7, so ascending value order
selects `/charlie`.

## Error window

Window: `--since 2026-07-12T10:00:00+00:00 --until
2026-07-12T11:00:00+00:00`, with both endpoints included.

| status | path | count |
|---:|---|---:|
| 404 | `/alpha` | 3 |
| 404 | `/beta` | 2 |
| 500 | `/charlie` | 2 |
| 500 | `/delta` | 1 |
| 503 | `/epsilon` | 1 |

The `09:59:59` and `11:00:01` errors are excluded. There are 9 errors in the
window. Ordering among equal-count groups is not constrained by the project
spec.

## Hourly counts

| hour | count | hour | count | hour | count |
|---:|---:|---:|---:|---:|---:|
| 00 | 3 | 08 | 0 | 16 | 0 |
| 01 | 2 | 09 | 3 | 17 | 0 |
| 02 | 0 | 10 | 9 | 18 | 0 |
| 03 | 0 | 11 | 4 | 19 | 0 |
| 04 | 0 | 12 | 5 | 20 | 0 |
| 05 | 0 | 13 | 0 | 21 | 0 |
| 06 | 4 | 14 | 0 | 22 | 0 |
| 07 | 0 | 15 | 5 | 23 | 5 |
