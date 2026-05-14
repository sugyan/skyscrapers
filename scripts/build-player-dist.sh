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
#   - skyscrapers-generator/pkg/      (from `wasm-pack build --target web`)
#   - skyscrapers-player/dist/styles.css  (from `npm run build:css`)
#
# Why this script exists as a separate file: the assembly logic is large
# enough that embedding it in the GitHub Actions workflow yaml hurt
# readability and locked it away from `shellcheck` and local execution.
# Run it locally to inspect the produced tree before relying on CI.

set -euo pipefail

REPO_ROOT="$(git rev-parse --show-toplevel)"
PLAYER_DIR="$REPO_ROOT/skyscrapers-player"
GENERATOR_PKG="$REPO_ROOT/skyscrapers-generator/pkg"

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

for cmd in jq sed find cp; do
  if ! command -v "$cmd" >/dev/null 2>&1; then
    echo "::error::Required tool '$cmd' is not on PATH." >&2
    exit 1
  fi
done

if [ ! -d "$GENERATOR_PKG" ]; then
  echo "::error::Missing $GENERATOR_PKG. Run \`wasm-pack build --target web skyscrapers-generator\` first." >&2
  exit 1
fi
if [ ! -f "$GENERATOR_PKG/skyscrapers_generator.js" ]; then
  echo "::error::$GENERATOR_PKG exists but skyscrapers_generator.js is missing." >&2
  exit 1
fi
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

# Vendor the wasm-pack output as a sibling directory of `src/`.
# wasm-pack writes a `.gitignore` of `*` inside pkg/ — npm pack honors
# nested .gitignores and would silently drop the WASM artifact from the
# tarball, so strip it explicitly.
cp -r "$GENERATOR_PKG" "$OUT_DIR/skyscrapers-generator"
rm -f "$OUT_DIR/skyscrapers-generator/.gitignore"

# The published wasm-engine imports the generator by a *relative* path
# rather than the bare specifier it uses in the monorepo: nested `file:`
# deps don't resolve through `npm install <git url>` because the dependency
# tree is flattened from the tarball, but a plain relative import to the
# sibling directory works regardless.
ENGINE="$OUT_DIR/src/engine/wasm-engine.ts"
REL_IMPORT='../../skyscrapers-generator/skyscrapers_generator.js'
sed -i.bak "s|from \"skyscrapers-generator\"|from \"$REL_IMPORT\"|" "$ENGINE"
rm -f "$ENGINE.bak"

# Rewrite package.json:
# - drop fields that don't apply to a published package
# - remove the skyscrapers-generator dep entry — the import now resolves
#   through a relative path, so npm doesn't need to install the generator
#   as a separate package
# - swap the styles.css export from the Tailwind source entry to the
#   pre-built artifact
# - declare `files` so `npm pack`/git-install stay bounded
jq '
  del(.private, .scripts, .devDependencies, .dependencies["skyscrapers-generator"]) |
  if (.dependencies | length) == 0 then del(.dependencies) else . end |
  .files = ["src", "dist", "skyscrapers-generator", "README.md"] |
  .exports["./styles.css"] = "./dist/styles.css"
' "$PLAYER_DIR/package.json" > "$OUT_DIR/package.json"

# ─── Post-assembly assertions ─────────────────────────────────────────────
# `sed -i` exits 0 on no-match, so a future refactor that renames the bare
# specifier or the vendored filename would silently produce a dist tree
# that imports an unresolved module. Fail loudly here instead.

if grep -q 'from "skyscrapers-generator"' "$ENGINE"; then
  echo "::error::wasm-engine.ts still contains a bare 'skyscrapers-generator' import after rewrite. The dist tree would be broken." >&2
  exit 1
fi
if ! grep -q "from \"$REL_IMPORT\"" "$ENGINE"; then
  echo "::error::wasm-engine.ts is missing the expected relative import to $REL_IMPORT. The dist tree would be broken." >&2
  exit 1
fi
if [ ! -f "$OUT_DIR/skyscrapers-generator/skyscrapers_generator.js" ]; then
  echo "::error::Vendored generator JS missing at expected path." >&2
  exit 1
fi

echo "$OUT_DIR"
