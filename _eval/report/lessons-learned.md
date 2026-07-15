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

## From the run 1.5 expansion (06-gstack + 07-baseline-claude, 2026-07-15)

The first exercise of the ADDING-A-HARNESS.md incremental process: one new
harness, one new runner, one new control, delta grading, frozen priors.
What it revealed:

15. **The runner-covariate rule worked — and the covariate turned out to be
    the biggest effect in the eval.** Pairing the native-runner harness with
    a same-runner control (07) made 06 scoreable at all: efficiency was
    judged against 07 plus wall-clock instead of incomparable cost units.
    But the control itself then topped the whole field (95), beating every
    harness on either runner — the runner (system prompt, review habits,
    tool loop) is a quality intervention of the same order as the harnesses
    being measured. v2's harness × runner matrix is not optional; a
    harness ranking that doesn't control for runner is measuring the wrong
    variable.
16. **HOME-sandbox profiles work for Claude Code but have sharp edges.**
    Three found during setup: (a) a sandbox does NOT inherit the real
    account's permission settings — each profile needs its own
    `settings.json` with bypass-permissions for hands-off runs; (b) with a
    HOME override, Claude Code reads `$HOME/.claude.json` at the profile
    ROOT, not `.claude/.claude.json`; (c) **build a profile under its final
    name** — gstack's installer baked absolute profile paths into its
    session hook and compiled `browse` binary, so renaming a profile
    strands them. All three are now codified in ADDING-A-HARNESS.md §2.
17. **The Phoenix-contamination mistake almost repeated with new paint.**
    gstack's global piece initially landed in the harness-free
    `claude-clean` profile — which would have made the control run a gstack
    run, invalidating exactly the 06-vs-07 comparison the expansion existed
    to make. Caught at setup and split into `claude-clean`/`claude-gstack`
    (the staging repo's install commit still records the near-miss). The
    generalized rule: one profile per environment flavor, and a
    harness-free control may never share a profile with anything. Lesson 1's
    "automated preflight that proves the environment" applies to every new
    runner, not just Copilot.
18. **Frozen-prior delta grading re-litigates adjudicated questions.** The
    A5 window-boundary dispute recurred *identically* on repo 06: sol's
    fixture probes the boundary, the repo documents half-open bounds, sol
    scored FAIL — and, correctly barred from reading `final.md` and
    `reconcile/`, had no way to know run 1 had already adjudicated the
    exact question. The adjudication was a copy-paste of precedent; the
    grader tokens were wasted. v2: adjudicated *interpretations* (not
    scores) belong in a sealed addendum future graders DO read — or better,
    in the executable acceptance suite where the question can't arise.
19. **Run-log discipline decays without automation.** Both 1.5 run logs
    left the `claude-code version` field unfilled even though the template
    asked for it (kai's binary reported 2.1.209 at consensus time the same
    day, so the build is *probably* known — but "probably" is the point).
    With auto-updating CLIs, tool version is a real covariate. v2 preflight
    must capture `<runner> --version`, model ID, and profile name
    mechanically, not as a checklist item.
20. **Calibration-from-sheets held up.** The delta graders' scores on 07
    landed within 2 points of each other (96/94) and their 06 disagreement
    traced to run 1's two known fault lines (boundary semantics, sol's
    stricter machinery prior) rather than to scale drift. One
    reconciliation round closed the only ≥2 gap with both graders moving
    to the same score from opposite directions. Two-grader + mean-on-≤1 +
    reconcile-on-≥2 survives the frozen-delta regime; keep it for v2.
21. **The checklist still doesn't discriminate — now 7 for 7.** Both new
    runs passed 12/12 (after the precedent adjudication). Two runs, two
    graders, and a consensus pass added zero information via the objective
    tier; all signal again came from robustness edges and cost. This
    hardens lesson 9: v2's hard acceptance tier is the single
    highest-value change to the eval.
