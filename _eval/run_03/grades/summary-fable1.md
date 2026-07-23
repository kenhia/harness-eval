# run_03 summary — grader: fable1 (Haiku 4.5 tier)

Scores are tier-absolute (calibrated to this field's own controls: 05 for
Copilot cells, 07 for Claude cells). Correctness applied mechanically from
the rubric's (core, hard) mapping. Weights: 30/20/15/10/10/10/5.

| repo | correct | code | tests | docs | process | effic | auton | core | hard | weighted |
|---|---|---|---|---|---|---|---|---|---|---|
| 01-atv-starterkit | 3 | 3 | 3 | 4 | 2 | 2 | 4 | 11/12 | 4/9 | **59** |
| 02-atv-phoenix | 2 | 2 | 2 | 3 | 2 | 4 | 4 | 9/12 | 4/9 | **48** |
| 03-working-skill-repo | 5 | 4 | 3 | 3 | 2 | 5 | 4 | 12/12 | 8/9 | **79** |
| 04-kprojects | 3.5 | 3 | 3 | 4 | 3 | 5 | 4 | 11/12 | 8/9 | **70** |
| 05-baseline (Copilot ctl) | 2 | 3 | 3 | 4 | 3 | 5 | 5 | 9/12 | 4/9 | **62** |
| 06-gstack | 2.5 | 3 | 3 | 4 | 3 | 2 | 5 | 9/12 | 8/9 | **59** |
| 07-baseline-claude (Claude ctl) | 3 | 3 | 3 | 4 | 2 | 5 | 5 | 11/12 | 4/9 | **66** |

Ranking: 03 (79) > 04 (70) > 07 (66) > 05 (62) > 01 (59) = 06 (59) > 02 (48).

> Cells 03 and 06 re-scored after suite defect S2 (see DEFECTS.md): the
> `loglens_json` fixture's argparse-specific retry failed click repos on
> JSON checks they pass; S2b made the A12 justfile probe case-tolerant.
> Corrected tallies: 03 → 12/12, 8/9; 06 → 9/12, 8/9. Other rows frozen.

## Grader notes

- **Suite defect S2 — flagged by this grader, since adjudicated and fixed**
  (see DEFECTS.md §S2). The `loglens_json` fixture's leading-form fallback
  keyed on argparse's "unrecognized" wording, which click never emits, so
  click repos failed JSON checks they pass. Cells 03 and 06 were re-graded
  under the corrected tallies (03: A3/A7/H7/H8 flipped; 06: A3/H7/H8).
  Agent-owned residue stands as scored: 03's README documents the trailing
  `--format` form its own CLI rejects; 06 suppresses the required stderr
  malformed count in json mode (`fmt == "text"` guard, residual A7) and
  its dev-extras layout breaks clean-env `pytest`/`just check` (A9, A12)
  regardless of S2b's now-case-tolerant justfile probe.
- **Tier-wide finding:** no repo normalized naive/aware datetimes on both
  sides. The naive-parser repos (01, 02, 05, 07) crash on aware
  `--since/--until` (A5 + H2/H3/H4/H6/H7); the aware-parser repos (03, 04,
  06) crash on naive `--since` (H9). Complementary halves of the same
  missed edge — nobody handled the mix.
- Four repos (04, 05, 06, 07) committed `__pycache__` `.pyc` files; 06 and
  07 have no .gitignore at all. Two repos (05, 06) declared dev tooling as
  optional extras, breaking `uv run pytest`/`just check` from a clean
  checkout. Both patterns look like tier-characteristic hygiene lapses.
- **Blackout compliance:** klams/korg were not queried for anything
  eval-related this session, and no eval-related memory surfaced unbidden.
- All inspection done on clones under `/tmp/grade-fable1-run03/`; run repos
  untouched.
