#!/usr/bin/env bash
# Install Finder Quick Actions for ParseKit (default + replace-original).
set -euo pipefail

SERVICES_DIR="${HOME}/Library/Services"

if [[ -d "/Applications/ParseKit.app" ]]; then
  APP_BUNDLE="/Applications/ParseKit.app"
elif [[ -d "${HOME}/Applications/ParseKit.app" ]]; then
  APP_BUNDLE="${HOME}/Applications/ParseKit.app"
else
  ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
  if [[ -d "${ROOT}/target/release/bundle/macos/ParseKit.app" ]]; then
    APP_BUNDLE="${ROOT}/target/release/bundle/macos/ParseKit.app"
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

ICON_CANDIDATES=(
  "${APP_BUNDLE}/Contents/Resources/macos/finder-quick-action.icns"
  "$(cd "$(dirname "$0")" && pwd)/finder-quick-action.icns"
  "$(cd "$(dirname "$0")/../../.." && pwd)/assets/branding/finder-quick-action.icns"
)
WORKFLOW_ICON=""
for candidate in "${ICON_CANDIDATES[@]}"; do
  if [[ -f "${candidate}" ]]; then
    WORKFLOW_ICON="${candidate}"
    break
  fi
done

install_workflow() {
  local service_name="$1"
  local bundle_id="$2"
  local command_string="$3"
  local workflow_dir="${SERVICES_DIR}/${service_name}.workflow"
  local input_uuid output_uuid action_uuid

  input_uuid="$(uuidgen)"
  output_uuid="$(uuidgen)"
  action_uuid="$(uuidgen)"

  rm -rf "${workflow_dir}"
  mkdir -p "${workflow_dir}/Contents/Resources"

  if [[ -n "${WORKFLOW_ICON}" ]]; then
    cp "${WORKFLOW_ICON}" "${workflow_dir}/Contents/Resources/WorkflowIcon.icns"
  fi

  cat > "${workflow_dir}/Contents/document.wflow" << WFLOW
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
          <string>${command_string}</string>
          <key>CheckedForUserDefaultShell</key><true/>
          <key>inputMethod</key><integer>1</integer>
          <key>shell</key><string>/bin/bash</string>
          <key>source</key><string></string>
        </dict>
        <key>BundleIdentifier</key><string>com.apple.RunShellScript</string>
        <key>Class Name</key><string>RunShellScriptAction</string>
        <key>InputUUID</key><string>${input_uuid}</string>
        <key>OutputUUID</key><string>${output_uuid}</string>
        <key>UUID</key><string>${action_uuid}</string>
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

  local icon_plist=""
  if [[ -n "${WORKFLOW_ICON}" ]]; then
    icon_plist=$'  <key>CFBundleIconFile</key>\n  <string>WorkflowIcon</string>\n'
  fi

  cat > "${workflow_dir}/Contents/Info.plist" << PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleName</key>
  <string>${service_name}</string>
  <key>CFBundleIdentifier</key>
  <string>${bundle_id}</string>
${icon_plist}  <key>NSServices</key>
  <array>
    <dict>
      <key>NSMenuItem</key>
      <dict><key>default</key><string>${service_name}</string></dict>
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

  echo "Installed Finder action: ${workflow_dir}"
}

install_workflow \
  "Parse to Markdown with ParseKit" \
  "com.harshabala.parsekit.finder-action" \
  "\"${OPEN_SCRIPT}\" \"\$@\""

install_workflow \
  "Parse to Markdown with ParseKit (Replace Original)" \
  "com.harshabala.parsekit.finder-action-replace" \
  "PARSEKIT_REPLACE_ORIGINAL=1 \"${OPEN_SCRIPT}\" \"\$@\""

/System/Library/CoreServices/pbs -update 2>/dev/null || true

echo "Enable actions in System Settings → Keyboard → Keyboard Shortcuts → Services (or Finder Quick Actions)."