//! Parse clipboard file paths via sidecar and copy Markdown (or configured format) to the clipboard.

use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::Deserialize;
use serde_json::{json, Value};
use tauri::{AppHandle, Manager, Runtime};

use crate::clipboard_paths::get_clipboard_file_paths;
use crate::sidecar_helpers::validate_output_format;

pub const AUTO_CONVERT_SETTINGS_KEY: &str = "autoConvertOnCopy";

#[derive(Default)]
pub struct ClipboardWatchState {
    pub auto_convert_enabled: AtomicBool,
    last_clipboard_signature: Mutex<String>,
    converting: AtomicBool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AppSidecarSettings {
    #[serde(default = "default_format")]
    format: String,
    #[serde(default = "default_true")]
    ocr_enabled: bool,
    #[serde(default = "default_ocr_language")]
    ocr_language: String,
    #[serde(default)]
    workers: u32,
}

fn default_format() -> String {
    "md".to_string()
}

fn default_true() -> bool {
    true
}

fn default_ocr_language() -> String {
    "eng".to_string()
}

fn settings_path() -> Result<PathBuf, String> {
    let home = std::env::var("HOME").map_err(|e| e.to_string())?;
    Ok(PathBuf::from(home)
        .join("Library/Application Support/com.harshabala.parsekit/settings.json"))
}

pub fn read_auto_convert_from_settings() -> bool {
    let path = match settings_path() {
        Ok(path) => path,
        Err(_) => return false,
    };
    let raw = match std::fs::read_to_string(path) {
        Ok(raw) => raw,
        Err(_) => return false,
    };
    let value: Value = match serde_json::from_str(&raw) {
        Ok(value) => value,
        Err(_) => return false,
    };
    value
        .get(AUTO_CONVERT_SETTINGS_KEY)
        .and_then(Value::as_bool)
        .unwrap_or(false)
}

fn load_app_settings() -> AppSidecarSettings {
    let path = match settings_path() {
        Ok(path) => path,
        Err(_) => return AppSidecarSettings {
            format: default_format(),
            ocr_enabled: true,
            ocr_language: default_ocr_language(),
            workers: 4,
        },
    };
    let raw = match std::fs::read_to_string(path) {
        Ok(raw) => raw,
        Err(_) => return AppSidecarSettings {
            format: default_format(),
            ocr_enabled: true,
            ocr_language: default_ocr_language(),
            workers: 4,
        },
    };
    serde_json::from_str(&raw).unwrap_or(AppSidecarSettings {
        format: default_format(),
        ocr_enabled: true,
        ocr_language: default_ocr_language(),
        workers: 4,
    })
}

fn normalize_user_path(path: String) -> String {
    let trimmed = path.trim();
    if trimmed.starts_with("file://") {
        if let Ok(url) = url::Url::parse(trimmed) {
            if let Ok(p) = url.to_file_path() {
                return p.to_string_lossy().into_owned();
            }
        }
    }
    trimmed.to_string()
}

fn is_supported_extension(path: &str) -> bool {
    Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .map(|ext| crate::SUPPORTED_EXTENSIONS.contains(&ext.to_lowercase().as_str()))
        .unwrap_or(false)
}

fn expand_path_to_supported_files(path: &str) -> Vec<String> {
    let normalized = normalize_user_path(path.to_string());
    if normalized.is_empty() {
        return Vec::new();
    }
    let p = Path::new(&normalized);
    if p.is_file() {
        return if is_supported_extension(&normalized) {
            vec![normalized]
        } else {
            Vec::new()
        };
    }
    if p.is_dir() {
        return crate::scan_directory_sync(normalized).unwrap_or_default();
    }
    Vec::new()
}

/// Supported files referenced by the current clipboard contents.
pub fn resolve_clipboard_supported_files() -> Vec<String> {
    let mut files = Vec::new();
    for path in get_clipboard_file_paths() {
        files.extend(expand_path_to_supported_files(&path));
    }
    files.sort();
    files.dedup();
    files
}

fn host_triple() -> String {
    match std::env::consts::OS {
        "macos" => format!("{}-apple-darwin", std::env::consts::ARCH),
        "linux" => format!("{}-unknown-linux-gnu", std::env::consts::ARCH),
        "windows" => format!("{}-pc-windows-msvc", std::env::consts::ARCH),
        other => format!("{}-{other}", std::env::consts::ARCH),
    }
}

fn resolve_sidecar() -> Result<PathBuf, String> {
    if let Ok(path) = std::env::var("PARSEKIT_SIDECAR") {
        let candidate = PathBuf::from(&path);
        if candidate.is_file() {
            return Ok(candidate);
        }
        return Err(format!("PARSEKIT_SIDECAR not found: {path}"));
    }

    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            let sibling = dir.join("parsekit-sidecar");
            if sibling.is_file() {
                return Ok(sibling);
            }
        }
    }

    if let Ok(exe) = std::env::current_exe() {
        let mut dir = exe.parent().map(Path::to_path_buf);
        for _ in 0..8 {
            let Some(current) = dir else { break };
            let triple = host_triple();
            let bundled = current
                .join("src-tauri/binaries")
                .join(format!("parsekit-sidecar-{triple}"));
            if bundled.is_file() {
                return Ok(bundled);
            }
            let plain = current.join("src-tauri/binaries/parsekit-sidecar");
            if plain.is_file() {
                return Ok(plain);
            }
            dir = current.parent().map(Path::to_path_buf);
        }
    }

    Err(
        "parsekit-sidecar not found. Build with: npm run build:sidecar (or set PARSEKIT_SIDECAR)"
            .to_string(),
    )
}

