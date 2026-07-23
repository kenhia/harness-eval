# 04-kprojects — grader: fable1 (run_03, Haiku 4.5 tier)

| dim | score /5 | note |
|---|---|---|
| correctness | 3.5 | Mechanical mapping: core 11/12, hard 8/9 → base 3 + 0.5 (hard ≥ 6/9) = 3.5. Best tally of the field. The two failures: A1 — `loglens --help` exits 1 with a one-line usage (the hand-rolled parser has no help support); H9 — naive `--since` vs aware entries crash (`analysis.py:71`) |
| code quality | 3 | Good separation (parser/analysis/formatter/cli), tz-aware `%z` parsing, malformed count on stderr in every command, file-existence check with proper exit 2. Docked hard for the hand-rolled `sys.argv` parser (`cli.py:166-236`): no argparse/click, `--help` unsupported, unknown options silently skipped (`else: i += 1`), usage printed to stdout. It accidentally makes `--format` position-independent, but it's the wrong way to build a Python CLI and directly caused the A1 failure. Committed `.pyc` files under `tests/__pycache__/` |
| tests | 3 | 23 unit tests across parser/analysis/formatter: tie-break, time filter, malformed line, missing-bytes, JSON formatters parsed. No CLI-level tests at all — nothing invokes the binary, so no exit-code, stderr, or `--help` coverage; a single smoke test of `loglens --help` would have caught A1 |
| docs | 4 | README (176 lines) is the only one in the field with an explicit Exit Codes section; general syntax line documents the global `--format`, plus realistic workflow examples and testing/dev instructions. It nowhere mentions `--help` (consistent with reality, but a stranger will try it and get rc 1) |
| process | 3 | Best commit sequence of the Copilot cells: 4 logical commits (setup → core modules → CLI → tests/fixtures) with accurate messages. Properly wired the kproject-prescribed `just check` gate (replacing the pre-run TODO stub) — harness-prescribed artifact done right, and A12 passes from a clean env. Docked for: no sprint record despite kproject conventions, committed `.pyc` files, tree dirty at agent-done |
| efficiency | 5 | 48.1 AI credits vs control 05's 57.1 = 0.84× — *cheaper than the control* — at essentially identical wall clock (4m42s vs 4m39s), while delivering the field's best acceptance tally |
| autonomy | 4 | Headless, zero interventions, runner exit 0; docked one for the uncommitted-changes tree at agent-done (runlog) |

**Weighted total:** 70/100
(3.5/5×30 + 3/5×20 + 3/5×15 + 4/5×10 + 3/5×10 + 5/5×10 + 4/5×5)

**Best thing:** Value for money: the field's best acceptance result (11/12 core, 8/9 hard) at 16% *less* cost than the no-harness control, in a tidy four-commit sequence with the check gate actually wired up and green from a clean checkout.

**Worst thing:** Reinventing argument parsing by hand. The manual `sys.argv` walk silently ignores unknown options, prints usage to stdout, and cannot answer `--help` — the only repo in the field to fail A1, on the most basic CLI affordance there is.

**Narrative (≤150 words):** The strongest artifact of this tier, and the cheapest harness cell. The kproject setup seems to have channeled effort where it paid: a real module split, tz-aware parsing (8/9 on the hard tier — only H9's naive-`--since` mix trips it), stderr discipline, an Exit Codes section in the README, and a `just check` gate that actually passes from a clean clone. The one striking sub-frontier behavior is the CLI layer itself: rather than reach for argparse, the agent hand-walked `sys.argv`, which cost it `--help` (A1) and left unknown options silently ignored. Tests are decent unit-level work but never touch the installed binary, which is exactly where its two failures live. Process is the best of the Copilot cells — four accurate commits — though the tree was dirty at done, stray `.pyc` files got committed, and no sprint record was written.
