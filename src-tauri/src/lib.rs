pub mod clipboard_convert;
pub mod clipboard_paths;
pub mod global_hotkey;
pub mod macos_popover;
#[cfg(target_os = "macos")]
pub mod macos_open_files;
#[cfg(target_os = "macos")]
pub mod macos_notification;
#[cfg(target_os = "macos")]
pub mod macos_trash;
pub mod popover_trace;
pub mod sidecar_helpers;
pub mod token_count;
pub mod token_stats;

use popover_trace::trace as popover_trace;

pub use popover_trace::startup_trace;

use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{
    MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent, TrayIconId,
};
use tauri::{AppHandle, Manager, PhysicalPosition, Position, Rect, Runtime, Size, State, WebviewWindow};
use tauri_plugin_dialog::DialogExt;
use tauri_plugin_updater::UpdaterExt;
use walkdir::WalkDir;

/// Menu bar template glyph (black on transparent); see `icons/tray/icon*.png`.
const TRAY_ICON: tauri::image::Image<'static> = tauri::include_image!("icons/tray/icon.png");
/// Ignore focus-loss hides briefly after `Window.show()` so activation does not collapse the panel.
/// Just long enough to ride out activation focus churn; short enough that click-away still dismisses.
const POPOVER_SHOW_GRACE_MS: u64 = 900;

/// Set `false` only when diagnosing focus-loss; production should keep `true`.
const FOCUS_LOSS_AUTO_HIDE_ENABLED: bool = true;

#[derive(Clone)]
pub(crate) struct PopoverState {
    /// Authoritative open/closed state — do not use `window.is_visible()` for toggling;
    /// macOS can report visible=true while the accessory popover is not on screen.
    is_open: Arc<AtomicBool>,
    last_opened_at: Arc<Mutex<Option<Instant>>>,
    picker_active: Arc<AtomicBool>,
}

impl PopoverState {
    fn new() -> Self {
        Self {
            is_open: Arc::new(AtomicBool::new(false)),
            last_opened_at: Arc::new(Mutex::new(None)),
            picker_active: Arc::new(AtomicBool::new(false)),
        }
    }

    fn is_open(&self) -> bool {
        self.is_open.load(Ordering::SeqCst)
    }

    fn set_open(&self, open: bool) {
        self.is_open.store(open, Ordering::SeqCst);
        if !open {
            if let Ok(mut last_opened_at) = self.last_opened_at.lock() {
                *last_opened_at = None;
            }
        }
    }

    /// Anchor the grace window at the moment the panel is shown. Focus-loss events that
    /// arrive within `POPOVER_SHOW_GRACE_MS` of this are activation churn and are ignored.
    pub(crate) fn mark_opening(&self) {
        if let Ok(mut last_opened_at) = self.last_opened_at.lock() {
            *last_opened_at = Some(Instant::now());
        }
    }

    fn begin_picker(&self) {
        self.picker_active.store(true, Ordering::SeqCst);
    }

    fn end_picker(&self) {
        self.picker_active.store(false, Ordering::SeqCst);
    }

    fn should_hide_on_focus_loss(&self) -> bool {
        if !FOCUS_LOSS_AUTO_HIDE_ENABLED {
            return false;
        }
        if !self.is_open() {
            return false;
        }
        if self.picker_active.load(Ordering::SeqCst) {
            return false;
        }
        self.last_opened_at
            .lock()
            .ok()
            .and_then(|last_opened_at| *last_opened_at)
            .map(|opened_at| opened_at.elapsed() > Duration::from_millis(POPOVER_SHOW_GRACE_MS))
            .unwrap_or(false)
    }
}

/// Keeps the tray icon alive for the app lifetime (tray-icon drops NSStatusItem when the last clone goes away).
#[derive(Clone)]
#[allow(dead_code)]
struct TrayHandle<R: Runtime> {
    icon: TrayIcon<R>,
}

#[derive(Clone, Default)]
struct TrayRectState(Arc<Mutex<Option<Rect>>>);

#[derive(Clone)]
struct TrayMenuState {
    tray_id: Arc<Mutex<TrayIconId>>,
    open_label: Arc<Mutex<String>>,
    clipboard_label: Arc<Mutex<String>>,
    quit_label: Arc<Mutex<String>>,
}

/// Collapses the burst of duplicate tray `Click` events macOS delivers for a single
/// physical click into one toggle.
///
/// AppKit emits a non-deterministic number of Down/Up pairs per physical tray click
/// (3–5 observed in traces), so counting spurious events cannot work — the count varies.
/// Instead we debounce by time: the first Up toggles; any further Ups within the debounce
/// window belong to the same physical click and are ignored, regardless of how many arrive.
#[derive(Clone)]
struct TrayClickDebounce {
    last_toggle_at: Arc<Mutex<Option<Instant>>>,
}

/// Tray `Up` events closer together than this belong to one physical click and collapse to
/// a single toggle. Bursts arrive within a few ms; a deliberate human re-click is far
/// slower, so this never swallows an intentional second click.
const TRAY_TOGGLE_DEBOUNCE_MS: u64 = 400;

impl TrayClickDebounce {
    fn new() -> Self {
        Self {
            last_toggle_at: Arc::new(Mutex::new(None)),
        }
    }

    /// Returns `true` if this click should toggle the popover; `false` if it is a burst
    /// duplicate of a click already handled within the debounce window.
    fn accept_click(&self) -> bool {
        let Ok(mut last_toggle_at) = self.last_toggle_at.lock() else {
            return true;
        };
        let now = Instant::now();
        if let Some(prev) = *last_toggle_at {
            if now.duration_since(prev) < Duration::from_millis(TRAY_TOGGLE_DEBOUNCE_MS) {
                return false;
            }
        }
        *last_toggle_at = Some(now);
        true
    }
}

fn build_tray_menu<R: Runtime>(
    app: &AppHandle<R>,
    open_label: &str,
    clipboard_label: &str,
    quit_label: &str,
) -> tauri::Result<Menu<R>> {
    let open_item = MenuItem::with_id(app, "open_parsekit", open_label, true, None::<&str>)?;
    let clipboard_item = MenuItem::with_id(
        app,
        "parse_clipboard",
        clipboard_label,
        true,
        None::<&str>,
    )?;
    let separator = PredefinedMenuItem::separator(app)?;
    let quit_item = PredefinedMenuItem::quit(app, Some(quit_label))?;
    Menu::with_items(
        app,
        &[&open_item, &clipboard_item, &separator, &quit_item],
    )
}

fn popup_tray_menu<R: Runtime>(app: &AppHandle<R>, tray_state: &TrayMenuState) -> Result<(), String> {
    popover_trace("Tray right-click: popup_menu");
    let open_label = tray_state
        .open_label
        .lock()
        .map_err(|e| e.to_string())?;
    let clipboard_label = tray_state
        .clipboard_label
        .lock()
        .map_err(|e| e.to_string())?;
    let quit_label = tray_state
        .quit_label
        .lock()
        .map_err(|e| e.to_string())?;
    let menu = build_tray_menu(app, &open_label, &clipboard_label, &quit_label)
        .map_err(|e| e.to_string())?;
    let window = app
        .get_webview_window("main")
        .ok_or_else(|| "Main window not found".to_string())?;
    window.popup_menu(&menu).map_err(|e| e.to_string())
}

