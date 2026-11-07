use tauri::{AppHandle, Emitter, Manager};
use crate::commands::snapshot::DefaultStringsChanges;

/// Emit default strings changed event
pub fn emit_default_strings_changed(app: &AppHandle, changes: &DefaultStringsChanges) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.emit("default-strings-changed", changes);
    }
}

/// Emit new default strings detected event (for auto-translate)
pub fn emit_new_default_strings_detected(app: &AppHandle, new_strings: &[String]) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.emit("new-default-strings-detected", new_strings);
    }
}

/// Emit strings file changed event
pub fn emit_strings_file_changed(app: &AppHandle, paths: &[String]) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.emit("strings-file-changed", paths);
    }
}

