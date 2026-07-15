#!/usr/bin/env bash
# use-profile.sh — swap ~/.copilot between eval profiles.
#
# Usage:
#   use-profile.sh bootstrap          # one-time: move real ~/.copilot -> profiles/original, symlink to it
#   use-profile.sh clean|phoenix|original
#   use-profile.sh status
set -euo pipefail

PROFILES="$HOME/src/ai-agents/harness-eval/_eval/profiles"
LINK="$HOME/.copilot"

cmd="${1:?usage: use-profile.sh bootstrap|clean|phoenix|original|status}"

case "$cmd" in
    status)
        if [[ -L $LINK ]]; then
            echo "~/.copilot -> $(readlink "$LINK")"
        elif [[ -d $LINK ]]; then
            echo "~/.copilot is a real directory (not bootstrapped)"
        else
            echo "~/.copilot does not exist"
        fi
        exit 0
        ;;
    bootstrap)
        if [[ -L $LINK ]]; then
            echo "already bootstrapped: ~/.copilot -> $(readlink "$LINK")"
            exit 0
        fi
        [[ -d $LINK ]] || { echo "no ~/.copilot to bootstrap" >&2; exit 1; }
        [[ -e $PROFILES/original ]] && { echo "profiles/original already exists; refusing" >&2; exit 1; }
        mkdir -p "$PROFILES/original"
        mv "$LINK" "$PROFILES/original/.copilot"
        ln -s "$PROFILES/original/.copilot" "$LINK"
        echo "bootstrapped: ~/.copilot -> $PROFILES/original/.copilot"
        exit 0
        ;;
    clean|phoenix|original)
        target="$PROFILES/$cmd/.copilot"
        [[ -d $target ]] || { echo "no such profile: $target" >&2; exit 1; }
        if [[ -e $LINK && ! -L $LINK ]]; then
            echo "~/.copilot is a real directory; run 'use-profile.sh bootstrap' first" >&2
            exit 1
        fi
        ln -sfn "$target" "$LINK"
        echo "~/.copilot -> $target"
        ;;
    *)
        echo "unknown command: $cmd" >&2
        exit 1
        ;;
esac
