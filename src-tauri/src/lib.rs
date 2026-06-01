use std::path::Path;
use tauri::tray::{TrayIconBuilder, TrayIconEvent};
use tauri::{Manager, PhysicalPosition, Position, Size};
use walkdir::WalkDir;

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
const SUPPORTED_EXTENSIONS: &[&str] = &[
    "pdf", "docx", "doc", "pptx", "ppt", "xlsx", "xls", "png", "jpg", "jpeg", "tiff", "tif", "bmp",
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
            trigger_haptic
        ])
        .setup(|app| {
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            let window = app.get_webview_window("main").unwrap();

            let w = window.clone();
            window.on_window_event(move |event| {
                if let tauri::WindowEvent::Focused(false) = event {
                    let _ = w.hide();
                }
            });

            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click { rect, .. } = event {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            let is_visible = window.is_visible().unwrap_or(false);
                            if is_visible {
                                let _ = window.hide();
                            } else {
                                // Position the popover window just below the tray icon,
                                // horizontally centered on it.
                                if let Ok(win_size) = window.outer_size() {
                                    // Resolve icon rect to physical pixels.
                                    // Tray icon positions are already in screen physical coords.
                                    let icon_x = match rect.position {
                                        Position::Physical(p) => p.x as f64,
                                        Position::Logical(p) => p.x,
                                    };
                                    let icon_y = match rect.position {
                                        Position::Physical(p) => p.y as f64,
                                        Position::Logical(p) => p.y,
                                    };
                                    let icon_w = match rect.size {
                                        Size::Physical(s) => s.width as f64,
                                        Size::Logical(s) => s.width,
                                    };
                                    let icon_h = match rect.size {
                                        Size::Physical(s) => s.height as f64,
                                        Size::Logical(s) => s.height,
                                    };
                                    let win_w = win_size.width as f64;

                                    // Center window horizontally under the icon, clamp to avoid
                                    // going off the left edge of the screen.
                                    let x = (icon_x + icon_w / 2.0 - win_w / 2.0).max(0.0) as i32;
                                    let y = (icon_y + icon_h) as i32;

                                    let _ = window.set_position(
                                        PhysicalPosition::new(x, y),
                                    );
                                }
                                let _ = window.show();
                                let _ = window.set_focus();
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
