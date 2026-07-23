# run_03 suite defect + incident log

Same protocol as run_02: post-freeze suite defects are recorded and
adjudicated, never silently patched; every completed cell is re-run
under the fixed suite.

## S1 — project-root assumption (2026-07-22, found on cells 01 & 03)

**Symptom.** Cells 01-atv-starterkit and 03-working-skill-repo scored
**0/12 core, 0/9 hard** — every test ERRORed at session-fixture setup
with `uv sync failed: No pyproject.toml found`.

**Root cause: the suite, not the implementations.** Both agents built
complete, working CLIs but nested them one level down
(`loglens/pyproject.toml`) instead of at the repo root. The suite
hardcoded the repo root as the project root, so `uv sync` found nothing
and every test errored before touching the code.

**Adjudication.** The frozen loglens spec says "build this project
start to finish in the current repository" and does **not** pin the
directory level; a nested project is spec-permissible (same reasoning
class as precedent P1: don't fail a repo on a spec silence). Scoring a
working CLI 0/12 for a directory level would have been a suite artifact
reported as a capability finding — precisely the error this tier exists
to avoid.

**Fix.** `conftest.find_project_root()` accepts the repo root, else a
single `*/pyproject.toml` one level down; nesting prints a NOTE into
the acceptance output as a **scoreable process observation** (graders
weigh it under process/docs, correctness is unaffected). Regression-
checked against the run_01 trees: unchanged (20 passed / 1 known H9
failure on the same repos as before).

**Re-run under the fixed suite** (pre-fix archives in
`.scratch/run_03-pre-S1/`):

| cell | before | after |
|---|---|---|
| 01-atv-starterkit | 0/12, 0/9 | **11/12 core, 4/9 hard** |
| 02-atv-phoenix | 9/12, 4/9 | 9/12, 4/9 (unchanged — root-level) |
| 03-working-skill-repo | 0/12, 0/9 | **10/12 core, 6/9 hard** |
| 04-kprojects | 11/12, 8/9 | 11/12, 8/9 (unchanged — root-level) |

**Finding preserved:** nesting the project is itself a sub-frontier
behavior worth reporting — no run_01/run_02 frontier cell did it, two
of four Haiku cells did.

## I1 — interrupted run, cell 05 (2026-07-22)

The matrix stopped after cell 04: cell 05-baseline was mid-run when the
controlling ssh session died (cleo reboot). Evidence: 05 had an
uncommitted partial worktree and **no runlog** (run-eval.sh writes the
log only after the runner exits), while 06/07 were untouched. Not a
cell failure, not a matrix bug — a killed controller.

**Handling** (run_02 void protocol): partial worktree preserved to
`.scratch/run_03-void-05/partial-worktree.tar`, staging repo reset to
`pre-run`, no runlog written so `run-matrix.sh run_03` resumes cleanly
at 05 → 06 → 07. Cells 01–04 keep their runlogs and are skipped.

**Tooling follow-up (v3):** long matrix runs should survive controller
disconnects — run the matrix under `tmux`/`nohup` on kai, or have
run-matrix.sh detect and report an interrupted cell rather than leaving
a silent partial worktree.
