#!/usr/bin/env bash
# Decide release version: respect manual bumps, otherwise auto-increment patch.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
REPO="${GITHUB_REPOSITORY:-harshabala/parsekit}"
OUT="${GITHUB_OUTPUT:-}"

CURRENT="$(node -p "require('$ROOT/package.json').version")"
LATEST_TAG="$(gh release list --repo "$REPO" --limit 1 --json tagName --jq '.[0].tagName // empty' 2>/dev/null || true)"
LATEST="${LATEST_TAG#v}"

emit() {
  local key="$1"
  local value="$2"
  if [[ -n "$OUT" ]]; then
    echo "${key}=${value}" >> "$OUT"
  else
    echo "${key}=${value}"
  fi
}

# HEAD already points at the latest release tag — nothing new to ship.
if [[ -n "$LATEST_TAG" ]] && git rev-parse "refs/tags/${LATEST_TAG}" >/dev/null 2>&1; then
  TAG_SHA="$(git rev-list -n 1 "$LATEST_TAG")"
  HEAD_SHA="$(git rev-parse HEAD)"
  if [[ "$TAG_SHA" == "$HEAD_SHA" ]]; then
    echo "HEAD is already ${LATEST_TAG}; skipping release."
    emit skip true
    emit version "$CURRENT"
    exit 0
  fi
fi

emit skip false

if [[ -z "$LATEST" ]]; then
  echo "No prior GitHub release; using v${CURRENT}"
  bash "$ROOT/scripts/sync-version.sh" "$CURRENT"
  emit version "$CURRENT"
  exit 0
fi

# Maintainer set a higher version than the latest published release.
if [[ "$CURRENT" != "$LATEST" ]] && [[ "$(printf '%s\n%s\n' "$CURRENT" "$LATEST" | sort -V | tail -1)" == "$CURRENT" ]]; then
  echo "Keeping manual version bump: v${CURRENT} (latest published: v${LATEST})"
  bash "$ROOT/scripts/sync-version.sh" "$CURRENT"
  emit version "$CURRENT"
  exit 0
fi

# Latest release already uses this version — bump patch for the new commits.
NEW="$(node -e "
const parts = '$CURRENT'.split('.').map((n) => Number(n));
if (parts.length !== 3 || parts.some((n) => Number.isNaN(n))) {
  throw new Error('Expected semver like 0.2.4');
}
parts[2] += 1;
process.stdout.write(parts.join('.'));
")"

echo "Auto-bumping v${CURRENT} → v${NEW} (v${LATEST} already published)"
bash "$ROOT/scripts/sync-version.sh" "$NEW"
emit version "$NEW"