# Do agent harnesses earn their keep? — Harness eval POC, run 1

*harness-eval, 2026-07-15. Five identical greenfield builds, one per harness,
same model, same prompt. Two independent LLM graders, an objective acceptance
checklist, and a consensus pass.*

## The question

Agent harnesses — skill packs, planning rituals, sprint records, handoff
documents — promise better outcomes from the same model. Each one also costs
tokens, wall-clock, and attention. This POC asks the simplest version of the
question: **give the same capable model the same task five times, once under
each of four harnesses and once bare — does any harness produce a better
artifact, and at what cost?**

## Method

**Task.** Build `loglens`, a Python CLI that analyzes Combined Log Format
web-access logs: four subcommands (summary/top/errors/hourly), text and JSON
output, malformed-line accounting on stderr, spec-pinned exit codes, pytest,
ruff, README, and a one-command check. The spec deliberately hides traps:
a tie-break rule, a time-window filter, stdout/stderr separation. Every run
got the identical prompt body; each harness's own "go" incantation
(`/lfg`, `/phoenix-goal`, `kb-start`, ambient instructions, or nothing)
was the only difference.

**Contenders.**

| run | harness | scope |
|---|---|---|
| 01 | ATV-StarterKit 2.x | repo-local (94 installed files) |
| 02 | ATV-Phoenix | global (`~/.copilot` profile) |
| 03 | working-skill-repo (KB) | repo-local skills + AGENTS.md |
| 04 | kprojects | repo-local instruction block + sprints/ |
| 05 | baseline | none — raw prompt |

**Runner.** GitHub Copilot CLI on kai with Claude Opus 4.8 selected. Three of
the four harnesses are Copilot-first; running under a different CLI would
have silently disabled most of their machinery. This is also a confound —
see threats.

**Isolation.** The single most important setup finding predates the first
run: kai's live `~/.copilot` had Phoenix globally installed — skills, agent,
MCP server — meaning *every* run including the baseline would have had
Phoenix active. The fix was symlink-swapped profiles (`use-profile.sh`):
a `clean` profile (auth + standard MCP, zero skills/agents) for runs
01/03/04/05 and a `phoenix` profile for run 02 only. Personal skills were
excluded from both. Without this, the entire eval would have been silently
invalid: environment isolation is not hygiene, it is the experiment.

**Protocol.** Empty baseline commit → harness install commit → `pre-run` tag
→ paste prompt → hands off. Everything after `pre-run` is agent-authored;
graders diff exactly that. Zero interventions occurred in any run.

**Grading.** Two independent graders — Fable (fable1) and GPT Sol (sol) —
each ran the 12 sealed acceptance checks against their own independently
generated fixture, then scored a 7-dimension weighted rubric. A consensus
session (this author) adjudicated factual disputes by re-running checks in
fresh clones, drove one reconciliation round, and computed final scores.

## Results

All five runs delivered a working, lint-clean, fully documented tool with
zero interventions, and — after adjudication — **all five passed all 12
acceptance checks.** The objective checklist did not discriminate at all.

| rank | repo | correct | quality | tests | docs | process | effcy | auton | **total** |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|
| 1 | 04-kprojects | 5 | 4.5 | 4.5 | 5 | 4.5 | 3.5 | 5 | **92.5** |
| 2= | 03-working-skill-repo | 5 | 4 | 4 | 5 | 5 | 4.5 | 5 | **92** |
| 2= | 05-baseline | 5 | 4 | 4 | 5 | 4.5 | 5 | 5 | **92** |
| 4 | 02-atv-phoenix | 5 | 4 | 4 | 4.5 | 5 | 3.5 | 5 | **89** |
| 5 | 01-atv-starterkit | 5 | 4.5 | 5 | 4.5 | 3.5 | 2 | 5 | **88** |

A 4.5-point spread on a 100-point scale, with n=1 per cell, is **inside the
noise floor**. The honest headline is not "kprojects wins"; it is: *the
baseline tied for second, and every harness's margin over it is smaller than
the cost it added.*

Where the runs actually differed was robustness beyond the checklist. An
offset-less ISO 8601 window bound (`--since 2026-07-14T07:00:00`, no
timezone) crashes runs 02, 03, and 05 with a raw naive-vs-aware `TypeError`
— in run 02's case, the README's own example is the crashing form. Run 04
normalizes naive timestamps to UTC in a documented helper *and is the only
repo that tests that edge*; run 01 documents explicit half-open window
semantics end to end. That — not the acceptance list — is where the two
process-heavy runs spent their extra tokens, and it is the only quality
signal in the data that separates any run from any other.

## What the machinery actually did

**01 · ATV-StarterKit — the full ritual, at full price.** The `/lfg` pipeline
ran exactly as designed: a 220-line plan document with requirements tracing
(`docs/plans/…-loglens-cli-plan.md`), a feature branch, implementation, a
genuine self-review loop — the fix commit `a99b165` (invalid-date handling →
clean exit 2) is real evidence the review caught something — then a
completion-marking docs commit and a self-merge. The result is the field's
best-tested artifact with explicitly documented boundary semantics. It also
cost 440 credits and 10m54s: 3.1× the baseline's credits for an artifact the
consensus scored *below* the baseline, because the rubric (rightly) charges
for efficiency. Verdict in one line: deepest process, priciest ticket,
and the plan prose never repaid its token bill.

