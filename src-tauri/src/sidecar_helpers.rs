//! Shared sidecar logic for ParseKit (used by the parsekit-sidecar binary).

use crate::token_count::{self, TokenSavings};
use liteparse::{LiteParseConfig, OutputFormat, ParseResult, ParsedPage, TextItem};
use serde::Serialize;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Extensions that always export as JSON (spreadsheets / tabular).
pub const SPREADSHEET_EXTENSIONS: &[&str] = &["xls", "xlsx", "xlsm", "ods", "csv", "tsv"];

pub fn is_spreadsheet_ext(ext: &str) -> bool {
    SPREADSHEET_EXTENSIONS.contains(&ext.to_lowercase().as_str())
}

pub fn validate_output_format(format: &str) -> Result<(), String> {
    match format {
        "md" | "json" | "txt" => Ok(()),
        _ => Err(format!(
            "Invalid format \"{format}\". Expected md, json, or txt."
        )),
    }
}

pub fn build_liteparse_config(
    ocr_enabled: bool,
    ocr_language: String,
    num_workers: usize,
) -> LiteParseConfig {
    let mut config = LiteParseConfig::default();
    config.ocr_enabled = ocr_enabled;
    config.ocr_language = ocr_language;
    config.quiet = true;
    // Page-level parallelism WITHIN a single file. Files themselves are parsed
    // one at a time by the sidecar (the engine is not safe to run as multiple
    // concurrent instances), so this is the only place worker count applies.
    config.num_workers = num_workers.max(1);
    config.output_format = OutputFormat::Json;
    config
}

pub fn output_paths(file_path: &Path, out_dir: &Path, format: &str) -> (PathBuf, String, bool) {
    let ext = file_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");
    let base_name = file_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("document")
        .to_string();
    let is_spreadsheet = is_spreadsheet_ext(ext);
    let out_ext = if is_spreadsheet {
        "json"
    } else if format == "json" {
        "json"
    } else if format == "txt" {
        "txt"
    } else {
        "md"
    };
    let out_path = out_dir.join(format!("{base_name}.{out_ext}"));
    (out_path, base_name, is_spreadsheet)
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct JsonTextItem {
    text: String,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    font_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    font_size: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    confidence: Option<f32>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct JsonParsedPage {
    page_num: usize,
    width: f32,
    height: f32,
    text: String,
    text_items: Vec<JsonTextItem>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct JsonParseResult {
    pages: Vec<JsonParsedPage>,
    text: String,
}

fn to_json_result(result: &ParseResult) -> JsonParseResult {
    fn map_item(item: &TextItem) -> JsonTextItem {
        JsonTextItem {
            text: item.text.clone(),
            x: item.x,
            y: item.y,
            width: item.width,
            height: item.height,
            font_name: item.font_name.clone(),
            font_size: item.font_size,
            confidence: item.confidence,
        }
    }
    fn map_page(page: &ParsedPage) -> JsonParsedPage {
        JsonParsedPage {
            page_num: page.page_number,
            width: page.page_width,
            height: page.page_height,
            text: page.text.clone(),
            text_items: page.text_items.iter().map(map_item).collect(),
        }
    }
    JsonParseResult {
        pages: result.pages.iter().map(map_page).collect(),
        text: result.text.clone(),
    }
}

/// Fast naive PDF baseline: `pdftotext` text layer only (no OCR, no layout engine).
pub fn extract_pdf_baseline_text(path: &Path) -> String {
    try_pdftotext(path).unwrap_or_default()
}

fn try_pdftotext(path: &Path) -> Option<String> {
    let path_str = path.to_string_lossy();
    let output = Command::new("pdftotext")
        .args(["-layout", path_str.as_ref(), "-"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    String::from_utf8(output.stdout).ok()
}

/// Compute per-file token savings from parse output and emit-ready JSON fields.
pub fn compute_token_savings(
    source_path: &Path,
    output_content: &str,
    parse_result: &ParseResult,
) -> TokenSavings {
    let baseline_text = if token_count::file_type_from_path(source_path) == "pdf" {
        extract_pdf_baseline_text(source_path)
    } else {
        String::new()
    };
    token_count::compute_token_savings(source_path, output_content, parse_result, &baseline_text)
}

pub fn token_savings_event(file_name: &str, savings: &TokenSavings) -> Value {
    json!({
        "type": "token_savings",
        "file": file_name,
        "tokens_saved": savings.tokens_saved,
        "pages_unlocked": savings.pages_unlocked,
        "documents_unlocked": savings.documents_unlocked,
        "file_type": savings.file_type,
    })
}

pub fn format_output(result: &ParseResult, base_name: &str, format: &str, is_spreadsheet: bool) -> String {
    if is_spreadsheet || format == "json" {
        let json_result = to_json_result(result);
        return serde_json::to_string_pretty(&json_result).unwrap_or_else(|e| {
            json!({ "error": e.to_string() }).to_string()
        });
    }
    if format == "txt" {
        if !result.pages.is_empty() {
            return result
                .pages
                .iter()
                .map(|p| p.text.as_str())
                .collect::<Vec<_>>()
                .join("\n\n---\n\n");
        }
        return result.text.clone();
    }
    // Markdown
    let pages: Vec<String> = result
        .pages
        .iter()
        .enumerate()
        .map(|(i, p)| format!("## Page {}\n\n{}", i + 1, p.text))
        .collect();
    if pages.is_empty() {
        format!("# {base_name}\n\n{}", result.text)
    } else {
        format!("# {base_name}\n\n{}", pages.join("\n\n---\n\n"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spreadsheet_extensions_force_json_output() {
        for ext in ["xls", "xlsx", "xlsm", "ods", "csv", "tsv"] {
            assert!(is_spreadsheet_ext(ext), "{ext}");
            let (path, _, is_ss) =
                output_paths(Path::new(&format!("report.{ext}")), Path::new("/out"), "md");
            assert!(is_ss);
            assert_eq!(path.extension().and_then(|e| e.to_str()), Some("json"));
        }
    }

    #[test]
    fn non_spreadsheet_respects_format() {
        let (path, _, is_ss) = output_paths(Path::new("doc.pdf"), Path::new("/out"), "txt");
        assert!(!is_ss);
        assert_eq!(path.extension().and_then(|e| e.to_str()), Some("txt"));
    }

    #[test]
    fn validate_format_rejects_unknown() {
        assert!(validate_output_format("md").is_ok());
        assert!(validate_output_format("bogus").is_err());
    }

    #[test]
    fn liteparse_config_uses_explicit_worker_count() {
        let cfg = build_liteparse_config(true, "eng".to_string(), 4);
        assert_eq!(cfg.num_workers, 4);
        let cfg_zero = build_liteparse_config(true, "eng".to_string(), 0);
        assert_eq!(cfg_zero.num_workers, 1, "worker count is clamped to >= 1");
    }
}