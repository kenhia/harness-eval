# Do agent harnesses earn their keep? — Harness eval POC, runs 1 & 1.5

*harness-eval, 2026-07-15. Seven identical greenfield builds: five harnesses,
two bare-model controls, two runners, same model, same prompt. Two
independent LLM graders, an objective acceptance checklist, and a consensus
pass. Run 1 covered the five Copilot-runner cells; run 1.5 added a
Claude-Code-native harness and its matching control.*

## The question

Agent harnesses — skill packs, planning rituals, sprint records, handoff
documents — promise better outcomes from the same model. Each one also costs
tokens, wall-clock, and attention. This POC asks the simplest version of the
question: **give the same capable model the same task, once under each
harness and once bare — does any harness produce a better artifact, and at
what cost?**

## Method

**Task.** Build `loglens`, a Python CLI that analyzes Combined Log Format
web-access logs: four subcommands (summary/top/errors/hourly), text and JSON
output, malformed-line accounting on stderr, spec-pinned exit codes, pytest,
ruff, README, and a one-command check. The spec deliberately hides traps:
a tie-break rule, a time-window filter, stdout/stderr separation. Every run
got the identical prompt body; each harness's own "go" incantation
(`/lfg`, `/phoenix-goal`, `kb-start`, ambient instructions, `/autoplan`, or
nothing) was the only difference.

**Contenders.**

| run | harness | runner | scope |
|---|---|---|---|
| 01 | ATV-StarterKit 2.x | Copilot CLI | repo-local (94 installed files) |
| 02 | ATV-Phoenix | Copilot CLI | global (`~/.copilot` profile) |
| 03 | working-skill-repo (KB) | Copilot CLI | repo-local skills + AGENTS.md |
| 04 | kprojects | Copilot CLI | repo-local instruction block + sprints/ |
| 05 | baseline (control) | Copilot CLI | none — raw prompt |
| 06 | gstack | Claude Code | repo CLAUDE.md routing + hook; global skills in a HOME-profile |
| 07 | baseline (control) | Claude Code | none — raw prompt |

**Runner.** Run 1 used GitHub Copilot CLI on kai with Claude Opus 4.8
selected: three of the four run-1 harnesses are Copilot-first, and running
under a different CLI would have silently disabled their machinery. Run 1.5
introduced the second runner: gstack is Claude-Code-native, so per the
runner-covariate rule (ADDING-A-HARNESS.md §1) it ran under Claude Code CLI
with the same model (claude-opus-4-8) — together with `07-baseline-claude`,
a no-harness control on that runner. The pairing is what makes the covariate
analyzable: **05 vs 07 isolates the runner effect on a bare baseline, and
06 vs 07 isolates gstack's contribution on its native runner.** Efficiency
units (Copilot credits vs Claude Code dollars/tokens) are not comparable
across runners; graders scored 06 against 07 plus absolute wall-clock.

**Isolation.** The single most important setup finding predates the first
run: kai's live `~/.copilot` had Phoenix globally installed — skills, agent,
MCP server — meaning *every* run including the baseline would have had
Phoenix active. The fix was symlink-swapped profiles (`use-profile.sh`):
a `clean` profile (auth + standard MCP, zero skills/agents) for runs
01/03/04/05 and a `phoenix` profile for run 02 only. Personal skills were
excluded from both. Without this, the entire eval would have been silently
invalid: environment isolation is not hygiene, it is the experiment.

Run 1.5 ported the same discipline to Claude Code as HOME-sandboxes
(`env HOME=<profile> claude …`): `claude-clean` for the control and a
separate `claude-gstack` carrying gstack's global piece. The split exists
because the near-miss almost repeated — gstack's global skills initially
landed in the harness-free `claude-clean` profile (the staging repo's
install commit still records it), which would have made the control a
gstack run. It was caught at setup and split into two profiles, one
environment flavor each.

**Protocol.** Empty baseline commit → harness install commit → `pre-run` tag
→ paste prompt → hands off. Everything after `pre-run` is agent-authored;
graders diff exactly that. Zero interventions occurred in any run.

**Grading.** Two independent graders — Fable (fable1) and GPT Sol (sol) —
each ran the 12 sealed acceptance checks against their own independently
generated fixture, then scored a 7-dimension weighted rubric. A consensus
session (this author) adjudicated factual disputes by re-running checks in
fresh clones, drove reconciliation rounds, and computed final scores. Run
1.5 used fresh grader sessions under the same identities, calibrated from
their predecessors' written sheets; run-1 scores were frozen and never
re-derived.