/// `window.center()` can land on the wrong monitor in multi-display setups (e.g. pos -1600).
fn center_on_primary_monitor<R: Runtime>(window: &WebviewWindow<R>) -> bool {
    let Ok(win_size) = window.outer_size() else {
        return false;
    };
    let Some(monitor) = window
        .primary_monitor()
        .ok()
        .flatten()
    else {
        return window.center().is_ok();
    };
    let mon_pos = monitor.position();
    let mon_size = monitor.size();
    let x = mon_pos.x + (mon_size.width as i32 - win_size.width as i32) / 2;
    let y = mon_pos.y + (mon_size.height as i32 - win_size.height as i32) / 2;
    window
        .set_position(PhysicalPosition::new(x, y))
        .is_ok()
}

fn position_popover_under_tray<R: Runtime>(window: &WebviewWindow<R>, rect: &Rect) -> bool {
    popover_trace("Positioning: start (under tray)");
    let Ok(win_size) = window.outer_size() else {
        popover_trace("Positioning: FAILED (outer_size)");
        return false;
    };
    let scale = window.scale_factor().unwrap_or(1.0);
    let (icon_x, icon_y, icon_w, icon_h) = physical_xywh(rect, scale);
    let win_w = win_size.width as f64;
    let win_h = win_size.height as f64;
    let (monitor_width, monitor_height) = monitor_size_f64(window, (f64::MAX, f64::MAX));

    let x = (icon_x + icon_w / 2.0 - win_w / 2.0)
        .max(0.0)
        .min(monitor_width - win_w) as i32;
    let y = (icon_y + icon_h) as i32;

    // Clamp to visible menu-bar region; never push the panel off-screen.
    let y = y.clamp(24, (monitor_height - win_h - 8.0).max(24.0) as i32);
    let x = x.clamp(8, (monitor_width - win_w - 8.0).max(8.0) as i32);

    let _ = window.set_position(PhysicalPosition::new(x, y));
    popover_trace(&format!("Positioning: OK x={x} y={y} win={win_w}x{win_h}"));
    true
}

fn show_popover<R: Runtime>(
    window: &WebviewWindow<R>,
    rect: Option<&Rect>,
    popover: &PopoverState,
) {
    popover_trace("PopoverManager.show()");
    // Start grace before any show/focus work so tray-click focus-loss cannot instantly hide.
    popover.mark_opening();

    // Center on primary display first so the panel is on-screen even if tray coords are wrong.
    let _ = center_on_primary_monitor(window);
    popover_trace("Positioning: center(primary)");
    if let Some(r) = rect {
        let _ = position_popover_under_tray(window, r);
    } else {
        popover_trace("Positioning: skipped (no tray rect)");
    }

    macos_popover::activate_app_for_popover(window, popover);
    let visible = window.is_visible().unwrap_or(false);
    popover.set_open(visible);
    if !visible {
        popover_trace("PopoverManager.show(): window not visible after activate — is_open=false");
    }
    log_window_visibility(window, "after show_popover");
}

fn hide_popover<R: Runtime>(window: &WebviewWindow<R>, popover: &PopoverState) {
    popover_trace("PopoverManager.hide()");
    popover.set_open(false);
    let _ = window.hide();
    log_window_visibility(window, "after hide_popover");
}

fn log_window_visibility<R: Runtime>(window: &WebviewWindow<R>, context: &str) {
    let visible = window.is_visible().unwrap_or(false);
    let size = window
        .outer_size()
        .map(|s| format!("{}x{}", s.width, s.height))
        .unwrap_or_else(|_| "?".into());
    let pos = window
        .outer_position()
        .map(|p| format!("({}, {})", p.x, p.y))
        .unwrap_or_else(|_| "?".into());
    let scale = window.scale_factor().unwrap_or(0.0);
    popover_trace(&format!(
        "Window.visibility [{context}]: visible={visible} size={size} pos={pos} scale={scale}"
    ));
}

fn toggle_popover_from_tray<R: Runtime>(
    window: &WebviewWindow<R>,
    rect: &Rect,
    popover: &PopoverState,
) {
    popover_trace("PopoverManager.toggle()");
    if popover.is_open() {
        popover_trace("toggle: branch hide (open)");
        hide_popover(window, popover);
    } else {
        popover_trace("toggle: branch show (closed)");
        show_popover(window, Some(rect), popover);
    }
}

fn open_popover_from_menu<R: Runtime>(
    app: &AppHandle<R>,
    window: &WebviewWindow<R>,
    tray_state: &TrayMenuState,
    popover: &PopoverState,
) {
    popover_trace("Tray menu: Open ParseKit → show");
    let Ok(tray_id) = tray_state.tray_id.lock().map(|id| id.clone()) else {
        popover_trace("Tray menu: ABORT (tray_id lock poisoned)");
        return;
    };
    let rect = app
        .tray_by_id(&tray_id)
        .and_then(|t| t.rect().ok().flatten());
    show_popover(window, rect.as_ref(), popover);
}

#[tauri::command]
fn update_tray_menu_labels(
    tray_state: State<TrayMenuState>,
    open_label: String,
    clipboard_label: String,
    quit_label: String,
) -> Result<(), String> {
    // Only update labels — never call tray.set_menu(), which re-attaches the menu to
    // NSStatusItem and causes macOS to swallow left-clicks.
    *tray_state
        .open_label
        .lock()
        .map_err(|e| e.to_string())? = open_label;
    *tray_state
        .clipboard_label
        .lock()
        .map_err(|e| e.to_string())? = clipboard_label;
    *tray_state
        .quit_label
        .lock()
        .map_err(|e| e.to_string())? = quit_label;
    Ok(())
}

#[cfg(target_os = "macos")]
fn elevate_app_for_file_dialog<R: Runtime>(app: &AppHandle<R>) {
    let _ = app.set_activation_policy(tauri::ActivationPolicy::Regular);
}

#[cfg(target_os = "macos")]
fn restore_accessory_app_policy<R: Runtime>(app: &AppHandle<R>) {
    let _ = app.set_activation_policy(tauri::ActivationPolicy::Accessory);
}

pub(crate) fn normalize_user_path(path: impl AsRef<str>) -> String {
    let trimmed = path.as_ref().trim();
    if trimmed.starts_with("file://") {
        if let Ok(url) = url::Url::parse(trimmed) {
            if let Ok(p) = url.to_file_path() {
                return p.to_string_lossy().into_owned();
            }
        }
    }
    trimmed.to_string()
}

pub fn app_support_dir() -> Result<std::path::PathBuf, String> {
    let home = std::env::var("HOME").map_err(|e| e.to_string())?;
    Ok(std::path::PathBuf::from(home)
        .join("Library/Application Support/com.harshabala.parsekit"))
}

pub fn settings_json_path() -> Result<std::path::PathBuf, String> {
    Ok(app_support_dir()?.join("settings.json"))
}

pub fn read_settings_json() -> Option<serde_json::Value> {
    let raw = std::fs::read_to_string(settings_json_path().ok()?).ok()?;
    serde_json::from_str(&raw).ok()
}

/// Resolve and validate a user-supplied path before spawning `open` or similar OS commands.
fn validate_user_path(path: &str) -> Result<std::path::PathBuf, String> {
    let normalized = normalize_user_path(path);
    if normalized.is_empty() {
        return Err("Path is empty".into());
    }
    if normalized.as_bytes().contains(&0) || normalized.chars().any(|c| c.is_control()) {
        return Err("Invalid path".into());
    }
    std::fs::canonicalize(&normalized).map_err(|e| format!("Path does not exist: {e}"))
}

