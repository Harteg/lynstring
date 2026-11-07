use std::path::Path;
use std::fs;
use tauri::AppHandle;
use crate::commands::strings::parse_strings_xml;
use crate::commands::snapshot::{
    ProjectSnapshot, detect_changes, filter_auto_undo,
    create_snapshot_from_strings, get_project_snapshots,
    load_snapshot, save_snapshot
};
use crate::commands::events::{emit_default_strings_changed, emit_new_default_strings_detected};
use crate::commands::auto_translate::get_auto_translate_state;
use std::collections::HashSet;

/// Check for changes in default strings.xml and emit event if found
/// Returns true if changes were detected and emitted, false otherwise
pub fn check_for_default_strings_changes(
    app: &AppHandle,
    default_strings_path: &Path,
    project_path: &str,
) -> bool {
    // project_path is already canonicalized when passed from emit_file_changes
    // Read and parse the default strings.xml
    let xml_content = match fs::read_to_string(default_strings_path) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Failed to read default strings.xml: {}", e);
            return false;
        }
    };
    
    let strings = match parse_strings_xml(&xml_content) {
        Ok(strings) => strings,
        Err(e) => {
            eprintln!("Failed to parse default strings.xml: {}", e);
            return false;
        }
    };
    
    // Get snapshot for this project
    let snapshots = get_project_snapshots();
    let snapshot: ProjectSnapshot = {
        let snapshots_guard = snapshots.lock().unwrap();
        snapshots_guard.get(project_path).cloned().unwrap_or_else(|| {
            // Not in memory, try to load from disk
            drop(snapshots_guard); // Release lock before disk I/O
            load_snapshot(app, project_path)
        })
    };
    
    // If snapshot is empty, create initial snapshot and don't emit changes
    // This happens on first load when no snapshot exists yet
    if snapshot.strings.is_empty() {
        let new_snapshot = create_snapshot_from_strings(&strings);
        let mut snapshots_guard = snapshots.lock().unwrap();
        snapshots_guard.insert(project_path.to_string(), new_snapshot.clone());
        drop(snapshots_guard); // Release lock before disk I/O
        // Save to disk
        if let Err(e) = save_snapshot(app, project_path, &new_snapshot) {
            eprintln!("Failed to save initial snapshot: {}", e);
        }
        return false; // Don't emit changes on initial snapshot creation
    }
    
    // Detect changes (hash-based comparison)
    // Use the baseline snapshot (loaded at start) for change detection
    // This ensures we compare against the "original" state before any updates
    eprintln!("[DEBUG] check_for_default_strings_changes: Checking for changes in {}", project_path);
    eprintln!("[DEBUG] Baseline snapshot has {} strings", snapshot.strings.len());
    
    // Detect changes against the baseline snapshot
    let changes = detect_changes(&strings, &snapshot);
    
    eprintln!("[DEBUG] Detected changes: {} new, {} modified, {} deleted", 
        changes.new.len(), changes.modified.len(), changes.deleted.len());
    
    // Filter out changes that have been reverted (auto-undo)
    let filtered_changes = filter_auto_undo(&changes, &snapshot);
    
    // ALWAYS emit an event when the file changes, even if no changes are detected
    // This is critical: when a string is reverted, detect_changes returns 0 changes,
    // but we still need to notify the frontend so it can remove strings that are no longer changed
    // The file watcher only calls this function when the file actually changes, so it's safe to always emit
    eprintln!("[DEBUG] Emitting changes: {} new, {} modified, {} deleted", 
        filtered_changes.new.len(), filtered_changes.modified.len(), filtered_changes.deleted.len());
    
    // IMPORTANT: Do NOT update snapshot here - only update when user acknowledges!
    // The snapshot remains as the baseline reference until user clicks Ignore/Update
    // This allows changes to accumulate across multiple saves
    emit_default_strings_changed(app, &filtered_changes);
    
    // Also emit legacy event for auto-translate (only new strings)
    if !filtered_changes.new.is_empty() {
        let auto_translate_state = get_auto_translate_state();
        let mut state = auto_translate_state.lock().unwrap();
        
        if state.enabled {
            // Update tracked set with all current string names
            let current_string_names: HashSet<String> = strings
                .iter()
                .filter(|s| s.is_translatable)
                .map(|s| s.name.clone())
                .collect();
            
            state.tracked_string_names = current_string_names;
            
            // Emit legacy event for backward compatibility
            emit_new_default_strings_detected(app, &filtered_changes.new);
        }
    }
    
    true // Event was emitted
}

