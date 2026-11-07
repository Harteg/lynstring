use std::collections::HashMap;
use serde::{Deserialize, Serialize};

// Hash-based snapshot entry for change detection
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StringSnapshotEntry {
    pub hash: String,              // SHA256 hash of key + canonicalized text
    pub text: String,              // Current text (for showing diffs)
    pub original_hash: Option<String>, // Original hash before any changes (for auto-undo)
    pub is_translatable: bool,     // translatable attribute
    pub last_changed: Option<String>, // ISO 8601 timestamp when last changed
    pub ignored: bool,             // User has ignored this string
}

// Project snapshot: maps string_name -> StringSnapshotEntry
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq))]
pub struct ProjectSnapshot {
    pub version: u32,
    pub default_locale: String,
    pub strings: HashMap<String, StringSnapshotEntry>,
}

// Project snapshot state (always active when project is watched)
// Maps project_path -> ProjectSnapshot
pub type ProjectSnapshotState = std::sync::Arc<std::sync::Mutex<HashMap<String, ProjectSnapshot>>>;
static PROJECT_SNAPSHOTS: std::sync::OnceLock<ProjectSnapshotState> = std::sync::OnceLock::new();

pub fn get_project_snapshots() -> &'static ProjectSnapshotState {
    PROJECT_SNAPSHOTS.get_or_init(|| std::sync::Arc::new(std::sync::Mutex::new(HashMap::new())))
}

pub fn create_empty_snapshot() -> ProjectSnapshot {
    ProjectSnapshot {
        version: 1,
        default_locale: "en".to_string(),
        strings: HashMap::new(),
    }
}

