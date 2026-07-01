//! ParseKit CLI: script-friendly wrapper around the parsekit-sidecar binary.
//! Same JSON-lines protocol and parse logic as the GUI — no fork of the engine.

use parsekit_lib::sidecar_helpers::validate_output_format;
use parsekit_lib::token_stats::{self, RecordInput};
use serde::Deserialize;
use serde_json::{json, Value};
use std::env;
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use walkdir::WalkDir;

const SUPPORTED_EXTENSIONS: &[&str] = &[
    "pdf", "doc", "docx", "docm", "odt", "rtf", "ppt", "pptx", "pptm", "odp", "xls", "xlsx",
    "xlsm", "ods", "csv", "tsv", "png", "jpg", "jpeg", "gif", "bmp", "tiff", "tif", "webp",
    "svg",
];

const HELP: &str = "\
ParseKit — convert documents to Markdown/JSON locally

Usage:
  parsekit convert <file> [--out <path>] [--format md|txt|json]
  parsekit convert <folder> --batch [--out <folder>]
  parsekit trash <file>...          (macOS only — move files to Trash)

Options:
  --out <path>     Output file or folder (default: same directory as input)
  --format <fmt>   Output format: md, txt, or json (default: md)
  --batch          Convert all supported files under a folder (recursive)
  --help, -h       Show this help

On success, prints the output path (one line per file in batch mode) and exits 0.
Failures print to stderr and exit non-zero.
";

#[derive(Debug, Clone, PartialEq, Eq)]
struct ConvertArgs {
    input: PathBuf,
    batch: bool,
    out: Option<PathBuf>,
    format: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SidecarSettings {
    #[serde(default = "default_format")]
    _format: String,
    #[serde(default = "default_ocr_enabled")]
    ocr_enabled: bool,
    #[serde(default = "default_ocr_language")]
    ocr_language: String,
    #[serde(default)]
    workers: u32,
}

fn default_format() -> String {
    "md".to_string()
}

fn default_ocr_enabled() -> bool {
    true
}

fn default_ocr_language() -> String {
    "eng".to_string()
}

fn default_sidecar_settings() -> SidecarSettings {
    SidecarSettings {
        _format: default_format(),
        ocr_enabled: default_ocr_enabled(),
        ocr_language: default_ocr_language(),
        workers: 0,
    }
}

fn main() {
    if let Err(message) = run() {
        eprintln!("{message}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let mut args: Vec<String> = env::args().skip(1).collect();
    if args.is_empty() || matches!(args[0].as_str(), "--help" | "-h" | "help") {
        print!("{HELP}");
        return Ok(());
    }

    if args[0] == "convert" {
        args.remove(0);
        if args.is_empty() || matches!(args[0].as_str(), "--help" | "-h") {
            print!("{HELP}");
            return Ok(());
        }
        let parsed = parse_convert_args(&args)?;
        return execute_convert(parsed);
    }

    if args[0] == "trash" {
        args.remove(0);
        return execute_trash(&args);
    }

    Err(format!(
        "Unknown command \"{}\". Run parsekit --help.",
        args[0]
    ))
}

fn parse_convert_args(args: &[String]) -> Result<ConvertArgs, String> {
    let mut input: Option<PathBuf> = None;
    let mut batch = false;
    let mut out: Option<PathBuf> = None;
    let mut format = "md".to_string();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--help" | "-h" => {
                print!("{HELP}");
                std::process::exit(0);
            }
            "--batch" => {
                batch = true;
                i += 1;
            }
            "--out" => {
                let value = args
                    .get(i + 1)
                    .ok_or_else(|| "--out requires a path".to_string())?;
                out = Some(PathBuf::from(value));
                i += 2;
            }
            "--format" => {
                let value = args
                    .get(i + 1)
                    .ok_or_else(|| "--format requires md, txt, or json".to_string())?;
                format = value.clone();
                i += 2;
            }
            flag if flag.starts_with('-') => {
                return Err(format!("Unknown option \"{flag}\""));
            }
            path => {
                if input.is_some() {
                    return Err("Only one input path is allowed".to_string());
                }
                input = Some(PathBuf::from(path));
                i += 1;
            }
        }
    }

    let input = input.ok_or_else(|| "Missing input path. Usage: parsekit convert <path>".to_string())?;
    Ok(ConvertArgs {
        input,
        batch,
        out,
        format,
    })
}

