#!/usr/bin/env bash
set -euo pipefail
pkill -9 parsekit 2>/dev/null || true
pkill -9 parsedock 2>/dev/null || true
pkill -9 parsekit-sidecar 2>/dev/null || true
pkill -9 parsedock-sidecar 2>/dev/null || true
lsof -ti :1420 2>/dev/null | xargs kill -9 2>/dev/null || true
pkill -f "tauri dev" 2>/dev/null || true
pkill -f "cargo run" 2>/dev/null || true
sleep 1
echo "All ParseKit processes stopped."