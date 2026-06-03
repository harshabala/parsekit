#!/usr/bin/env bash
set -euo pipefail
pkill -9 parsedock 2>/dev/null || true
pkill -9 parsedock-sidecar 2>/dev/null || true
# Kill anything holding port 1420 (vite dev server)
lsof -ti :1420 2>/dev/null | xargs kill -9 2>/dev/null || true
# Kill tauri CLI watcher
pkill -f "tauri dev" 2>/dev/null || true
pkill -f "cargo run" 2>/dev/null || true
# Wait for port to free
sleep 1
echo "All ParseDock processes stopped."