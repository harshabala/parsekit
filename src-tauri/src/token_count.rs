//! Token counting (cl100k_base) and savings formula for the sidecar tracker.

use liteparse::ParseResult;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;
use tiktoken_rs::cl100k_base_singleton;

const BENCHMARK_RATIOS_JSON: &str = include_str!("../../scripts/benchmark-ratios.json");

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TokenSavings {
    pub tokens_saved: u64,
    pub pages_unlocked: u64,
    pub documents_unlocked: u64,
    pub file_type: String,
}

#[derive(Debug, Deserialize)]
struct BenchmarkRatiosFile {
    #[serde(default)]
    by_file_type: HashMap<String, BenchmarkFileTypeEntry>,
}

#[derive(Debug, Deserialize)]
struct BenchmarkFileTypeEntry {
    #[serde(default)]
    avg_reduction_ratio: f64,
}

/// Count tokens using OpenAI `cl100k_base` (GPT-4 family reference encoding).
pub fn count_tokens(text: &str) -> u64 {
    let bpe = cl100k_base_singleton();
    bpe.encode_with_special_tokens(text).len() as u64
}

pub fn file_type_from_path(path: &Path) -> String {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|ext| ext.trim_start_matches('.').to_ascii_lowercase())
        .filter(|ext| !ext.is_empty())
        .unwrap_or_else(|| "unknown".to_string())
}

/// Whether OCR contributed text to the parse (any text item carries confidence).
pub fn ocr_was_used(result: &ParseResult) -> bool {
    result.pages.iter().any(|page| {
        page.text_items
            .iter()
            .any(|item| item.confidence.is_some())
    })
}

/// Pages that gained readable text via OCR.
pub fn pages_unlocked_from_result(result: &ParseResult) -> u64 {
    result
        .pages
        .iter()
        .filter(|page| {
            page.text_items
                .iter()
                .any(|item| item.confidence.is_some() && !item.text.trim().is_empty())
        })
        .count() as u64
}

/// Scanned/OCR-only path: naive PDF baseline is empty and OCR unlocked content.
pub fn is_scanned_unlock(baseline_text: &str, result: &ParseResult, file_type: &str) -> bool {
    file_type == "pdf" && baseline_text.trim().is_empty() && ocr_was_used(result)
}

fn avg_reduction_ratio(file_type: &str) -> f64 {
    let parsed: BenchmarkRatiosFile =
        serde_json::from_str(BENCHMARK_RATIOS_JSON).unwrap_or(BenchmarkRatiosFile {
            by_file_type: HashMap::new(),
        });
    let key = file_type.trim().trim_start_matches('.').to_ascii_lowercase();
    parsed
        .by_file_type
        .get(&key)
        .map(|entry| entry.avg_reduction_ratio)
        .unwrap_or(0.0)
        .clamp(0.0, 1.0)
}

/// Estimate baseline tokens from ParseKit output and benchmark reduction ratio.
/// Formula (Section 8.1): `baseline ≈ output / (1 - ratio)` when ratio > 0.
pub fn estimate_baseline_tokens(output_tokens: u64, reduction_ratio: f64) -> u64 {
    if output_tokens == 0 {
        return 0;
    }
    if reduction_ratio <= 0.0 {
        return output_tokens;
    }
    if reduction_ratio >= 1.0 {
        return output_tokens;
    }
    let baseline = (output_tokens as f64) / (1.0 - reduction_ratio);
    baseline.ceil().max(0.0) as u64
}

/// `saved = max(0, baseline - output)` — never negative.
pub fn tokens_saved_from_counts(baseline_tokens: u64, output_tokens: u64) -> u64 {
    baseline_tokens.saturating_sub(output_tokens)
}

