#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CORPUS_DIR="$SCRIPT_DIR/corpus"

clone_if_missing() {
    local repo="$1"
    local branch="$2"
    local dest="$CORPUS_DIR/$(basename "$repo")"

    if [ -d "$dest" ]; then
        echo "Already exists: $dest"
        return
    fi

    echo "Cloning $repo ($branch) -> $dest"
    git clone --depth 1 --branch "$branch" "https://github.com/$repo.git" "$dest"
}

mkdir -p "$CORPUS_DIR"

clone_if_missing "godotengine/godot-demo-projects" "4.3"
clone_if_missing "godotengine/tps-demo" "master"

echo "Corpus ready at $CORPUS_DIR"
