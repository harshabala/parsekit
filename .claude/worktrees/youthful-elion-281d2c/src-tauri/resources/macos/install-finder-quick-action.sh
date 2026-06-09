#!/usr/bin/env bash
# Install Finder Quick Action: "Parse to Markdown with ParseKit"
set -euo pipefail

SERVICE_NAME="Parse to Markdown with ParseKit"
SERVICES_DIR="${HOME}/Library/Services"
WORKFLOW_DIR="${SERVICES_DIR}/${SERVICE_NAME}.workflow"

if [[ -d "/Applications/ParseKit.app" ]]; then
  APP_BUNDLE="/Applications/ParseKit.app"
elif [[ -d "${HOME}/Applications/ParseKit.app" ]]; then
  APP_BUNDLE="${HOME}/Applications/ParseKit.app"
else
  ROOT="$(cd "$(dirname "$0")/.." && pwd)"
  if [[ -d "${ROOT}/src-tauri/target/release/bundle/macos/ParseKit.app" ]]; then
    APP_BUNDLE="${ROOT}/src-tauri/target/release/bundle/macos/ParseKit.app"
  else
    echo "error: ParseKit.app not found in /Applications — install the app first" >&2
    exit 1
  fi
fi

OPEN_SCRIPT="${APP_BUNDLE}/Contents/Resources/macos/open-with-parsekit.sh"
if [[ ! -x "${OPEN_SCRIPT}" ]]; then
  echo "error: missing ${OPEN_SCRIPT} — rebuild ParseKit.app" >&2
  exit 1
fi

INPUT_UUID=$(uuidgen)
OUTPUT_UUID=$(uuidgen)
ACTION_UUID=$(uuidgen)

rm -rf "${WORKFLOW_DIR}"
mkdir -p "${WORKFLOW_DIR}/Contents"

cat > "${WORKFLOW_DIR}/Contents/document.wflow" << WFLOW
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>AMApplicationBuild</key><string>523</string>
  <key>AMApplicationVersion</key><string>2.10</string>
  <key>AMDocumentVersion</key><string>2</string>
  <key>actions</key>
  <array>
    <dict>
      <key>action</key>
      <dict>
        <key>AMAccepts</key>
        <dict>
          <key>Container</key><string>List</string>
          <key>Optional</key><false/>
          <key>Types</key>
          <array><string>com.apple.cocoa.path</string></array>
        </dict>
        <key>AMActionVersion</key><string>2.0.3</string>
        <key>AMApplication</key><array><string>Automator</string></array>
        <key>AMProvides</key>
        <dict>
          <key>Container</key><string>List</string>
          <key>Types</key>
          <array><string>com.apple.cocoa.path</string></array>
        </dict>
        <key>ActionBundlePath</key>
        <string>/System/Library/Automator/Run Shell Script.action</string>
        <key>ActionName</key><string>Run Shell Script</string>
        <key>ActionParameters</key>
        <dict>
          <key>COMMAND_STRING</key>
          <string>"${OPEN_SCRIPT}" "\$@"</string>
          <key>CheckedForUserDefaultShell</key><true/>
          <key>inputMethod</key><integer>1</integer>
          <key>shell</key><string>/bin/bash</string>
          <key>source</key><string></string>
        </dict>
        <key>BundleIdentifier</key><string>com.apple.RunShellScript</string>
        <key>Class Name</key><string>RunShellScriptAction</string>
        <key>InputUUID</key><string>${INPUT_UUID}</string>
        <key>OutputUUID</key><string>${OUTPUT_UUID}</string>
        <key>UUID</key><string>${ACTION_UUID}</string>
      </dict>
    </dict>
  </array>
  <key>connectors</key><dict/>
  <key>workflowMetaData</key>
  <dict>
    <key>workflowTypeIdentifier</key>
    <string>com.apple.Automator.servicesMenu</string>
    <key>serviceInputTypeIdentifier</key>
    <string>com.apple.Automator.fileSystemObject</string>
    <key>serviceApplicationBundleID</key>
    <string>com.apple.finder</string>
    <key>serviceApplicationPath</key>
    <string>/System/Library/CoreServices/Finder.app</string>
    <key>serviceProcessesInput</key><integer>0</integer>
  </dict>
</dict>
</plist>
WFLOW

cat > "${WORKFLOW_DIR}/Contents/Info.plist" << PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleName</key>
  <string>${SERVICE_NAME}</string>
  <key>CFBundleIdentifier</key>
  <string>com.harshabala.parsekit.finder-action</string>
  <key>NSServices</key>
  <array>
    <dict>
      <key>NSMenuItem</key>
      <dict><key>default</key><string>${SERVICE_NAME}</string></dict>
      <key>NSMessage</key><string>runWorkflowAsService</string>
      <key>NSSendFileTypes</key>
      <array>
        <string>public.pdf</string>
        <string>com.adobe.pdf</string>
        <string>public.plain-text</string>
        <string>public.image</string>
        <string>public.content</string>
        <string>public.data</string>
        <string>public.item</string>
      </array>
    </dict>
  </array>
</dict>
</plist>
PLIST

/System/Library/CoreServices/pbs -update 2>/dev/null || true

echo "Installed Finder action: ${WORKFLOW_DIR}"
echo "Enable it in System Settings → Keyboard → Keyboard Shortcuts → Services (or Finder Quick Actions)."