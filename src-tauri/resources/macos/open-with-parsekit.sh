#!/bin/bash
# Finder Quick Action: headless parse when output folder is saved; otherwise queue files and open ParseKit.
# Set PARSEKIT_REPLACE_ORIGINAL=1 to move originals to Trash after a successful parse (recoverable).
set -euo pipefail

APP_NAME="ParseKit"
SUPPORT="$HOME/Library/Application Support/com.harshabala.parsekit"
SETTINGS="$SUPPORT/settings.json"
QUEUE="$SUPPORT/open-queue.json"
REPLACE_ORIGINAL="${PARSEKIT_REPLACE_ORIGINAL:-0}"

if [[ -d "/Applications/ParseKit.app" ]]; then
  APP_BUNDLE="/Applications/ParseKit.app"
elif [[ -d "${HOME}/Applications/ParseKit.app" ]]; then
  APP_BUNDLE="${HOME}/Applications/ParseKit.app"
else
  echo "Install ParseKit in Applications first." >&2
  exit 1
fi

SIDECAR="$APP_BUNDLE/Contents/MacOS/parsekit-sidecar"
CLI="$APP_BUNDLE/Contents/MacOS/parsekit-cli"

mkdir -p "$SUPPORT"

export PARSEKIT_APP="$APP_BUNDLE"
export PARSEKIT_SIDECAR="$SIDECAR"
export PARSEKIT_CLI="$CLI"
export PARSEKIT_SETTINGS="$SETTINGS"
export PARSEKIT_QUEUE="$QUEUE"
export PARSEKIT_APP_NAME="$APP_NAME"
export PARSEKIT_REPLACE_ORIGINAL="$REPLACE_ORIGINAL"

exec python3 - "$@" << 'PY'
import json
import os
import subprocess
import sys

paths = [p for p in sys.argv[1:] if p and os.path.exists(p)]
if not paths:
    sys.exit(0)

settings_path = os.environ["PARSEKIT_SETTINGS"]
settings = {}
if os.path.isfile(settings_path):
    try:
        with open(settings_path) as f:
            settings = json.load(f)
    except Exception:
        pass

output_dir = (settings.get("outputDir") or "").strip()
app_name = os.environ["PARSEKIT_APP_NAME"]
app_bundle = os.environ["PARSEKIT_APP"]
sidecar = os.environ["PARSEKIT_SIDECAR"]
cli = os.environ.get("PARSEKIT_CLI", "")
queue_path = os.environ["PARSEKIT_QUEUE"]
replace_original = os.environ.get("PARSEKIT_REPLACE_ORIGINAL") == "1"
stats_path = os.path.join(
    os.path.dirname(settings_path), "token-stats.json"
)

def default_token_stats():
    return {
        "total_files_converted": 0,
        "total_tokens_saved": 0,
        "total_pages_unlocked": 0,
        "total_documents_unlocked": 0,
        "by_file_type": {},
        "events": [],
    }

def load_token_stats():
    if not os.path.isfile(stats_path):
        return default_token_stats()
    try:
        with open(stats_path) as f:
            data = json.load(f)
    except Exception:
        return default_token_stats()
    base = default_token_stats()
    base.update({k: data.get(k, v) for k, v in base.items()})
    base["by_file_type"] = data.get("by_file_type") or {}
    base["events"] = data.get("events") or []
    return base

def save_token_stats(stats):
    os.makedirs(os.path.dirname(stats_path), exist_ok=True)
    with open(stats_path, "w") as f:
        json.dump(stats, f, indent=2)

def record_token_savings(file_type, tokens_saved, pages_unlocked, documents_unlocked):
    """Mirror src-tauri/src/token_stats.rs — CLI will call the same path in Task 8."""
    normalized = (file_type or "").strip().lstrip(".").lower()
    if not normalized:
        return
    stats = load_token_stats()
    tokens_saved = max(0, int(tokens_saved or 0))
    pages_unlocked = max(0, int(pages_unlocked or 0))
    documents_unlocked = max(0, int(documents_unlocked or 0))
    stats["total_files_converted"] = int(stats.get("total_files_converted", 0)) + 1
    stats["total_tokens_saved"] = int(stats.get("total_tokens_saved", 0)) + tokens_saved
    stats["total_pages_unlocked"] = int(stats.get("total_pages_unlocked", 0)) + pages_unlocked
    stats["total_documents_unlocked"] = int(
        stats.get("total_documents_unlocked", 0)
    ) + documents_unlocked
    by_type = stats.setdefault("by_file_type", {})
    entry = by_type.setdefault(normalized, {"files": 0, "tokens_saved": 0})
    entry["files"] = int(entry.get("files", 0)) + 1
    entry["tokens_saved"] = int(entry.get("tokens_saved", 0)) + tokens_saved
    stats.setdefault("events", []).append({
        "ts": __import__("datetime").datetime.now(
            __import__("datetime").timezone.utc
        ).strftime("%Y-%m-%dT%H:%M:%S+00:00"),
        "file_type": normalized,
        "tokens_saved": tokens_saved,
        "pages_unlocked": pages_unlocked,
    })
    save_token_stats(stats)