fn file_path_to_string(path: tauri_plugin_dialog::FilePath) -> Option<String> {
    path.into_path()
        .ok()
        .and_then(|p| p.into_os_string().into_string().ok())
        .map(normalize_user_path)
}

fn physical_xywh(rect: &Rect, scale: f64) -> (f64, f64, f64, f64) {
    let x = match rect.position {
        Position::Physical(p) => p.x as f64,
        Position::Logical(p) => p.x * scale,
    };
    let y = match rect.position {
        Position::Physical(p) => p.y as f64,
        Position::Logical(p) => p.y * scale,
    };
    let w = match rect.size {
        Size::Physical(s) => s.width as f64,
        Size::Logical(s) => s.width * scale,
    };
    let h = match rect.size {
        Size::Physical(s) => s.height as f64,
        Size::Logical(s) => s.height * scale,
    };
    (x, y, w, h)
}

fn monitor_size_f64<R: Runtime>(window: &WebviewWindow<R>, fallback: (f64, f64)) -> (f64, f64) {
    window
        .current_monitor()
        .ok()
        .flatten()
        .map(|m| (m.size().width as f64, m.size().height as f64))
        .unwrap_or(fallback)
}

/// Open native pickers without a parent window. Borderless accessory popovers cannot host
/// sheet modals; attaching the main window as parent causes a system beep and no Finder UI.
async fn run_file_picker_async<R: Runtime, Fut>(
    app: AppHandle<R>,
    popover: &PopoverState,
    pick: impl FnOnce(AppHandle<R>) -> Fut,
) -> Result<Option<Vec<String>>, String>
where
    Fut: std::future::Future<Output = Option<Vec<String>>>,
{
    popover_trace("File picker: async session begin");
    popover.begin_picker();
    #[cfg(target_os = "macos")]
    {
        elevate_app_for_file_dialog(&app);
        popover_trace("File picker: activation policy → Regular");
    }

    let result = pick(app.clone()).await;

    #[cfg(target_os = "macos")]
    {
        let popover_still_open = popover.is_open();
        if !popover_still_open {
            restore_accessory_app_policy(&app);
            popover_trace("File picker: activation policy → Accessory (popover closed)");
        } else {
            popover_trace("File picker: keeping activation policy (popover still open)");
        }
    }
    popover.end_picker();
    popover_trace("File picker: async session end");
    Ok(result)
}

async fn await_pick_files(app: &AppHandle, title: &str) -> Result<Option<Vec<String>>, String> {
    let (tx, rx) = tokio::sync::oneshot::channel();
    app.dialog()
        .file()
        .set_title(title)
        .add_filter("Supported documents", SUPPORTED_EXTENSIONS)
        .pick_files(move |paths| {
            let _ = tx.send(paths);
        });
    match rx.await {
        Ok(Some(files)) => Ok(Some(
            files.into_iter().filter_map(file_path_to_string).collect(),
        )),
        Ok(None) => Ok(None),
        Err(_) => Ok(None),
    }
}

async fn await_pick_folder(app: &AppHandle, title: &str) -> Result<Option<String>, String> {
    let (tx, rx) = tokio::sync::oneshot::channel();
    app.dialog()
        .file()
        .set_title(title)
        .pick_folder(move |path| {
            let _ = tx.send(path);
        });
    match rx.await {
        Ok(Some(path)) => Ok(file_path_to_string(path)),
        Ok(None) => Ok(None),
        Err(_) => Ok(None),
    }
}

#[tauri::command]
async fn pick_input_files(
    app: AppHandle,
    popover: State<'_, PopoverState>,
) -> Result<Option<Vec<String>>, String> {
    run_file_picker_async(app, popover.inner(), |app| async move {
        await_pick_files(&app, "Select files to parse")
            .await
            .ok()
            .flatten()
    })
    .await
}

async fn pick_named_folder(
    app: AppHandle,
    popover: State<'_, PopoverState>,
    title: &'static str,
) -> Result<Option<String>, String> {
    run_file_picker_async(app, popover.inner(), move |app| async move {
        await_pick_folder(&app, title)
            .await
            .ok()
            .flatten()
            .map(|path| vec![path])
    })
    .await
    .map(|paths| paths.and_then(|mut v| v.pop()))
}

#[tauri::command]
async fn pick_input_folder(
    app: AppHandle,
    popover: State<'_, PopoverState>,
) -> Result<Option<String>, String> {
    pick_named_folder(app, popover, "Select folder").await
}

#[tauri::command]
async fn pick_output_folder(
    app: AppHandle,
    popover: State<'_, PopoverState>,
) -> Result<Option<String>, String> {
    pick_named_folder(app, popover, "Choose output folder").await
}

#[cfg(target_os = "macos")]
fn detect_apple_silicon() -> bool {
    std::process::Command::new("sysctl")
        .arg("-n")
        .arg("machdep.cpu.brand_string")
        .output()
        .map(|o| {
            let output = String::from_utf8_lossy(&o.stdout);
            output.contains("Apple")
                || output.contains("M1")
                || output.contains("M2")
                || output.contains("M3")
                || output.contains("M4")
        })
        .unwrap_or(false)
}

#[cfg(target_os = "macos")]
fn get_optimal_workers() -> usize {
    if detect_apple_silicon() {
        6
    } else {
        4
    }
}

#[cfg(not(target_os = "macos"))]
fn get_optimal_workers() -> usize {
    4
}

#[derive(serde::Serialize)]
struct DependencyStatus {
    id: String,
    #[serde(rename = "labelKey")]
    label_key: String,
    installed: bool,
    optional: bool,
    #[serde(rename = "brewHint")]
    brew_hint: String,
}

/// PATH used for `which` and inherited by child processes (sidecar → liteparse shells out).
const HOMEBREW_AUGMENTED_PATH: &str =
    "/opt/homebrew/bin:/usr/local/bin:/opt/local/bin:/usr/bin:/bin:/usr/sbin:/sbin";

const TOOL_PATH_PREFIXES: &[&str] = &["/opt/homebrew/bin", "/usr/local/bin", "/opt/local/bin"];

fn path_list_contains(path_var: &str, dir: &str) -> bool {
    path_var.split(':').any(|p| p == dir)
}

/// Prepend common macOS tool dirs so Finder-launched apps (minimal PATH) can run `soffice` / `magick`.
fn ensure_homebrew_on_process_path() {
    let current = std::env::var("PATH").unwrap_or_default();
    let mut prefix = String::new();
    for dir in TOOL_PATH_PREFIXES {
        if !path_list_contains(&current, dir) {
            prefix.push_str(dir);
            prefix.push(':');
        }
    }
    if prefix.is_empty() {
        return;
    }
    if current.is_empty() {
        std::env::set_var("PATH", HOMEBREW_AUGMENTED_PATH);
    } else {
        std::env::set_var("PATH", format!("{prefix}{current}"));
    }
}

