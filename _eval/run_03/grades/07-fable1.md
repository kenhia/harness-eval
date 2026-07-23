# 07-baseline-claude — grader: fable1 (run_03, Haiku 4.5 tier) [Claude Code control]

| dim | score /5 | note |
|---|---|---|
| correctness | 3 | Mechanical mapping: core 11/12, hard 4/9 → base 3, no adjustment (hard < 6/9). All six failures share one root: `parse_timestamp` keeps only `split(' ')[0]`, discarding the offset, so any aware `--since/--until` crashes (`cli.py:102`). Notably passes H9 (naive-vs-naive compares fine) — the exact mirror image of the tz-aware repos |
| code quality | 3 | Deliberately minimal: stdlib-only (zero runtime deps — `dependencies = []`), flat cli.py + parser.py, single file-load then dispatch (no per-command reload boilerplate), tie-break with `(-count, str(value))`, malformed count on stderr everywhere. Docked for: the naive-timestamp shortcut (six failures); dead code — `format_output` returns `data` on both branches and is never meaningfully used; the no-valid-entries check duplicated in `main` *and* all four handlers; committed `.pyc` files with no .gitignore |
| tests | 3 | 25 tests (16 CLI via monkeypatch/capsys + 9 parser): exit codes for missing file and no-valid-lines, JSON validity per subcommand, since/until filters, default `-n` and default format. Crucially, the suite runs from a clean checkout (A9/A12 green) — the only Claude-runner repo where the tests actually execute for a stranger. Missing: tie-break assertion, malformed-count-on-stderr assertion, any tz coverage |
| docs | 4 | README (166 lines): uv install, every subcommand with realistic examples (default/-n variants, time-range errors), error handling with exit codes, log format, dev commands — all of which work from a clean clone. Enough for a stranger, and accurate |
| process | 2 | One commit for the whole project — `70530fd` "build: initialize loglens project" (18 files, 953 lines) — against the spec's explicit "meaningful git commits as you go"; the message misdescribes a complete implementation as initialization. No .gitignore, `.pyc` files committed. Tree clean and committed at done, which is the floor it stands on |
| efficiency | 5 | Control anchor for the Claude cells: 22.2k output / 2.5m cache read / 42.2k cache write, 54 turns, 4m16s wall — the cheapest, fastest run of the entire field |
| autonomy | 5 | Headless, zero interventions, runner exit 0, tree clean and fully committed at agent-done |

**Weighted total:** 66/100
(3/5×30 + 3/5×20 + 3/5×15 + 4/5×10 + 2/5×10 + 5/5×10 + 5/5×5)

**Best thing:** The zero-dependency decision: stdlib argparse and no runtime deps make the package trivially portable — this is the only Claude-runner cell whose tests and `just check` pass from a clean clone, and it was also the field's cheapest and fastest run.

**Worst thing:** Git process reduced to a single mislabeled commit — "build: initialize loglens project" is actually the entire finished project, discarding the spec's commits-as-you-go requirement and any ability to follow the work.

**Narrative (≤150 words):** The Claude control is a study in effective minimalism with one blind spot and one bad habit. Minimalism: stdlib-only, flat layout, one parse pass feeding four handlers — and because nothing needs installing, its checks are the only Claude-cell ones that work everywhere (A9, A12, A12's `just check` all green at 11/12 core). The blind spot is the tier's signature: the timestamp offset is thrown away at parse time, so every timezone-aware filter crashes — six failures from one line, while its naive-on-naive H9 pass mirrors the aware repos' failure. The bad habit is process: the entire project landed in a single commit whose message says "initialize", there is no .gitignore, and compiled `.pyc` files are in history. As the efficiency anchor it is formidable — 4m16s and the smallest token bill in the field for the second-best core tally.
