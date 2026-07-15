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