pub fn compute_token_savings(
    file_path: &Path,
    output_content: &str,
    parse_result: &ParseResult,
    baseline_text: &str,
) -> TokenSavings {
    let file_type = file_type_from_path(file_path);
    let output_tokens = count_tokens(output_content);

    if is_scanned_unlock(baseline_text, parse_result, &file_type) {
        let pages = pages_unlocked_from_result(parse_result);
        return TokenSavings {
            tokens_saved: 0,
            pages_unlocked: pages.max(1),
            documents_unlocked: 1,
            file_type,
        };
    }

    let baseline_tokens = if file_type == "pdf" && !baseline_text.trim().is_empty() {
        count_tokens(baseline_text)
    } else {
        let ratio = avg_reduction_ratio(&file_type);
        estimate_baseline_tokens(output_tokens, ratio)
    };

    TokenSavings {
        tokens_saved: tokens_saved_from_counts(baseline_tokens, output_tokens),
        pages_unlocked: 0,
        documents_unlocked: 0,
        file_type,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use liteparse::{ParsedPage, ParseResult, TextItem};
    use liteparse::types::Region;

    fn empty_parse_result() -> ParseResult {
        ParseResult {
            pages: Vec::new(),
            text: String::new(),
            outline: Vec::new(),
            images: Vec::new(),
        }
    }

    fn ocr_parse_result(page_text: &str) -> ParseResult {
        ParseResult {
            pages: vec![ParsedPage {
                page_number: 1,
                page_width: 612.0,
                page_height: 792.0,
                text: page_text.to_string(),
                text_items: vec![TextItem {
                    text: page_text.to_string(),
                    confidence: Some(0.92),
                    ..Default::default()
                }],
                projected_lines: Vec::new(),
                regions: Region::default(),
                graphics: Vec::new(),
                figures: Vec::new(),
                struct_nodes: Vec::new(),
                image_refs: Vec::new(),
            }],
            text: page_text.to_string(),
            outline: Vec::new(),
            images: Vec::new(),
        }
    }

    #[test]
    fn count_tokens_empty_is_zero() {
        assert_eq!(count_tokens(""), 0);
    }

    #[test]
    fn count_tokens_non_empty() {
        assert!(count_tokens("Hello, world!") > 0);
    }

    #[test]
    fn tokens_saved_floors_at_zero_when_output_larger() {
        assert_eq!(tokens_saved_from_counts(50, 80), 0);
        assert_eq!(tokens_saved_from_counts(100, 100), 0);
    }

    #[test]
    fn tokens_saved_positive_when_baseline_larger() {
        assert_eq!(tokens_saved_from_counts(200, 120), 80);
    }

    #[test]
    fn estimate_baseline_from_ratio_formula() {
        // baseline = 50 / (1 - 0.5) = 100
        assert_eq!(estimate_baseline_tokens(50, 0.5), 100);
        // ratio 0 -> no estimate lift
        assert_eq!(estimate_baseline_tokens(50, 0.0), 50);
    }

    #[test]
    fn scanned_path_skips_token_savings() {
        let result = ocr_parse_result("Invoice total $512");
        let savings = compute_token_savings(
            Path::new("/tmp/scanned.pdf"),
            "## Page 1\n\nInvoice total $512",
            &result,
            "",
        );
        assert_eq!(savings.tokens_saved, 0);
        assert_eq!(savings.pages_unlocked, 1);
        assert_eq!(savings.documents_unlocked, 1);
        assert_eq!(savings.file_type, "pdf");
    }

    #[test]
    fn live_pdf_baseline_used_when_text_present() {
        let result = empty_parse_result();
        let baseline = "word ".repeat(40);
        let output = "short";
        let savings = compute_token_savings(
            Path::new("/tmp/doc.pdf"),
            output,
            &result,
            &baseline,
        );
        let baseline_tokens = count_tokens(&baseline);
        let output_tokens = count_tokens(output);
        assert_eq!(
            savings.tokens_saved,
            tokens_saved_from_counts(baseline_tokens, output_tokens)
        );
        assert_eq!(savings.pages_unlocked, 0);
        assert_eq!(savings.documents_unlocked, 0);
    }

    #[test]
    fn fallback_ratio_when_pdf_baseline_empty_but_no_ocr() {
        let result = empty_parse_result();
        let savings = compute_token_savings(
            Path::new("/tmp/empty.pdf"),
            "some output",
            &result,
            "",
        );
        // benchmark-ratios.json has 0.0 for pdf -> baseline == output -> saved 0
        assert_eq!(savings.tokens_saved, 0);
        assert_eq!(savings.pages_unlocked, 0);
        assert_eq!(savings.documents_unlocked, 0);
    }

    #[test]
    fn file_type_from_path_normalizes_extension() {
        assert_eq!(file_type_from_path(Path::new("Report.PDF")), "pdf");
        assert_eq!(file_type_from_path(Path::new("noext")), "unknown");
    }
}