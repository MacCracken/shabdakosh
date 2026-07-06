#!/usr/bin/env bash
# version-bump.sh — Bump the shabdakosh version.
#
# VERSION is the source of truth: cyrius.cyml reads `version = "${file:VERSION}"`,
# so bumping the version means writing VERSION and regenerating the distlib bundle
# (dist/shabdakosh.cyr) so its `# Version:` header carries the new value.
set -euo pipefail

[ $# -ne 1 ] && echo "Usage: $0 <version>" && exit 1
NEW_VERSION="$1"
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

echo "$NEW_VERSION" > "$REPO_ROOT/VERSION"

# Regenerate dist/shabdakosh.cyr (+ .deps) so the bundle carries the new version.
cd "$REPO_ROOT" && cyrius distlib

echo
echo "Bumped to ${NEW_VERSION}. Next:"
echo "  - Update CHANGELOG.md (Keep a Changelog: Added/Changed/Fixed/Removed)"
echo "  - Commit, then tag: git tag v${NEW_VERSION}"
