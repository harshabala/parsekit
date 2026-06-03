pub mod sidecar_helpers;

use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager, PhysicalPosition, Position, Runtime, Size, State, WebviewWindow};
use tauri_plugin_dialog::DialogExt;
use walkdir::WalkDir;

/// Ignore focus-loss hides briefly after opening so tray mouse-up does not collapse the panel.
const POPOVER_SHOW_GRACE_MS: u64 = 500;
const TRAY_ICON: tauri::image::Image<'static> = tauri::include_image!("icons/tray/icon.png");

fn show_popover<R: Runtime>(app: &AppHandle<R>, window: &WebviewWindow<R>) {
    if let Some(state) = app.try_state::<PopoverState>() {
        state.mark_opening();
    }
    let _ = window.show();
    let _ = window.set_focus();
}

fn toggle_popover<R: Runtime>(app: AppHandle<R>, window: WebviewWindow<R>) {
    let is_visible = window.is_visible().unwrap_or(false);
    if is_visible {
        let _ = window.hide();
        return;
    }
    show_popover(&app, &window);
}

#[derive(Clone)]
struct PopoverState {
    last_opened_at: Arc<Mutex<Option<Instant>>>,
    /// While a native file picker is open, do not auto-hide the popover on focus loss.
    picker_active: Arc<AtomicBool>,
}

impl PopoverState {
    fn new() -> Self {
        Self {
            last_opened_at: Arc::new(Mutex::new(None)),
            picker_active: Arc::new(AtomicBool::new(false)),
        }
    }

    fn mark_opening(&self) {
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
        if self.picker_active.load(Ordering::SeqCst) {
            return false;
        }
        self.last_opened_at
            .lock()
            .ok()
            .and_then(|last_opened_at| *last_opened_at)
            .map(|opened_at| opened_at.elapsed() > Duration::from_millis(POPOVER_SHOW_GRACE_MS))
            .unwrap_or(true)
    }
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
fn run_file_picker<R: Runtime, F: FnOnce(&AppHandle<R>) -> Option<Vec<String>>>(
    app: AppHandle<R>,
    popover: &PopoverState,
    pick: F,
) -> Result<Option<Vec<String>>, String> {
    popover.begin_picker();
    #[cfg(target_os = "macos")]
    elevate_app_for_file_dialog(&app);

    let result = pick(&app);

    #[cfg(target_os = "macos")]
    restore_accessory_app_policy(&app);
    popover.end_picker();
    Ok(result)
}

#[tauri::command]
fn pick_input_files(
    app: AppHandle,
    popover: State<PopoverState>,
) -> Result<Option<Vec<String>>, String> {
    run_file_picker(app, popover.inner(), |app| {
        app.dialog()
            .file()
            .set_title("Select files to parse")
            .add_filter("Supported documents", SUPPORTED_EXTENSIONS)
            .blocking_pick_files()
            .map(|files| {
                files
                    .into_iter()
                    .filter_map(file_path_to_string)
                    .collect::<Vec<_>>()
            })
    })
}

#[tauri::command]
fn pick_input_folder(
    app: AppHandle,
    popover: State<PopoverState>,
) -> Result<Option<String>, String> {
    let paths = run_file_picker(app, popover.inner(), |app| {
        app.dialog()
            .file()
            .set_title("Select folder")
            .blocking_pick_folder()
            .and_then(file_path_to_string)
            .map(|path| vec![path])
    })?;
    Ok(paths.and_then(|mut v| v.pop()))
}

#[tauri::command]
fn pick_output_folder(
    app: AppHandle,
    popover: State<PopoverState>,
) -> Result<Option<String>, String> {
    let paths = run_file_picker(app, popover.inner(), |app| {
        app.dialog()
            .file()
            .set_title("Choose output folder")
            .blocking_pick_folder()
            .and_then(file_path_to_string)
            .map(|path| vec![path])
    })?;
    Ok(paths.and_then(|mut v| v.pop()))
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

#[tauri::command]
fn scan_directory(path: String) -> Result<Vec<String>, String> {
    let dir = Path::new(&path);
    if !dir.is_dir() {
        return Err(format!("Path is not a directory: {}", path));
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
            open_in_finder,
            copy_file_to_clipboard,
            get_system_info,
            trigger_haptic,
            pick_input_files,
            pick_input_folder,
            pick_output_folder
        ])
        .setup(|app| {
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            app.manage(PopoverState::new());

            let window = app.get_webview_window("main").unwrap();

            let w = window.clone();
            let popover_state = app.state::<PopoverState>().inner().clone();
            window.on_window_event(move |event| {
                if let tauri::WindowEvent::Focused(false) = event {
                    if popover_state.should_hide_on_focus_loss() {
                        let _ = w.hide();
                    }
                }
            });

            let open_item = MenuItem::with_id(app, "open_parsedock", "Open ParseDock", true, None::<&str>)?;
            let separator = PredefinedMenuItem::separator(app)?;
            let quit_item = PredefinedMenuItem::quit(app, Some("Quit ParseDock"))?;
            let tray_menu = Menu::with_items(app, &[&open_item, &separator, &quit_item])?;

            let _tray = TrayIconBuilder::new()
                .icon(TRAY_ICON)
                .icon_as_template(true)
                .tooltip("ParseDock")
                .menu(&tray_menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|tray, event| {
                    if event.id().as_ref() == "open_parsedock" {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            show_popover(&app, &window);
                        }
                    }
                })
                .on_tray_icon_event(|tray, event| {
                    // tray-icon emits Click on mouse-down and mouse-up; only act on release
                    // so a single click opens the panel instead of open-on-down/close-on-up.
                    if let TrayIconEvent::Click {
                        rect,
                        button_state,
                        ..
                    } = event
                    {
                        if button_state != MouseButtonState::Up {
                            return;
                        }
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            let is_visible = window.is_visible().unwrap_or(false);
                            if is_visible {
                                toggle_popover(app.clone(), window);
                            } else {
                                // Position the popover window just below the tray icon,
                                // horizontally centered on it.
                                if let Ok(win_size) = window.outer_size() {
                                    // Scale factor for converting logical → physical pixels.
                                    let scale = window.scale_factor().unwrap_or(1.0);

                                    // Resolve icon rect to physical pixels.
                                    // Tray icon positions are already in screen physical coords.
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

                                    // Monitor width for right-edge clamping (physical px).
                                    let monitor_width = window
                                        .current_monitor()
                                        .ok()
                                        .flatten()
                                        .map(|m| m.size().width as f64)
                                        .unwrap_or(f64::MAX);

                                    // Center window horizontally under the icon, clamp to avoid
                                    // going off either edge of the screen.
                                    let x = (icon_x + icon_w / 2.0 - win_w / 2.0)
                                        .max(0.0)
                                        .min(monitor_width - win_w)
                                        as i32;
                                    let y = (icon_y + icon_h) as i32;

                                    let _ = window.set_position(PhysicalPosition::new(x, y));
                                }
                                toggle_popover(app.clone(), window);
                            }
                        }
                    }
                })
                .build(app)?;

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running ParseDock");
}
