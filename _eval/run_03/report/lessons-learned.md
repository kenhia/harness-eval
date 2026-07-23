# Lessons learned — run 03 (Haiku 4.5 tier)

Continues the series (1–21 run_01, 22–33 run_02). Input for eval v3.

## The finding itself

34. **Harness value is a function of two variables, not one.** Run 03's
    cross-tier deltas split cleanly by harness *weight*: light
    convention/skill harnesses gained sharply as capability dropped (KB
    0 → +15.25, kprojects +0.5 → +7.75) while heavy go-command
    harnesses lost more (Phoenix −3 → −12, gstack −8 → −9.25). The
    mechanism: **encoded knowledge substitutes for capability; an
    autonomous process demands it.** Any future harness claim should
    name the capability tier it applies to.
35. **A weaker model fails differently, not just more.** The tier's
    organizing failure — every repo solving exactly one half of the
    naive/aware datetime problem, in complementary halves — did not
    exist at the frontier, where the same edge produced scattered
    misses. Sub-frontier behaviors absent from both frontier runs:
    projects nested a directory down, dev tooling in optional extras
    breaking clean-env checks, committed `.pyc`, dirty trees at done.
    **Capability tiers need their own failure taxonomies**, not a
    difficulty knob on the frontier one.
36. **The instrument must be re-calibrated per tier, not just reused.**
    Run_02's correctness mapping ("12/12 → 4, any failure caps at ≤3")
    would have flattened all seven Haiku cells into one band. A
    tier-calibrated mapping table preserved a 27-point spread — and
    kept the mapping mechanical, so correctness still matched 7/7
    across graders.

## Method

37. **A retro-validated suite is worth building before the tier that
    needs it.** Porting run 1's checklist to executable form and
    running it against the seven *already-graded* frontier trees proved
    calibration (7/7 at core 12/12) and, as a bonus, retroactively
    discriminated run 1's dead heat: H9 flagged exactly the repos its
    graders had hand-flagged. Free validation from artifacts you
    already have.
38. **Graders find suite defects that self-testing cannot.** S2 — the
    JSON fallback matching only argparse's error wording — survived my
    regression suite, the retro-validation, and a full field run,
    because every run_01 tree happened to use argparse. A grader hit it
    on the first click-based CLI, **flagged it for adjudication instead
    of self-adjudicating, and left the scores as supplied.** The other
    grader independently observed the same symptom with the opposite
    attribution (repo fault, not suite fault) — that disagreement is
    what made it certain. Design grading prompts so raising a defect is
    an expected, protocol-blessed move.
39. **Zero-tolerance for silently-zero results.** S1 scored two working
    CLIs 0/12 because they nested the project one level. A 0 that means
    "harness error" is indistinguishable from a 0 that means "total
    failure" unless you look — and at a sub-frontier tier, "total
    failure" is *plausible*, which is exactly when the mistake gets
    believed. Any all-zero cell should be treated as suspect until
    proven.
40. **Mechanical vetting of grader output is cheap and worth it.**
    `vet-grades.py` checks sheet completeness, score ranges, the
    correctness mapping against the acceptance tallies, weighted-total
    arithmetic, text integrity (control chars, mojibake, degeneration,
    truncation), and independence leakage. Self-testing it against
    known-good run_02 sheets caught bugs in the checker itself.
41. **Shared memory is an independence hazard when graders run
    locally.** klams had indexed this repo — including prior graders'
    sheets, reconciliation prompts with verbatim justifications, and
    `final.md` — and the standing global instruction is to search it
    first. Grader prompts now carry an explicit blackout overriding
    that. Any shared retrieval layer must be considered part of the
    grading environment.
42. **Interrupted runs need a signature, not a guess.** I1 (controller
    ssh death mid-cell) was identifiable mechanically: uncommitted
    worktree + no runlog, because the runlog is written only after the
    runner exits. Long matrices should run under tmux; the driver
    should detect and report a partial cell rather than leaving it
    silent.

## Carried forward

43. **N≥3 remains the top debt.** Three consecutive rounds of zero
    reconciliations mean grader noise is no longer the limiting factor
    — trajectory variance is (run_02 measured 44% wall-clock spread on
    one cell, with reps landing on opposite sides of a decisive check).
    The local-model tier (free inference) is the affordable place to
    finally buy reps.
44. **Cross-tier comparisons need a designated statistic, decided
    before grading.** Assigning the harness-minus-control delta to the
    consensus session — and explicitly barring raw-total comparison —
    kept the tier's headline honest and stopped graders from
    anchoring across capability classes.
