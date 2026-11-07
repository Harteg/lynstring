use crate::commands::snapshot::{ProjectSnapshot, compute_string_hash, canonicalize_text};
use crate::models::AndroidString;

// Change detection result
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct DefaultStringsChanges {
    pub new: Vec<String>, // New string names
    pub modified: Vec<ModifiedString>, // Modified strings with old/new values
    pub deleted: Vec<String>, // Deleted string names
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct ModifiedString {
    pub name: String,
    pub old_value: String,
    pub new_value: String,
    pub translatable_changed: bool, // True if is_translatable changed
}

/// Compare current strings with snapshot and detect changes (hash-based)
pub fn detect_changes(
    current_strings: &[AndroidString],
    snapshot: &ProjectSnapshot,
) -> DefaultStringsChanges {
    let mut new = Vec::new();
    let mut modified = Vec::new();
    let mut deleted = Vec::new();
    
    // Find new and modified strings
    for current_string in current_strings {
        let current_hash = compute_string_hash(&current_string.name, &current_string.text);
        
        match snapshot.strings.get(&current_string.name) {
            None => {
                // New string - not in snapshot
                new.push(current_string.name.clone());
            }
            Some(entry) => {
                let hash_changed = current_hash != entry.hash;
                let translatable_changed = current_string.is_translatable != entry.is_translatable;
                
                if hash_changed || translatable_changed {
                    modified.push(ModifiedString {
                        name: current_string.name.clone(),
                        old_value: entry.text.clone(),
                        new_value: current_string.text.clone(),
                        translatable_changed,
                    });
                }
            }
        }
    }
    
    // Find deleted strings (only if not ignored)
    for (name, entry) in &snapshot.strings {
        if !current_strings.iter().any(|s| s.name == *name) && !entry.ignored {
            deleted.push(name.clone());
        }
    }
    
    DefaultStringsChanges {
        new,
        modified,
        deleted,
    }
}

/// Filter out changes that have been reverted (auto-undo)
/// Compares current value against original_hash from baseline snapshot to detect reverts
pub fn filter_auto_undo(
    changes: &DefaultStringsChanges,
    baseline_snapshot: &ProjectSnapshot,
) -> DefaultStringsChanges {
    let mut filtered = DefaultStringsChanges {
        new: changes.new.clone(),
        modified: Vec::new(),
        deleted: changes.deleted.clone(),
    };

    // Filter out modified strings that have been reverted to original
    for modified in &changes.modified {
        if let Some(entry) = baseline_snapshot.strings.get(&modified.name) {
            let current_hash = compute_string_hash(&modified.name, &modified.new_value);
            
            // Compare against original_hash if available, otherwise use current hash
            // If original_hash exists and matches, the string was reverted to its original value
            let original_hash = entry.original_hash.as_ref().unwrap_or(&entry.hash);
            
            // Also compare canonicalized text directly as a fallback (in case of hash mismatches due to edge cases)
            let current_canonical = canonicalize_text(&modified.new_value);
            let original_canonical = canonicalize_text(&entry.text);
            
            if current_hash == *original_hash || current_canonical == original_canonical {
                continue; // Skip - it's been reverted to original
            }
        }

        filtered.modified.push(modified.clone());
    }

    filtered
}

