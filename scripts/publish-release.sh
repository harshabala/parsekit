#!/usr/bin/env bash
# Build ParseKit, seal-sign .app (postbuild-macos), tar signed .app, sign tarball, upload.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
VERSION="$(node -p "require('$ROOT/package.json').version")"
TAG="v${VERSION}"
REPO="harshabala/parsekit"
KEY_PATH="${TAURI_SIGNING_PRIVATE_KEY_PATH:-$HOME/.tauri/parsekit.key}"

APP="$ROOT/src-tauri/target/release/bundle/macos/ParseKit.app"
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

echo "[1/6] tauri build + postbuild-macos (whole-bundle codesign + DMG) ..."
cd "$ROOT"
npm run release:macos

if [[ ! -d "$APP" ]]; then
  echo "error: signed app missing at $APP" >&2
  exit 1
fi

echo "[2/6] Fail-fast codesign on release .app ..."
xattr -cr "$APP" 2>/dev/null || true
codesign --verify --deep --strict --verbose=2 "$APP"

echo "[3/6] Create updater .tar.gz from signed .app (not Tauri pre-sign artifact) ..."
bash "$ROOT/scripts/create-updater-tarball.sh" "$APP" "$UPDATER_STAGED"

SIG_FILE="${UPDATER_STAGED}.sig"
rm -f "$SIG_FILE"
echo "[4/6] Sign updater tarball with minisign ..."
(
  cd "$ROOT"
  # Avoid TAURI_SIGNING_PRIVATE_KEY path conflicting with -f (tauri signer rejects both).
  env -u TAURI_SIGNING_PRIVATE_KEY npx tauri signer sign -f "$KEY_PATH" -p "${TAURI_SIGNING_PRIVATE_KEY_PASSWORD}" "$UPDATER_STAGED"
)

if [[ ! -f "$SIG_FILE" ]]; then
  echo "error: missing signature file $SIG_FILE" >&2
  exit 1
fi

echo "[5/6] Write parsekit-latest.json ..."
# Prefer docs/releases/vX.Y.Z.md; fall back to a clear install-focused body.
# RELEASE_NOTES env still overrides when set (workflow / maintainer).
NOTES_FILE="$(mktemp)"
bash "$ROOT/scripts/release-notes.sh" "$VERSION" >"$NOTES_FILE"
RELEASE_NOTES="$(cat "$NOTES_FILE")"
export VERSION SIG_FILE LATEST_JSON UPDATER_NAME RELEASE_NOTES
# Updater banner uses a short plain-text summary (first line of notes, stripped).
SHORT_NOTES="$(head -n 1 "$NOTES_FILE" | sed 's/^#* *//')"
export SHORT_NOTES
node <<'NODE'
const fs = require("fs");
const version = process.env.VERSION;
const signature = fs.readFileSync(process.env.SIG_FILE, "utf8").trim();
const json = {
  version,
  notes: process.env.SHORT_NOTES || `ParseKit v${version}`,
  pub_date: new Date().toISOString(),
  platforms: {
    "darwin-aarch64": {
      signature,
      url: `https://github.com/harshabala/parsekit/releases/download/v${version}/${process.env.UPDATER_NAME}`,
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

echo "[6/6] Upload to GitHub release ${TAG} (manifest + signed tar.gz + DMG only) ..."
if gh release view "$TAG" --repo "$REPO" &>/dev/null; then
  gh release upload "$TAG" "$DMG" --repo "$REPO" --clobber
  gh release upload "$TAG" "$UPDATER_STAGED" --repo "$REPO" --clobber
  gh release upload "$TAG" "$SIG_FILE" --repo "$REPO" --clobber 2>/dev/null || true
  gh release upload "$TAG" "$LATEST_JSON" --repo "$REPO" --clobber
  gh release edit "$TAG" --repo "$REPO" --title "ParseKit v${VERSION}" --notes-file "$NOTES_FILE"
  echo "Updated existing release ${TAG}"
else
  gh release create "$TAG" \
    "$DMG" \
    "$UPDATER_STAGED" \
    "$LATEST_JSON" \
    --repo "$REPO" \
    --title "ParseKit v${VERSION}" \
    --notes-file "$NOTES_FILE"
  echo "Created release ${TAG}"
fi
rm -f "$NOTES_FILE"

# Remove stray latest.json if a prior upload added it.
if gh release view "$TAG" --repo "$REPO" --json assets -q '.assets[].name' 2>/dev/null | grep -qx 'latest.json'; then
  echo "Removing duplicate asset latest.json from ${TAG} ..."
  gh release delete-asset "$TAG" latest.json --repo "$REPO" --yes 2>/dev/null || true
fi

echo "Done."
echo "  DMG:          $DMG"
echo "  Updater:      $UPDATER_STAGED"
echo "  Manifest:     $LATEST_JSON"
echo "  Release URL:  https://github.com/${REPO}/releases/tag/${TAG}"