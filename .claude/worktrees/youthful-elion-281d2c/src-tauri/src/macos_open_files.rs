//! macOS: queue file paths from Finder Quick Actions / second-instance launches.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::Duration;
use tauri::{AppHandle, Emitter};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct OpenQueueFile {
    paths: Vec<String>,
}

fn queue_path() -> Result<PathBuf, String> {
    let home = std::env::var("HOME").map_err(|e| e.to_string())?;
    Ok(PathBuf::from(home)
        .join("Library/Application Support/com.harshabala.parsekit")
        .join("open-queue.json"))
}

fn ensure_support_dir() -> Result<PathBuf, String> {
    let path = queue_path()?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    Ok(path)
}

/// Append paths for the running instance (called from a duplicate process before exit).
pub fn enqueue_paths(paths: Vec<String>) -> Result<(), String> {
    let supported: Vec<String> = paths
        .into_iter()
        .filter(|p| !p.is_empty() && !p.starts_with('-'))
        .collect();
    if supported.is_empty() {
        return Ok(());
    }

    let path = ensure_support_dir()?;
    let mut existing = std::fs::read_to_string(&path)
        .ok()
        .and_then(|s| serde_json::from_str::<OpenQueueFile>(&s).ok())
        .unwrap_or(OpenQueueFile {
            paths: Vec::new(),
        });
    existing.paths.extend(supported);

    let json = serde_json::to_string_pretty(&existing).map_err(|e| e.to_string())?;
    std::fs::write(&path, json).map_err(|e| e.to_string())?;
    Ok(())
}

/// Forward CLI args when a second instance would have started (single-instance guard).
pub fn forward_argv_from_duplicate_instance() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() {
        return;
    }
    if let Err(e) = enqueue_paths(args) {
        eprintln!("ParseKit: failed to queue open files: {e}");
        return;
    }
    // Wake the menu-bar app without passing duplicate argv to a new process.
    let _ = std::process::Command::new("open")
        .args(["-ga", "ParseKit"])
        .status();
}

fn drain_queue() -> Vec<String> {
    let path = match queue_path() {
        Ok(p) => p,
        Err(_) => return Vec::new(),
    };
    let raw = match std::fs::read_to_string(&path) {
        Ok(s) if !s.trim().is_empty() => s,
        _ => return Vec::new(),
    };
    let _ = std::fs::remove_file(&path);
    serde_json::from_str::<OpenQueueFile>(&raw)
        .map(|q| q.paths)
        .unwrap_or_default()
}

pub fn emit_opened_urls(app: &AppHandle, urls: Vec<url::Url>) {
    let paths: Vec<String> = urls
        .into_iter()
        .filter_map(|u| u.to_file_path().ok())
        .filter_map(|p| p.into_os_string().into_string().ok())
        .collect();
    if paths.is_empty() {
        return;
    }
    let _ = app.emit("open-files", paths);
}

static DRAIN_LOCK: Mutex<()> = Mutex::new(());

pub fn start_open_queue_watcher(app: AppHandle) {
    std::thread::spawn(move || {
        loop {
            std::thread::sleep(Duration::from_millis(800));
            let paths = if let Ok(_guard) = DRAIN_LOCK.lock() {
                drain_queue()
            } else {
                continue;
            };
            if paths.is_empty() {
                continue;
            }
            let _ = app.emit("open-files", paths);
        }
    });
}