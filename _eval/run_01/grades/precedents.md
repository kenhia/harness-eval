# Grading precedents — run_01 field

Adjudicated **interpretations** (not scores) from consensus sessions.
Delta graders for this field READ THIS FILE before grading — settled
questions are not re-litigated (`final.md` and `reconcile/` remain
off-limits). Consensus sessions append here.

## P1 — time-window boundary semantics (A5)

Where the project spec and `acceptance.md` are silent on `--since/--until`
boundary semantics, a repo's boundary choice **passes** A5 if it is
deliberate, documented, and internally consistent (e.g. a documented
half-open window `since <= t < until`). Do not fail a repo because your
sealed fixture places a record exactly on a bound.

Established: run 1 consensus (repo 01, dispute 1); reapplied verbatim in
run 1.5 (repo 06). Root cause is a spec-authoring gap — future specs pin
their own edge semantics (see lessons 10, 11, 18).
