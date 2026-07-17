# Reconciliation round 1 — repo 01-atv-starterkit, dimension: tests — for grader: fable1

You are grader **fable1** in the harness-eval POC run 1. You and grader sol
disagree by 2 points on the **tests** dimension for **01-atv-starterkit**.
This prompt is self-contained; you do not need any other context beyond your
own grading session.

**Your score: 5/5.** Your justification, verbatim:
> 25 tests across parser/analyze/cli; explicitly cover tie-break
> (`test_top_by_ip_desc_with_tiebreak`), exit codes 1/2, stderr-vs-stdout
> separation, JSON single-document validity, time window, invalid `--since`
> — would catch real regressions

**Sol's score: 3/5.** Sol's justification, verbatim:
> Twenty-five tests cover parsing, ties, JSON, exits, invalid timestamps,
> grouping, and histogram shape. However, the suite explicitly asserts the
> incorrect exclusive-`until` behavior, so it protects a spec divergence
> instead of catching it.

**Background fact (verified by the consensus adjudicator in a fresh clone):**
repo 01 implements `--since` inclusive / `--until` exclusive. This is
documented in its README, `--help` text, and the `analyze.errors` docstring,
and asserted by `tests/test_analyze.py:77-79`. Sol's sealed fixture, whose
ground truth defined both endpoints as included, therefore observed a
boundary record at the exact `--until` timestamp being excluded. The project
spec is silent on window boundary semantics (unlike the `top` tie-break,
which it pins explicitly).

**Crux question:** For the *tests* dimension specifically — when a test suite
asserts a boundary semantic on a point where the spec is silent, is that (a)
legitimate regression protection of a documented contract, or (b) canonizing
an interpretation so that the suite can never catch the divergence sol
observed? Would the ideal suite have flagged the ambiguity (e.g. tested the
boundary record explicitly against the spec's plain reading)? Does your 5/5
survive the observation that the suite locks in the field's only
non-inclusive window semantics, or does it warrant a dock — and if so, is
sol's 3/5 the right size, given the suite's otherwise-broad coverage?

**Instruction:** Append a section `## Reconciliation round 1` to your own
grade file `_eval/grades/01-fable1.md` containing your revised-or-defended
tests score (0–5, integer) and 2–5 sentences of rationale engaging with the
crux above. Do not change any other dimension.
