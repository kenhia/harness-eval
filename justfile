# List available recipes
default:
    @just --list

# Run CI gates: syntax-check the eval tooling
check:
    bash -n _eval/bin/new-run.sh _eval/bin/run-eval.sh _eval/bin/run-acceptance.sh _eval/bin/run-matrix.sh _eval/bin/use-profile.sh
    python3 -m py_compile _eval/bin/collect-session.py _eval/bin/vet-grades.py _eval/run_02/acceptance/*.py _eval/run_03/acceptance/*.py
    @echo ok

# Stamp out a staging repo for one eval run (see _eval/README.md)
new-run run_group name *flags:
    _eval/bin/new-run.sh {{run_group}} {{name}} {{flags}}
