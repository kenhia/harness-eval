# 05-baseline — grader: fable1 (run_03, Haiku 4.5 tier) [Copilot control]

| dim | score /5 | note |
|---|---|---|
| correctness | 2 | Mechanical mapping: core 9/12, hard 4/9 → base 2, no adjustment (hard < 6/9). Failures split into two roots: (1) naive parser — `timestamp_str[:20]` slice drops the offset, so aware windows crash (A5, H2, H3, H4, H6, H7 at `cli.py:157`); (2) packaging — pytest lives only in the `dev` optional extra, so `uv run pytest` and `just check` fail from a clean checkout with `No module named 'click'` (A9, A12) |
| code quality | 3 | Idiomatic click CLI (227 lines) + separate parser module; malformed count on stderr (A7 passes), correct exit codes, invalid `--since` handled. Docked for: the tz-stripping slice; dev tooling declared as `[project.optional-dependencies].dev` instead of a dependency group, which breaks its own test/check commands anywhere but the agent's venv; `.pyc` files committed and never untracked after the .gitignore commit |
| tests | 3 | 29 CliRunner tests — the best CLI-boundary coverage in the Copilot field: exit codes 0/1/2 each asserted, malformed-lines warning asserted on output, JSON parsed for every subcommand, since/until/both filters, invalid-since error. But zero tz coverage, and the whole suite is unrunnable from a clean clone (A9) — regression protection that only exists on the machine that wrote it |
| docs | 4 | README (157 lines): prerequisites, uv install, all four subcommands with output, log format, error handling, dev commands. Complete and accurate apart from inheriting the broken clean-env test invocation |
| process | 3 | Six sequential commits with accurate messages (init → parser → tests/fixtures → justfile → gitignore → uv.lock) — a coherent narrative arc, and the tree was clean at agent-done. Docked for committing `.pyc` files in the early commits and then adding a .gitignore without removing the already-tracked caches |
| efficiency | 5 | Control anchor for the Copilot cells: 57.1 AI credits, 4m39s wall, 0.33 premium requests |
| autonomy | 5 | Headless, zero interventions, runner exit 0, tree clean and fully committed at agent-done |

**Weighted total:** 62/100
(2/5×30 + 3/5×20 + 3/5×15 + 4/5×10 + 3/5×10 + 5/5×10 + 5/5×5)

**Best thing:** The test suite's CLI-boundary discipline — all three exit codes, stderr content, and JSON validity are individually asserted through CliRunner; among Copilot cells only this control tested the interface the spec actually specifies.

**Worst thing:** Putting pytest/ruff in an optional extra: `uv run pytest`, the justfile's `check`, and the README's dev instructions all fail on a fresh clone (A9, A12), silently converting 29 good tests into zero effective ones.

**Narrative (≤150 words):** The no-harness Copilot control is a respectable middle: clean click code, the field's most behavior-complete test suite, honest commit history, and a clean tree at done — with two structural mistakes that cost it five core points between them. The timezone slice (`[:20]`) is the tier's signature bug in its bluntest form, taking down the windowed-errors path and four hard tests. The subtler one is packaging: dev tools declared as an optional extra mean the project's own quality gates only run in the environment that built it — acceptance's clean-env `pytest` and `just check` both die on imports. Nothing here is careless in the small; both failures are single early decisions with wide blast radius. As the efficiency and autonomy anchor for the Copilot cells, it sets a brisk baseline: 57 credits, 4m39s, six commits, no loose ends in the tree.
