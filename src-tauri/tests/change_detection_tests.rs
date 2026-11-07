mod test_config;
mod helpers;

use test_config::TestConfig;
use helpers::*;
use lynstring_tauri_lib::models::AndroidString;
use lynstring_tauri_lib::commands::snapshot::{
    detect_changes, create_snapshot_from_strings, filter_auto_undo,
    compute_string_hash
};

macro_rules! skip_if_disabled {
    ($config:expr, $test_name:literal) => {
        if !$config {
            eprintln!("⏭️  [SKIP] {}", $test_name);
            return;
        }
    };
}

// Use the real functions from snapshot module - no need for custom helpers

#[test]
fn test_auto_undo_single_string() {
    let config = TestConfig::load();
    skip_if_disabled!(config.integration.change_detection, "test_auto_undo_single_string");

    // Create initial strings
    let initial_strings = vec![
        AndroidString {
            name: "test_string".to_string(),
            text: "Original value".to_string(),
            is_translatable: true,
            is_commented_out: false,
            line_number: 1,
            has_cdata: false,
        },
    ];

    // Create baseline snapshot
    let baseline_snapshot = create_snapshot_from_strings(&initial_strings);

    // Change the string
    let changed_strings = vec![
        AndroidString {
            name: "test_string".to_string(),
            text: "Modified value".to_string(),
            is_translatable: true,
            is_commented_out: false,
            line_number: 1,
            has_cdata: false,
        },
    ];

    let changes = detect_changes(&changed_strings, &baseline_snapshot);
    assert_eq!(changes.modified.len(), 1, "Should detect one modified string");
    assert_eq!(changes.modified[0].name, "test_string");
    assert_eq!(changes.modified[0].old_value, "Original value");
    assert_eq!(changes.modified[0].new_value, "Modified value");

    // Change it back to original
    let reverted_strings = vec![
        AndroidString {
            name: "test_string".to_string(),
            text: "Original value".to_string(),
            is_translatable: true,
            is_commented_out: false,
            line_number: 1,
            has_cdata: false,
        },
    ];

    let changes_after_revert = detect_changes(&reverted_strings, &baseline_snapshot);
    // When reverted to original, detect_changes should return 0 changes (hash matches baseline)
    assert_eq!(changes_after_revert.modified.len(), 0, "Should detect 0 changes when reverted to original");

    // Apply auto-undo filter (should still be 0)
    let filtered = filter_auto_undo(&changes_after_revert, &baseline_snapshot);
    assert_eq!(filtered.modified.len(), 0, "Auto-undo should keep 0 changes when reverted");
    assert_eq!(filtered.new.len(), 0);
    assert_eq!(filtered.deleted.len(), 0);
}

#[test]
fn test_multiple_string_changes() {
    let config = TestConfig::load();
    skip_if_disabled!(config.integration.change_detection, "test_multiple_string_changes");

    // Create initial strings
    let initial_strings = vec![
        AndroidString {
            name: "string1".to_string(),
            text: "Value 1".to_string(),
            is_translatable: true,
            is_commented_out: false,
            line_number: 1,
            has_cdata: false,
        },
        AndroidString {
            name: "string2".to_string(),
            text: "Value 2".to_string(),
            is_translatable: true,
            is_commented_out: false,
            line_number: 1,
            has_cdata: false,
        },
        AndroidString {
            name: "string3".to_string(),
            text: "Value 3".to_string(),
            is_translatable: true,
            is_commented_out: false,
            line_number: 1,
            has_cdata: false,
        },
    ];

    // Create baseline snapshot
    let baseline_snapshot = create_snapshot_from_strings(&initial_strings);

    // Change multiple strings
    let changed_strings = vec![
        AndroidString {
            name: "string1".to_string(),
            text: "Modified Value 1".to_string(),
            is_translatable: true,
            is_commented_out: false,
            line_number: 1,
            has_cdata: false,
        },
        AndroidString {
            name: "string2".to_string(),
            text: "Modified Value 2".to_string(),
            is_translatable: true,
            is_commented_out: false,
            line_number: 1,
            has_cdata: false,
        },
        AndroidString {
            name: "string3".to_string(),
            text: "Value 3".to_string(), // Unchanged
            is_translatable: true,
            is_commented_out: false,
            line_number: 1,
            has_cdata: false,
        },
    ];

    let changes = detect_changes(&changed_strings, &baseline_snapshot);
    assert_eq!(changes.modified.len(), 2, "Should detect 2 modified strings");
    assert_eq!(changes.new.len(), 0, "Should have no new strings");
    assert_eq!(changes.deleted.len(), 0, "Should have no deleted strings");

    // Verify the modified strings
    let modified_names: Vec<&String> = changes.modified.iter().map(|m| &m.name).collect();
    assert!(modified_names.contains(&&"string1".to_string()));
    assert!(modified_names.contains(&&"string2".to_string()));
    assert!(!modified_names.contains(&&"string3".to_string()));
}

