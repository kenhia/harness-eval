# 06-gstack — grader: fable1

Runner covariate: Claude Code CLI (not Copilot CLI as in 01–05). Efficiency is
scored primarily against 07-baseline-claude (same runner) plus absolute
wall-clock, per ADDING-A-HARNESS.md §1; `/cost` dollars/tokens are not
comparable to Copilot AI credits and are not held against the runner.

| dim | score /5 | note |
|---|---|---|
| correctness | 5 | 12/12 acceptance (see 06-acceptance-fable1.md); both designed ties (path and ip) break correctly, window and streams exact |
| code quality | 5 | Five-layer split (source/parse/analyze/render/cli) where each boundary earns its keep; escaped-quote-safe CLF regex; locale-proof month map; naive `--since` coerced to UTC at the argparse boundary — the exact hole that crashed runs 02/03/05; `-` bytes → None; BrokenPipe and exit-70 policy. Nits: LogEntry carries ident/authuser/referer nothing consumes; parse_line's blank-line branch is unreachable via source.py, which skips blanks first |
| tests | 5 | 137 tests, the field's strongest suite by a distance: tie-breaks fed in REVERSE order so insertion-order ranking fails loudly; `json.loads(stdout)` on a file *with* malformed lines; full exit-code matrix incl. directory and chmod-000 (guarded skip as root); naive-since; half-open boundary pinned; same-instant-two-offsets hourly bucketing; locale, ReDoS bound, broken pipe, invalid UTF-8, truncated final line |
| docs | 5 | README alone suffices: both install paths, every subcommand with real captured output, JSON contract ("on exit 0, stdout is a single valid JSON document"), exit-code table, malformed-vs-valid table, honest limits section. Nit: the primary install command points at a placeholder GitHub URL that doesn't exist; the from-a-clone path works as documented |
| process | 5 | Six commits in a textbook sequence (scaffold → parser/reader → analyze/render → CLI → fixtures+tests → docs), each message recording decisions and their failure-mode rationale, not diff restatement; TODOS.md is the anti-ceremony artifact — reviewer findings deliberately deferred with reasons, not built. The /autoplan plan itself was kept out of the repo; nothing committed is ceremony |
| efficiency | 2 | Longest wall-clock of the seven-run field: 19m05s vs 8m38s for the same-runner baseline (2.2×), $10.84 / 122.7k output tokens vs $3.61 / 43.5k (~3×). Run log notes ~8 minutes of up-front analysis before the first edit — proportionate for a big system, not a 700-line CLI. Result is the field's most robust, but the rubric prices burn *for the result delivered*, and 07 shows the same runner delivering 12/12 at a third of the spend |
| autonomy | 5 | Zero interventions; declared done; committed |

**Weighted total:** 94/100
(5/5×30 + 5/5×20 + 5/5×15 + 5/5×10 + 5/5×10 + 2/5×10 + 5/5×5)

**Best thing:** Threat-model discipline in the parser: the escaped-quote regex (silent field-shift corruption), the locale-proof month map, `-` bytes as None, and junk-request-lines-stay-countable are each implemented, tested, *and* argued in the commit message — the only run where the parser's failure modes were enumerated before being coded against.

**Worst thing:** Spend. Nineteen minutes and ~3× the same-runner baseline's tokens, with the first ~8 minutes producing no edits — planning depth sized for a project an order of magnitude larger (and the run log's own observation agrees).

**Narrative (≤150 words):** The strongest artifact of the seven runs. Everything 01–05 got dinged for is handled here: naive ISO-8601 bounds are coerced and tested, escaped quotes can't shift fields, months parse under any locale, tie-breaks are asserted against adversarially-ordered input, and the README documents every observable decision including its limits. The hostile.log fixture and the 137-test suite target exactly where regressions hide. Commit messages read like design reviews; TODOS.md records what the review chose *not* to build and why — process artifacts with genuine archaeological value. The cost is real: 2.2× the wall-clock and ~3× the tokens of the identical-runner baseline for margins the spec never asks for, echoing run 01's pattern — the harness bought robustness, not throughput. One factual note: the run log's observation that this project "didn't split analyze/cli/parser" is wrong — it has the field's cleanest five-module split.

## Reconciliation round 1

**Code quality: revised 5 → 4.**

The fact that moves me is (i), the verified README-vs-code contradiction. My
5 treated the unused LogEntry fields and the unreachable blank-line branch as
nits because nothing observable depended on them — but the memory claim is
different in kind: README line 185 states a behavioral guarantee ("memory
scales with distinct values, not file size") that `read_entries`
(source.py:43, `entries = list(_iter_entries(...))`) does not honor — peak
memory scales with valid-line count, and the `_iter_entries` generator whose
only caller immediately materializes it is exactly the speculative-streaming
machinery the rubric's "no speculative abstraction" language targets. Taken
together with my own recorded dead-code nits, that is a real dock, not a
footnote.

Sol's 3 overshoots, though, because the remaining charges don't hold up
against the artifact: the five-module split is load-bearing rather than
sized-for-a-larger-system — each seam is a distinct concern the 137-test
suite attaches to directly, and two of the seams produced verified
correctness wins (naive-UTC coercion at the argparse boundary; a single
render path that keeps `--format json` from being a second code path). The
lone `except Exception` is a top-level exit-70 policy that reports to stderr
and never masks a result — a deliberate sysexits convention, not a broad
swallow — and `dict[str, Any]` across a one-way analyze→render boundary of
four fixed shapes is the *lighter* choice; typed result classes would add
the machinery sol is objecting to. Query-stripping is a documented, tested,
RFC-grounded interpretation, not a defect. Parser rigor (escaped-quote
safety, locale-proof months, `-`→None) is verified and undisputed; it keeps
this at 4.

**Revised weighted total: 90/100** (4/5×20 on code quality; all other
dimensions unchanged).
