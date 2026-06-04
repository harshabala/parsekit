#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    parsedock_lib::startup_trace(&format!(
        "main() entered pid={}",
        std::process::id()
    ));
    let instance = single_instance::SingleInstance::new("com.parsedock.app").unwrap();
    if !instance.is_single() {
        parsedock_lib::startup_trace(
            "single-instance guard: another ParseDock is running, exiting",
        );
        #[cfg(target_os = "macos")]
        {
            let _ = std::process::Command::new("osascript").args([
                "-e",
                r#"display notification "ParseDock is already running. Click the blue P icon in the menu bar (top-right of your screen)." with title "ParseDock""#,
            ]).spawn();
        }
        eprintln!(
            "ParseDock is already running. Look for the blue P icon in the menu bar (top-right), or quit ParseDock in Activity Monitor."
        );
        std::process::exit(0);
    }
    parsedock_lib::startup_trace("single-instance OK, entering tauri::run");
    parsedock_lib::run();
}