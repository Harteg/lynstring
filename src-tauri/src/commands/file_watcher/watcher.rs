use notify::{RecommendedWatcher};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::mpsc;

// Global state to track active watchers
type WatcherState = Arc<Mutex<Option<RecommendedWatcher>>>;
static WATCHER: std::sync::OnceLock<WatcherState> = std::sync::OnceLock::new();

// Channel sender for file change events
type FileChangeSender = Arc<Mutex<Option<mpsc::UnboundedSender<PathBuf>>>>;
static FILE_CHANGE_SENDER: std::sync::OnceLock<FileChangeSender> = std::sync::OnceLock::new();

// Flag to stop the polling task
type PollingFlag = Arc<AtomicBool>;
static POLLING_ACTIVE: std::sync::OnceLock<PollingFlag> = std::sync::OnceLock::new();

pub fn get_watcher_state() -> &'static WatcherState {
    WATCHER.get_or_init(|| Arc::new(Mutex::new(None)))
}

pub fn get_file_change_sender() -> &'static FileChangeSender {
    FILE_CHANGE_SENDER.get_or_init(|| Arc::new(Mutex::new(None)))
}

pub fn get_polling_flag() -> &'static PollingFlag {
    POLLING_ACTIVE.get_or_init(|| Arc::new(AtomicBool::new(false)))
}

pub fn start_polling() {
    get_polling_flag().store(true, Ordering::SeqCst);
}

pub fn stop_polling() {
    get_polling_flag().store(false, Ordering::SeqCst);
}

pub fn is_polling_active() -> bool {
    get_polling_flag().load(Ordering::SeqCst)
}

/// Check if a path is a strings.xml file we should watch
pub fn is_strings_xml_file(path: &Path) -> bool {
    if let Some(file_name) = path.file_name() {
        if file_name == "strings.xml" {
            // Check if it's in a values or values-* directory
            if let Some(parent) = path.parent() {
                if let Some(dir_name) = parent.file_name() {
                    let dir_str = dir_name.to_string_lossy();
                    return dir_str == "values" || dir_str.starts_with("values-");
                }
            }
        }
    }
    false
}

/// Check if a path is the default strings.xml (in values/ directory)
pub fn is_default_strings_xml(path: &Path) -> bool {
    if let Some(file_name) = path.file_name() {
        if file_name == "strings.xml" {
            if let Some(parent) = path.parent() {
                if let Some(dir_name) = parent.file_name() {
                    return dir_name == "values";
                }
            }
        }
    }
    false
}

/// Check if a path is a language folder (values-* directory)
/// Note: This checks the path name, not whether it exists (useful for deleted folders)
pub fn is_language_folder(path: &Path) -> bool {
    if let Some(dir_name) = path.file_name() {
        let dir_str = dir_name.to_string_lossy();
        return dir_str.starts_with("values-") && dir_str != "values";
    }
    false
}

/// Check if a path is inside a language folder (values-* directory)
/// This handles cases where files inside the folder are deleted
pub fn is_in_language_folder(path: &Path) -> bool {
    let mut current = path;
    loop {
        if let Some(parent) = current.parent() {
            if is_language_folder(parent) {
                return true;
            }
            current = parent;
        } else {
            break;
        }
    }
    false
}

/// Handle file change events - sends to channel for processing in tokio runtime
pub fn handle_file_change(changed_path: PathBuf) {
    let sender = get_file_change_sender();
    if let Some(sender) = sender.lock().unwrap().as_ref() {
        let _ = sender.send(changed_path);
    }
}

