#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn notify_already_running() {
    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("osascript").args([
            "-e",
            r#"display notification "ParseKit is already running. Click the ParseKit icon in the menu bar (top-right of your screen)." with title "ParseKit""#,
        ]).spawn();
    }
    eprintln!(
        "ParseKit is already running. Look for the ParseKit icon in the menu bar (top-right), or quit ParseKit in Activity Monitor."
    );
}

fn main() {
    parsekit_lib::startup_trace(&format!(
        "main() entered pid={}",
        std::process::id()
    ));

    let lock_path = std::env::temp_dir().join("com.harshabala.parsekit.lock");
    let lock_id = lock_path.to_string_lossy().into_owned();

    match single_instance::SingleInstance::new(&lock_id) {
        Ok(instance) if !instance.is_single() => {
            parsekit_lib::startup_trace(
                "single-instance guard: another ParseKit is running, exiting",
            );
            #[cfg(target_os = "macos")]
            parsekit_lib::macos_open_files::forward_argv_from_duplicate_instance();
            notify_already_running();
            return;
        }
        Ok(instance) => {
            parsekit_lib::startup_trace("single-instance OK, entering tauri::run");
            parsekit_lib::run();
            drop(instance);
        }
        Err(e) => {
            parsekit_lib::startup_trace(&format!(
                "single-instance unavailable: {e}; continuing"
            ));
            parsekit_lib::run();
        }
    }
}