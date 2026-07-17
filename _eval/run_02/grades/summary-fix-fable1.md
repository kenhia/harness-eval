# Summary — run_02.1 fix-round delta — grader: fable1

Delta-only pass over the six fixed repos, graded in order 01→07 per
protocol (03 absent: passed run_02 26/26, nothing to fix). Acceptance
was uniform — every cell 14/14 core, 12/12 hard, 3/3 fix addendum from
the supplied `runs/NN-fix-acceptance.txt` — so fix correctness
differentiates only on how robustly the pass is achieved; all six fixes
landed at the XML parse layer (none patched the fetch loop or handler),
and all six detect truncation generically (open-element state at EOF),
so no correctness deductions were warranted. Spread comes from test
depth, mechanism cleanliness, scope hygiene, and cost. Per-repo detail
in `NN-fix-fable1.md`. No ranking commentary beyond the table per
protocol; synthesis belongs to consensus.

| repo | fix correctness (35) | fix quality (30) | tests (20) | scope & process (10) | efficiency (5) | weighted total /100 |
|---|---|---|---|---|---|---|
| 01-atv-starterkit | 5 | 5 | 4 | 4 | 5 | 94 |
| 02-atv-phoenix | 5 | 5 | 5 | 4 | 3 | 96 |
| 04-kprojects | 5 | 4 | 5 | 5 | 4 | 93 |
| 05-baseline | 5 | 5 | 3 | 5 | 5 | 92 |
| 06-gstack | 5 | 5 | 5 | 5 | 4 | 99 |
| 07-baseline-claude | 5 | 5 | 4 | 5 | 5 | 96 |

Weighted total = Σ(score/5 × weight); weights 35/30/20/10/5.

Cross-cutting observations recorded for consensus (not a ranking):

1. **Five of six fixes are the same fix.** 01, 02, 05, 06, 07 all added
   an open-element-stack check in the `Event::Eof` arm — the stack
   already existed in each parser; the fix was consulting it. 04, whose
   parser had no stack (balanced-text helper architecture), added a
   depth counter across four match arms plus a second EOF guard inside
   `read_text` — the only repo where the same bug had two mouths to
   close, and the only typed error variant (`ParseError::Truncated`).
2. **Tests are where the field actually spread.** Ends of the spectrum:
   05 shipped a single unit test that is the bug report's repro verbatim;
   02 shipped three unit tests including a well-formed-empty guard that
   independently anticipates the F2 addendum trap, plus fixture + e2e.
   Four repos (02, 04, 06, 07) wired a `truncated.xml` fixture into
   their own corpus and e2e — regression coverage at the layer the bug
   actually manifests. Sample-shaped suites (05 entirely, 07's variants)
   were docked even where the fix itself is generic.
3. **Cost tracked test depth almost linearly on the copilot runner**
   (67.4 → 86.2 → 151.7 → 261.6 credits for 05 → 01 → 04 → 02, wall
   1m29s → 6m42s in the same order): the efficiency dimension and the
   tests dimension are close to a paid trade this round. The Claude
   pair shows the same shape in miniature (07: 8.5k output/2m57s lean;
   06: 12.8k/3m44s deeper coverage).
4. **Harness machinery visible in deltas, for better and worse**: 04's
   unprompted sprint doc is an accurate postmortem artifact (scored as
   process signal); 02 retargeted its phoenix done gate from `just
   check` to a single e2e test — a verification narrowing bundled into
   the fix commit; 01 again finished with untracked `.atv` hook
   telemetry, the only dirty tree of the round.
