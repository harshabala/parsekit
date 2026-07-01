//! Native macOS notifications from the ParseKit process (correct app icon).

#[cfg(target_os = "macos")]
pub fn display_notification(title: &str, body: &str) -> Result<(), String> {
    let mut notification = notify_rust::Notification::new();
    notification.summary(title).body(body);

    if let Some(icon) = resolve_notification_icon() {
        notification.icon(&icon);
    }

    notification
        .show()
        .map_err(|e| format!("notification failed: {e}"))?;
    Ok(())
}

#[cfg(target_os = "macos")]
fn resolve_notification_icon() -> Option<String> {
    if let Ok(exe) = std::env::current_exe() {
        let bundled = exe
            .parent()?
            .parent()?
            .parent()?
            .join("Resources")
            .join("icon.icns");
        if bundled.is_file() {
            return Some(bundled.to_string_lossy().into_owned());
        }
    }
    // Dev builds: icon next to crate root
    let dev_icon = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("icons/icon.icns");
    if dev_icon.is_file() {
        return Some(dev_icon.to_string_lossy().into_owned());
    }
    None
}

#[cfg(not(target_os = "macos"))]
pub fn display_notification(_title: &str, _body: &str) -> Result<(), String> {
    Ok(())
}