fn run_sidecar_for_files(
    files: &[String],
    output_dir: &Path,
    settings: &AppSidecarSettings,
) -> Result<Vec<PathBuf>, String> {
    validate_output_format(&settings.format)?;
    let sidecar = resolve_sidecar()?;
    let workers = if settings.workers == 0 {
        4
    } else {
        settings.workers as usize
    };
    let payload = json!({
        "files": files,
        "outputDir": output_dir.to_string_lossy(),
        "format": settings.format,
        "ocrEnabled": settings.ocr_enabled,
        "ocrLanguage": settings.ocr_language,
        "workers": workers,
    });

    let mut child = Command::new(&sidecar)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to run sidecar ({}): {e}", sidecar.display()))?;

    if let Some(mut stdin) = child.stdin.take() {
        let line = format!(
            "{}\n",
            serde_json::to_string(&payload).map_err(|e| e.to_string())?
        );
        stdin
            .write_all(line.as_bytes())
            .map_err(|e| format!("Failed to write sidecar config: {e}"))?;
    }

    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| "Sidecar stdout unavailable".to_string())?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| "Sidecar stderr unavailable".to_string())?;

    let reader = BufReader::new(stdout);
    let mut output_paths: Vec<PathBuf> = Vec::new();
    let mut errors = 0usize;

    for line in reader.lines() {
        let line = line.map_err(|e| format!("Failed to read sidecar output: {e}"))?;
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let event: Value = serde_json::from_str(trimmed)
            .map_err(|e| format!("Invalid sidecar JSON line ({trimmed}): {e}"))?;
        match event.get("type").and_then(Value::as_str) {
            Some("error") => {
                let message = event
                    .get("message")
                    .and_then(Value::as_str)
                    .unwrap_or("Sidecar error");
                return Err(message.to_string());
            }
            Some("progress") => {
                let status = event.get("status").and_then(Value::as_str).unwrap_or("");
                if status == "error" {
                    errors += 1;
                } else if matches!(status, "completed" | "skipped") {
                    if let Some(path) = event.get("path").and_then(Value::as_str) {
                        output_paths.push(PathBuf::from(path));
                    }
                }
            }
            Some("done") => {
                if let Some(count) = event.get("errors").and_then(Value::as_u64) {
                    errors = errors.max(count as usize);
                }
            }
            _ => {}
        }
    }

    let status = child
        .wait()
        .map_err(|e| format!("Failed waiting for sidecar: {e}"))?;
    if !status.success() {
        let mut err_text = String::new();
        for line in BufReader::new(stderr).lines() {
            if let Ok(line) = line {
                if !line.trim().is_empty() {
                    err_text.push_str(&line);
                    err_text.push('\n');
                }
            }
        }
        if err_text.trim().is_empty() {
            return Err(format!("Sidecar exited with status {status}"));
        }
        return Err(err_text.trim().to_string());
    }

    if errors > 0 {
        return Err(format!("{errors} file(s) failed to convert"));
    }
    if output_paths.is_empty() {
        return Err("Sidecar produced no output paths".to_string());
    }
    Ok(output_paths)
}

