//! Temporary parity-audit tracing. Logs use PopoverManager step names so dev
//! console output can be compared to the expected click→show sequence.
//! Remove or gate behind a feature flag after the audit is complete.

use std::fs::OpenOptions;
use std::io::Write;
use std::sync::Mutex;

pub const TRACE_FILE: &str = "/tmp/parsedock-popover-trace.log";

static TRACE_LOCK: Mutex<()> = Mutex::new(());

fn write_trace(line: &str) {
    eprintln!("{line}");
    if let Ok(_guard) = TRACE_LOCK.lock() {
        if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(TRACE_FILE) {
            let _ = writeln!(file, "{line}");
            let _ = file.flush();
        }
    }
}

pub fn trace(step: &str) {
    write_trace(&format!("[PopoverManager] {step}"));
}

pub fn trace_fmt(args: std::fmt::Arguments<'_>) {
    write_trace(&format!("[PopoverManager] {args}"));
}

pub fn tray_guard_trace(step: &str) {
    write_trace(&format!("[TrayGuard] {step}"));
}

pub fn startup_trace(step: &str) {
    write_trace(&format!("[ParseDock] {step}"));
}

#[macro_export]
macro_rules! popover_trace {
    ($msg:expr) => {
        $crate::popover_trace::trace($msg);
    };
    ($fmt:expr, $($arg:tt)*) => {
        $crate::popover_trace::trace_fmt(format_args!($fmt, $($arg)*));
    };
}