//! Local token savings counter persisted in app support as `token-stats.json`.

use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

const STATS_FILE_NAME: &str = "token-stats.json";
const MAX_TOKEN_EVENTS: usize = 500;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct FileTypeStats {
    pub files: u64,
    pub tokens_saved: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TokenEvent {
    pub ts: String,
    pub file_type: String,
    pub tokens_saved: u64,
    pub pages_unlocked: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct TokenStats {
    pub total_files_converted: u64,
    pub total_tokens_saved: u64,
    pub total_pages_unlocked: u64,
    pub total_documents_unlocked: u64,
    #[serde(default)]
    pub by_file_type: HashMap<String, FileTypeStats>,
    #[serde(default)]
    pub events: Vec<TokenEvent>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordInput {
    pub file_type: String,
    pub tokens_saved: u64,
    pub pages_unlocked: u64,
    pub documents_unlocked: u64,
}

fn support_dir() -> Result<PathBuf, String> {
    let home = std::env::var("HOME").map_err(|e| e.to_string())?;
    Ok(PathBuf::from(home).join("Library/Application Support/com.harshabala.parsekit"))
}

fn stats_path() -> Result<PathBuf, String> {
    Ok(support_dir()?.join(STATS_FILE_NAME))
}

fn ensure_support_dir() -> Result<PathBuf, String> {
    let dir = support_dir()?;
    std::fs::create_dir_all(&dir).map_err(|e| format!("Could not create support dir: {e}"))?;
    Ok(dir)
}

fn normalize_file_type(file_type: &str) -> String {
    file_type.trim().trim_start_matches('.').to_ascii_lowercase()
}

fn now_iso8601() -> String {
    Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
}

pub fn load() -> TokenStats {
    let path = match stats_path() {
        Ok(path) => path,
        Err(_) => return TokenStats::default(),
    };
    load_from_path(&path).unwrap_or_default()
}

fn load_from_path(path: &Path) -> Result<TokenStats, String> {
    let raw = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    serde_json::from_str(&raw).map_err(|e| format!("Invalid token stats JSON: {e}"))
}

pub fn save(stats: &TokenStats) -> Result<(), String> {
    let _ = ensure_support_dir()?;
    let path = stats_path()?;
    let json = serde_json::to_string_pretty(stats).map_err(|e| e.to_string())?;
    std::fs::write(&path, json).map_err(|e| format!("Could not write token stats: {e}"))
}

pub fn reset() -> Result<TokenStats, String> {
    let stats = TokenStats::default();
    save(&stats)?;
    Ok(stats)
}

pub fn record(input: RecordInput) -> Result<TokenStats, String> {
    let mut stats = load();
    apply_record(&mut stats, &input);
    save(&stats)?;
    Ok(stats)
}

fn apply_record(stats: &mut TokenStats, input: &RecordInput) {
    let file_type = normalize_file_type(&input.file_type);
    if file_type.is_empty() {
        return;
    }

    let tokens_saved = input.tokens_saved;
    let pages_unlocked = input.pages_unlocked;
    let documents_unlocked = input.documents_unlocked;

    stats.total_files_converted = stats.total_files_converted.saturating_add(1);
    stats.total_tokens_saved = stats
        .total_tokens_saved
        .saturating_add(tokens_saved);
    stats.total_pages_unlocked = stats
        .total_pages_unlocked
        .saturating_add(pages_unlocked);
    stats.total_documents_unlocked = stats
        .total_documents_unlocked
        .saturating_add(documents_unlocked);

    let entry = stats.by_file_type.entry(file_type.clone()).or_default();
    entry.files = entry.files.saturating_add(1);
    entry.tokens_saved = entry.tokens_saved.saturating_add(tokens_saved);

    stats.events.push(TokenEvent {
        ts: now_iso8601(),
        file_type,
        tokens_saved,
        pages_unlocked,
    });
    if stats.events.len() > MAX_TOKEN_EVENTS {
        let overflow = stats.events.len() - MAX_TOKEN_EVENTS;
        stats.events.drain(0..overflow);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Mutex, MutexGuard};
    use std::time::{SystemTime, UNIX_EPOCH};

    static TEST_DIR_LOCK: Mutex<()> = Mutex::new(());

    struct TestDirGuard {
        _lock: MutexGuard<'static, ()>,
        path: PathBuf,
    }

    impl TestDirGuard {
        fn new() -> Self {
            let lock = TEST_DIR_LOCK.lock().expect("test dir lock poisoned");
            let nanos = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("clock before epoch")
                .as_nanos();
            let path = std::env::temp_dir().join(format!("parsekit-token-stats-test-{nanos}"));
            std::fs::create_dir_all(&path).expect("create temp support dir");
            std::env::set_var("HOME", &path);
            Self {
                _lock: lock,
                path,
            }
        }
    }

    impl Drop for TestDirGuard {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.path);
            std::env::remove_var("HOME");
        }
    }

    #[test]
    fn default_stats_when_file_missing() {
        let _guard = TestDirGuard::new();
        let stats = load();
        assert_eq!(stats, TokenStats::default());
    }

    #[test]
    fn record_increments_totals_and_by_file_type() {
        let _guard = TestDirGuard::new();
        let stats = record(RecordInput {
            file_type: "PDF".into(),
            tokens_saved: 340,
            pages_unlocked: 0,
            documents_unlocked: 0,
        })
        .expect("record");

        assert_eq!(stats.total_files_converted, 1);
        assert_eq!(stats.total_tokens_saved, 340);
        assert_eq!(stats.total_pages_unlocked, 0);
        assert_eq!(stats.total_documents_unlocked, 0);
        assert_eq!(stats.by_file_type.get("pdf").map(|v| v.files), Some(1));
        assert_eq!(
            stats.by_file_type.get("pdf").map(|v| v.tokens_saved),
            Some(340)
        );
        assert_eq!(stats.events.len(), 1);
        assert_eq!(stats.events[0].file_type, "pdf");
        assert_eq!(stats.events[0].tokens_saved, 340);
        assert!(!stats.events[0].ts.is_empty());
    }

    #[test]
    fn record_tracks_pages_and_documents_separately() {
        let _guard = TestDirGuard::new();
        let stats = record(RecordInput {
            file_type: "pdf".into(),
            tokens_saved: 0,
            pages_unlocked: 12,
            documents_unlocked: 1,
        })
        .expect("record");

        assert_eq!(stats.total_tokens_saved, 0);
        assert_eq!(stats.total_pages_unlocked, 12);
        assert_eq!(stats.total_documents_unlocked, 1);
        assert_eq!(stats.events[0].pages_unlocked, 12);
    }

    #[test]
    fn record_persists_across_load() {
        let _guard = TestDirGuard::new();
        record(RecordInput {
            file_type: "docx".into(),
            tokens_saved: 100,
            pages_unlocked: 0,
            documents_unlocked: 0,
        })
        .expect("record");

        let loaded = load();
        assert_eq!(loaded.total_files_converted, 1);
        assert_eq!(loaded.total_tokens_saved, 100);
        assert_eq!(loaded.by_file_type.get("docx").map(|v| v.files), Some(1));
    }

    #[test]
    fn reset_clears_all_fields() {
        let _guard = TestDirGuard::new();
        record(RecordInput {
            file_type: "pdf".into(),
            tokens_saved: 50,
            pages_unlocked: 2,
            documents_unlocked: 1,
        })
        .expect("record");

        let stats = reset().expect("reset");
        assert_eq!(stats, TokenStats::default());
        assert_eq!(load(), TokenStats::default());
    }

    #[test]
    fn events_are_capped_at_max() {
        let _guard = TestDirGuard::new();
        for i in 0..(MAX_TOKEN_EVENTS + 25) {
            record(RecordInput {
                file_type: format!("pdf{i}"),
                tokens_saved: 1,
                pages_unlocked: 0,
                documents_unlocked: 0,
            })
            .expect("record");
        }
        let stats = load();
        assert_eq!(stats.events.len(), MAX_TOKEN_EVENTS);
    }

    #[test]
    fn save_load_roundtrip() {
        let _guard = TestDirGuard::new();
        let mut stats = TokenStats::default();
        stats.total_files_converted = 2;
        stats.total_tokens_saved = 500;
        stats.by_file_type.insert(
            "pptx".into(),
            FileTypeStats {
                files: 2,
                tokens_saved: 500,
            },
        );
        stats.events.push(TokenEvent {
            ts: "2026-07-01T12:00:00Z".into(),
            file_type: "pptx".into(),
            tokens_saved: 250,
            pages_unlocked: 0,
        });
        save(&stats).expect("save");

        let loaded = load();
        assert_eq!(loaded, stats);
    }
}