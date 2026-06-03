//! ParseDock sidecar: JSON-lines stdin/stdout protocol for batch document parsing.
//! Uses LiteParse v2 (Rust core) — same contract as the legacy Node sidecar.

use liteparse::LiteParse;
use parsedock_lib::sidecar_helpers::{
    build_liteparse_config, format_output, output_paths, validate_output_format,
};
use serde::Deserialize;
use serde_json::json;
use std::io::{self, BufRead, Write};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Semaphore;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SidecarConfig {
    files: Vec<String>,
    output_dir: String,
    format: String,
    #[serde(default = "default_ocr_enabled")]
    ocr_enabled: bool,
    #[serde(default = "default_ocr_language")]
    ocr_language: String,
    #[serde(default = "default_workers")]
    workers: usize,
}

fn default_ocr_enabled() -> bool {
    true
}

fn default_ocr_language() -> String {
    "eng".to_string()
}

fn default_workers() -> usize {
    4
}

fn emit(value: serde_json::Value) {
    let mut stdout = io::stdout().lock();
    let _ = writeln!(stdout, "{}", value);
}

async fn process_file(
    file_path: String,
    out_dir: PathBuf,
    format: String,
    lp_config: liteparse::LiteParseConfig,
) -> &'static str {
    let path = PathBuf::from(&file_path);
    let file_name = path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or(&file_path)
        .to_string();
    let (out_path, base_name, is_spreadsheet) = output_paths(&path, &out_dir, &format);

    if let Ok(meta) = tokio::fs::metadata(&out_path).await {
        if meta.len() > 0 {
            emit(json!({
                "type": "progress",
                "file": file_name,
                "status": "skipped",
                "path": out_path.to_string_lossy(),
            }));
            return "skipped";
        }
    }

    emit(json!({
        "type": "progress",
        "file": file_name,
        "status": "parsing",
    }));

    let parser = LiteParse::new(lp_config);
    match parser.parse(&file_path).await {
        Ok(result) => {
            let content = format_output(&result, &base_name, &format, is_spreadsheet);
            if let Err(e) = tokio::fs::write(&out_path, content).await {
                emit(json!({
                    "type": "progress",
                    "file": file_name,
                    "status": "error",
                    "error": e.to_string(),
                }));
                return "error";
            }
            emit(json!({
                "type": "progress",
                "file": file_name,
                "status": "completed",
                "path": out_path.to_string_lossy(),
            }));
            "completed"
        }
        Err(e) => {
            emit(json!({
                "type": "progress",
                "file": file_name,
                "status": "error",
                "error": e.to_string(),
            }));
            "error"
        }
    }
}

async fn run(config: SidecarConfig) -> Result<(), String> {
    validate_output_format(&config.format)?;

    let out_dir = PathBuf::from(&config.output_dir);
    tokio::fs::create_dir_all(&out_dir)
        .await
        .map_err(|e| e.to_string())?;

    let file_concurrency = config.workers.max(1);
    let format = config.format.clone();
    let ocr_enabled = config.ocr_enabled;
    let ocr_language = config.ocr_language.clone();

    emit(json!({
        "type": "start",
        "total": config.files.len(),
    }));

    let semaphore = Arc::new(Semaphore::new(file_concurrency));
    let mut parsed = 0usize;
    let mut skipped = 0usize;
    let mut errors = 0usize;

    let mut handles = Vec::with_capacity(config.files.len());
    for file_path in config.files {
        let permit = semaphore
            .clone()
            .acquire_owned()
            .await
            .map_err(|e| e.to_string())?;
        let out_dir = out_dir.clone();
        let format = format.clone();
        let ocr_language = ocr_language.clone();
        let lp_config = build_liteparse_config(ocr_enabled, ocr_language, file_concurrency);

        handles.push(tokio::spawn(async move {
            let result = process_file(file_path, out_dir, format, lp_config).await;
            drop(permit);
            result
        }));
    }

    for handle in handles {
        match handle.await {
            Ok("skipped") => skipped += 1,
            Ok("completed") => parsed += 1,
            Ok("error") | Ok(_) => errors += 1,
            Err(e) => {
                errors += 1;
                emit(json!({
                    "type": "error",
                    "message": e.to_string(),
                }));
            }
        }
    }

    emit(json!({
        "type": "done",
        "parsed": parsed,
        "skipped": skipped,
        "errors": errors,
    }));
    Ok(())
}

#[tokio::main]
async fn main() {
    let stdin = io::stdin();
    let mut lines = stdin.lock().lines();
    let line = match lines.next() {
        Some(Ok(l)) => l,
        Some(Err(e)) => {
            emit(json!({ "type": "error", "message": e.to_string() }));
            std::process::exit(1);
        }
        None => {
            emit(json!({ "type": "error", "message": "No input on stdin" }));
            std::process::exit(1);
        }
    };

    let config: SidecarConfig = match serde_json::from_str(&line) {
        Ok(c) => c,
        Err(e) => {
            emit(json!({ "type": "error", "message": format!("Invalid JSON input: {e}") }));
            std::process::exit(1);
        }
    };

    if let Err(e) = run(config).await {
        emit(json!({ "type": "error", "message": e }));
        std::process::exit(1);
    }
}