fn copy_text_to_system_clipboard(text: &str) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
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

fn temp_output_dir() -> Result<PathBuf, String> {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("parsekit-clipboard-{stamp}"));
    std::fs::create_dir_all(&dir).map_err(|e| format!("Could not create temp dir: {e}"))?;
    Ok(dir)
}

/// Parse supported clipboard files and copy the combined output text to the system clipboard.
pub fn convert_clipboard_files_to_clipboard() -> Result<usize, String> {
    let files = resolve_clipboard_supported_files();
    if files.is_empty() {
        return Err("No supported file on clipboard".into());
    }

    let settings = load_app_settings();
    let temp_dir = temp_output_dir()?;
    let output_paths = run_sidecar_for_files(&files, &temp_dir, &settings)?;

    let mut combined = String::new();
    for path in &output_paths {
        let content =
            std::fs::read_to_string(path).map_err(|e| format!("Could not read output: {e}"))?;
        if !combined.is_empty() {
            combined.push_str("\n\n---\n\n");
        }
        combined.push_str(&content);
    }

    copy_text_to_system_clipboard(&combined)?;
    let _ = std::fs::remove_dir_all(&temp_dir);
    Ok(files.len())
}

pub fn run_clipboard_convert_with_notification(title: &str, success_body: &str, error_prefix: &str) {
    match convert_clipboard_files_to_clipboard() {
        Ok(_) => {
            let _ = crate::display_notification(title, success_body);
        }
        Err(e) => {
            let _ = crate::display_notification(title, &format!("{error_prefix}: {e}"));
        }
    }
}

pub fn setup_clipboard_watcher<R: Runtime>(app: AppHandle<R>) {
    let state = app.state::<ClipboardWatchState>().inner().clone();
    state
        .auto_convert_enabled
        .store(read_auto_convert_from_settings(), Ordering::SeqCst);

    thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_millis(800));
            if !state.auto_convert_enabled.load(Ordering::SeqCst) {
                continue;
            }
            if state.converting.load(Ordering::SeqCst) {
                continue;
            }

            let paths = get_clipboard_file_paths();
            if paths.is_empty() {
                continue;
            }

            let signature = paths.join("\n");
            let supported = resolve_clipboard_supported_files();
            if supported.is_empty() {
                continue;
            }

            let should_convert = {
                let Ok(mut last) = state.last_clipboard_signature.lock() else {
                    continue;
                };
                if *last == signature {
                    false
                } else {
                    *last = signature;
                    true
                }
            };
            if !should_convert {
                continue;
            }

            state.converting.store(true, Ordering::SeqCst);
            run_clipboard_convert_with_notification(
                "ParseKit",
                "Markdown copied to clipboard",
                "Clipboard convert failed",
            );
            state.converting.store(false, Ordering::SeqCst);
        }
    });
}

impl Clone for ClipboardWatchState {
    fn clone(&self) -> Self {
        Self {
            auto_convert_enabled: AtomicBool::new(
                self.auto_convert_enabled.load(Ordering::SeqCst),
            ),
            last_clipboard_signature: Mutex::new(
                self.last_clipboard_signature
                    .lock()
                    .map(|s| s.clone())
                    .unwrap_or_default(),
            ),
            converting: AtomicBool::new(self.converting.load(Ordering::SeqCst)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_file_url_path() {
        assert_eq!(
            normalize_user_path("file:///tmp/report.pdf".to_string()),
            "/tmp/report.pdf"
        );
    }

    #[test]
    fn unsupported_extension_filtered() {
        assert!(!is_supported_extension("/tmp/readme.txt"));
        assert!(is_supported_extension("/tmp/report.PDF"));
    }
}