//! Dev-only popover tracing. Release builds compile these to no-ops.

#[cfg(debug_assertions)]
use std::fs::OpenOptions;
#[cfg(debug_assertions)]
use std::io::Write;
#[cfg(debug_assertions)]
use std::sync::Mutex;

#[cfg(debug_assertions)]
pub const TRACE_FILE: &str = "/tmp/parsedock-popover-trace.log";

#[cfg(debug_assertions)]
static TRACE_LOCK: Mutex<()> = Mutex::new(());

#[cfg(debug_assertions)]
fn write_trace(line: &str) {
    eprintln!("{line}");
    if let Ok(_guard) = TRACE_LOCK.lock() {
        if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(TRACE_FILE) {
            let _ = writeln!(file, "{line}");
            let _ = file.flush();
        }
    }
}

#[cfg(debug_assertions)]
pub fn trace(step: &str) {
    write_trace(&format!("[PopoverManager] {step}"));
}

#[cfg(not(debug_assertions))]
pub fn trace(_step: &str) {}

#[cfg(debug_assertions)]
pub fn trace_fmt(args: std::fmt::Arguments<'_>) {
    write_trace(&format!("[PopoverManager] {args}"));
}

#[cfg(not(debug_assertions))]
pub fn trace_fmt(_args: std::fmt::Arguments<'_>) {}

#[cfg(debug_assertions)]
pub fn tray_guard_trace(step: &str) {
    write_trace(&format!("[TrayGuard] {step}"));
}

#[cfg(not(debug_assertions))]
pub fn tray_guard_trace(_step: &str) {}

#[cfg(debug_assertions)]
pub fn startup_trace(step: &str) {
    write_trace(&format!("[ParseDock] {step}"));
}

#[cfg(not(debug_assertions))]
pub fn startup_trace(_step: &str) {}

#[macro_export]
macro_rules! popover_trace {
    ($msg:expr) => {
        $crate::popover_trace::trace($msg);
    };
    ($fmt:expr, $($arg:tt)*) => {
        $crate::popover_trace::trace_fmt(format_args!($fmt, $($arg)*));
    };
}