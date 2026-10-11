#!/usr/bin/env bash
# Keep package.json, package-lock.json, tauri.conf.json, Cargo.toml, and
# DMG download links in README/INSTALL in sync with the release version.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
VERSION="${1:?Usage: sync-version.sh <version>}"

cd "$ROOT"
npm version "$VERSION" --no-git-tag-version --allow-same-version >/dev/null

export VERSION ROOT
node <<'NODE'
const fs = require("fs");
const path = require("path");

const version = process.env.VERSION;
const root = process.env.ROOT;
const dmgName = `ParseKit_${version}_aarch64.dmg`;
const dmgPattern = /ParseKit_\d+\.\d+\.\d+_aarch64\.dmg/g;

const tauriConfPath = path.join(root, "src-tauri/tauri.conf.json");
const tauriConf = JSON.parse(fs.readFileSync(tauriConfPath, "utf8"));
tauriConf.version = version;
fs.writeFileSync(tauriConfPath, JSON.stringify(tauriConf, null, 2) + "\n");

const cargoPath = path.join(root, "src-tauri/Cargo.toml");
let cargo = fs.readFileSync(cargoPath, "utf8");
cargo = cargo.replace(/^version = ".*"$/m, 'version = "' + version + '"');
fs.writeFileSync(cargoPath, cargo);

// Keep public download links pointing at the current release DMG filename.
// GitHub's /releases/latest/download/<name> still requires the exact asset name.
for (const rel of ["README.md", "docs/INSTALL.md"]) {
  const filePath = path.join(root, rel);
  if (!fs.existsSync(filePath)) continue;
  const before = fs.readFileSync(filePath, "utf8");
  const after = before.replace(dmgPattern, dmgName);
  if (after !== before) fs.writeFileSync(filePath, after);
}
NODE

echo "Synced version to $VERSION"