fn shell_which_on_augmented_path(name: &str) -> bool {
    std::process::Command::new("/usr/bin/which")
        .arg(name)
        .env("PATH", HOMEBREW_AUGMENTED_PATH)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// True if the tool exists at a known install path or is found via augmented `which`.
fn dependency_tool_installed(
    candidate_paths: &[&str],
    which_names: &[&str],
    path_exists: impl Fn(&str) -> bool,
    which: impl Fn(&str) -> bool,
) -> bool {
    candidate_paths.iter().any(|p| path_exists(p))
        || which_names.iter().any(|n| which(n))
}

const LIBREOFFICE_CANDIDATE_PATHS: &[&str] = &[
    "/Applications/LibreOffice.app/Contents/MacOS/soffice",
    "/opt/homebrew/bin/soffice",
    "/usr/local/bin/soffice",
    "/opt/local/bin/soffice",
    "/opt/homebrew/bin/libreoffice",
    "/usr/local/bin/libreoffice",
    "/opt/local/bin/libreoffice",
];

const IMAGEMAGICK_CANDIDATE_PATHS: &[&str] = &[
    "/opt/homebrew/bin/magick",
    "/usr/local/bin/magick",
    "/opt/local/bin/magick",
    "/opt/homebrew/bin/convert",
    "/usr/local/bin/convert",
    "/opt/local/bin/convert",
];

fn libreoffice_available() -> bool {
    dependency_tool_installed(
        LIBREOFFICE_CANDIDATE_PATHS,
        &["soffice", "libreoffice"],
        |p| Path::new(p).exists(),
        shell_which_on_augmented_path,
    )
}

fn imagemagick_available() -> bool {
    dependency_tool_installed(
        IMAGEMAGICK_CANDIDATE_PATHS,
        &["magick", "convert"],
        |p| Path::new(p).exists(),
        shell_which_on_augmented_path,
    )
}

#[cfg(test)]
mod path_helper_tests {
    use super::{normalize_user_path, path_has_supported_extension};
    use std::path::Path;

    #[test]
    fn normalize_file_url_and_trim() {
        assert_eq!(
            normalize_user_path("file:///tmp/report.pdf"),
            "/tmp/report.pdf"
        );
        assert_eq!(normalize_user_path("  /tmp/a.pdf  "), "/tmp/a.pdf");
    }

    #[test]
    fn supported_extension_is_case_insensitive() {
        assert!(path_has_supported_extension(Path::new("Report.PDF")));
        assert!(!path_has_supported_extension(Path::new("notes.txt")));
        assert!(!path_has_supported_extension(Path::new("noext")));
    }
}

#[cfg(test)]
mod security_path_tests {
    use super::sanitize_report_file_name;

    #[test]
    fn sanitize_report_file_name_accepts_simple_names() {
        assert_eq!(
            sanitize_report_file_name("parsekit-errors.txt").unwrap(),
            "parsekit-errors.txt"
        );
    }

    #[test]
    fn sanitize_report_file_name_rejects_traversal() {
        assert!(sanitize_report_file_name("../secrets.txt").is_err());
        assert!(sanitize_report_file_name("sub/file.txt").is_err());
        assert!(sanitize_report_file_name("").is_err());
    }
}

#[cfg(test)]
mod dependency_detect_tests {
    use super::*;

    #[test]
    fn detects_tool_at_absolute_path_without_which() {
        assert!(dependency_tool_installed(
            &["/fake/opt/homebrew/bin/magick"],
            &["magick"],
            |p| p == "/fake/opt/homebrew/bin/magick",
            |_| false,
        ));
    }

    #[test]
    fn detects_via_which_when_path_missing() {
        assert!(dependency_tool_installed(
            &["/missing/magick"],
            &["magick"],
            |_| false,
            |n| n == "magick",
        ));
    }

    #[test]
    fn intel_local_bin_candidate_in_libreoffice_list() {
        assert!(LIBREOFFICE_CANDIDATE_PATHS.contains(&"/usr/local/bin/soffice"));
    }

    #[test]
    fn intel_local_bin_candidate_in_imagemagick_list() {
        assert!(IMAGEMAGICK_CANDIDATE_PATHS.contains(&"/usr/local/bin/magick"));
    }

    #[test]
    fn libreoffice_app_bundle_path_listed() {
        assert!(LIBREOFFICE_CANDIDATE_PATHS
            .contains(&"/Applications/LibreOffice.app/Contents/MacOS/soffice"));
    }

    #[test]
    fn macports_bin_candidate_in_imagemagick_list() {
        assert!(IMAGEMAGICK_CANDIDATE_PATHS.contains(&"/opt/local/bin/magick"));
    }
}

#[tauri::command]
fn check_dependencies() -> Result<Vec<DependencyStatus>, String> {
    Ok(vec![
        DependencyStatus {
            id: "libreoffice".into(),
            label_key: "deps.libreoffice".into(),
            installed: libreoffice_available(),
            optional: true,
            brew_hint: "brew install --cask libreoffice".into(),
        },
        DependencyStatus {
            id: "imagemagick".into(),
            label_key: "deps.imagemagick".into(),
            installed: imagemagick_available(),
            optional: true,
            brew_hint: "brew install imagemagick".into(),
        },
        // Tesseract is bundled with ParseKit (liteparse); no external install check.
        DependencyStatus {
            id: "tesseract".into(),
            label_key: "deps.tesseract".into(),
            installed: true,
            optional: false,
            brew_hint: String::new(),
        },
    ])
}

#[cfg(target_os = "macos")]
fn running_from_applications() -> bool {
    std::env::current_exe()
        .map(|p| p.to_string_lossy().contains("/Applications/ParseKit.app/"))
        .unwrap_or(false)
}

#[cfg(target_os = "macos")]
fn resolve_bundle_from_macos_dir(macos_dir: &Path) -> Option<std::path::PathBuf> {
    let bundle = macos_dir.parent()?.parent()?;
    let name = bundle.file_name()?.to_string_lossy();
    if name == "ParseKit.app" || name.ends_with(".app") {
        Some(bundle.to_path_buf())
    } else {
        None
    }
}

#[cfg(target_os = "macos")]
fn resolve_app_bundle_from_exe(exe: &Path) -> Option<std::path::PathBuf> {
    exe.parent()
        .and_then(resolve_bundle_from_macos_dir)
        .or_else(|| {
            let bundle = exe.parent()?.parent()?.parent()?;
            let name = bundle.file_name()?.to_string_lossy();
            if name == "ParseKit.app" || name.ends_with(".app") {
                Some(bundle.to_path_buf())
            } else {
                None
            }
        })
}

#[cfg(target_os = "macos")]
fn resolve_running_bundle(app: &AppHandle) -> Option<std::path::PathBuf> {
    if let Ok(exe_dir) = app.path().executable_dir() {
        if let Some(bundle) = resolve_bundle_from_macos_dir(&exe_dir) {
            return Some(bundle);
        }
    }
    std::env::current_exe()
        .ok()
        .and_then(|exe| resolve_app_bundle_from_exe(&exe))
}

#[cfg(target_os = "macos")]
fn play_install_complete_sound() {
    const SOUNDS: &[&str] = &[
        "/System/Library/Components/CoreAudio.component/Contents/SharedSupport/systemsound.bundle/usernames/Pop.aiff",
        "/System/Library/Components/CoreAudio.component/Contents/SharedSupport/systemsound.bundle/usernames/Glass.aiff",
    ];
    for sound in SOUNDS {
        if Path::new(sound).is_file() {
            let _ = std::process::Command::new("afplay").arg(sound).spawn();
            return;
        }
    }
}

#[cfg(target_os = "macos")]
#[tauri::command]
fn is_installed_in_applications() -> bool {
    running_from_applications() || Path::new("/Applications/ParseKit.app").is_dir()
}

#[cfg(target_os = "macos")]
#[tauri::command]
fn should_show_install_step(app: AppHandle) -> bool {
    !running_from_applications() && resolve_running_bundle(&app).is_some()
}

#[cfg(target_os = "macos")]
fn copy_bundle_to_applications(bundle: &Path) -> Result<(), String> {
    let dest = Path::new("/Applications/ParseKit.app");
    if dest.exists() {
        trash::delete(dest).map_err(|e| format!("Could not replace existing ParseKit.app: {e}"))?;
    }

    let status = std::process::Command::new("/usr/bin/ditto")
        .arg(bundle)
        .arg(dest)
        .status()
        .map_err(|e| format!("Failed to copy ParseKit to Applications: {e}"))?;
    if !status.success() {
        return Err(
            "Could not copy ParseKit to Applications. Check disk space and try again.".to_string(),
        );
    }
    Ok(())
}

#[cfg(target_os = "macos")]
#[tauri::command]
fn install_and_relaunch(app: AppHandle) -> Result<(), String> {
    if running_from_applications() {
        return Ok(());
    }

    let bundle = resolve_running_bundle(&app).ok_or_else(|| {
        "Open ParseKit from the DMG (double-click ParseKit.app), then try again.".to_string()
    })?;

    copy_bundle_to_applications(&bundle)?;
    play_install_complete_sound();

    std::process::Command::new("/usr/bin/open")
        .args(["-a", "/Applications/ParseKit.app"])
        .spawn()
        .map_err(|e| format!("Installed, but could not reopen ParseKit: {e}"))?;

    app.exit(0);
    Ok(())
}

#[cfg(not(target_os = "macos"))]
#[tauri::command]
fn should_show_install_step(_app: AppHandle) -> bool {
    false
}

#[cfg(not(target_os = "macos"))]
#[tauri::command]
fn install_and_relaunch(_app: AppHandle) -> Result<(), String> {
    Err("Installing to Applications is only supported on macOS.".to_string())
}

#[cfg(not(target_os = "macos"))]
#[tauri::command]
fn is_installed_in_applications() -> bool {
    true
}

#[cfg(target_os = "macos")]
#[tauri::command]
fn open_privacy_security_settings() -> Result<(), String> {
    std::process::Command::new("open")
        .arg("x-apple.systempreferences:com.apple.settings.PrivacySecurity.extension")
        .spawn()
        .map_err(|e| format!("Failed to open System Settings: {e}"))?;
    Ok(())
}

#[cfg(not(target_os = "macos"))]
#[tauri::command]
fn open_privacy_security_settings() -> Result<(), String> {
    Err("System Settings link is only available on macOS".into())
}

#[tauri::command]
fn gatekeeper_fix_command() -> String {
    "xattr -cr /Applications/ParseKit.app && xattr -d com.apple.FinderInfo /Applications/ParseKit.app 2>/dev/null || true".to_string()
}

/// Reject path components in report filenames (IPC boundary).
pub(crate) fn sanitize_report_file_name(file_name: &str) -> Result<String, String> {
    let trimmed = file_name.trim();
    if trimmed.is_empty() {
        return Err("File name is empty".into());
    }
    if trimmed.contains('/') || trimmed.contains('\\') || trimmed.contains("..") {
        return Err("Invalid file name".into());
    }
    if trimmed.chars().any(|c| c.is_control()) {
        return Err("Invalid file name".into());
    }
    Ok(trimmed.to_string())
}

/// Ensure the parent directory of `target` stays under `base_dir`.
fn path_stays_within_base(base_dir: &std::path::Path, target: &std::path::Path) -> Result<(), String> {
    let base = std::fs::canonicalize(base_dir)
        .map_err(|e| format!("Could not resolve output folder: {e}"))?;
    let parent = target
        .parent()
        .ok_or_else(|| "Invalid report path".to_string())?;
    let parent_resolved = std::fs::canonicalize(parent)
        .map_err(|e| format!("Could not resolve report folder: {e}"))?;
    if !parent_resolved.starts_with(&base) {
        return Err("Report path escapes the output folder".into());
    }
    Ok(())
}

/// Write a text report (error log) into `dir/file_name`, then reveal it in Finder.
/// Returns the full path of the written file.
#[tauri::command]
fn save_error_report(dir: String, file_name: String, contents: String) -> Result<String, String> {
    let safe_name = sanitize_report_file_name(&file_name)?;
    let mut path = std::path::PathBuf::from(&dir);
    if !path.is_dir() {
        // Fall back to ~/Downloads if the original output dir is gone.
        let home = std::env::var("HOME")
            .map_err(|_| "Could not resolve a folder to save the report".to_string())?;
        let downloads = std::path::PathBuf::from(&home).join("Downloads");
        path = if downloads.is_dir() {
            downloads
        } else {
            std::path::PathBuf::from(home)
        };
    }
    let base_dir = path.clone();
    path.push(&safe_name);
    path_stays_within_base(&base_dir, &path)?;
    std::fs::write(&path, contents).map_err(|e| format!("Could not write report: {e}"))?;
    let path_str = path.to_string_lossy().to_string();
    #[cfg(target_os = "macos")]
    {
        if let Ok(canonical) = validate_user_path(&path_str) {
            let _ = std::process::Command::new("open")
                .arg("-R")
                .arg(canonical)
                .spawn();
        }
    }
    Ok(path_str)
}

/// Show a macOS notification (shared by IPC and background clipboard conversion).
pub(crate) fn display_notification(title: &str, body: &str) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        macos_notification::display_notification(title, body)
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = (title, body);
        Ok(())
    }
}

