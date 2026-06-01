#!/usr/bin/env bash
#
# Assemble a self-contained `skyscrapers-player` package tree, suitable for
# force-pushing to the `player-dist` branch (which external projects install
# via `npm install github:sugyan/skyscrapers#player-dist`).
#
# Usage:
#   scripts/build-player-dist.sh [OUTPUT_DIR]
#
# If OUTPUT_DIR is omitted, a fresh temp directory is created. The chosen
# output path is printed to stdout on success.
#
# Prerequisites (the script verifies these and exits with a clear error if
# anything is missing):
#   - skyscrapers-player/dist/styles.css  (from `npm run build:css`)
#
# Why this script exists as a separate file: the assembly logic is large
# enough that embedding it in the GitHub Actions workflow yaml hurt
# readability and locked it away from `shellcheck` and local execution.
# Run it locally to inspect the produced tree before relying on CI.

set -euo pipefail

REPO_ROOT="$(git rev-parse --show-toplevel)"
PLAYER_DIR="$REPO_ROOT/skyscrapers-player"

OUT_DIR="${1:-}"
if [ -z "$OUT_DIR" ]; then
  OUT_DIR="$(mktemp -d)"
else
  # A caller-supplied path may not exist yet (e.g. "$RUNNER_TEMP/dist" in CI);
  # ensure it does before the first `cp` walks into it.
  mkdir -p "$OUT_DIR"
  # Refuse to clobber a non-empty directory: `cp -r src/ dst/` where dst/src
  # already exists nests it into dst/src/src instead of replacing, producing
  # an invalid tree. Forcing the caller to start clean keeps repeated local
  # runs deterministic.
  if [ -n "$(ls -A "$OUT_DIR" 2>/dev/null)" ]; then
    echo "::error::Output directory $OUT_DIR is not empty. Remove it (or pass a fresh path) before re-running." >&2
    exit 1
  fi
fi

# ─── Prerequisite checks ──────────────────────────────────────────────────

for cmd in jq find cp; do
  if ! command -v "$cmd" >/dev/null 2>&1; then
    echo "::error::Required tool '$cmd' is not on PATH." >&2
    exit 1
  fi
done

if [ ! -f "$PLAYER_DIR/dist/styles.css" ]; then
  echo "::error::Missing $PLAYER_DIR/dist/styles.css. Run \`npm run build:css\` in skyscrapers-player first." >&2
  exit 1
fi

# ─── Assemble the tree ────────────────────────────────────────────────────

# Player source — strip tests; consumers don't need them.
cp -r "$PLAYER_DIR/src" "$OUT_DIR/src"
find "$OUT_DIR/src" -name "*.test.ts" -delete

# Pre-built Tailwind output.
cp -r "$PLAYER_DIR/dist" "$OUT_DIR/dist"

# README explaining the install/usage.
cp "$PLAYER_DIR/README.md" "$OUT_DIR/README.md"

# Rewrite package.json:
# - drop fields that don't apply to a published package
# - swap the styles.css export from the Tailwind source entry to the
#   pre-built artifact
# - declare `files` so `npm pack`/git-install stay bounded
jq '
  del(.private, .scripts, .devDependencies) |
  .files = ["src", "dist", "README.md"] |
  .exports["./styles.css"] = "./dist/styles.css"
' "$PLAYER_DIR/package.json" > "$OUT_DIR/package.json"

echo "$OUT_DIR"
