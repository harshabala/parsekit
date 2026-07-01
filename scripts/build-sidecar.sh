#!/usr/bin/env bash
set -euo pipefail

TRIPLE=$(rustc -Vv | grep host | awk '{print $2}')

if [ -z "$TRIPLE" ]; then
  echo "Error: could not determine host triple from rustc. Is rustc installed?" >&2
  exit 1
fi

mkdir -p src-tauri/binaries

OUT="src-tauri/binaries/parsekit-sidecar-$TRIPLE"
LEGACY="src-tauri/binaries/parsedock-sidecar-$TRIPLE"

# Tauri build.rs requires the sidecar resource to exist before compiling the crate.
if [[ ! -f "$OUT" && -f "$LEGACY" ]]; then
  cp "$LEGACY" "$OUT"
  chmod +x "$OUT"
  echo "Bootstrapped $OUT from legacy sidecar (will rebuild if sources changed)"
fi

if [[ ! -f "$OUT" ]]; then
  for app in "/Applications/ParseKit.app" "${HOME}/Applications/ParseKit.app"; do
    bundled="${app}/Contents/MacOS/parsekit-sidecar"
    if [[ -x "$bundled" ]]; then
      cp "$bundled" "$OUT"
      chmod +x "$OUT"
      echo "Bootstrapped $OUT from installed $app (will rebuild if sources changed)"
      break
    fi
  done
fi
SOURCES=(
  src-tauri/src/bin/parsekit-sidecar.rs
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

# Tauri build.rs requires binaries/parsekit-sidecar-<triple> before any cargo build
# in this crate (including --bin parsekit-sidecar). Seed a valid Mach-O stub on CI
# and fresh clones where no legacy/install bootstrap exists.
if [[ ! -f "$OUT" ]]; then
  cp /usr/bin/true "$OUT"
  chmod +x "$OUT"
  echo "Bootstrapped $OUT from /usr/bin/true (replaced after sidecar compile)"
fi

echo "Building LiteParse v2 sidecar (first build may take ~10 minutes)..."
cargo build --release --manifest-path src-tauri/Cargo.toml --bin parsekit-sidecar

cp "src-tauri/target/release/parsekit-sidecar" "$OUT"
chmod +x "$OUT"

echo "Built sidecar: $OUT"