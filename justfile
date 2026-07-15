# List available recipes
default:
    @just --list

# Run CI gates (lint, typecheck, tests) — wire these up for the project
check:
    @echo "TODO: wire project checks, e.g.:"
    @echo "  uv run ruff check . && uv run ty check && uv run pytest"
