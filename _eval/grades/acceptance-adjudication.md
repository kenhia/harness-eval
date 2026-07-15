# Acceptance adjudication — consensus session (fable2)

Graders agreed on 59 of 60 acceptance checks (5 repos × 12). One factual
dispute, adjudicated below in a fresh clone per protocol. Clone location:
`/tmp/grade-consensus/01-atv-starterkit` @ HEAD (bc5b3bf).

## Dispute 1 — repo 01, check A5 (errors + time window)

**fable1 claimed PASS:** "no window: 404/missing 4, 500/admin 2, 503/admin 1;
window 07:00→14:00 → 404/missing 2, 500/admin 1, 503/admin 1; most-frequent
first" (fixture `sealed-fixture-fable1`, which has no record exactly on a
window boundary — the check could not distinguish inclusive from exclusive
`--until`).

**sol claimed FAIL:** "Windowed errors reported `(404,/alpha,2)` instead of
expected count 3 because the `11:00:00` record was excluded by
`timestamp >= until`" (fixture `sealed-fixture-sol`, whose ground truth was
designed "with both endpoints included" and has a 404 /alpha record at
exactly `2026-07-12T11:00:00`).

**What I ran:** fresh clone, `uv sync`, then
`loglens errors sealed-fixture-sol/sealed.log --since 2026-07-12T10:00:00+00:00
--until 2026-07-12T11:00:00+00:00`. Output: alpha 2, beta 2, charlie 2,
delta 1, epsilon 1 — the 11:00:00 record is excluded. Sol's factual
observation **reproduces exactly**; the two graders' raw observations do not
actually conflict (fable1's fixture was blind to the boundary).

**The real question is interpretive:** is exclusive-`--until` "filtering
correctly" (A5's pass criterion)? Evidence gathered:

- The project spec (`prompts/00-project-spec.md`) says only
  `[--since ISO8601] [--until ISO8601]`. It specifies tie-break semantics for
  `top` explicitly, but is silent on window boundary semantics.
- `acceptance.md` A5 says "`--since`/`--until` filter correctly" — it does
  not pin inclusivity either. "Both endpoints included" was a choice made in
  sol's fixture design, not a sealed requirement.
- Repo 01's choice is deliberate and consistent: README line 79 ("`--since`
  (inclusive) and `--until` (exclusive)"), `--help` text (cli.py:54-55),
  `analyze.errors` docstring (analyze.py:63), and a dedicated test
  (test_analyze.py:77-79) all agree. Half-open `[since, until)` is a
  standard windowing convention.

**Verdict: PASS.** Where the spec and the sealed checklist are both silent, a
documented, internally consistent, tested interpretation cannot fail an
objective acceptance check; failing it would grade the grader's unstated
assumption, not the artifact. Runs 02–05 chose inclusive-`--until` and also
pass — both semantics are spec-permissible.

**Eval-design lesson (logged for lessons-learned):** A5 was decidable only by
accident of fixture design. v2's executable sealed acceptance must pin
boundary semantics in the spec or test both conventions as acceptable.

## Adjudicated correctness scores (final — not returned to graders)

| repo | adjudicated passes | correctness /5 |
|---|---|---|
| 01-atv-starterkit | 12/12 | 5 |
| 02-atv-phoenix | 12/12 | 5 |
| 03-working-skill-repo | 12/12 | 5 |
| 04-kprojects | 12/12 | 5 |
| 05-baseline | 12/12 | 5 |

## Run 1.5 — delta pass (06-gstack, 07-baseline-claude)

Graders agreed on 23 of 24 acceptance checks (2 repos × 12). Run 07 had no
disputes (both graders: 12/12, all PASS). One factual dispute on run 06,
adjudicated below in a fresh clone per protocol. Clone:
`/tmp/grade-consensus/06-gstack` @ HEAD (0886dc4).

### Dispute — repo 06, check A5 (errors + time window)

**fable1 claimed PASS:** window 07:00→14:00 UTC filtered correctly against
`sealed-fixture-fable1` — which, as in run 1, has no record exactly on a
window boundary, so the check could not distinguish inclusive from exclusive
`--until`.

**sol claimed FAIL:** "The window returned `(404,/alpha,2)` and 8 total
errors because `filter_entries` applies an exclusive upper bound and drops
the record exactly at `11:00:00`" (`sealed-fixture-sol`, whose ground truth
is defined "with both endpoints included" and places a 404 `/alpha` record
at exactly `2026-07-12T11:00:00`).

**What I ran:** fresh clone, `uv sync`, then `uv run loglens errors
sealed-fixture-sol/sealed.log --since 2026-07-12T10:00:00+00:00 --until
2026-07-12T11:00:00+00:00`. Output: alpha 2, beta 2, charlie 2, delta 1,
epsilon 1 — total 8; the 11:00:00 record is excluded. Sol's factual
observation **reproduces exactly**; the graders' raw observations do not
conflict (fable1's fixture was again blind to the boundary).

**Verdict: PASS — by direct application of run 1's Dispute 1 precedent.**
The situation is point-for-point the one adjudicated for repo 01: the spec
and `acceptance.md` are silent on boundary semantics, and repo 06's
half-open choice is deliberate, documented, and internally consistent —
README line 93 ("Bounds are **half-open** (`since <= t < until`)", with a
seam-chaining rationale), `--help` ("only requests strictly before this
time"), the `analyze.py` window docstring, and a dedicated test
(`tests/test_analyze.py:189` `test_bounds_are_half_open`). A documented,
tested, spec-permissible interpretation cannot fail an objective acceptance
check; ruling otherwise would also retroactively contradict the frozen
run-1 verdict that passed repo 01 for the identical choice. (Delta graders
correctly could not read `final.md`/`reconcile/`, so sol had no access to
that precedent — the disagreement is a protocol artifact, not a grading
error.)

**Eval-design lesson (again):** this is the second occurrence of the exact
failure mode run 1 predicted — A5 is decidable only by accident of fixture
design. v2's sealed acceptance must pin boundary semantics in the spec or
accept both conventions explicitly.

### Adjudicated correctness scores, run 1.5 (final — not returned to graders)

| repo | adjudicated passes | correctness /5 |
|---|---|---|
| 06-gstack | 12/12 | 5 |
| 07-baseline-claude | 12/12 | 5 |