#[test]
fn test_new_string_detection() {
    let config = TestConfig::load();
    skip_if_disabled!(config.integration.change_detection, "test_new_string_detection");

    // Create initial strings
    let initial_strings = vec![
        AndroidString {
            name: "existing_string".to_string(),
            text: "Existing value".to_string(),
            is_translatable: true,
            is_commented_out: false,
            line_number: 1,
            has_cdata: false,
        },
    ];

    // Create baseline snapshot
    let baseline_snapshot = create_snapshot_from_strings(&initial_strings);

    // Add new strings
    let updated_strings = vec![
        AndroidString {
            name: "existing_string".to_string(),
            text: "Existing value".to_string(),
            is_translatable: true,
            is_commented_out: false,
            line_number: 1,
            has_cdata: false,
        },
        AndroidString {
            name: "new_string1".to_string(),
            text: "New value 1".to_string(),
            is_translatable: true,
            is_commented_out: false,
            line_number: 1,
            has_cdata: false,
        },
        AndroidString {
            name: "new_string2".to_string(),
            text: "New value 2".to_string(),
            is_translatable: true,
            is_commented_out: false,
            line_number: 1,
            has_cdata: false,
        },
    ];

    let changes = detect_changes(&updated_strings, &baseline_snapshot);
    assert_eq!(changes.new.len(), 2, "Should detect 2 new strings");
    assert_eq!(changes.modified.len(), 0, "Should have no modified strings");
    assert_eq!(changes.deleted.len(), 0, "Should have no deleted strings");

    // Verify new string names
    assert!(changes.new.contains(&"new_string1".to_string()));
    assert!(changes.new.contains(&"new_string2".to_string()));
}

#[test]
fn test_deleted_string_detection() {
    let config = TestConfig::load();
    skip_if_disabled!(config.integration.change_detection, "test_deleted_string_detection");

    // Create initial strings
    let initial_strings = vec![
        AndroidString {
            name: "string1".to_string(),
            text: "Value 1".to_string(),
            is_translatable: true,
            is_commented_out: false,
            line_number: 1,
            has_cdata: false,
        },
        AndroidString {
            name: "string2".to_string(),
            text: "Value 2".to_string(),
            is_translatable: true,
            is_commented_out: false,
            line_number: 1,
            has_cdata: false,
        },
        AndroidString {
            name: "string3".to_string(),
            text: "Value 3".to_string(),
            is_translatable: true,
            is_commented_out: false,
            line_number: 1,
            has_cdata: false,
        },
    ];

    // Create baseline snapshot
    let baseline_snapshot = create_snapshot_from_strings(&initial_strings);

    // Remove some strings
    let updated_strings = vec![
        AndroidString {
            name: "string1".to_string(),
            text: "Value 1".to_string(),
            is_translatable: true,
            is_commented_out: false,
            line_number: 1,
            has_cdata: false,
        },
        // string2 and string3 are deleted
    ];

    let changes = detect_changes(&updated_strings, &baseline_snapshot);
    assert_eq!(changes.deleted.len(), 2, "Should detect 2 deleted strings");
    assert_eq!(changes.new.len(), 0, "Should have no new strings");
    assert_eq!(changes.modified.len(), 0, "Should have no modified strings");

    // Verify deleted string names
    assert!(changes.deleted.contains(&"string2".to_string()));
    assert!(changes.deleted.contains(&"string3".to_string()));
}

