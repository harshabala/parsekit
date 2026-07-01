//! Native macOS notifications from the ParseKit process (correct app icon, not Script Editor).

use std::sync::Once;

static INIT_BUNDLE: Once = Once::new();

const PARSEKIT_BUNDLE_ID: &str = "com.harshabala.parsekit";

/// Call once at app startup so Notification Center shows ParseKit, not Script Editor.
#[cfg(target_os = "macos")]
pub fn init_notification_bundle() {
    INIT_BUNDLE.call_once(|| {
        let _ = mac_notification_sys::set_application(PARSEKIT_BUNDLE_ID);
    });
}

#[cfg(not(target_os = "macos"))]
pub fn init_notification_bundle() {}

#[cfg(target_os = "macos")]
pub fn display_notification(title: &str, body: &str) -> Result<(), String> {
    init_notification_bundle();
    let subtitle: Option<&str> = None;
    let sound: Option<&str> = None;
    mac_notification_sys::send_notification(title, &subtitle, body, &sound)
        .map_err(|e| format!("notification failed: {e}"))
}

#[cfg(not(target_os = "macos"))]
pub fn display_notification(_title: &str, _body: &str) -> Result<(), String> {
    Ok(())
}