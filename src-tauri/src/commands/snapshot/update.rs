use chrono::Utc;
use crate::commands::snapshot::{
    ProjectSnapshot, StringSnapshotEntry, compute_string_hash
};
use crate::models::AndroidString;

/// Create snapshot from AndroidString list (computes hashes)
pub fn create_snapshot_from_strings(strings: &[AndroidString]) -> ProjectSnapshot {
    let mut snapshot_entries = std::collections::HashMap::new();
    let now = Utc::now().to_rfc3339();
    
    for s in strings {
        let hash = compute_string_hash(&s.name, &s.text);
        snapshot_entries.insert(
            s.name.clone(),
            StringSnapshotEntry {
                hash: hash.clone(),
                text: s.text.clone(), // Store original text for diff display
                original_hash: Some(hash.clone()), // Store original hash for auto-undo
                is_translatable: s.is_translatable,
                last_changed: Some(now.clone()),
                ignored: false,
            },
        );
    }
    
    ProjectSnapshot {
        version: 1,
        default_locale: "en".to_string(),
        strings: snapshot_entries,
    }
}

/// Update snapshot with current strings state (called when user acknowledges changes)
/// This updates the snapshot to the current state, making it the new baseline
pub fn update_snapshot_with_current_strings(
    snapshot: &ProjectSnapshot,
    current_strings: &[AndroidString],
    ignored_string_names: &[String],
) -> ProjectSnapshot {
    let mut updated = snapshot.clone();
    let now = Utc::now().to_rfc3339();
    let ignored_set: std::collections::HashSet<&String> = ignored_string_names.iter().collect();
    
    // Update entries for all current strings
    for current_string in current_strings {
        let hash = compute_string_hash(&current_string.name, &current_string.text);
        let was_ignored = updated.strings
            .get(&current_string.name)
            .map(|e| e.ignored)
            .unwrap_or(false);
        
        let should_be_ignored = ignored_set.contains(&current_string.name);
        
        // If hash changed and was ignored, reset ignore flag
        let should_reset_ignore = updated.strings
            .get(&current_string.name)
            .map(|e| e.ignored && e.hash != hash)
            .unwrap_or(false);
        
        // When acknowledging, update original_hash to current hash (new baseline)
        // This makes the current state the new reference point for future change detection
        // Whether ignoring or updating, we update the baseline to current state
        updated.strings.insert(
            current_string.name.clone(),
            StringSnapshotEntry {
                hash: hash.clone(),
                text: current_string.text.clone(),
                original_hash: Some(hash.clone()),
                is_translatable: current_string.is_translatable,
                last_changed: Some(now.clone()),
                ignored: (should_be_ignored || was_ignored) && !should_reset_ignore,
            },
        );
    }
    
    updated
}

/// Add only strings that are not already in the snapshot (e.g. after translating new strings)
/// Preserves existing snapshot entries so modified strings remain detectable
pub fn add_new_strings_to_snapshot(
    snapshot: &ProjectSnapshot,
    current_strings: &[AndroidString],
) -> ProjectSnapshot {
    let mut updated = snapshot.clone();
    let now = Utc::now().to_rfc3339();

    for current_string in current_strings {
        if updated.strings.contains_key(&current_string.name) {
            continue; // Skip - already in snapshot, preserve existing state
        }
        let hash = compute_string_hash(&current_string.name, &current_string.text);
        updated.strings.insert(
            current_string.name.clone(),
            StringSnapshotEntry {
                hash: hash.clone(),
                text: current_string.text.clone(),
                original_hash: Some(hash.clone()),
                is_translatable: current_string.is_translatable,
                last_changed: Some(now.clone()),
                ignored: false,
            },
        );
    }

    updated
}

