//! Global hotkey: parse Finder selection or clipboard file paths in the background.

use std::sync::Mutex;

use tauri::{AppHandle, Emitter, Manager, Runtime};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

pub const DEFAULT_GLOBAL_SHORTCUT: &str = "Control+Shift+KeyM";
const SETTINGS_KEY: &str = "globalShortcut";

#[derive(Default)]
pub struct GlobalHotkeyState {
    current_shortcut: Mutex<String>,
}

pub fn read_global_shortcut_from_settings() -> String {
    crate::read_settings_json()
        .and_then(|value| {
            value
                .get(SETTINGS_KEY)
                .and_then(|v| v.as_str())
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(str::to_string)
        })
        .unwrap_or_else(|| DEFAULT_GLOBAL_SHORTCUT.to_string())
}

#[cfg(target_os = "macos")]
fn get_finder_selection_paths() -> Vec<String> {
    let script = r#"
tell application "Finder"
    set sel to selection
    if sel is {} then return ""
    set out to ""
    repeat with itemRef in sel
        set end of out to (POSIX path of (itemRef as alias)) & linefeed
    end repeat
    return text 1 thru -2 of out
end tell
"#;
    crate::clipboard_paths::run_osascript(script)
        .map(|text| {
            text.lines()
                .map(str::trim)
                .filter(|line| !line.is_empty())
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}

#[cfg(not(target_os = "macos"))]
fn get_finder_selection_paths() -> Vec<String> {
    Vec::new()
}

fn handle_hotkey_pressed<R: Runtime>(app: &AppHandle<R>) {
    let finder_paths = get_finder_selection_paths();
    if !finder_paths.is_empty() {
        let files = crate::collect_supported_files(finder_paths);
        if files.is_empty() {
            let _ = crate::display_notification(
                "ParseKit",
                "No supported files in Finder selection.",
            );
            return;
        }
        let _ = app.emit("background-parse", files);
        return;
    }

    let clipboard_files = crate::clipboard_convert::resolve_clipboard_supported_files();
    if !clipboard_files.is_empty() {
        tauri::async_runtime::spawn_blocking(|| {
            crate::clipboard_convert::run_clipboard_convert_with_notification(
                "ParseKit",
                "Markdown copied to clipboard",
                "Clipboard convert failed",
            );
        });
        return;
    }

    let _ = crate::display_notification(
        "ParseKit",
        "No supported files in Finder selection or clipboard.",
    );
}

pub fn register_global_hotkey<R: Runtime>(
    app: &AppHandle<R>,
    shortcut: &str,
    state: &GlobalHotkeyState,
) -> Result<(), String> {
    let gs = app.global_shortcut();
    if let Ok(mut current) = state.current_shortcut.lock() {
        if !current.is_empty() && current.as_str() != shortcut {
            let _ = gs.unregister(current.as_str());
        }
        gs.on_shortcut(shortcut, |app, _shortcut, event| {
            if event.state == ShortcutState::Pressed {
                handle_hotkey_pressed(app);
            }
        })
        .map_err(|e| e.to_string())?;
        *current = shortcut.to_string();
    }
    Ok(())
}

pub fn setup_global_hotkey<R: Runtime>(app: &AppHandle<R>) -> Result<(), String> {
    let shortcut = read_global_shortcut_from_settings();
    let state = app
        .try_state::<GlobalHotkeyState>()
        .ok_or_else(|| "GlobalHotkeyState missing".to_string())?;
    register_global_hotkey(app, &shortcut, state.inner())
}

#[tauri::command]
pub fn get_global_shortcut(state: tauri::State<'_, GlobalHotkeyState>) -> String {
    state
        .current_shortcut
        .lock()
        .map(|s| s.clone())
        .unwrap_or_else(|_| DEFAULT_GLOBAL_SHORTCUT.to_string())
}

#[tauri::command]
pub fn update_global_shortcut<R: Runtime>(
    app: AppHandle<R>,
    state: tauri::State<'_, GlobalHotkeyState>,
    shortcut: String,
) -> Result<(), String> {
    let shortcut = shortcut.trim();
    if shortcut.is_empty() {
        return Err("Shortcut cannot be empty".into());
    }
    register_global_hotkey(&app, shortcut, state.inner())
}