/// Copy plain text to the system clipboard (macOS `pbcopy` — reliable for menu-bar apps).
pub(crate) fn copy_text_to_system_clipboard(text: &str) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        use std::io::Write;
        use std::process::{Command, Stdio};
        let mut child = Command::new("/usr/bin/pbcopy")
            .stdin(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Could not run pbcopy: {e}"))?;
        if let Some(mut stdin) = child.stdin.take() {
            stdin
                .write_all(text.as_bytes())
                .map_err(|e| format!("Could not write to pbcopy: {e}"))?;
        }
        let status = child
            .wait()
            .map_err(|e| format!("pbcopy wait failed: {e}"))?;
        if status.success() {
            Ok(())
        } else {
            Err("pbcopy failed to copy to clipboard".into())
        }
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = text;
        Err("Clipboard copy is only supported on macOS".into())
    }
}

#[tauri::command]
fn copy_text_to_clipboard(text: String) -> Result<(), String> {
    copy_text_to_system_clipboard(&text)
}

#[tauri::command]
fn get_system_info() -> Result<serde_json::Value, String> {
    Ok(serde_json::json!({
        "version": env!("CARGO_PKG_VERSION"),
        "optimalWorkers": get_optimal_workers(),
        "isAppleSilicon": cfg!(target_os = "macos") && detect_apple_silicon()
    }))
}

#[tauri::command]
fn trigger_haptic() -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        use objc2_app_kit::{
            NSHapticFeedbackManager, NSHapticFeedbackPattern, NSHapticFeedbackPerformanceTime,
            NSHapticFeedbackPerformer,
        };
        let performer = NSHapticFeedbackManager::defaultPerformer();
        performer.performFeedbackPattern_performanceTime(
            NSHapticFeedbackPattern::Generic,
            NSHapticFeedbackPerformanceTime::Default,
        );
    }
    Ok(())
}

// Canonical list of supported file extensions — single source of truth used for both
// the preview file count (scan_directory) and the actual parse file set passed to the sidecar.
// Aligned with LiteParse v2 multi-format support (LibreOffice / ImageMagick where noted).
pub const SUPPORTED_EXTENSIONS: &[&str] = &[
    "pdf", "doc", "docx", "docm", "odt", "rtf", "ppt", "pptx", "pptm", "odp", "xls", "xlsx",
    "xlsm", "ods", "csv", "tsv", "png", "jpg", "jpeg", "gif", "bmp", "tiff", "tif", "webp", "svg",
];

