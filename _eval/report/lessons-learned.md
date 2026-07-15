# Lessons learned — harness eval POC run 1

Consolidates the setup-time findings from `_eval/notes/lessons.md`, run-log
observations, grading and consensus friction, and the grader-agreement
analysis into the design input for evaluation harness v2.

## From setup (recap of the seeded list)

1. **Environment isolation is the experiment, not hygiene.** kai's live
   `~/.copilot` had Phoenix globally installed; without symlink-swapped
   profiles, every run — including the baseline — would have been a Phoenix
   run and the whole POC silently invalid. v2: per-run containers or
   throwaway users, with an automated preflight that *proves* the
   environment is what the run label claims.
2. **Install scope is itself a finding.** Repo-local harnesses (StarterKit,
   KB, kprojects) are versionable and reviewable; global ones (Phoenix)
   bleed across projects and require the profile machinery above.
3. **Harness self-checks barely exist.** The registered Phoenix MCP binary
   was missing before the eval and nothing had noticed. v2 preflight should
   run each harness's doctor equivalent where one exists — and the absence
   of one is a scoreable fact.
4. **Runner choice is a confound** (Copilot-first harnesses forced Copilot
   CLI + Opus for all runs). v2 adds a harness × runner axis where
   dual-target harnesses exist.
5. **Commit-boundary discipline worked perfectly.** empty baseline →
   install commit → `pre-run` tag → agent-authored diff. Keep exactly as is.

## From the runs

6. **Eval-harness tension is measurable friction.** Run 03 hit conflicting
   instructions ("don't write in the harness project" vs "build here") and
   harness-owned `.github` files that failed the project's own lint; the
   agent burned time resolving *our* setup problem (and resolved it well —
   scoped `extend-exclude`, no out-of-mandate edits). The run log's
   "give-back" seconds are a hack; v2 should eliminate the tension (harness
   files outside the lint path by construction) rather than compensate.
7. **MCP availability is part of the environment definition.** Run 04 wrote
   a korg work item mid-run — korg/klams MCP was deliberately in both
   profiles as "standard machine environment," but that choice gave one
   harness a visible process artifact the others don't produce. v2 must
   decide per-cell whether ambient services are in or out, and record it.
8. **Manual timing is not reproducible.** Start/end times were reconstructed
   from memory ± the last commit; token/credit numbers were transcribed from
   a TUI usage panel. v2: headless scripted runs (`copilot -p …`) with
   wall-clock, session-log token parse, and `git diff --stat` captured
   automatically.

## From grading and consensus

9. **A 12-check objective list did not discriminate at frontier-model
   capability.** All five runs passed all 12 checks (after adjudication).
   The entire quality spread came from robustness edges *outside* the
   checklist — the naive-vs-aware datetime crash (three of five repos),
   boundary semantics, tested-vs-untested edges. v2 acceptance needs
   graduated difficulty: a core tier every competent run passes and a
   hard tier (adversarial inputs, unusual-but-valid ISO 8601 forms,
   locale/timezone traps) designed to spread the field.
10. **Sealed-but-unshared fixtures caused the only acceptance dispute.**
    Each grader generated their own fixture; sol's placed an error record
    exactly on the `--until` bound, fable1's had no boundary records at all.
    Same repo, same check, honest PASS and FAIL. The consensus adjudication
    ruled the repo's documented half-open window spec-permissible — but the
    root cause is that "filter correctly" was never pinned. v2: sealed
    acceptance is **one shared executable pytest suite** with explicit
    boundary semantics, run verbatim by every grader (or by CI, removing
    graders from objective checks entirely).
11. **One ambiguous spec point cost a 20-point grader swing.** Sol
    propagated the boundary reading into four dimensions of repo 01
    (correctness, tests, docs, process): 92 vs 72 pre-consensus. The
    reconciliation protocol worked — one round, both graders at 5 on the
    disputed dimension, with sol revising on the spec-silence argument and
    fable1 defending on regression-protection grounds — but the cheaper fix
    is specs that pin their own edge semantics (the spec pinned tie-breaks
    explicitly and got five identical, correct tie-break implementations;
    it left window bounds unpinned and got two conventions and a dispute).
12. **Grader disagreement was otherwise noise-band.** 33 of 35 non-
    correctness dimension cells were within 1 point. Mild systematic
    flavors: fable1 credited robustness-beyond-spec more (repo 04); sol
    read post-hoc artifacts more skeptically (01's plan, 04's roadmap) and
    was slightly more generous on mid-field efficiency. Two graders +
    mean-on-≤1 + reconcile-on-≥2 is a solid, cheap protocol — keep it.
13. **Efficiency judgment needs an anchor.** Both graders scored efficiency
    against the observed field (baseline = 5, StarterKit = 2), which is
    circular when the field is n=5. v2: define efficiency bands relative to
    the baseline cell's median across N reps, not grader intuition.
14. **"Weighted total" hides the story.** 88–92.5 reads as a dead heat, yet
    the artifacts differ meaningfully (tested tz-handling vs README examples
    that crash). Report dimension-level deltas and named qualitative
    findings alongside the headline number.

## v2 design (concrete)

- **Task-type matrix**: (a) greenfield build (run 1's cell), (b) bug-fix on
  a planted-bug repo, (c) behavior-preserving refactor on a messy repo,
  (d) resume-from-handoff — a half-finished project continued from its own
  docs. (d) is the cell where handoff-heavy harnesses (KB, kprojects
  sprints) should shine or be exposed; run 1 structurally couldn't test it.
- **N ≥ 3 reps per cell**; report medians and spread, not single runs.
- **Headless scripted runs**: one justfile executes the whole matrix
  (`copilot -p "$(cat prompt.md)"` per run dir, fresh profile per run).
- **Automated metric capture**: wall-clock, tokens/turns from session logs,
  `git diff --stat`, test count, coverage — zero human transcription.
- **Executable sealed acceptance**: a shared pytest suite with pinned edge
  semantics, two difficulty tiers, run identically for every repo; graders
  only score the subjective dimensions.
- **Preflight self-check step**: environment-isolation proof + harness
  doctor + MCP availability manifest, recorded into the run log.
- **Keep**: commit-boundary protocol (5), two-grader consensus protocol
  (12), the rubric's weights and anti-ceremony framing — all earned their
  keep in run 1.
