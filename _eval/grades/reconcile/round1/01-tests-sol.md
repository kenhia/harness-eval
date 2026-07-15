# Reconciliation round 1 — repo 01-atv-starterkit, dimension: tests — for grader: sol

You are grader **sol** in the harness-eval POC run 1. You and grader fable1
disagree by 2 points on the **tests** dimension for **01-atv-starterkit**.
This prompt is self-contained; you do not need any other context beyond your
own grading session.

**Your score: 3/5.** Your justification, verbatim:
> Twenty-five tests cover parsing, ties, JSON, exits, invalid timestamps,
> grouping, and histogram shape. However, the suite explicitly asserts the
> incorrect exclusive-`until` behavior, so it protects a spec divergence
> instead of catching it.

**Fable1's score: 5/5.** Fable1's justification, verbatim:
> 25 tests across parser/analyze/cli; explicitly cover tie-break
> (`test_top_by_ip_desc_with_tiebreak`), exit codes 1/2, stderr-vs-stdout
> separation, JSON single-document validity, time window, invalid `--since`
> — would catch real regressions

**Background fact (verified by the consensus adjudicator in a fresh clone):**
repo 01's exclusive `--until` is deliberate and consistent: documented in its
README ("`--since` (inclusive) and `--until` (exclusive)"), in the `--help`
text, and in the `analyze.errors` docstring, and asserted by
`tests/test_analyze.py:77-79`. The project spec is silent on window boundary
semantics — it pins the `top` tie-break explicitly but says nothing about
`--since`/`--until` inclusivity. "Both endpoints included" was a design
choice of your sealed fixture's ground truth, not a stated spec requirement.

**Crux question:** For the *tests* dimension specifically — given the spec's
silence, is a suite that asserts the repo's own documented, internally
consistent boundary contract "protecting a spec divergence", or is it doing
exactly what a regression suite should do (pinning documented behavior)?
Your 3/5 treats the exclusive-`until` interpretation as *incorrect*; if it is
instead *spec-permissible-but-different-from-your-fixture*, does a 2-point
dock still follow? Conversely, if you believe the plain reading of
`--until` implies inclusion and the suite should have caught that, defend
why that outweighs the suite's otherwise-broad coverage (tie-break, exit
codes, stream separation, JSON validity, invalid timestamps).

**Instruction:** Append a section `## Reconciliation round 1` to your own
grade file `_eval/grades/01-sol.md` containing your revised-or-defended
tests score (0–5, integer) and 2–5 sentences of rationale engaging with the
crux above. Do not change any other dimension.