fn execute_convert(args: ConvertArgs) -> Result<(), String> {
    validate_output_format(&args.format)?;

    let input = fs::canonicalize(&args.input)
        .map_err(|e| format!("Input path not found ({}): {e}", args.input.display()))?;

    let files = if args.batch {
        if !input.is_dir() {
            return Err(format!(
                "--batch requires a folder, got file: {}",
                input.display()
            ));
        }
        scan_supported_files(&input)?
    } else if input.is_dir() {
        return Err(format!(
            "Input is a folder; use --batch or pass a file path: {}",
            input.display()
        ));
    } else {
        if !is_supported_file(&input) {
            return Err(format!(
                "Unsupported file type: {}",
                input.display()
            ));
        }
        vec![input.clone()]
    };

    if files.is_empty() {
        return Err(format!(
            "No supported files found under {}",
            input.display()
        ));
    }

    let (output_dir, rename_targets) = resolve_output_layout(&input, &args, &files)?;
    fs::create_dir_all(&output_dir)
        .map_err(|e| format!("Could not create output directory ({}): {e}", output_dir.display()))?;

    let settings = load_sidecar_settings();
    let sidecar = resolve_sidecar()?;
    let payload = json!({
        "files": files.iter().map(|p| p.to_string_lossy().to_string()).collect::<Vec<_>>(),
        "outputDir": output_dir.to_string_lossy(),
        "format": args.format,
        "ocrEnabled": settings.ocr_enabled,
        "ocrLanguage": settings.ocr_language,
        "workers": if settings.workers == 0 { 4 } else { settings.workers },
    });

    let mut child = Command::new(&sidecar)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to run sidecar ({}): {e}", sidecar.display()))?;

    if let Some(mut stdin) = child.stdin.take() {
        let line = format!("{}\n", serde_json::to_string(&payload).map_err(|e| e.to_string())?);
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
                    let file = event
                        .get("file")
                        .and_then(Value::as_str)
                        .unwrap_or("file");
                    let detail = event
                        .get("error")
                        .and_then(Value::as_str)
                        .unwrap_or("parse failed");
                    eprintln!("{file}: {detail}");
                } else if matches!(status, "completed" | "skipped") {
                    if let Some(path) = event.get("path").and_then(Value::as_str) {
                        output_paths.push(PathBuf::from(path));
                    }
                }
            }
            Some("token_savings") => {
                record_token_savings_event(&event)?;
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

    let mut final_paths = Vec::with_capacity(output_paths.len());
    for produced in output_paths {
        let final_path = apply_rename_target(&produced, &rename_targets)?;
        println!("{}", final_path.display());
        final_paths.push(final_path);
    }

    let _ = final_paths;
    Ok(())
}

fn record_token_savings_event(event: &Value) -> Result<(), String> {
    let file_type = event
        .get("file_type")
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim()
        .to_string();
    if file_type.is_empty() {
        return Ok(());
    }
    token_stats::record(RecordInput {
        file_type,
        tokens_saved: event
            .get("tokens_saved")
            .and_then(Value::as_u64)
            .unwrap_or(0),
        pages_unlocked: event
            .get("pages_unlocked")
            .and_then(Value::as_u64)
            .unwrap_or(0),
        documents_unlocked: event
            .get("documents_unlocked")
            .and_then(Value::as_u64)
            .unwrap_or(0),
    })
    .map(|_| ())
}

fn resolve_output_layout(
    input: &Path,
    args: &ConvertArgs,
    files: &[PathBuf],
) -> Result<(PathBuf, Vec<Option<PathBuf>>), String> {
    if args.batch {
        let output_dir = match &args.out {
            Some(path) => {
                if path.is_file() {
                    return Err("--out must be a folder when using --batch".to_string());
                }
                path.clone()
            }
            None => input.to_path_buf(),
        };
        let rename_targets = vec![None; files.len()];
        return Ok((output_dir, rename_targets));
    }

    let file = &files[0];
    let desired_out = args.out.clone();
    let output_dir = match &desired_out {
        Some(path) if path.is_dir() || path.to_string_lossy().ends_with('/') => path.clone(),
        Some(path) => path
            .parent()
            .map(Path::to_path_buf)
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or_else(|| {
                file.parent()
                    .map(Path::to_path_buf)
                    .unwrap_or_else(|| PathBuf::from("."))
            }),
        None => file
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| PathBuf::from(".")),
    };

    let rename_target = match desired_out {
        Some(path) if !(path.is_dir() || path.to_string_lossy().ends_with('/')) => Some(path),
        _ => None,
    };
    Ok((output_dir, vec![rename_target]))
}

