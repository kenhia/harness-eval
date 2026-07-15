# Lessons learned — running list

Seeded during setup (2026-07-14, before any runs). Add to this after every
run and during grading; this file becomes the "evaluation harness v2" design
input.

## Found during setup

1. **Global-scope harnesses contaminate everything.** kai's live `~/.copilot`
   already had Phoenix (19 skills, agent, MCP server) globally installed —
   the "baseline" run would have had Phoenix active. Environment isolation is
   not optional; it is the first thing an eval harness must provide. POC
   answer: symlink-swapped `~/.copilot` profiles. Real answer: per-run
   containers or throwaway users.
2. **Install scope varies wildly and is itself a finding.** StarterKit and
   working-skill-repo install repo-local; Phoenix is global-only; kprojects
   is repo-local + one optional global skill. Repo-local scope is a
   *feature* for teams (versioned, reviewable, no cross-project bleed).
3. **The registered Phoenix MCP binary was missing** (`target/release`
   cleaned at some point) — a silent global breakage no harness surfaced.
   Eval harness v2 should include a preflight "harness self-check" step
   (Phoenix has `phoenix-mcp doctor`; others have nothing equivalent).
4. **Runner choice is a confound.** All three external harnesses are
   Copilot-first; "give the prompt to Opus" required picking Copilot CLI +
   Opus model as the runner so every harness's machinery actually loads.
   A harness×runner matrix is a future axis.
5. **working-skill-repo's documented install is global**, but the installer
   has an undocumented-ish `--target repo` that is repo-local — reading the
   installer beat reading the README.
6. **Commit-boundary discipline enables grading.** empty-baseline commit →
   install commit → `pre-run` tag makes "agent-authored work" a clean
   `git diff pre-run..HEAD`. Keep this in v2.

## To evaluate after run 1 (fill in)

- Did any harness's ceremony (plans, reviews, handoffs) measurably change the
  outcome vs baseline?
- Token burn per harness (harness overhead = tokens minus baseline tokens).
- Intervention count — which harness needed the most hand-holding?
- Grader agreement — where did Fable and GPT Sol diverge and why?

## v2 orchestration ideas (from the brainstorm)

- **Task-type matrix**, not just greenfield: (a) greenfield build (this run),
  (b) bug-fix session on a planted-bug repo, (c) refactor-without-behavior-
  change on a messy repo, (d) "resume a half-finished project from its own
  docs" — the last one is where handoff-heavy harnesses (KB, kprojects
  sprints) should shine or be exposed.
- **N≥3 repetitions per cell** — single runs are noise; report medians.
- **Scripted runs**: `copilot -p "$(cat prompt.md)"`-style headless
  invocation per run dir, so a justfile can execute the whole matrix.
- **Automatic metrics capture**: wall-clock, session log parse for
  tokens/turns, `git diff --stat`, test/coverage counts — no human notes.
- **Sealed acceptance as executable pytest**, not a checklist a grader
  interprets.
- **Cross-model axis** later: same harness × {Opus, Sonnet, GPT-x} to test
  the "harnesses matter less as models improve" hypothesis explicitly.
