# Grading precedents — run_03 field (Haiku 4.5 tier)

Read before grading (see rubric §Grader instructions). Inherits
run_01's precedents (`../../run_01/grades/precedents.md`) — P1 (window
boundary semantics) remains live wherever a spec is silent on bounds —
and run_02's (no new entries). New entries below were adjudicated
during run_03 (recorded in `../DEFECTS.md`, restated here so future
tiers inherit the interpretation without re-reading the defect log).

## P2 — project directory level is spec-permissible (S1, 2026-07-22)

Where the spec says "build this project in the current repository"
without pinning the directory level, a project nested one level down
(`loglens/pyproject.toml`) is **spec-permissible** — same reasoning
class as P1 (don't fail a repo on a spec silence). The acceptance suite
must locate the project rather than hardcode the repo root; nesting is
surfaced as a NOTE in the acceptance output and is a **scoreable
process observation** (convention/discoverability, weighed under
Process/docs), never a correctness failure.

## P3 — global-option placement and probe parser-agnosticism (S2/S2b, 2026-07-22)

Where a spec calls an option **global**, accepting it before the
subcommand (`loglens --format json summary …`) is a legitimate —
indeed conventional — reading; a CLI must not fail acceptance for
supporting only the leading form. Acceptance probes that fall back to
an alternative invocation must be **parser-agnostic** (retry on any
nonzero exit, keep the alternative only if it succeeds), never keyed to
one library's error wording. Corollary S2b: filename probes accept any
casing the underlying tool accepts (`justfile`/`Justfile`/`.justfile`).
Agent-owned residue is still scoreable: documenting an invocation form
the repo's own CLI rejects remains a real docs defect (03), and
packaging that breaks the repo's own checks in a clean environment
remains a real delivery defect (06).

## Run_03 consensus (2026-07-23)

No new interpretation precedents from consensus itself. All 42
non-mechanical dimension cells landed within 1 point (no reconciliation
held); the one factual discrepancy (cell 01's hard-miss count) was
resolved from the supplied acceptance output with no score impact (see
`adjudication.md`). The dirty-tree-at-done → Autonomy −1 treatment
applied to cells 01–04 follows run_02's pre-specified treatment, not a
new ruling.
