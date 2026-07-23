#!/usr/bin/env python3
"""vet-grades.py — sanity-check a grader's committed sheets before consensus.

Usage:
  vet-grades.py <run-group> <grader-id>        e.g. vet-grades.py run_03 sol

Checks (mechanical only — judgment stays with the consensus session):
  1. all expected sheets + a summary exist
  2. every rubric dimension present per sheet, scores within 0–5
  3. correctness score matches the run_03 mapping table for that repo's
     (core, hard) acceptance tally
  4. weighted total arithmetic
  5. text integrity: control chars, mojibake, absurd repetition,
     truncation, stray token-ish noise
  6. independence: no reference to the other grader or their scores

Exit 0 = clean, 1 = findings. Findings are advisory: read them, don't
auto-trust them.
"""

import re
import sys
import unicodedata
from pathlib import Path

WEIGHTS = {
    "correctness": 30, "code quality": 20, "tests": 15,
    "docs": 10, "process": 10, "efficiency": 10, "autonomy": 5,
}
OTHER = {"sol": "fable1", "fable1": "sol"}


def correctness_from_tally(core: int, hard: int) -> float:
    if core >= 12:
        return 4 + (1 if hard >= 8 else 0)
    if core == 11:
        base = 3
    elif core >= 9:
        base = 2
    elif core >= 7:
        base = 1
    else:
        return 0
    return base + (0.5 if hard >= 6 else 0)


def read_tallies(runs: Path) -> dict[str, tuple[int, int]]:
    out = {}
    for f in sorted(runs.glob("*-acceptance.txt")):
        nn = f.name.split("-")[0]
        text = f.read_text(errors="replace")
        core = hard = None
        for line in text.splitlines():
            if m := re.search(r"#\s*core:\s*(\d+)/(\d+)", line):
                core = int(m.group(1))
            elif m := re.search(r"#\s*hard:\s*(\d+)/(\d+)", line):
                hard = int(m.group(1))
        if core is not None and hard is not None:
            out[nn] = (core, hard)
    return out


def parse_sheet(text: str) -> tuple[dict[str, float], float | None]:
    scores: dict[str, float] = {}
    for line in text.splitlines():
        if not line.strip().startswith("|"):
            continue
        cells = [c.strip() for c in line.strip().strip("|").split("|")]
        if len(cells) < 2:
            continue
        dim = cells[0].lower()
        for known in WEIGHTS:
            if dim.startswith(known.split()[0]) and known.split()[0] not in ("dim",):
                if m := re.match(r"^(\d+(?:\.\d+)?)", cells[1]):
                    scores[known] = float(m.group(1))
                break
    total = None
    if m := re.search(r"weighted total:?\**\s*:?\s*\**\s*(\d+(?:\.\d+)?)", text, re.I):
        total = float(m.group(1))
    return scores, total


def text_findings(name: str, text: str) -> list[str]:
    out = []
    ctrl = {c for c in text if unicodedata.category(c) == "Cc" and c not in "\n\t\r"}
    if ctrl:
        out.append(f"{name}: control characters present: {[hex(ord(c)) for c in ctrl][:5]}")
    for bad in ("�", "â€", "Ã©", "<|", "|>", "<0x"):
        if bad in text:
            out.append(f"{name}: suspicious sequence {bad!r} (encoding/token leakage?)")
    # absurd repetition: same 40-char chunk many times
    for chunk, n in ((c, text.count(c)) for c in {text[i:i + 40] for i in range(0, max(0, len(text) - 40), 400)}):
        if n >= 6 and chunk.strip():
            out.append(f"{name}: chunk repeated {n}x: {chunk[:50]!r}")
            break
    if re.search(r"\b(\w+)( \1\b){4,}", text):
        out.append(f"{name}: a word repeats 5+ times consecutively (degeneration?)")
    if text.rstrip().endswith((",", "the", "and", "-", "|")):
        out.append(f"{name}: ends mid-sentence — truncated?")
    if len(text.strip()) < 200:
        out.append(f"{name}: suspiciously short ({len(text.strip())} chars)")
    return out


def main() -> int:
    if len(sys.argv) != 3:
        sys.exit(__doc__)
    group, grader = sys.argv[1], sys.argv[2]
    root = Path(__file__).resolve().parent.parent / group
    grades, runs = root / "grades", root / "runs"
    tallies = read_tallies(runs)
    findings: list[str] = []

    # only the run's own per-cell sheets: NN-<grader>.md (excludes
    # summary-*, and variant rounds like NN-fix-<grader>.md which carry a
    # different dimension set)
    sheets = sorted(
        s for s in grades.glob(f"*-{grader}.md")
        if re.fullmatch(rf"\d{{2}}-{re.escape(grader)}\.md", s.name)
    )
    expected = sorted(tallies)
    found = sorted({s.name.split("-")[0] for s in sheets})
    if missing := [n for n in expected if n not in found]:
        findings.append(f"MISSING sheets for cells: {missing}")
    if not (grades / f"summary-{grader}.md").is_file():
        findings.append(f"MISSING summary-{grader}.md")

    for sheet in sheets:
        nn = sheet.name.split("-")[0]
        text = sheet.read_text(errors="replace")
        findings += text_findings(sheet.name, text)
        scores, total = parse_sheet(text)
        for dim in WEIGHTS:
            if dim not in scores:
                findings.append(f"{sheet.name}: dimension '{dim}' not parsed/absent")
            elif not 0 <= scores[dim] <= 5:
                findings.append(f"{sheet.name}: {dim} score {scores[dim]} out of range")
        # the mechanical mapping is defined for run_03's tier; other run
        # groups use their own correctness rules — don't cross-apply it
        if group == "run_03" and nn in tallies and "correctness" in scores:
            core, hard = tallies[nn]
            want = correctness_from_tally(core, hard)
            if abs(scores["correctness"] - want) > 0.001:
                findings.append(
                    f"{sheet.name}: correctness {scores['correctness']} != mapping "
                    f"{want} for (core {core}, hard {hard})"
                )
        if total is not None and len(scores) == len(WEIGHTS):
            calc = sum(scores[d] / 5 * w for d, w in WEIGHTS.items())
            if abs(calc - total) > 0.6:
                findings.append(
                    f"{sheet.name}: weighted total {total} != computed {calc:.1f}"
                )
        elif total is None:
            findings.append(f"{sheet.name}: no weighted total found")
        if other := OTHER.get(grader):
            if re.search(rf"\b{other}\b", text, re.I):
                findings.append(f"{sheet.name}: references the other grader ({other}) — independence")

    print(f"vet-grades: {group} / {grader} — {len(sheets)} sheets, "
          f"{len(tallies)} cells with tallies")
    for f in findings:
        print(f"  ! {f}")
    if not findings:
        print("  clean — no mechanical findings")
    return 1 if findings else 0


if __name__ == "__main__":
    sys.exit(main())