pub fn path_has_supported_extension(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .is_some_and(|ext| {
            SUPPORTED_EXTENSIONS
                .iter()
                .any(|candidate| candidate.eq_ignore_ascii_case(ext))
        })
}

pub(crate) fn expand_path_to_supported_files(path: &str) -> Vec<String> {
    let normalized = normalize_user_path(path);
    if normalized.is_empty() {
        return Vec::new();
    }
    let p = Path::new(&normalized);
    if p.is_file() {
        return if path_has_supported_extension(p) {
            vec![normalized]
        } else {
            Vec::new()
        };
    }
    if p.is_dir() {
        return scan_directory_sync(normalized).unwrap_or_default();
    }
    Vec::new()
}

pub(crate) fn collect_supported_files<I, S>(raw_paths: I) -> Vec<String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let mut files = Vec::new();
    for path in raw_paths {
        files.extend(expand_path_to_supported_files(path.as_ref()));
    }
    files.sort();
    files.dedup();
    files
}

const SCAN_MAX_DEPTH: usize = 32;
const SCAN_MAX_FILES: usize = 10_000;

pub(crate) fn scan_directory_sync(path: String) -> Result<Vec<String>, String> {
    let path = normalize_user_path(path);
    let dir = Path::new(&path);
    if !dir.is_dir() {
        return Err(format!("Path is not a directory: {path}"));
    }

    let mut files = Vec::new();
    for entry in WalkDir::new(dir)
        .max_depth(SCAN_MAX_DEPTH)
        .follow_links(false)
        .into_iter()
    {
        let entry = entry.map_err(|e| format!("Could not scan directory: {e}"))?;
        let entry_path = entry.path();
        if entry_path.is_file() {
            if files.len() >= SCAN_MAX_FILES {
                return Err(format!(
                    "Too many supported files (max {SCAN_MAX_FILES}). Choose a smaller folder."
                ));
            }
            if path_has_supported_extension(entry_path) {
                if let Some(s) = entry_path.to_str() {
                    files.push(s.to_string());
                }
            }
        }
    }

    files.sort();
    Ok(files)
}

#[tauri::command]
fn path_is_directory(path: String) -> bool {
    Path::new(&normalize_user_path(path)).is_dir()
}

/// Resolve symlinks (/var vs /private/var) so sidecar progress events match UI row ids.
#[tauri::command]
fn canonicalize_paths(paths: Vec<String>) -> Vec<String> {
    paths
        .into_iter()
        .map(|p| {
            let p = normalize_user_path(p);
            std::fs::canonicalize(&p)
                .map(|c| c.to_string_lossy().into_owned())
                .unwrap_or(p)
        })
        .collect()
}

/// Walks the tree off the UI thread so large folders do not beach-ball the webview.
#[tauri::command]
async fn scan_directory(path: String) -> Result<Vec<String>, String> {
    tauri::async_runtime::spawn_blocking(move || scan_directory_sync(path))
        .await
        .map_err(|e| format!("scan_directory task failed: {e}"))?
}

#[cfg(target_os = "macos")]
fn maybe_show_menu_bar_hint() {
    let Ok(dir) = app_support_dir() else {
        return;
    };
    let marker = dir.join(".menu_bar_hint_shown");
    if marker.exists() {
        return;
    }
    let _ = std::fs::create_dir_all(&dir);
    let _ = std::fs::write(&marker, "");
    let _ = show_completion_notification(
        "ParseKit".to_string(),
        "Look for the ParseKit icon in your menu bar (top-right). Click it to open.".to_string(),
    );
}

#[cfg(target_os = "macos")]
#[tauri::command]
fn show_completion_notification(title: String, body: String) -> Result<(), String> {
    display_notification(&title, &body)
}

#[cfg(not(target_os = "macos"))]
#[tauri::command]
fn show_completion_notification(_title: String, _body: String) -> Result<(), String> {
    Ok(())
}

#[cfg(target_os = "macos")]
#[tauri::command]
fn set_launch_at_login(enabled: bool) -> Result<(), String> {
    use std::process::Command;
    let home = std::env::var("HOME").map_err(|e| e.to_string())?;
    let plist_dir = format!("{home}/Library/LaunchAgents");
    let plist_path = format!("{plist_dir}/com.harshabala.parsekit.plist");
    if enabled {
        std::fs::create_dir_all(&plist_dir).map_err(|e| e.to_string())?;
        let exe = std::env::current_exe().map_err(|e| e.to_string())?;
        let plist = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
  <key>Label</key><string>com.harshabala.parsekit</string>
  <key>ProgramArguments</key><array><string>{}</string></array>
  <key>RunAtLoad</key><true/>
</dict></plist>"#,
            exe.display()
        );
        std::fs::write(&plist_path, plist).map_err(|e| e.to_string())?;
        let _ = Command::new("launchctl")
            .args(["load", "-w", &plist_path])
            .status();
    } else {
        let _ = Command::new("launchctl")
            .args(["unload", "-w", &plist_path])
            .status();
        let _ = std::fs::remove_file(&plist_path);
    }
    Ok(())
}

#[cfg(not(target_os = "macos"))]
#[tauri::command]
fn set_launch_at_login(_enabled: bool) -> Result<(), String> {
    Err("Launch at login is only supported on macOS".into())
}

#[tauri::command]
fn open_in_finder(path: String) -> Result<(), String> {
    let canonical = validate_user_path(&path)?;
    std::process::Command::new("open")
        .arg(canonical)
        .spawn()
        .map_err(|e| format!("Failed to open in Finder: {}", e))?;
    Ok(())
}

#[tauri::command]
fn show_main_window(
    app: tauri::AppHandle,
    popover: State<'_, PopoverState>,
) -> Result<(), String> {
    let window = app
        .get_webview_window("main")
        .ok_or_else(|| "Main window not found".to_string())?;
    show_popover(&window, None, popover.inner());
    Ok(())
}

#[tauri::command]
fn show_onboarding_window(app: tauri::AppHandle) -> Result<(), String> {
    let window = app
        .get_webview_window("onboarding")
        .ok_or_else(|| "Onboarding window not found".to_string())?;
    #[cfg(target_os = "macos")]
    {
        let _ = app.set_activation_policy(tauri::ActivationPolicy::Regular);
    }
    let _ = window.center();
    let _ = window.show();
    let _ = window.set_focus();
    Ok(())
}

#[tauri::command]
fn close_onboarding_window(app: tauri::AppHandle) -> Result<(), String> {
    let window = app
        .get_webview_window("onboarding")
        .ok_or_else(|| "Onboarding window not found".to_string())?;
    let _ = window.hide();
    #[cfg(target_os = "macos")]
    {
        let _ = app.set_activation_policy(tauri::ActivationPolicy::Accessory);
    }
    Ok(())
}

