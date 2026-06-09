#!/bin/bash
# Finder Quick Action: headless parse when output folder is saved; otherwise queue files and open ParseKit.
set -euo pipefail

APP_NAME="ParseKit"
SUPPORT="$HOME/Library/Application Support/com.harshabala.parsekit"
SETTINGS="$SUPPORT/settings.json"
QUEUE="$SUPPORT/open-queue.json"

if [[ -d "/Applications/ParseKit.app" ]]; then
  APP_BUNDLE="/Applications/ParseKit.app"
elif [[ -d "${HOME}/Applications/ParseKit.app" ]]; then
  APP_BUNDLE="${HOME}/Applications/ParseKit.app"
else
  osascript -e 'display notification "Install ParseKit in Applications first." with title "ParseKit"' 2>/dev/null || true
  exit 1
fi

SIDECAR="$APP_BUNDLE/Contents/MacOS/parsekit-sidecar"

mkdir -p "$SUPPORT"

export PARSEKIT_APP="$APP_BUNDLE"
export PARSEKIT_SIDECAR="$SIDECAR"
export PARSEKIT_SETTINGS="$SETTINGS"
export PARSEKIT_QUEUE="$QUEUE"
export PARSEKIT_APP_NAME="$APP_NAME"

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
queue_path = os.environ["PARSEKIT_QUEUE"]

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

if not output_dir:
    queue_paths()
    open_app()
    sys.exit(0)

if not os.path.isfile(sidecar) or not os.access(sidecar, os.X_OK):
    subprocess.run([
        "osascript", "-e",
        'display notification "Parse engine missing. Reinstall ParseKit." with title "ParseKit"',
    ], check=False)
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
parsed = errors = 0
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
        elif st == "error":
            errors += 1

msg = f"Done: {parsed} parsed, {errors} errors."
subprocess.run(
    ["osascript", "-e", f'display notification "{msg}" with title "ParseKit"'],
    check=False,
)
PY