## Results

All seven runs delivered a working, lint-clean, fully documented tool with
zero interventions, and — after adjudication — **all seven passed all 12
acceptance checks.** The objective checklist did not discriminate at all.

| rank | repo | runner | correct | quality | tests | docs | process | effcy | auton | **total** |
|---|---|---|---:|---:|---:|---:|---:|---:|---:|---:|
| 1 | 07-baseline-claude (control) | Claude | 5 | 4 | 5 | 5 | 5 | 4.5 | 5 | **95** |
| 2 | 04-kprojects | Copilot | 5 | 4.5 | 4.5 | 5 | 4.5 | 3.5 | 5 | **92.5** |
| 3= | 03-working-skill-repo | Copilot | 5 | 4 | 4 | 5 | 5 | 4.5 | 5 | **92** |
| 3= | 05-baseline (control) | Copilot | 5 | 4 | 4 | 5 | 4.5 | 5 | 5 | **92** |
| 5 | 02-atv-phoenix | Copilot | 5 | 4 | 4 | 4.5 | 5 | 3.5 | 5 | **89** |
| 6 | 01-atv-starterkit | Copilot | 5 | 4.5 | 5 | 4.5 | 3.5 | 2 | 5 | **88** |
| 7 | 06-gstack | Claude | 5 | 4 | 5 | 4.5 | 4.5 | 1.5 | 5 | **87** |

Within the run-1 field the spread is 4.5 points — **inside the noise floor**
of n=1 per cell, and that read is unchanged. Run 1.5 stretches the total
spread to 8 points, but both extremes are Claude-runner entries, so the
extra spread is runner + harness, not harness alone. The honest headline
hardened rather than changed: *both bare controls now beat or match every
harness on their own runner.*

Where the runs actually differed was robustness beyond the checklist. An
offset-less ISO 8601 window bound (`--since 2026-07-14T07:00:00`, no
timezone) crashes runs 02, 03, and 05 with a raw naive-vs-aware `TypeError`
— in run 02's case, the README's own example is the crashing form. Run 04
normalizes naive timestamps to UTC in a documented helper and tests that
edge; run 01 documents explicit half-open window semantics end to end. Run
1.5 sharpened this signal: **both Claude-runner runs — including the bare
control — handle the naive-timestamp edge**, and both carry a self-review
loop that demonstrably changed the outcome (06's reviewer findings became a
deliberate deferral record; 07's final commit fixed two real bugs its own
review caught). The robustness that in run 1 separated the two
process-heavy harnesses from the field showed up on the second runner
without any harness at all.

### Runner effect (05 vs 07)

Same bare model, same prompt, no harness — only the CLI differs.

| dim | 05 (Copilot) | 07 (Claude Code) | Δ |
|---|---:|---:|---:|
| tests | 4 | 5 | +1 |
| process | 4.5 | 5 | +0.5 |
| efficiency | 5 | 4.5 | −0.5 |
| everything else | — | — | 0 |
| **total** | **92** | **95** | **+3** |

The Claude-runner control wrote 77 tests to 05's 23, produced a seven-commit
scaffold → layers → tests → docs → self-review sequence to 05's three coarse
commits, and its unprompted review commit (`00f8445`) fixed a histogram
rendering bug and a JSON typing inconsistency — plus two tests that weren't
testing what they claimed. It also dodged the naive-timestamp crash that
bites 05. The price was wall-clock: 8m38s vs 4m32s. With n=1 and an
unrecorded tool-build delta, +3 points is a direction, not a measurement —
but the direction is that the runner (its system prompt, review habits, and
tool loop) is itself a quality intervention of the same order as the
harnesses being measured.

### gstack vs its own baseline (06 vs 07)

Same runner, same model — the harness is the only variable, which makes this
the cleanest harness-contribution readout in the eval.

| dim | 07 (control) | 06 (gstack) | Δ |
|---|---:|---:|---:|
| docs | 5 | 4.5 | −0.5 |
| process | 5 | 4.5 | −0.5 |
| efficiency | 4.5 | 1.5 | −3 |
| everything else | — | — | 0 |
| **total** | **95** | **87** | **−8** |

gstack's machinery improved no dimension over the bare control. What it
bought was margin: the field's most hardened parser (escaped-quote-safe
regex, locale-proof month parsing, hostile-byte and terminal-escape
handling), 137 tests to the control's 77, and the eval's best deferred-work
artifact. What it cost was 2.2× the control's wall-clock (19m05s vs 8m38s)
and roughly 3× its tokens and dollars ($10.84 vs $3.61), with ~8 minutes of
up-front analysis before the first edit — planning depth sized for a much
larger system. This is run 01's robustness-for-spend trade replayed on the
other runner, which is not a coincidence: ATV-StarterKit bundles gstack
(see threats).

