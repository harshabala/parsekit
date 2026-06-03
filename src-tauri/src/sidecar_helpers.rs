//! Shared sidecar logic for ParseDock (used by the parsedock-sidecar binary).

use liteparse::{LiteParseConfig, OutputFormat, ParseResult, ParsedPage, TextItem};
use serde::Serialize;
use serde_json::json;
use std::path::{Path, PathBuf};

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

/// OCR threads per file: when multiple files run in parallel, avoid N×M thread explosion.
pub fn ocr_num_workers(file_concurrency: usize, ocr_enabled: bool) -> usize {
    if ocr_enabled && file_concurrency > 1 {
        1
    } else {
        file_concurrency.max(1)
    }
}

pub fn build_liteparse_config(
    ocr_enabled: bool,
    ocr_language: String,
    file_concurrency: usize,
) -> LiteParseConfig {
    let mut config = LiteParseConfig::default();
    config.ocr_enabled = ocr_enabled;
    config.ocr_language = ocr_language;
    config.quiet = true;
    config.num_workers = ocr_num_workers(file_concurrency, ocr_enabled);
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
    fn ocr_num_workers_capped_when_parallel_files() {
        assert_eq!(ocr_num_workers(4, true), 1);
        assert_eq!(ocr_num_workers(4, false), 4);
        assert_eq!(ocr_num_workers(1, true), 1);
    }
}