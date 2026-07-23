# run_03 summary — grader: fable1 (Haiku 4.5 tier)

Scores are tier-absolute (calibrated to this field's own controls: 05 for
Copilot cells, 07 for Claude cells). Correctness applied mechanically from
the rubric's (core, hard) mapping. Weights: 30/20/15/10/10/10/5.

| repo | correct | code | tests | docs | process | effic | auton | core | hard | weighted |
|---|---|---|---|---|---|---|---|---|---|---|
| 01-atv-starterkit | 3 | 3 | 3 | 4 | 2 | 2 | 4 | 11/12 | 4/9 | **59** |
| 02-atv-phoenix | 2 | 2 | 2 | 3 | 2 | 4 | 4 | 9/12 | 4/9 | **48** |
| 03-working-skill-repo | 2.5 | 4 | 3 | 3 | 2 | 5 | 4 | 10/12 | 6/9 | **64** |
| 04-kprojects | 3.5 | 3 | 3 | 4 | 3 | 5 | 4 | 11/12 | 8/9 | **70** |
| 05-baseline (Copilot ctl) | 2 | 3 | 3 | 4 | 3 | 5 | 5 | 9/12 | 4/9 | **62** |
| 06-gstack | 1.5 | 3 | 3 | 4 | 3 | 2 | 5 | 8/12 | 6/9 | **53** |
| 07-baseline-claude (Claude ctl) | 3 | 3 | 3 | 4 | 2 | 5 | 5 | 11/12 | 4/9 | **66** |

Ranking: 04 (70) > 07 (66) > 03 (64) > 05 (62) > 01 (59) > 06 (53) > 02 (48).

## Grader notes

- **Candidate suite defect (flagged for adjudication, scores unchanged).**
  The `loglens_json` fixture tries trailing `--format json` and falls back
  to the leading form only when stderr contains "unrecognized" — argparse's
  error wording. Click emits "No such option", so the fallback never fires
  for click repos even where the leading form works (verified by running
  both clones: 03 and 06 return rc 0 with valid JSON on
  `loglens --format json summary FILE`). Implicated: A3, A7, H7, H8 in
  repos 03 and 06 (same reasoning class as S1). Tempering facts: 03's own
  README documents the trailing form its CLI rejects (agent-owned doc bug);
  06's README documents the working leading form. Also noted: A12 probes
  lowercase `justfile` only — 06 ships capital-J `Justfile` (which `just`
  accepts), though its recipes use bare `pytest`/`ruff` and would fail from
  a clean env anyway.
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
