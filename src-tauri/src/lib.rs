pub mod macos_popover;
pub mod popover_trace;
pub mod sidecar_helpers;

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
use walkdir::WalkDir;

/// Colored app mark — `icons/tray/icon@2x.png` was a solid black square and was invisible
/// in the menu bar when used as a template image.
const TRAY_ICON: tauri::image::Image<'static> = tauri::include_image!("icons/32x32.png");
/// Ignore focus-loss hides briefly after `Window.show()` so activation does not collapse the panel.
/// Just long enough to ride out activation focus churn; short enough that click-away still dismisses.
const POPOVER_SHOW_GRACE_MS: u64 = 500;

/// Set `false` only when diagnosing focus-loss; production should keep `true`.
const FOCUS_LOSS_AUTO_HIDE_ENABLED: bool = true;

#[derive(Clone)]
pub(crate) struct PopoverState {
    last_opened_at: Arc<Mutex<Option<Instant>>>,
    picker_active: Arc<AtomicBool>,
}

impl PopoverState {
    fn new() -> Self {
        Self {
            last_opened_at: Arc::new(Mutex::new(None)),
            picker_active: Arc::new(AtomicBool::new(false)),
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

#[derive(Clone)]
struct TrayMenuState {
    tray_id: TrayIconId,
    open_label: Arc<Mutex<String>>,
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
    quit_label: &str,
) -> tauri::Result<Menu<R>> {
    let open_item = MenuItem::with_id(app, "open_parsedock", open_label, true, None::<&str>)?;
    let separator = PredefinedMenuItem::separator(app)?;
    let quit_item = PredefinedMenuItem::quit(app, Some(quit_label))?;
    Menu::with_items(app, &[&open_item, &separator, &quit_item])
}

fn popup_tray_menu<R: Runtime>(app: &AppHandle<R>, tray_state: &TrayMenuState) -> Result<(), String> {
    popover_trace("Tray right-click: popup_menu");
    let open_label = tray_state
        .open_label
        .lock()
        .map_err(|e| e.to_string())?;
    let quit_label = tray_state
        .quit_label
        .lock()
        .map_err(|e| e.to_string())?;
    let menu = build_tray_menu(app, &open_label, &quit_label).map_err(|e| e.to_string())?;
    let window = app
        .get_webview_window("main")
        .ok_or_else(|| "Main window not found".to_string())?;
    window.popup_menu(&menu).map_err(|e| e.to_string())
}

fn position_popover_under_tray<R: Runtime>(window: &WebviewWindow<R>, rect: &Rect) -> bool {
    popover_trace("Positioning: start (under tray)");
    let Ok(win_size) = window.outer_size() else {
        popover_trace("Positioning: FAILED (outer_size)");
        return false;
    };
    let scale = window.scale_factor().unwrap_or(1.0);

    let icon_x = match rect.position {
        Position::Physical(p) => p.x as f64,
        Position::Logical(p) => p.x * scale,
    };
    let icon_y = match rect.position {
        Position::Physical(p) => p.y as f64,
        Position::Logical(p) => p.y * scale,
    };
    let icon_w = match rect.size {
        Size::Physical(s) => s.width as f64,
        Size::Logical(s) => s.width * scale,
    };
    let icon_h = match rect.size {
        Size::Physical(s) => s.height as f64,
        Size::Logical(s) => s.height * scale,
    };
    let win_w = win_size.width as f64;
    let win_h = win_size.height as f64;

    let (monitor_width, monitor_height) = window
        .current_monitor()
        .ok()
        .flatten()
        .map(|m| (m.size().width as f64, m.size().height as f64))
        .unwrap_or((f64::MAX, f64::MAX));

    let x = (icon_x + icon_w / 2.0 - win_w / 2.0)
        .max(0.0)
        .min(monitor_width - win_w) as i32;
    let y = (icon_y + icon_h) as i32;

    // Reject coordinates that would place the window off-screen (bad tray rect).
    if y < 0 || y as f64 > monitor_height - 80.0 || x < -win_w as i32 / 2 {
        popover_trace(&format!(
            "Positioning: REJECTED x={x} y={y} monitor={monitor_width}x{monitor_height}"
        ));
        return false;
    }

    let _ = window.set_position(PhysicalPosition::new(x, y));
    popover_trace(&format!("Positioning: OK x={x} y={y} win={win_w}x{win_h}"));
    let _ = win_h; // keep height available for future vertical clamping
    true
}

fn show_popover<R: Runtime>(
    window: &WebviewWindow<R>,
    rect: Option<&Rect>,
    popover: &PopoverState,
) {
    popover_trace("PopoverManager.show()");

    // Center first so the panel is on-screen even if tray coordinates are wrong.
    let _ = window.center();
    popover_trace("Positioning: center()");
    if let Some(r) = rect {
        let _ = position_popover_under_tray(window, r);
    } else {
        popover_trace("Positioning: skipped (no tray rect)");
    }

    macos_popover::activate_app_for_popover(window, popover);
    log_window_visibility(window, "after show_popover");
}

fn hide_popover<R: Runtime>(window: &WebviewWindow<R>) {
    popover_trace("PopoverManager.hide()");
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
    if window.is_visible().unwrap_or(false) {
        popover_trace("toggle: branch hide (already visible)");
        hide_popover(window);
    } else {
        popover_trace("toggle: branch show (not visible)");
        show_popover(window, Some(rect), popover);
    }
}

fn open_popover_from_menu<R: Runtime>(
    app: &AppHandle<R>,
    window: &WebviewWindow<R>,
    tray_state: &TrayMenuState,
    popover: &PopoverState,
) {
    popover_trace("Tray menu: Open ParseDock → show");
    let rect = app
        .tray_by_id(&tray_state.tray_id)
        .and_then(|t| t.rect().ok().flatten());
    show_popover(window, rect.as_ref(), popover);
}

#[tauri::command]
fn update_tray_menu_labels(
    tray_state: State<TrayMenuState>,
    open_label: String,
    quit_label: String,
) -> Result<(), String> {
    // Only update labels — never call tray.set_menu(), which re-attaches the menu to
    // NSStatusItem and causes macOS to swallow left-clicks.
    *tray_state
        .open_label
        .lock()
        .map_err(|e| e.to_string())? = open_label;
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

fn file_path_to_string(path: tauri_plugin_dialog::FilePath) -> Option<String> {
    path.into_path()
        .ok()
        .and_then(|p| p.into_os_string().into_string().ok())
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
        let popover_still_open = app
            .get_webview_window("main")
            .map(|w| w.is_visible().unwrap_or(false))
            .unwrap_or(false);
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

#[tauri::command]
async fn pick_input_folder(
    app: AppHandle,
    popover: State<'_, PopoverState>,
) -> Result<Option<String>, String> {
    run_file_picker_async(app, popover.inner(), |app| async move {
        await_pick_folder(&app, "Select folder")
            .await
            .ok()
            .flatten()
            .map(|path| vec![path])
    })
    .await
    .map(|paths| paths.and_then(|mut v| v.pop()))
}

#[tauri::command]
async fn pick_output_folder(
    app: AppHandle,
    popover: State<'_, PopoverState>,
) -> Result<Option<String>, String> {
    run_file_picker_async(app, popover.inner(), |app| async move {
        await_pick_folder(&app, "Choose output folder")
            .await
            .ok()
            .flatten()
            .map(|path| vec![path])
    })
    .await
    .map(|paths| paths.and_then(|mut v| v.pop()))
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
const SUPPORTED_EXTENSIONS: &[&str] = &[
    "pdf", "doc", "docx", "docm", "odt", "rtf", "ppt", "pptx", "pptm", "odp", "xls", "xlsx",
    "xlsm", "ods", "csv", "tsv", "png", "jpg", "jpeg", "gif", "bmp", "tiff", "tif", "webp", "svg",
];

fn scan_directory_sync(path: String) -> Result<Vec<String>, String> {
    let dir = Path::new(&path);
    if !dir.is_dir() {
        return Err(format!("Path is not a directory: {path}"));
    }

    let mut files = Vec::new();
    for entry in WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
        let entry_path = entry.path();
        if entry_path.is_file() {
            if let Some(ext) = entry_path.extension().and_then(|e| e.to_str()) {
                if SUPPORTED_EXTENSIONS.contains(&ext.to_lowercase().as_str()) {
                    if let Some(s) = entry_path.to_str() {
                        files.push(s.to_string());
                    }
                }
            }
        }
    }

    files.sort();
    Ok(files)
}

#[tauri::command]
fn path_is_directory(path: String) -> bool {
    Path::new(&path).is_dir()
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
    let Ok(home) = std::env::var("HOME") else {
        return;
    };
    let marker = format!("{home}/Library/Application Support/ParseDock/.menu_bar_hint_shown");
    if Path::new(&marker).exists() {
        return;
    }
    if let Some(parent) = Path::new(&marker).parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let _ = std::fs::write(&marker, "");
    let _ = show_completion_notification(
        "ParseDock".to_string(),
        "Look for the blue P icon in your menu bar (top-right). Click it to open.".to_string(),
    );
}

#[cfg(target_os = "macos")]
#[tauri::command]
fn show_completion_notification(title: String, body: String) -> Result<(), String> {
    let script = format!(
        "display notification {} with title {}",
        serde_json::to_string(&body).map_err(|e| e.to_string())?,
        serde_json::to_string(&title).map_err(|e| e.to_string())?
    );
    std::process::Command::new("osascript")
        .args(["-e", &script])
        .spawn()
        .map_err(|e| format!("notification failed: {e}"))?;
    Ok(())
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
    let plist_path = format!("{plist_dir}/com.parsedock.app.plist");
    if enabled {
        std::fs::create_dir_all(&plist_dir).map_err(|e| e.to_string())?;
        let exe = std::env::current_exe().map_err(|e| e.to_string())?;
        let plist = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
  <key>Label</key><string>com.parsedock.app</string>
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
    std::process::Command::new("open")
        .arg(&path)
        .spawn()
        .map_err(|e| format!("Failed to open in Finder: {}", e))?;
    Ok(())
}

#[tauri::command]
fn copy_file_to_clipboard(path: String) -> Result<Vec<u8>, String> {
    std::fs::read(&path).map_err(|e| format!("Failed to read file: {}", e))
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .invoke_handler(tauri::generate_handler![
            scan_directory,
            path_is_directory,
            open_in_finder,
            copy_file_to_clipboard,
            get_system_info,
            trigger_haptic,
            pick_input_files,
            pick_input_folder,
            pick_output_folder,
            update_tray_menu_labels,
            show_completion_notification,
            set_launch_at_login,
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
            let tray_click_debounce = TrayClickDebounce::new();

            let window = app.get_webview_window("main").expect("main window");
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
                        hide_popover(&w);
                    } else {
                        popover_trace("Focus-loss: suppressed (grace or picker active)");
                    }
                }
            });

            let tray = TrayIconBuilder::new()
                .icon(TRAY_ICON)
                .icon_as_template(false)
                .show_menu_on_left_click(false)
                .tooltip("ParseDock")
                // Do NOT attach .menu() — macOS captures left-clicks for the status item menu.
                .on_menu_event(|app, event| {
                    if event.id().as_ref() == "open_parsedock" {
                        popover_trace("Tray menu event: open_parsedock");
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
                    }
                })
                .on_tray_icon_event({
                    let popover = popover_state.clone();
                    let tray_click_debounce = tray_click_debounce.clone();
                    move |tray, event| {
                    let app = tray.app_handle();
                    let Some(tray_state) = app.try_state::<TrayMenuState>() else {
                        popover_trace("Tray Click: ABORT (TrayMenuState missing)");
                        return;
                    };
                    let tray_state = tray_state.inner().clone();

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
                                    let _ = popup_tray_menu(&app, &tray_state);
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

            app.manage(TrayMenuState {
                tray_id,
                open_label: Arc::new(Mutex::new("Open ParseDock".to_string())),
                quit_label: Arc::new(Mutex::new("Quit ParseDock".to_string())),
            });

            #[cfg(debug_assertions)]
            popover_trace(&format!(
                "Setup complete — trace file: {}",
                popover_trace::TRACE_FILE
            ));
            #[cfg(not(debug_assertions))]
            popover_trace("Setup complete");
            startup_trace("setup() complete");

            #[cfg(target_os = "macos")]
            maybe_show_menu_bar_hint();

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running ParseDock");
}