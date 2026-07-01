//! Move files to the macOS Trash (recoverable via Finder).

use std::path::Path;

/// Move `path` to the user Trash. Returns an error if the path is missing or trash fails.
pub fn move_to_trash(path: &Path) -> Result<(), String> {
    if !path.exists() {
        return Err(format!("File not found: {}", path.display()));
    }
    trash::delete(path).map_err(|e| format!("Failed to move {} to Trash: {e}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn move_to_trash_rejects_missing_file() {
        let err = move_to_trash(Path::new("/tmp/parsekit-trash-missing-test-file"))
            .expect_err("missing file should fail");
        assert!(err.contains("not found"), "unexpected: {err}");
    }

    #[test]
    fn move_to_trash_moves_temp_file() {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("parsekit-trash-test-{stamp}.txt"));
        fs::write(&path, "parsekit trash test").expect("write temp file");
        move_to_trash(&path).expect("trash temp file");
        assert!(!path.exists(), "original path should be gone after trash");
    }
}