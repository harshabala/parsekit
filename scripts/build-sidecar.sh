#!/usr/bin/env bash
set -euo pipefail

TRIPLE=$(rustc -Vv | grep host | awk '{print $2}')

if [ -z "$TRIPLE" ]; then
  echo "Error: could not determine host triple from rustc. Is rustc installed?" >&2
  exit 1
fi

mkdir -p src-tauri/binaries

OUT="src-tauri/binaries/parsedock-sidecar-$TRIPLE"
SOURCES=(
  src-tauri/src/bin/parsedock-sidecar.rs
  src-tauri/src/sidecar_helpers.rs
  src-tauri/src/lib.rs
  src-tauri/Cargo.toml
  src-tauri/Cargo.lock
)

needs_build=true
if [[ -f "$OUT" ]]; then
  needs_build=false
  for src in "${SOURCES[@]}"; do
    if [[ "$src" -nt "$OUT" ]]; then
      needs_build=true
      break
    fi
  done
fi

if [[ "$needs_build" == "false" ]]; then
  echo "Sidecar up to date: $OUT"
  exit 0
fi

echo "Building LiteParse v2 sidecar (first build may take ~10 minutes)..."
cargo build --release --manifest-path src-tauri/Cargo.toml --bin parsedock-sidecar

cp "src-tauri/target/release/parsedock-sidecar" "$OUT"
chmod +x "$OUT"

echo "Built sidecar: $OUT"