fn position_progress_hud_under_tray<R: Runtime>(window: &WebviewWindow<R>, rect: &Rect) -> bool {
    let Ok(win_size) = window.outer_size() else {
        return false;
    };
    let scale = window.scale_factor().unwrap_or(1.0);
    let (icon_x, icon_y, icon_w, icon_h) = physical_xywh(rect, scale);
    let win_w = win_size.width as f64;
    let win_h = win_size.height as f64;
    let (monitor_width, monitor_height) = monitor_size_f64(window, (f64::MAX, f64::MAX));

    // Anchor below menu bar icon, right-aligned to tray.
    let x = (icon_x + icon_w - win_w)
        .max(8.0)
        .min(monitor_width - win_w - 8.0) as i32;
    let menu_bar = 28.0 * scale;
    let y = (icon_y + icon_h + 6.0) as i32;
    let y = y.clamp(menu_bar as i32, (monitor_height - win_h - 8.0).max(menu_bar) as i32);

    window
        .set_position(PhysicalPosition::new(x, y))
        .is_ok()
}

fn position_progress_hud_top_right<R: Runtime>(window: &WebviewWindow<R>) -> bool {
    let Ok(win_size) = window.outer_size() else {
        return false;
    };
    let scale = window.scale_factor().unwrap_or(1.0);
    let (monitor_width, _) = monitor_size_f64(window, (1920.0 * scale, 1080.0 * scale));

    let margin = 16.0 * scale;
    let menu_bar = 36.0 * scale;
    let x = (monitor_width - win_size.width as f64 - margin).max(8.0) as i32;
    window
        .set_position(PhysicalPosition::new(x, menu_bar as i32))
        .is_ok()
}

#[tauri::command]
fn show_progress_hud(
    app: tauri::AppHandle,
    tray_rect: State<'_, TrayRectState>,
) -> Result<(), String> {
    let window = app
        .get_webview_window("progress-hud")
        .ok_or_else(|| "Progress HUD window not found".to_string())?;

    let rect = tray_rect
        .0
        .lock()
        .ok()
        .and_then(|g| g.clone());
    if let Some(r) = rect.as_ref() {
        let _ = position_progress_hud_under_tray(&window, r);
    } else {
        let _ = position_progress_hud_top_right(&window);
    }

    macos_popover::present_hud_window(&window);
    Ok(())
}