fn apply_rename_target(
    produced: &Path,
    rename_targets: &[Option<PathBuf>],
) -> Result<PathBuf, String> {
    let target = rename_targets
        .iter()
        .find_map(|candidate| candidate.as_ref())
        .cloned();
    let Some(target) = target else {
        return Ok(produced.to_path_buf());
    };
    if produced == target {
        return Ok(target);
    }
    if target.exists() {
        fs::remove_file(&target).map_err(|e| {
            format!(
                "Could not replace existing output ({}): {e}",
                target.display()
            )
        })?;
    }
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent).map_err(|e| {
            format!(
                "Could not create output parent ({}): {e}",
                parent.display()
            )
        })?;
    }
    fs::rename(produced, &target).map_err(|e| {
        format!(
            "Could not move output to {}: {e}",
            target.display()
        )
    })?;
    Ok(target)
}

fn is_supported_file(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| {
            let lower = ext.to_ascii_lowercase();
            SUPPORTED_EXTENSIONS.contains(&lower.as_str())
        })
        .unwrap_or(false)
}

fn scan_supported_files(dir: &Path) -> Result<Vec<PathBuf>, String> {
    let mut files = Vec::new();
    for entry in WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_file() && is_supported_file(path) {
            files.push(path.to_path_buf());
        }
    }
    files.sort();
    Ok(files)
}

fn settings_path() -> Result<PathBuf, String> {
    let home = env::var("HOME").map_err(|e| e.to_string())?;
    Ok(PathBuf::from(home)
        .join("Library/Application Support/com.harshabala.parsekit/settings.json"))
}

fn load_sidecar_settings() -> SidecarSettings {
    let path = match settings_path() {
        Ok(path) => path,
        Err(_) => return default_sidecar_settings(),
    };
    let raw = match fs::read_to_string(path) {
        Ok(raw) => raw,
        Err(_) => return default_sidecar_settings(),
    };
    serde_json::from_str(&raw).unwrap_or_else(|_| default_sidecar_settings())
}

fn host_triple() -> String {
    match env::consts::OS {
        "macos" => format!("{}-apple-darwin", env::consts::ARCH),
        "linux" => format!("{}-unknown-linux-gnu", env::consts::ARCH),
        "windows" => format!("{}-pc-windows-msvc", env::consts::ARCH),
        other => format!("{}-{other}", env::consts::ARCH),
    }
}

fn resolve_sidecar() -> Result<PathBuf, String> {
    if let Ok(path) = env::var("PARSEKIT_SIDECAR") {
        let candidate = PathBuf::from(&path);
        if candidate.is_file() {
            return Ok(candidate);
        }
        return Err(format!("PARSEKIT_SIDECAR not found: {path}"));
    }

    if let Ok(exe) = env::current_exe() {
        if let Some(dir) = exe.parent() {
            let sibling = dir.join("parsekit-sidecar");
            if sibling.is_file() {
                return Ok(sibling);
            }
        }
    }

    if let Ok(exe) = env::current_exe() {
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

fn execute_trash(args: &[String]) -> Result<(), String> {
    if args.is_empty() || matches!(args[0].as_str(), "--help" | "-h") {
        print!("{HELP}");
        return Ok(());
    }

    #[cfg(not(target_os = "macos"))]
    {
        let _ = args;
        return Err("parsekit trash is only available on macOS.".into());
    }

    #[cfg(target_os = "macos")]
    {
        for raw in args {
            if raw.starts_with('-') {
                return Err(format!("Unknown option for trash: {raw}"));
            }
            let path = PathBuf::from(raw);
            parsekit_lib::macos_trash::move_to_trash(&path)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_single_file_args() {
        let args = parse_convert_args(&[
            "doc.pdf".into(),
            "--format".into(),
            "txt".into(),
            "--out".into(),
            "/tmp/out".into(),
        ])
        .expect("parse");
        assert_eq!(args.input, PathBuf::from("doc.pdf"));
        assert!(!args.batch);
        assert_eq!(args.format, "txt");
        assert_eq!(args.out, Some(PathBuf::from("/tmp/out")));
    }

    #[test]
    fn parse_batch_args() {
        let args = parse_convert_args(&["./folder".into(), "--batch".into()])
            .expect("parse");
        assert!(args.batch);
        assert_eq!(args.format, "md");
    }

    #[test]
    fn output_paths_for_single_file() {
        let input = PathBuf::from("/in/report.pdf");
        let (out_dir, _) = resolve_output_layout(
            &input,
            &ConvertArgs {
                input: input.clone(),
                batch: false,
                out: None,
                format: "md".into(),
            },
            &[input.clone()],
        )
        .expect("layout");
        assert_eq!(out_dir, PathBuf::from("/in"));
    }

    #[test]
    fn expected_sidecar_output_name() {
        let (path, _, _) = parsekit_lib::sidecar_helpers::output_paths(
            Path::new("/in/report.pdf"),
            Path::new("/out"),
            "md",
        );
        assert_eq!(path, PathBuf::from("/out/report.md"));
    }
}