**02 · ATV-Phoenix — machinery outside, textbook diff inside.** Phoenix's
apparatus lives globally, and the agent kept it there — its last commit adds
a `.gitignore` for the Phoenix workspace, leaving a clean five-commit
conventional sequence (scaffold → parser → analysis → CLI → chore). Both
graders independently called the commit hygiene the best of the field. But
nothing in the artifact shows the machinery *doing* anything: no plan, no
review loop, no robustness beyond the spec letter — plus the field's most
embarrassing flaw, a README example that crashes the tool. Mid-field cost
(265 credits, 1.9× baseline) for a fine, unremarkable result.

**03 · working-skill-repo — the cheapest harness, and the best judgment
call.** Five focused `loglens:`-prefixed commits, complete docs, 12/12 — at
216 credits (1.5× baseline), the least expensive harness run. Its
distinguishing moment was meta: the eval's own setup created a tension
(harness-owned `.github` files failing `ruff` on line length), and the agent
resolved it *in-repo* with a scoped `extend-exclude` instead of "fixing"
files outside its mandate. The run log gives back ~15s for that friction,
which was ours, not the harness's.

**04 · kprojects — the sprint record that wasn't ceremony.** Three
conventional commits (feat → test → docs), a korg work item (#445) filed via
MCP, and `sprints/001-loglens-cli.md` — which both graders, from opposite
priors, agreed contains *real* decisions: the UTC-normalization choice, the
stderr policy, the tie-break orders, each verifiable in the code. The
naive-timestamp discipline (handled, documented, tested — unique in the
field) is exactly the robustness the eval's hidden traps were designed to
detect. Fable1 scored it the best artifact of the five (96); sol docked the
roadmap bookkeeping as post-hoc (89). Consensus: narrow winner at 1.76×
baseline cost.

**05 · baseline — the control that set the bar.** No harness, no plan, no
prescribed process: three coarse commits, the thinnest test suite (23), one
typing wart (`lines: object` + `# type: ignore`), the same naive-datetime
crash as 02/03 — and 12/12 acceptance, a complete README, clean layering,
in 4m32s and 143 credits. Every harness above it must justify itself against
this number, and in run 1, none decisively did.

## Harness overhead

Baseline = 143 credits, 4m32s, 1.3M tokens up, for a 12/12 artifact.

| run | credits | × base | wall | × base | tokens ↑ | overhead buys |
|---|---:|---:|---:|---:|---:|---|
| 01 | 440 | 3.08× | 10m54s | 2.40× | 4.7M | plan doc, review loop (1 real catch), best tests, +0 acceptance |
| 02 | 265 | 1.85× | 7m12s | 1.59× | 2.9M | commit hygiene, +0 acceptance |
| 03 | 216 | 1.51× | 6m40s | 1.47× | 2.1M | commit hygiene, docs, +0 acceptance |
| 04 | 252 | 1.76× | 6m52s | 1.51× | 2.8M | decision record, tz robustness + its test, +0 acceptance |
| 05 | 143 | 1× | 4m32s | 1× | 1.3M | — |

Read that last column literally: **no harness improved the acceptance
outcome.** The overhead bought process artifacts and, in two cases (01, 04),
genuine robustness beyond the checklist. Whether ~110–300 extra credits per
task for that robustness is a good trade is a per-team judgment; for a task
of this size, the model alone was sufficient.

## Threats to validity

1. **n = 1 per cell.** One run per harness; wall-clock and credit numbers
   have unknown variance. The 4.5-point rubric spread is smaller than
   plausible run-to-run noise. Nothing here supports a strong ranking claim.
2. **Author conflict of interest.** The task spec, the eval design, and one
   contender (kprojects) — plus korg, which run 04 wrote a work item to —
   share an author. The winner is the evaluator's own harness. The blinding
   (independent graders, sealed checks) mitigates but cannot remove this;
   treat run 04's #1 with corresponding suspicion.
3. **Harness artifacts self-identify.** Sprint docs, phoenix workspaces, and
   plan files tell a grader exactly which harness ran. True blinding is
   impossible; the rubric compensates by demanding artifact-grounded
   justifications, but priors leak.
4. **Runner choice favors Copilot-first harnesses.** Everything ran under
   Copilot CLI because three harnesses require it. A harness × runner matrix
   is future work.
5. **Grader fixtures were not shared.** Each grader generated their own
   sealed fixture; one placed a record exactly on a window boundary and one
   did not, which single-handedly produced the eval's only acceptance
   dispute and a 20-point pre-consensus swing on repo 01. The spec was
   silent on boundary semantics; the adjudication (see
   `grades/acceptance-adjudication.md`) ruled both interpretations
   permissible. v2 needs *executable* shared acceptance tests with pinned
   semantics.
6. **Manual timing and self-reported token data.** Start/end times were
   hand-recorded (±5s); credit/token figures come from the CLI's own usage
   panel and cannot be independently audited.
7. **One task type.** Greenfield-from-spec is the setting where harness
   ceremony should matter *least* — the model needs no context recovery, no
   archaeology, no coordination. Handoff-heavy harnesses were structurally
   disadvantaged; see lessons-learned for the v2 task matrix.

## Conclusion

With a frontier model on a well-specified, single-session greenfield task,
**harness machinery did not change what shipped — it changed how much it
cost to ship it.** The two harnesses that spent their overhead on substance
(kprojects' decision record and timezone discipline, StarterKit's review
loop and test depth) produced measurably more robust artifacts at the
margins; the baseline matched everyone on every sealed check at a third of
the price. The interesting hypothesis for run 2 is that this inverts on
resume-from-handoff and long-horizon tasks, where the machinery's real
product — durable context — is actually consumed. Run 1 cannot test that;
it only establishes the control condition every future cell must beat.