#[test]
fn test_auto_undo_multiple_strings() {
    let config = TestConfig::load();
    skip_if_disabled!(config.integration.change_detection, "test_auto_undo_multiple_strings");

    // Create initial strings
    let initial_strings = vec![
        AndroidString {
            name: "string1".to_string(),
            text: "Original 1".to_string(),
            is_translatable: true,
            is_commented_out: false,
            line_number: 1,
            has_cdata: false,
        },
        AndroidString {
            name: "string2".to_string(),
            text: "Original 2".to_string(),
            is_translatable: true,
            is_commented_out: false,
            line_number: 1,
            has_cdata: false,
        },
    ];

    // Create baseline snapshot
    let baseline_snapshot = create_snapshot_from_strings(&initial_strings);

    // Change both strings
    let changed_strings = vec![
        AndroidString {
            name: "string1".to_string(),
            text: "Modified 1".to_string(),
            is_translatable: true,
            is_commented_out: false,
            line_number: 1,
            has_cdata: false,
        },
        AndroidString {
            name: "string2".to_string(),
            text: "Modified 2".to_string(),
            is_translatable: true,
            is_commented_out: false,
            line_number: 1,
            has_cdata: false,
        },
    ];

    let changes = detect_changes(&changed_strings, &baseline_snapshot);
    assert_eq!(changes.modified.len(), 2, "Should detect 2 modified strings");

    // Revert string1 but keep string2 modified
    let partially_reverted = vec![
        AndroidString {
            name: "string1".to_string(),
            text: "Original 1".to_string(), // Reverted
            is_translatable: true,
            is_commented_out: false,
            line_number: 1,
            has_cdata: false,
        },
        AndroidString {
            name: "string2".to_string(),
            text: "Modified 2".to_string(), // Still modified
            is_translatable: true,
            is_commented_out: false,
            line_number: 1,
            has_cdata: false,
        },
    ];

    let changes_after_revert = detect_changes(&partially_reverted, &baseline_snapshot);
    // string1 is reverted (hash matches baseline), string2 is still modified
    assert_eq!(changes_after_revert.modified.len(), 1, "Should detect 1 change (string2), string1 matches baseline");

    // Apply auto-undo filter (should still be 1, since string1 wasn't detected as changed)
    let filtered = filter_auto_undo(&changes_after_revert, &baseline_snapshot);
    assert_eq!(filtered.modified.len(), 1, "Auto-undo should keep string2 (string1 already filtered by detect_changes)");
    assert_eq!(filtered.modified[0].name, "string2", "Should only have string2 in modified");
}

#[test]
fn test_real_world_strings_xml() {
    let config = TestConfig::load();
    skip_if_disabled!(config.integration.change_detection, "test_real_world_strings_xml");

    // Use the real strings.xml file provided by the user
    let strings_path = "/Users/jakobharteg/GitHub/crookcatcher-android/CrookCatcher/src/main/res/values/strings.xml";
    
    let strings = match load_strings_xml(&std::path::Path::new(strings_path)) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("⏭️  [SKIP] test_real_world_strings_xml - Failed to load strings.xml: {}", e);
            return;
        }
    };

    assert!(!strings.is_empty(), "Should load strings from real file");

    // Create baseline snapshot
    let baseline_snapshot = create_snapshot_from_strings(&strings);

    // Modify a few strings
    let mut modified_strings = strings.clone();
    for s in &mut modified_strings {
        if s.name == "app_name" {
            s.text = "Modified App Name".to_string();
        } else if s.name == "action_settings" {
            s.text = "Modified Settings".to_string();
        }
    }

    let changes = detect_changes(&modified_strings, &baseline_snapshot);
    
    // Should detect at least the strings we modified
    let modified_names: Vec<&String> = changes.modified.iter().map(|m| &m.name).collect();
    assert!(modified_names.contains(&&"app_name".to_string()) || 
            modified_names.contains(&&"action_settings".to_string()),
            "Should detect at least one of the modified strings");

    // Revert one of them
    let mut reverted_strings = modified_strings.clone();
    for s in &mut reverted_strings {
        if s.name == "app_name" {
            // Find original value
            if let Some(original) = strings.iter().find(|x| x.name == "app_name") {
                s.text = original.text.clone();
            }
        }
    }

    let changes_after_revert = detect_changes(&reverted_strings, &baseline_snapshot);
    let filtered = filter_auto_undo(&changes_after_revert, &baseline_snapshot);

    // If app_name was reverted, it should be filtered out
    let filtered_names: Vec<&String> = filtered.modified.iter().map(|m| &m.name).collect();
    if let Some(original) = strings.iter().find(|x| x.name == "app_name") {
        let reverted_hash = compute_string_hash("app_name", &original.text);
        let baseline_hash = baseline_snapshot.strings.get("app_name")
            .map(|e| e.hash.clone())
            .unwrap_or_default();
        
        if reverted_hash == baseline_hash {
            assert!(!filtered_names.contains(&&"app_name".to_string()),
                    "Reverted app_name should be filtered out by auto-undo");
        }
    }
}
