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
    if WINDOW_CONFIGURED.load(Ordering::SeqCst) {
        popover_trace("Activation: NSWindow already configured");
        return;
    }
    popover_trace("Activation: configure_popover_window (first open)");
    if window.ns_window().is_ok() {
        configure_popover_window(window);
        WINDOW_CONFIGURED.store(true, Ordering::SeqCst);
    } else {
        popover_trace("Activation: configure deferred (ns_window unavailable)");
    }
}

/// Raise the app and popover window so a borderless panel is actually visible.
#[cfg(target_os = "macos")]
pub(crate) fn activate_app_for_popover<R: tauri::Runtime>(
    window: &WebviewWindow<R>,
    _popover: &crate::PopoverState,
) {
    popover_trace("Activation: start");
    ensure_popover_window_configured(window);
    let Some(mtm) = MainThreadMarker::new() else {
        popover_trace("Activation: fallback (not main thread) — no NSApplication.activate");
        popover_trace("Window.show()");
        let _ = window.show();
        popover_trace("Window.focus()");
        let _ = window.set_focus();
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
    popover_trace("Activation: complete");
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
}

#[cfg(not(target_os = "macos"))]
pub fn configure_popover_window<R: tauri::Runtime>(_window: &WebviewWindow<R>) {}

static HUD_WINDOW_CONFIGURED: AtomicBool = AtomicBool::new(false);

/// Apply NSWindow settings for the floating progress HUD (non-activating, all spaces).
pub fn ensure_hud_window_configured<R: tauri::Runtime>(window: &WebviewWindow<R>) {
    if HUD_WINDOW_CONFIGURED.load(Ordering::SeqCst) {
        return;
    }
    if window.ns_window().is_ok() {
        configure_hud_window(window);
        HUD_WINDOW_CONFIGURED.store(true, Ordering::SeqCst);
    }
}

#[cfg(target_os = "macos")]
pub fn configure_hud_window<R: tauri::Runtime>(window: &WebviewWindow<R>) {
    let Ok(ns_ptr) = window.ns_window() else {
        return;
    };
    unsafe {
        let ns_window: &NSWindow = &*ns_ptr.cast();
        ns_window.setLevel(NSFloatingWindowLevel);
        let behavior = NSWindowCollectionBehavior::CanJoinAllSpaces
            | NSWindowCollectionBehavior::FullScreenAuxiliary;
        ns_window.setCollectionBehavior(behavior);
        ns_window.setHasShadow(true);
        ns_window.setOpaque(false);
        ns_window.setBackgroundColor(Some(&NSColor::clearColor()));
    }
}

#[cfg(not(target_os = "macos"))]
pub fn configure_hud_window<R: tauri::Runtime>(_window: &WebviewWindow<R>) {}