#[tauri::command]
fn hide_progress_hud(app: tauri::AppHandle) -> Result<(), String> {
    let window = app
        .get_webview_window("progress-hud")
        .ok_or_else(|| "Progress HUD window not found".to_string())?;
    window.hide().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn quit_app(app: tauri::AppHandle) {
    app.exit(0);
}

#[cfg(target_os = "macos")]
#[tauri::command]
fn install_finder_quick_action() -> Result<String, String> {
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let resources = exe
        .parent()
        .and_then(|m| m.parent())
        .and_then(|c| c.parent())
        .map(|c| c.join("Resources"))
        .ok_or_else(|| "Could not locate app Resources directory".to_string())?;

    let install_script = resources.join("macos/install-finder-quick-action.sh");
    if !install_script.is_file() {
        return Err(format!(
            "Installer script missing: {}. Reinstall ParseKit from the latest build.",
            install_script.display()
        ));
    }

    let output = std::process::Command::new("/bin/bash")
        .arg(&install_script)
        .output()
        .map_err(|e| e.to_string())?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        return Err(format!(
            "Finder action install failed.\n{stdout}{stderr}"
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

#[cfg(not(target_os = "macos"))]
#[tauri::command]
fn install_finder_quick_action() -> Result<String, String> {
    Err("Finder Quick Actions are only available on macOS.".into())
}

#[cfg(not(target_os = "macos"))]
#[tauri::command]
fn finder_quick_action_installed() -> Result<bool, String> {
    Ok(false)
}

#[cfg(target_os = "macos")]
#[tauri::command]
fn finder_quick_action_installed() -> Result<bool, String> {
    let home = std::env::var("HOME").map_err(|e| e.to_string())?;
    let services = Path::new(&home).join("Library/Services");
    let default_action = services.join("Parse to Markdown with ParseKit.workflow");
    let replace_action =
        services.join("Parse to Markdown with ParseKit (Replace Original).workflow");
    Ok(default_action.is_dir() && replace_action.is_dir())
}

#[derive(serde::Serialize)]
pub struct UpdateInfo {
    pub available: bool,
    pub version: Option<String>,
    pub body: Option<String>,
    pub download_url: Option<String>,
}

#[tauri::command]
async fn check_for_update(app: AppHandle) -> Result<UpdateInfo, String> {
    match app.updater().map_err(|e| e.to_string())?.check().await {
        Ok(Some(update)) => Ok(UpdateInfo {
            available: true,
            version: Some(update.version.clone()),
            body: update.body.clone(),
            download_url: Some(update.download_url.to_string()),
        }),
        Ok(None) => Ok(UpdateInfo {
            available: false,
            version: None,
            body: None,
            download_url: None,
        }),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
fn get_token_stats() -> Result<token_stats::TokenStats, String> {
    Ok(token_stats::load())
}

#[tauri::command]
fn reset_token_stats() -> Result<token_stats::TokenStats, String> {
    token_stats::reset()
}

#[tauri::command]
fn parse_clipboard_to_clipboard() -> Result<(), String> {
    clipboard_convert::convert_clipboard_files_to_clipboard().map(|_| ())
}

#[tauri::command]
fn set_auto_convert_on_copy(
    state: State<'_, clipboard_convert::ClipboardWatchState>,
    enabled: bool,
) -> Result<(), String> {
    state
        .auto_convert_enabled
        .store(enabled, std::sync::atomic::Ordering::SeqCst);
    Ok(())
}

#[tauri::command]
fn record_token_savings(
    file_type: String,
    tokens_saved: u64,
    pages_unlocked: u64,
    documents_unlocked: u64,
) -> Result<token_stats::TokenStats, String> {
    token_stats::record(token_stats::RecordInput {
        file_type,
        tokens_saved,
        pages_unlocked,
        documents_unlocked,
    })
}

#[tauri::command]
async fn install_update(app: AppHandle) -> Result<(), String> {
    let updater = app.updater().map_err(|e| e.to_string())?;
    let update = updater
        .check()
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "No update available".to_string())?;
    update
        .download_and_install(|_chunk, _total| {}, || {})
        .await
        .map_err(|e| e.to_string())?;
    app.restart();
}

pub fn run() {
    ensure_homebrew_on_process_path();
    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .invoke_handler(tauri::generate_handler![
            scan_directory,
            path_is_directory,
            canonicalize_paths,
            open_in_finder,
            save_error_report,
            get_system_info,
            check_dependencies,
            is_installed_in_applications,
            should_show_install_step,
            install_and_relaunch,
            open_privacy_security_settings,
            gatekeeper_fix_command,
            copy_text_to_clipboard,
            trigger_haptic,
            pick_input_files,
            pick_input_folder,
            pick_output_folder,
            update_tray_menu_labels,
            show_completion_notification,
            set_launch_at_login,
            install_finder_quick_action,
            finder_quick_action_installed,
            check_for_update,
            install_update,
            get_token_stats,
            reset_token_stats,
            record_token_savings,
            parse_clipboard_to_clipboard,
            set_auto_convert_on_copy,
            show_main_window,
            show_onboarding_window,
            close_onboarding_window,
            show_progress_hud,
            hide_progress_hud,
            quit_app,
            global_hotkey::get_global_shortcut,
            global_hotkey::update_global_shortcut,
        ])
        .setup(|app| {
            startup_trace("setup() begin");
            #[cfg(target_os = "macos")]
            {
                app.set_activation_policy(tauri::ActivationPolicy::Accessory);
                startup_trace("activation policy set to Accessory");
            }

            let popover_state = PopoverState::new();
            app.manage(popover_state.clone());
            app.manage(global_hotkey::GlobalHotkeyState::default());
            app.manage(clipboard_convert::ClipboardWatchState::default());
            let tray_click_debounce = TrayClickDebounce::new();

            macos_notification::init_notification_bundle();

            app.manage(TrayRectState::default());

            // Tray menu labels (tray_id filled in after the icon is built).
            app.manage(TrayMenuState {
                tray_id: Arc::new(Mutex::new(TrayIconId::new("pending"))),
                open_label: Arc::new(Mutex::new("Open ParseKit".to_string())),
                clipboard_label: Arc::new(Mutex::new(
                    "Parse Clipboard File → Copy Markdown".to_string(),
                )),
                quit_label: Arc::new(Mutex::new("Quit ParseKit".to_string())),
            });

            let window = app.get_webview_window("main").expect("main window");
            let _ = window.hide();
            popover_state.set_open(false);

            if let Some(hud) = app.get_webview_window("progress-hud") {
                let _ = hud.hide();
            }
            if let Some(onboarding) = app.get_webview_window("onboarding") {
                let _ = onboarding.hide();
                let app_for_onboarding = app.handle().clone();
                onboarding.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { .. } = event {
                        let _ = close_onboarding_window(app_for_onboarding.clone());
                    }
                });
            }
            let w = window.clone();
            let popover_for_events = popover_state.clone();
            if let Ok(url) = window.url() {
                startup_trace(&format!("webview url={url}"));
            } else {
                startup_trace("webview url: unavailable");
            }
            window.on_window_event(move |event| {
                if let tauri::WindowEvent::Focused(false) = event {
                    let should_hide = popover_for_events.should_hide_on_focus_loss();
                    popover_trace(&format!(
                        "Focus-loss: should_hide={should_hide} (grace/picker gate)"
                    ));
                    if should_hide {
                        popover_trace("Focus-loss: → hide");
                        hide_popover(&w, &popover_for_events);
                    } else {
                        popover_trace("Focus-loss: suppressed (grace or picker active)");
                    }
                }
            });

            let tray = TrayIconBuilder::new()
                .icon(TRAY_ICON)
                .icon_as_template(true)
                .show_menu_on_left_click(false)
                .tooltip("ParseKit")
                // Do NOT attach .menu() — macOS captures left-clicks for the status item menu.
                .on_menu_event(|app, event| {
                    if event.id().as_ref() == "open_parsekit" {
                        popover_trace("Tray menu event: open_parsekit");
                        if let (Some(window), Some(tray_state), Some(popover)) = (
                            app.get_webview_window("main"),
                            app.try_state::<TrayMenuState>(),
                            app.try_state::<PopoverState>(),
                        ) {
                            open_popover_from_menu(
                                &app,
                                &window,
                                tray_state.inner(),
                                popover.inner(),
                            );
                        } else {
                            popover_trace("Tray menu event: ABORT (missing window/state)");
                        }
                    } else if event.id().as_ref() == "parse_clipboard" {
                        popover_trace("Tray menu event: parse_clipboard");
                        tauri::async_runtime::spawn_blocking(|| {
                            clipboard_convert::run_clipboard_convert_with_notification(
                                "ParseKit",
                                "Markdown copied to clipboard",
                                "Clipboard convert failed",
                            );
                        });
                    }
                })
                .on_tray_icon_event({
                    let popover = popover_state.clone();
                    let tray_click_debounce = tray_click_debounce.clone();
                    move |tray, event| {
                    let app = tray.app_handle();

                    match event {
                        TrayIconEvent::Click {
                            rect,
                            button,
                            button_state,
                            ..
                        } => {
                            if button_state != MouseButtonState::Up {
                                popover_trace("Tray Click: ignored (not mouse-up)");
                                return;
                            }
                            let Some(window) = app.get_webview_window("main") else {
                                popover_trace("Tray Click: ABORT (main window missing)");
                                return;
                            };
                            if let Some(tray_rect_state) = app.try_state::<TrayRectState>() {
                                if let Ok(mut guard) = tray_rect_state.0.lock() {
                                    *guard = Some(rect.clone());
                                }
                            }

                            match button {
                                MouseButton::Left => {
                                    if !tray_click_debounce.accept_click() {
                                        popover_trace("Tray Click: debounced (burst duplicate)");
                                        return;
                                    }
                                    popover_trace("Tray Click → PopoverManager.toggle()");
                                    toggle_popover_from_tray(&window, &rect, &popover);
                                }
                                MouseButton::Right => {
                                    popover_trace("Tray Click: right → menu");
                                    let Some(tray_state) = app.try_state::<TrayMenuState>() else {
                                        popover_trace("Tray Click: ABORT (TrayMenuState missing)");
                                        return;
                                    };
                                    let _ = popup_tray_menu(&app, tray_state.inner());
                                }
                                _ => {
                                    popover_trace("Tray Click: ignored (other button)");
                                }
                            }
                        }
                        _ => {}
                    }
                }
                })
                .build(app)
                .map_err(|e| {
                    startup_trace(&format!("tray build FAILED: {e}"));
                    e
                })?;

            let tray_id = tray.id().clone();
            if let Some(tray_state) = app.try_state::<TrayMenuState>() {
                if let Ok(mut id) = tray_state.inner().tray_id.lock() {
                    *id = tray_id.clone();
                }
            }
            app.manage(TrayHandle { icon: tray.clone() });
            startup_trace(&format!("tray created and retained id={}", tray_id.0));

            if let Some(handle) = app.tray_by_id(&tray_id) {
                match handle.rect() {
                    Ok(Some(rect)) => startup_trace(&format!(
                        "tray rect OK position={:?} size={:?}",
                        rect.position, rect.size
                    )),
                    Ok(None) => startup_trace("tray rect: None (status item not yet laid out)"),
                    Err(e) => startup_trace(&format!("tray rect error: {e}")),
                }
            } else {
                startup_trace("tray lookup by id FAILED immediately after build");
            }

            #[cfg(debug_assertions)]
            popover_trace(&format!(
                "Setup complete — trace file: {}",
                popover_trace::TRACE_FILE
            ));
            #[cfg(not(debug_assertions))]
            popover_trace("Setup complete");
            startup_trace("setup() complete");

            #[cfg(target_os = "macos")]
            {
                maybe_show_menu_bar_hint();
                macos_open_files::start_open_queue_watcher(app.handle().clone());
                clipboard_convert::setup_clipboard_watcher(app.handle().clone());
            }

            #[cfg(desktop)]
            {
                if let Err(e) = global_hotkey::setup_global_hotkey(app.handle()) {
                    eprintln!("ParseKit: global hotkey registration failed: {e}");
                }
            }

            // Test-only: same `install_update` path as the gold banner "Install & Restart" button.
            #[cfg(debug_assertions)]
            if std::env::var("PARSEKIT_E2E_INSTALL_UPDATE").as_deref() == Ok("1") {
                let handle = app.handle().clone();
                tauri::async_runtime::spawn(async move {
                    tokio::time::sleep(std::time::Duration::from_secs(12)).await;
                    if let Err(e) = install_update(handle).await {
                        eprintln!("PARSEKIT_E2E_INSTALL_UPDATE failed: {e}");
                    }
                });
            }

            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while building ParseKit")
        .run(|app_handle, event| {
            #[cfg(target_os = "macos")]
            if let tauri::RunEvent::Opened { urls } = event {
                macos_open_files::emit_opened_urls(&app_handle, urls);
            }
        });
}