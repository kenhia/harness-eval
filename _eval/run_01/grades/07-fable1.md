# 07-baseline-claude — grader: fable1

Control run: no harness, Claude Code runner. Graded exactly as 05 was
(git hygiene and coherent commits under Process; no harness artifacts
expected). Efficiency scored on wall-clock and absence of thrash; `/cost`
dollars are not comparable to Copilot credits (07 is itself the anchor the
runner covariate is measured against).

| dim | score /5 | note |
|---|---|---|
| correctness | 5 | 12/12 acceptance (see 07-acceptance-fable1.md); both designed ties break correctly, window and streams exact |
| code quality | 4 | Clean four-module split (parser/analyze/render/cli), appropriately small (1,411 insertions); escaped-quote-safe regex; naive `--since`/`--until` normalized to UTC in `normalize_bound` — the hole that crashed 02/03/05 is handled here. Docked on accumulated small warts: `strptime %b` is locale-dependent (under a non-English LC_TIME every line goes malformed); `-` size becomes 0, conflating "no body" with "zero bytes"; `top` stringifies statuses that `render` casts back to int (self-acknowledged smell); `--format` is declared twice (top-level + parent) |
| tests | 5 | 77 tests covering every edge my run-1 sheets asked for: reverse-order tie-breaks, full exit-code matrix (missing/unreadable/directory/empty/all-malformed), stderr-vs-stdout with `json.loads(stdout)` on a malformed-bearing file, JSON single-doc for all four subcommands, both `--format` positions, naive-bound-as-UTC, inclusive window boundary pinned, JSON status-type stability, and a root-guard on the chmod test — fixed after its own review noticed it passed trivially |
| docs | 5 | README alone suffices: honest `git clone <this-repo>` placeholder, both install paths, every subcommand with transcribed real output, JSON contract, behavior notes (blank lines, timezone retention, bar scaling), exit-code table, dev section. Its naive-timestamp example (`--since 2026-07-12T12:00:00`) actually works — the exact example that crashed run 02 |
| process | 5 | Seven commits in a clean scaffold → parser → analysis → CLI → tests → docs → review-fix sequence, each message stating decisions, not diffs; the unprompted final commit is a genuine self-review that fixed two real user-visible bugs (empty bars for small nonzero hours; str-vs-int JSON statuses) and two tests that weren't testing what they claimed — the self-review 05 was docked for lacking |
| efficiency | 5 | 8m38s wall, single linear pass, zero thrash; one review loop that paid for itself in the final commit. Cheapest run on its runner ($3.61, 43.5k output tokens vs 06's $10.84/122.7k) and well under half of 06's 19m05s wall. Cross-runner cost units aren't comparable (this run exists to anchor them); on wall-clock and behavior this is what efficient looks like |
| autonomy | 5 | Zero interventions; declared done; committed |

**Weighted total:** 96/100
(5/5×30 + 4/5×20 + 5/5×15 + 5/5×10 + 5/5×10 + 5/5×10 + 5/5×5)

**Best thing:** The unprompted review-fix commit (00f8445): it caught a rendering bug, a JSON type inconsistency, and two self-deceiving tests — concrete evidence of a review loop that changed the outcome, with no harness prescribing it.

**Worst thing:** `strptime("%b")` for month parsing: under any non-English `LC_TIME` locale every single line silently becomes "malformed" and the tool exits 1 on a perfectly good log — latent, environment-dependent, and untested.

**Narrative (≤150 words):** The Claude-runner control turns in the strongest baseline of the eval — 12/12 acceptance, and the robustness edges that separated run 1's field are nearly all handled: naive ISO-8601 bounds normalize instead of crashing, tie-breaks are pinned against adversarial ordering, JSON types are stable, exit codes cover directory and permission-denied. The 77-test suite is triple 05's and includes a fix for a test that was passing for the wrong reason. Commit history reads scaffold → layers → tests → docs → self-review, with the review commit catching two real bugs. What keeps code quality off a 5 is an accumulation of small warts — locale-dependent `%b` month parsing, `-` sizes conflated with 0, a str/int status round-trip between layers. At 8m38s with zero rework, it sets a demanding efficiency anchor: whatever a harness adds on this runner, this is what it must beat.
