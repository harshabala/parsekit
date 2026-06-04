//! macOS-specific popover window configuration so the panel appears above the menu bar.

use crate::popover_trace::trace as popover_trace;
use std::sync::atomic::{AtomicBool, Ordering};

static WINDOW_CONFIGURED: AtomicBool = AtomicBool::new(false);

#[cfg(target_os = "macos")]
use objc2::MainThreadMarker;
#[cfg(target_os = "macos")]
use objc2_app_kit::{
    NSApplication, NSColor, NSFloatingWindowLevel, NSWindow, NSWindowCollectionBehavior,
};
#[cfg(target_os = "macos")]
use tauri::WebviewWindow;

/// Apply NSWindow popover settings once (defer until first open so webview init is stable).
pub fn ensure_popover_window_configured<R: tauri::Runtime>(window: &WebviewWindow<R>) {
    if WINDOW_CONFIGURED.swap(true, Ordering::SeqCst) {
        popover_trace("Activation: NSWindow already configured");
        return;
    }
    popover_trace("Activation: configure_popover_window (first open)");
    configure_popover_window(window);
}

/// Raise the app and popover window so a borderless panel is actually visible.
#[cfg(target_os = "macos")]
pub(crate) fn activate_app_for_popover<R: tauri::Runtime>(
    window: &WebviewWindow<R>,
    popover: &crate::PopoverState,
) {
    popover_trace("Activation: start");
    ensure_popover_window_configured(window);
    let Some(mtm) = MainThreadMarker::new() else {
        popover_trace("Activation: fallback (not main thread) — no NSApplication.activate");
        popover_trace("Window.show()");
        let _ = window.show();
        popover_trace("Window.focus()");
        let _ = window.set_focus();
        popover.mark_opening();
        popover_trace("show: grace period started (mark_opening after Window.show)");
        return;
    };
    let app = NSApplication::sharedApplication(mtm);
    popover_trace("Activation: NSApplication.activate(front)");
    #[allow(deprecated)]
    app.activateIgnoringOtherApps(true);
    app.activate();
    let _ = window.set_always_on_top(true);
    if let Ok(ns_ptr) = window.ns_window() {
        unsafe {
            let ns_window: &NSWindow = &*ns_ptr.cast();
            ns_window.makeKeyAndOrderFront(None);
        }
        popover_trace("Activation: NSWindow.makeKeyAndOrderFront");
    }
    popover_trace("Window.show()");
    let _ = window.show();
    popover_trace("Window.focus()");
    let _ = window.set_focus();
    popover.mark_opening();
    popover_trace("show: grace period started (mark_opening after Window.show)");
    popover_trace("Activation: complete");
    // Tray-open path re-blocks in TrayGuard; keep block until paired mouse-up finishes.
}

/// Configure the webview window as a floating popover (above menu bar, all spaces).
#[cfg(target_os = "macos")]
pub fn configure_popover_window<R: tauri::Runtime>(window: &WebviewWindow<R>) {
    let Ok(ns_ptr) = window.ns_window() else {
        popover_trace("Activation: configure FAILED (ns_window unavailable)");
        return;
    };
    unsafe {
        let ns_window: &NSWindow = &*ns_ptr.cast();
        ns_window.setLevel(NSFloatingWindowLevel);
        popover_trace("Activation: NSFloatingWindowLevel + clear background");
        let behavior = NSWindowCollectionBehavior::CanJoinAllSpaces
            | NSWindowCollectionBehavior::FullScreenAuxiliary;
        ns_window.setCollectionBehavior(behavior);
        ns_window.setHasShadow(true);
        ns_window.setOpaque(false);
        ns_window.setBackgroundColor(Some(&NSColor::clearColor()));
    }
}

#[cfg(not(target_os = "macos"))]
pub(crate) fn activate_app_for_popover<R: tauri::Runtime>(
    window: &WebviewWindow<R>,
    popover: &crate::PopoverState,
) {
    popover_trace("Activation: start (non-macOS)");
    ensure_popover_window_configured(window);
    let _ = window.set_always_on_top(true);
    popover_trace("Window.show()");
    let _ = window.show();
    popover_trace("Window.focus()");
    let _ = window.set_focus();
    popover.mark_opening();
    popover_trace("show: grace period started (mark_opening after Window.show)");
}

#[cfg(not(target_os = "macos"))]
pub fn configure_popover_window<R: tauri::Runtime>(_window: &WebviewWindow<R>) {}