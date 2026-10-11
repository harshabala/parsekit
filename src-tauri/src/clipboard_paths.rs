//! Read file paths from the macOS pasteboard (Finder copy → file URL).

#[cfg(target_os = "macos")]
pub(crate) fn run_osascript(script: &str) -> Option<String> {
    let output = std::process::Command::new("osascript")
        .arg("-e")
        .arg(script)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if text.is_empty() {
        None
    } else {
        Some(text)
    }
}

/// File paths currently on the clipboard (Finder copy puts file URLs on the pasteboard).
#[cfg(target_os = "macos")]
pub fn get_clipboard_file_paths() -> Vec<String> {
    let file_script = r#"
try
    set raw to the clipboard as «class furl»
    if class of raw is list then
        set out to ""
        repeat with f in raw
            set end of out to (POSIX path of f) & linefeed
        end repeat
        return text 1 thru -2 of out
    else
        return POSIX path of raw
    end if
on error
    return ""
end try
"#;
    if let Some(text) = run_osascript(file_script) {
        let paths: Vec<String> = text
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .map(str::to_string)
            .collect();
        if !paths.is_empty() {
            return paths;
        }
    }

    if let Some(text) = run_osascript("the clipboard as text") {
        let trimmed = text.trim();
        if !trimmed.is_empty() && (trimmed.starts_with('/') || trimmed.starts_with("file://")) {
            return vec![trimmed.to_string()];
        }
    }

    Vec::new()
}

#[cfg(not(target_os = "macos"))]
pub fn get_clipboard_file_paths() -> Vec<String> {
    Vec::new()
}

#[cfg(test)]
mod tests {
    #[test]
    fn non_macos_clipboard_paths_empty() {
        #[cfg(not(target_os = "macos"))]
        assert!(super::get_clipboard_file_paths().is_empty());
    }
}