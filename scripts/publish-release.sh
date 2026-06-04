#!/usr/bin/env bash
# Build ParseKit, sign updater bundle, write latest.json, upload to GitHub Releases.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
VERSION="$(node -p "require('$ROOT/package.json').version")"
TAG="v${VERSION}"
REPO="harshabala/parsedock"
KEY_PATH="${TAURI_SIGNING_PRIVATE_KEY_PATH:-$HOME/.tauri/parsekit.key}"

MACOS_DIR="$ROOT/src-tauri/target/release/bundle/macos"
DMG_DIR="$ROOT/src-tauri/target/release/bundle/dmg"
DMG="$DMG_DIR/ParseKit_${VERSION}_aarch64.dmg"
UPDATER_NAME="ParseKit_${VERSION}_aarch64.app.tar.gz"
UPDATER_STAGED="$DMG_DIR/$UPDATER_NAME"
LATEST_JSON="$DMG_DIR/parsekit-latest.json"

if [[ ! -f "$KEY_PATH" ]]; then
  echo "error: missing private key at $KEY_PATH" >&2
  echo "Generate with: npx tauri signer generate -w $KEY_PATH -f --ci -p ''" >&2
  exit 1
fi

export TAURI_SIGNING_PRIVATE_KEY="${TAURI_SIGNING_PRIVATE_KEY:-$KEY_PATH}"
export TAURI_SIGNING_PRIVATE_KEY_PASSWORD="${TAURI_SIGNING_PRIVATE_KEY_PASSWORD:-}"

echo "== publish-release: ParseKit v${VERSION} → ${REPO} ${TAG} =="

echo "[1/5] Build release (updater .tar.gz + DMG) ..."
cd "$ROOT"
npm run release:macos

UPDATER_PKG="$(find "$MACOS_DIR" -maxdepth 1 -name '*.app.tar.gz' ! -name '*.sig' -print -quit)"
if [[ -z "$UPDATER_PKG" ]]; then
  echo "error: no .app.tar.gz in $MACOS_DIR (is createUpdaterArtifacts enabled?)" >&2
  exit 1
fi

echo "[2/5] Stage updater bundle as $UPDATER_NAME ..."
mkdir -p "$DMG_DIR"
cp "$UPDATER_PKG" "$UPDATER_STAGED"

SIG_FILE="${UPDATER_STAGED}.sig"
if [[ -f "${UPDATER_PKG}.sig" ]]; then
  cp "${UPDATER_PKG}.sig" "$SIG_FILE"
  echo "Using signature from build: ${UPDATER_PKG}.sig"
else
  echo "Signing updater bundle ..."
  (
    cd "$ROOT"
    npx tauri signer sign -f "$KEY_PATH" -p "${TAURI_SIGNING_PRIVATE_KEY_PASSWORD}" "$UPDATER_STAGED"
  )
fi

if [[ ! -f "$SIG_FILE" ]]; then
  echo "error: missing signature file $SIG_FILE" >&2
  exit 1
fi

echo "[3/5] Write parsekit-latest.json ..."
RELEASE_NOTES="${RELEASE_NOTES:-ParseKit v${VERSION}}"
export VERSION SIG_FILE LATEST_JSON UPDATER_NAME RELEASE_NOTES
node <<'NODE'
const fs = require("fs");
const version = process.env.VERSION;
const signature = fs.readFileSync(process.env.SIG_FILE, "utf8").trim();
const json = {
  version,
  notes: process.env.RELEASE_NOTES,
  pub_date: new Date().toISOString(),
  platforms: {
    "darwin-aarch64": {
      signature,
      url: `https://github.com/harshabala/parsedock/releases/download/v${version}/${process.env.UPDATER_NAME}`,
    },
  },
};
fs.writeFileSync(process.env.LATEST_JSON, JSON.stringify(json, null, 2) + "\n");
NODE

echo "parsekit-latest.json:"
cat "$LATEST_JSON"

if [[ ! -f "$DMG" ]]; then
  echo "error: DMG not found at $DMG" >&2
  exit 1
fi

echo "[4/5] Upload to GitHub release ${TAG} ..."
if gh release view "$TAG" --repo "$REPO" &>/dev/null; then
  gh release upload "$TAG" "$DMG" --repo "$REPO" --clobber
  gh release upload "$TAG" "$UPDATER_STAGED" --repo "$REPO" --clobber
  gh release upload "$TAG" "$LATEST_JSON" --repo "$REPO" --clobber
  echo "Updated existing release ${TAG}"
else
  gh release create "$TAG" \
    "$DMG" \
    "$UPDATER_STAGED" \
    "$LATEST_JSON" \
    --repo "$REPO" \
    --title "ParseKit v${VERSION}" \
    --notes "$RELEASE_NOTES"
  echo "Created release ${TAG}"
fi

echo "[5/5] Done."
echo "  DMG:          $DMG"
echo "  Updater:      $UPDATER_STAGED"
echo "  Manifest:     $LATEST_JSON"
echo "  Release URL:  https://github.com/${REPO}/releases/tag/${TAG}"