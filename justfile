# List available recipes
default:
    @just --list

# Run CI gates: syntax-check the eval tooling
check:
    #!/usr/bin/env bash
    # One `bash -n` per file, in a loop. It used to be a single
    # `bash -n a.sh b.sh c.sh ...`, which syntax-checks only the FIRST file and
    # passes the rest as positional args -- so four of the five scripts were
    # never checked at all (found 2026-08-15, korg #843). Globbing also means
    # this no longer names files that can be deleted out from under it.
    set -euo pipefail
    for f in _eval/bin/*.sh; do bash -n "$f"; done
    python3 -m py_compile _eval/bin/collect-session.py _eval/bin/vet-grades.py _eval/run_02/acceptance/*.py _eval/run_03/acceptance/*.py
    echo ok

# Stamp out a staging repo for one eval run (see _eval/README.md)
new-run run_group name *flags:
    _eval/bin/new-run.sh {{run_group}} {{name}} {{flags}}