## What the machinery actually did

**01 · ATV-StarterKit — the full ritual, at full price.** The `/lfg` pipeline
ran exactly as designed: a 220-line plan document with requirements tracing
(`docs/plans/…-loglens-cli-plan.md`), a feature branch, implementation, a
genuine self-review loop — the fix commit `a99b165` (invalid-date handling →
clean exit 2) is real evidence the review caught something — then a
completion-marking docs commit and a self-merge. The result is run 1's
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
naive-timestamp discipline (handled, documented, tested — unique in run 1's
field) is exactly the robustness the eval's hidden traps were designed to
detect. Fable1 scored it the best artifact of the five (96); sol docked the
roadmap bookkeeping as post-hoc (89). Consensus: run 1's narrow winner at
1.76× baseline cost.

**05 · baseline — the control that set the bar.** No harness, no plan, no
prescribed process: three coarse commits, the thinnest test suite (23), one
typing wart (`lines: object` + `# type: ignore`), the same naive-datetime
crash as 02/03 — and 12/12 acceptance, a complete README, clean layering,
in 4m32s and 143 credits. Every harness above it must justify itself against
this number, and in run 1, none decisively did.

**06 · gstack — the strongest artifact, at the field's worst exchange
rate.** gstack's repo footprint is small but coercive: a CLAUDE.md that
demands a global-install check before any work and a `PreToolUse` hook that
hard-blocks skill use if gstack is missing. The `/autoplan` ritual spent ~8
minutes analyzing before the first edit, then delivered six textbook
commits (scaffold → parser/reader → analyze/render → CLI → fixtures+tests →
docs) whose messages read like design reviews — each records a decision and
the failure mode it prevents. The plan itself stayed out of the repo; what
was committed instead is `TODOS.md`, the eval's best anti-ceremony artifact:
seven reviewer findings deliberately *not* built, each with a reason and a
sketch. The artifact is the eval's most robust — escaped-quote-safe parsing,
locale-proof months, hostile-byte handling, 137 tests including a pinned
half-open boundary — and its one code-quality dock (a README memory claim
the code doesn't honor) survived reconciliation. But on the same runner, the
bare control matched it on every consensus dimension except docs/process
(where gstack scored *lower*) and shipped in less than half the time at a
third of the cost. The machinery bought hardening the spec never asked for.

**07 · baseline-claude — the control that won the eval.** No harness: seven
clean commits ending in an unprompted self-review that fixed two real,
user-visible bugs and two self-deceiving tests. 77 tests (triple 05's),
naive timestamps normalized instead of crashing, honest docs whose examples
all work. 8m38s, $3.61, zero thrash. Its remaining warts are small
(locale-dependent `%b` month parsing, a str/int status round-trip between
layers). As a control it did its analytical job almost too well: it set an
efficiency anchor no harness on its runner approached, and a quality bar
none exceeded.

## Harness overhead

Run 1 (Copilot runner) — baseline = 143 credits, 4m32s, 1.3M tokens up:

| run | credits | × base | wall | × base | tokens ↑ | overhead buys |
|---|---:|---:|---:|---:|---:|---|
| 01 | 440 | 3.08× | 10m54s | 2.40× | 4.7M | plan doc, review loop (1 real catch), best tests, +0 acceptance |
| 02 | 265 | 1.85× | 7m12s | 1.59× | 2.9M | commit hygiene, +0 acceptance |
| 03 | 216 | 1.51× | 6m40s | 1.47× | 2.1M | commit hygiene, docs, +0 acceptance |
| 04 | 252 | 1.76× | 6m52s | 1.51× | 2.8M | decision record, tz robustness + its test, +0 acceptance |
| 05 | 143 | 1× | 4m32s | 1× | 1.3M | — |

Run 1.5 (Claude Code runner) — baseline = $3.61, 8m38s, 43.5k output tokens.
Units are `/cost` dollars and output tokens, **not comparable to the credits
above**; the × columns compare within-runner only:

| run | cost | × base | wall | × base | tokens (out) | overhead buys |
|---|---:|---:|---:|---:|---:|---|
| 06 | $10.84 | 3.00× | 19m05s | 2.21× | 122.7k | hardened parser, 137 tests, TODOS.md, +0 acceptance |
| 07 | $3.61 | 1× | 8m38s | 1× | 43.5k | — |

Read the last columns literally: **no harness on either runner improved the
acceptance outcome.** The overhead bought process artifacts and, in three
cases (01, 04, 06), genuine robustness beyond the checklist. Whether that
robustness is worth ~2–3× the baseline's spend is a per-team judgment; for
a task of this size, the model alone was sufficient — on both runners.

## Threats to validity

1. **n = 1 per cell.** One run per harness; wall-clock and cost numbers
   have unknown variance. The rubric spreads are smaller than plausible
   run-to-run noise. Nothing here supports a strong ranking claim.
2. **Author conflict of interest.** The task spec, the eval design, and one
   contender (kprojects) — plus korg, which run 04 wrote a work item to —
   share an author. The top-ranked *harness* is the evaluator's own. The
   blinding (independent graders, sealed checks) mitigates but cannot
   remove this; treat run 04's position with corresponding suspicion.
3. **Harness artifacts self-identify.** Sprint docs, phoenix workspaces, and
   plan files tell a grader exactly which harness ran. True blinding is
   impossible; the rubric compensates by demanding artifact-grounded
   justifications, but priors leak.
4. **Runner is a covariate, only partially controlled.** Run 1 ran
   everything under Copilot CLI because three harnesses require it; run 1.5
   added Claude Code for gstack. The 07 control anchors that runner — 06
   is compared to 07, never to 01–05's cost numbers — but cross-runner
   comparisons (any Claude entry vs any Copilot entry) carry the full
   runner effect measured in §Runner effect, and the two runners' cost
   units are simply different currencies. A full harness × runner matrix
   is future work.
5. **Grader fixtures were not shared.** Each grader generated their own
   sealed fixture; one placed a record exactly on a window boundary and one
   did not, which single-handedly produced run 1's only acceptance dispute
   and a 20-point pre-consensus swing on repo 01 — **and then produced the
   identical dispute again in run 1.5 on repo 06.** The delta protocol
   correctly bars graders from reading consensus artifacts, so sol had no
   way to know the boundary question was already adjudicated. v2 needs
   *executable* shared acceptance tests with pinned semantics.
6. **Manual timing and self-reported token data.** Start/end times were
   hand-recorded (±5s); credit/token figures come from each CLI's own usage
   panel and cannot be independently audited. Both run-1.5 run logs left
   the `claude-code version` field unfilled (kai's binary reported 2.1.209
   at consensus time the same day, so the runs almost certainly shared one
   build — but it is formally unrecorded). Both CLIs auto-update, so
   later runs may sit on later tool/model builds than run 1.
7. **One task type.** Greenfield-from-spec is the setting where harness
   ceremony should matter *least* — the model needs no context recovery, no
   archaeology, no coordination. Handoff-heavy harnesses were structurally
   disadvantaged; see lessons-learned for the v2 task matrix.
8. **01 and 06 share DNA.** ATV-StarterKit bundles gstack in its install
   (run 01's tree includes `.gstack`), so runs 01 and 06 are not
   independent observations of unrelated harnesses. Their matching
   signature — deepest planning, top-tier robustness, worst-in-field
   efficiency, on different runners — is consistent with a family trait,
   but two correlated cells can't confirm it.
9. **Delta-grading continuity.** Run 1.5's graders were new sessions under
   the same identities, calibrated from their predecessors' written sheets
   rather than sharing run-1 session context. The observed consistency
   (07's near-agreement; 06's disagreement tracing to run 1's known fault
   lines) suggests the calibration held, but it is an approximation.

## Conclusion

With a frontier model on a well-specified, single-session greenfield task,
**harness machinery did not change what shipped — it changed how much it
cost to ship it.** Run 1.5 strengthened that conclusion from an unexpected
direction: the biggest quality delta in the whole eval came not from any
harness but from swapping the runner — the Claude Code control beat every
harness on either runner, matching the robustness edges (timezone handling,
self-review, deep tests) that run 1's process-heavy harnesses had spent 2–3×
baseline cost to obtain. The harnesses that spent their overhead on
substance (kprojects' decision record, StarterKit's review loop, gstack's
hardened parser) produced measurably more robust artifacts at the margins;
the bare controls matched everyone on every sealed check at a fraction of
the price. The interesting hypothesis for run 2 is unchanged: that this
inverts on resume-from-handoff and long-horizon tasks, where the
machinery's real product — durable context — is actually consumed. Runs 1
and 1.5 cannot test that; they establish the control conditions — now one
per runner — that every future cell must beat.
