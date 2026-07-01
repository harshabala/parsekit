#!/usr/bin/env bash
# Keep package.json, package-lock.json, tauri.conf.json, and Cargo.toml in sync.
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
const tauriConfPath = path.join(process.env.ROOT, "src-tauri/tauri.conf.json");
const tauriConf = JSON.parse(fs.readFileSync(tauriConfPath, "utf8"));
tauriConf.version = version;
fs.writeFileSync(tauriConfPath, JSON.stringify(tauriConf, null, 2) + "\n");

const cargoPath = path.join(process.env.ROOT, "src-tauri/Cargo.toml");
let cargo = fs.readFileSync(cargoPath, "utf8");
cargo = cargo.replace(/^version = ".*"$/m, 'version = "' + version + '"');
fs.writeFileSync(cargoPath, cargo);
NODE

echo "Synced version to $VERSION"