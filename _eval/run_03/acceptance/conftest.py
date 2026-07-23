"""Executable acceptance harness for the loglens scenario (run_03).
SEALED — never shown to agents.

Run against one contender repo:

    ACCEPTANCE_REPO=~/src/ai-agents/harness-eval-runs/run_03/05-baseline \
        uv run --with pytest pytest _eval/run_03/acceptance -v

Black-box: `uv sync` once, then drives the `loglens` CLI as a
subprocess. Output formats are implementation-defined, so core checks
assert VALUES (via substring/recursive-JSON scans), not layouts —
calibrated against the seven graded run_01 trees.
"""

import json
import os
import subprocess
from pathlib import Path

import pytest

import fixtures as fx

REPO = Path(
    os.environ.get("ACCEPTANCE_REPO") or os.environ.get("LOGLENS_REPO") or ""
).expanduser()
SYNC_TIMEOUT = 300
CLI_TIMEOUT = 60


def clean_env():
    """Drop the OUTER uv/venv context so the inner `uv run` binds to the
    contender repo's own environment (VIRTUAL_ENV leakage breaks it)."""
    env = {k: v for k, v in os.environ.items()
           if k not in ("VIRTUAL_ENV", "UV_PROJECT_ENVIRONMENT", "PYTHONPATH")}
    return env


def find_project_root(base: Path) -> tuple[Path, bool]:
    """Locate the Python project. Agents may build at the repo root or
    nest it one level (e.g. `loglens/pyproject.toml`) — the spec says
    'in the current repository' and does not pin the level, so both are
    accepted (defect S1). Returns (root, nested)."""
    if (base / "pyproject.toml").is_file():
        return base, False
    candidates = sorted(
        p.parent for p in base.glob("*/pyproject.toml")
        if not p.parent.name.startswith(".")
    )
    assert candidates, f"no pyproject.toml at {base} or one level below"
    assert len(candidates) == 1, f"ambiguous project roots: {candidates}"
    return candidates[0], True


@pytest.fixture(scope="session")
def repo() -> Path:
    assert REPO.is_dir(), f"ACCEPTANCE_REPO not set or not a dir: {REPO!r}"
    root, nested = find_project_root(REPO)
    if nested:
        print(f"\nNOTE: project nested at {root.relative_to(REPO)}/ "
              f"(not repo root) — scoreable process observation, not a failure")
    proc = subprocess.run(
        ["uv", "sync"], cwd=root, capture_output=True, text=True,
        timeout=SYNC_TIMEOUT, env=clean_env(),
    )
    assert proc.returncode == 0, f"uv sync failed:\n{proc.stderr[-2000:]}"
    return root


@pytest.fixture(scope="session")
def logfile(tmp_path_factory) -> Path:
    p = tmp_path_factory.mktemp("fixture") / "sealed.log"
    p.write_text(fx.render())
    return p


def write_log(tmp_path_factory, name: str, content: str) -> Path:
    p = tmp_path_factory.mktemp("fixture") / name
    p.write_text(content)
    return p


@pytest.fixture(scope="session")
def mixed_tz_log(tmp_path_factory) -> Path:
    return write_log(tmp_path_factory, "mixed_tz.log", fx.MIXED_TZ)


@pytest.fixture(scope="session")
def boundary_log(tmp_path_factory) -> Path:
    return write_log(tmp_path_factory, "boundary.log", fx.BOUNDARY)


class Result:
    def __init__(self, proc: subprocess.CompletedProcess):
        self.rc = proc.returncode
        self.out = proc.stdout
        self.err = proc.stderr

    def json(self):
        return json.loads(self.out)


@pytest.fixture(scope="session")
def loglens(repo):
    def run(*args: str) -> Result:
        proc = subprocess.run(
            ["uv", "run", "loglens", *args],
            cwd=repo, capture_output=True, text=True, timeout=CLI_TIMEOUT,
            env=clean_env(),
        )
        return Result(proc)

    return run


@pytest.fixture(scope="session")
def loglens_json(loglens):
    """Run a subcommand with --format json. The spec calls --format a
    'global option'; implementations legitimately put it before OR after
    the subcommand — try trailing, fall back to leading."""

    def run(*args: str) -> Result:
        r = loglens(*args, "--format", "json")
        if r.rc != 0 and "unrecognized" in r.err:
            r = loglens("--format", "json", *args)
        return r

    return run


# ------------------------------------------------------------- scan helpers
def walk_values(obj):
    """Yield every leaf value in a JSON document."""
    if isinstance(obj, dict):
        for k, v in obj.items():
            yield k
            yield from walk_values(v)
    elif isinstance(obj, list):
        for v in obj:
            yield from walk_values(v)
    else:
        yield obj


def doc_numbers(doc):
    out = []
    for v in walk_values(doc):
        if isinstance(v, bool):
            continue
        if isinstance(v, (int, float)):
            out.append(float(v))
        elif isinstance(v, str):
            try:
                out.append(float(v.rstrip("%")))
            except ValueError:
                pass
    return out


def has_number(doc, value, tol=0.05):
    return any(abs(n - value) <= tol for n in doc_numbers(doc))


def has_rate(doc, pct, tol=0.06):
    """Accept a percentage expressed as XX.X, XX.X%, or a 0.XXX fraction."""
    return has_number(doc, pct, tol) or has_number(doc, pct / 100, tol / 10)


def text_positions(text: str, needles):
    """Positions of each needle in text; -1 if absent."""
    return [text.find(n) for n in needles]
