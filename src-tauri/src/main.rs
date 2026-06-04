#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn notify_already_running() {
    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("osascript").args([
            "-e",
                r#"display notification "ParseDock is already running. Click the ParseDock icon in the menu bar (top-right of your screen)." with title "ParseDock""#,
        ]).spawn();
    }
    eprintln!(
            "ParseDock is already running. Look for the ParseDock icon in the menu bar (top-right), or quit ParseDock in Activity Monitor."
    );
}

fn main() {
    parsedock_lib::startup_trace(&format!(
        "main() entered pid={}",
        std::process::id()
    ));

    let lock_path = std::env::temp_dir().join("com.parsedock.app.lock");
    let lock_id = lock_path.to_string_lossy().into_owned();

    match single_instance::SingleInstance::new(&lock_id) {
        Ok(instance) if !instance.is_single() => {
            parsedock_lib::startup_trace(
                "single-instance guard: another ParseDock is running, exiting",
            );
            notify_already_running();
            return;
        }
        Ok(instance) => {
            parsedock_lib::startup_trace("single-instance OK, entering tauri::run");
            parsedock_lib::run();
            drop(instance);
        }
        Err(e) => {
            parsedock_lib::startup_trace(&format!(
                "single-instance unavailable: {e}; continuing"
            ));
            parsedock_lib::run();
        }
    }
}