def notify(message):
    candidates = [
        os.environ.get("PARSEKIT_CLI", ""),
        os.path.join(app_bundle, "Contents/MacOS/parsekit-cli"),
    ]
    for cli in candidates:
        if cli and os.path.isfile(cli) and os.access(cli, os.X_OK):
            subprocess.run([cli, "notify", message], check=False)
            return
    print(f"ParseKit notification skipped (CLI missing): {message}", file=sys.stderr)

def open_app():
    subprocess.run(["open", "-ga", app_name], check=False)
    if subprocess.run(["pgrep", "-xq", app_name]).returncode != 0:
        subprocess.run(["open", "-a", app_bundle], check=False)

def queue_paths():
    existing = {"paths": []}
    if os.path.isfile(queue_path):
        try:
            with open(queue_path) as f:
                existing = json.load(f)
        except Exception:
            pass
    existing.setdefault("paths", []).extend(paths)
    with open(queue_path, "w") as f:
        json.dump(existing, f, indent=2)

def trash_original(path):
    if not replace_original or not path or not os.path.exists(path):
        return False
    if os.path.isfile(cli) and os.access(cli, os.X_OK):
        result = subprocess.run([cli, "trash", path], capture_output=True, check=False)
        return result.returncode == 0
    escaped = path.replace("\\", "\\\\").replace('"', '\\"')
    result = subprocess.run(
        [
            "osascript",
            "-e",
            f'tell application "Finder" to delete POSIX file "{escaped}"',
        ],
        check=False,
    )
    return result.returncode == 0

if replace_original and not output_dir:
    notify("Set an output folder in ParseKit Settings before using Replace Original.")
    sys.exit(1)

if not output_dir:
    queue_paths()
    open_app()
    sys.exit(0)

show_hud = bool(settings.get("showFloatingHud", True))
if show_hud:
    with open(queue_path, "w") as f:
        json.dump({"paths": paths, "background": True}, f, indent=2)
    open_app()
    sys.exit(0)

if not os.path.isfile(sidecar) or not os.access(sidecar, os.X_OK):
    notify("Parse engine missing. Reinstall ParseKit.")
    sys.exit(1)

config = {
    "files": paths,
    "outputDir": output_dir,
    "format": settings.get("format") or "md",
    "ocrEnabled": settings.get("ocrEnabled", True),
    "ocrLanguage": settings.get("ocrLanguage") or "eng",
    "workers": int(settings.get("workers") or 0) or 4,
}
payload = (json.dumps(config) + "\n").encode()

proc = subprocess.run(
    [sidecar],
    input=payload,
    capture_output=True,
    check=False,
)
parsed = errors = trashed = 0
completed_sources = []
for line in proc.stdout.decode(errors="replace").splitlines():
    line = line.strip()
    if not line:
        continue
    try:
        ev = json.loads(line)
    except json.JSONDecodeError:
        continue
    if ev.get("type") == "progress":
        st = ev.get("status")
        if st in ("completed", "done"):
            parsed += 1
            source = ev.get("sourcePath") or ""
            if source:
                completed_sources.append(source)
        elif st == "error":
            errors += 1
    elif ev.get("type") == "token_savings":
        record_token_savings(
            ev.get("file_type"),
            ev.get("tokens_saved"),
            ev.get("pages_unlocked"),
            ev.get("documents_unlocked"),
        )

for source in completed_sources:
    if trash_original(source):
        trashed += 1

if replace_original:
    msg = f"Done: {parsed} parsed, {errors} errors, {trashed} moved to Trash."
else:
    msg = f"Done: {parsed} parsed, {errors} errors."
notify(msg)
PY