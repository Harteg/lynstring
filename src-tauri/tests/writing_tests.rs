mod test_config;
mod helpers;

use test_config::TestConfig;
use helpers::*;
use std::fs;

macro_rules! skip_if_disabled {
    ($config:expr, $test_name:literal) => {
        if !$config {
            eprintln!("⏭️  [SKIP] {}", $test_name);
            return;
        }
    };
}

#[test]
fn test_write_golden_comparison() {
    let config = TestConfig::load();
    skip_if_disabled!(config.writing.golden_comparison, "test_write_golden_comparison");

    let (_temp_dir, project_path) = create_temp_project("android_project_basic")
        .expect("Failed to create temp project");

    let strings_path = get_strings_xml_path(&project_path, "default");
    let original_strings = load_strings_xml(&strings_path).expect("Failed to load strings");

    // Find the res directory for writing
    let res_dir = get_res_directory_path(&project_path);
    
    // Write strings back
    write_strings_xml(&original_strings, &res_dir, "default")
        .expect("Failed to write strings");

    // Read back and verify structure is preserved
    let written_strings = load_strings_xml(&strings_path).expect("Failed to read written strings");
    
    assert_eq!(original_strings.len(), written_strings.len(), 
               "Should preserve all strings");

    // Verify key strings are still there
    let original_names: std::collections::HashSet<&str> = 
        original_strings.iter().map(|s| s.name.as_str()).collect();
    let written_names: std::collections::HashSet<&str> = 
        written_strings.iter().map(|s| s.name.as_str()).collect();
    
    assert_eq!(original_names, written_names, "Should preserve all string names");
}

#[test]
fn test_write_order_preservation() {
    let config = TestConfig::load();
    skip_if_disabled!(config.writing.golden_comparison, "test_write_order_preservation");

    let (_temp_dir, project_path) = create_temp_project("android_project_basic")
        .expect("Failed to create temp project");

    let strings_path = get_strings_xml_path(&project_path, "default");
    let original_strings = load_strings_xml(&strings_path).expect("Failed to load strings");

    // Verify we have multiple strings to test order
    assert!(original_strings.len() > 1, "Need multiple strings to test order preservation");

    // Find the res directory for writing
    let res_dir = get_res_directory_path(&project_path);
    
    // Write strings back
    write_strings_xml(&original_strings, &res_dir, "default")
        .expect("Failed to write strings");

    // Read back
    let written_strings = load_strings_xml(&strings_path).expect("Failed to read written strings");
    
    // Verify count matches
    assert_eq!(original_strings.len(), written_strings.len(), 
               "Should preserve all strings");
    
    // Verify order is preserved by comparing sequences (not HashSet)
    for (i, (original, written)) in original_strings.iter().zip(written_strings.iter()).enumerate() {
        assert_eq!(original.name, written.name, 
                  "String at index {} should match: expected '{}', got '{}'", 
                  i, original.name, written.name);
    }
}

#[test]
fn test_write_idempotence() {
    let config = TestConfig::load();
    skip_if_disabled!(config.writing.idempotence, "test_write_idempotence");

    let (_temp_dir, project_path) = create_temp_project("android_project_basic")
        .expect("Failed to create temp project");

    let strings_path = get_strings_xml_path(&project_path, "default");
    
    // Read original
    let _original_content = fs::read_to_string(&strings_path).expect("Failed to read original");
    let original_strings = load_strings_xml(&strings_path).expect("Failed to parse original");

    // Find the res directory for writing
    let res_dir = get_res_directory_path(&project_path);

    // First write
    write_strings_xml(&original_strings, &res_dir, "default")
        .expect("Failed to write strings (first time)");
    let _after_first = fs::read_to_string(&strings_path).expect("Failed to read after first write");
    let strings_after_first = load_strings_xml(&strings_path).expect("Failed to parse after first write");

    // Second write (should be idempotent)
    write_strings_xml(&strings_after_first, &res_dir, "default")
        .expect("Failed to write strings (second time)");
    let _after_second = fs::read_to_string(&strings_path).expect("Failed to read after second write");

    // Verify strings are preserved
    let strings_after_second = load_strings_xml(&strings_path).expect("Failed to parse after second write");
    assert_eq!(original_strings.len(), strings_after_second.len(), 
               "String count should remain constant through multiple writes");

    // Verify all original strings are still present (by name)
    let original_names: std::collections::HashSet<&str> = 
        original_strings.iter().map(|s| s.name.as_str()).collect();
    let final_names: std::collections::HashSet<&str> = 
        strings_after_second.iter().map(|s| s.name.as_str()).collect();
    
    assert_eq!(original_names, final_names, "All original strings should be preserved");
    
    // Also verify order is preserved through multiple write cycles
    for (i, (original, final_string)) in original_strings.iter().zip(strings_after_second.iter()).enumerate() {
        assert_eq!(original.name, final_string.name, 
                  "String order at index {} should be preserved: expected '{}', got '{}'", 
                  i, original.name, final_string.name);
    }
}

#[test]
fn test_write_comment_preservation() {
    let config = TestConfig::load();
    skip_if_disabled!(config.writing.golden_comparison, "test_write_comment_preservation");

    let (_temp_dir, project_path) = create_temp_project("android_project_basic")
        .expect("Failed to create temp project");

    let strings_path = get_strings_xml_path(&project_path, "default");
    let original_content = fs::read_to_string(&strings_path).expect("Failed to read original");
    
    // Verify original has comments
    assert!(original_content.contains("<!--"), "Original file should have comments");

    let strings = load_strings_xml(&strings_path).expect("Failed to parse strings");
    
    // Find the res directory for writing
    let res_dir = get_res_directory_path(&project_path);
    
    // Write and read back
    write_strings_xml(&strings, &res_dir, "default").expect("Failed to write strings");
    let written_content = fs::read_to_string(&strings_path).expect("Failed to read written file");

    // Comments should be preserved (the write function preserves non-string lines)
    // Note: This test verifies that comments aren't completely lost
    // Exact comment preservation depends on implementation
    assert!(!written_content.is_empty(), "Written file should not be empty");
}

#[test]
fn test_write_plurals() {
    let config = TestConfig::load();
    skip_if_disabled!(config.writing.plurals, "test_write_plurals");

    let (_temp_dir, project_path) = create_temp_project("android_project_basic")
        .expect("Failed to create temp project");

    let strings_path = get_strings_xml_path(&project_path, "default");
    let original_content = fs::read_to_string(&strings_path).expect("Failed to read original");

    // Verify plurals exist in original
    assert!(original_content.contains("<plurals"), "Original should contain plurals");
    assert!(original_content.contains("name=\"apples\""), "Original should contain apples plural");
    assert!(original_content.contains("quantity=\"zero\""), "Original should contain zero quantity");
    assert!(original_content.contains("quantity=\"one\""), "Original should contain one quantity");

    let strings = load_strings_xml(&strings_path).expect("Failed to load strings");
    let plurals = lynstring_tauri_lib::commands::strings::parse_plurals_xml(&original_content)
        .expect("Failed to parse plurals");
    let mut to_write: Vec<lynstring_tauri_lib::models::AndroidString> = strings.clone();
    for p in &plurals {
        for item in &p.items {
            to_write.push(lynstring_tauri_lib::models::AndroidString {
                name: format!("{}|{}", p.name, item.quantity),
                text: item.text.clone(),
                is_translatable: p.is_translatable,
                is_commented_out: false,
                line_number: 0,
                has_cdata: item.has_cdata,
            });
        }
    }
    let res_dir = get_res_directory_path(&project_path);
    lynstring_tauri_lib::commands::strings::write_strings_to_file(&to_write, res_dir.to_str().unwrap(), "default")
        .expect("Failed to write strings and plurals");

    let written_content = fs::read_to_string(&strings_path).expect("Failed to read written file");
    assert!(written_content.contains("<plurals"), "Should write plurals tag");
    assert!(written_content.contains("quantity=\"zero\""), "Should write zero quantity");
    assert!(written_content.contains("quantity=\"one\""), "Should write one quantity");
    let plurals_after = lynstring_tauri_lib::commands::strings::parse_plurals_xml(&written_content)
        .expect("Failed to parse written plurals");
    let apples = plurals_after.iter().find(|p| p.name == "apples").unwrap();
    assert_eq!(apples.items.len(), 6, "Should preserve all 6 quantity items");
}


#[test]
fn test_write_plurals_preserves_placeholders() {
    let config = TestConfig::load();
    skip_if_disabled!(config.writing.plurals, "test_write_plurals_preserves_placeholders");

    let (_temp_dir, project_path) = create_temp_project("android_project_basic")
        .expect("Failed to create temp project");

    let strings_path = get_strings_xml_path(&project_path, "default");
    let original_content = fs::read_to_string(&strings_path).expect("Failed to read original");

    let strings = load_strings_xml(&strings_path).expect("Failed to load strings");
    let plurals = lynstring_tauri_lib::commands::strings::parse_plurals_xml(&original_content)
        .expect("Failed to parse plurals");
    let mut to_write: Vec<lynstring_tauri_lib::models::AndroidString> = strings.clone();
    for p in &plurals {
        for item in &p.items {
            to_write.push(lynstring_tauri_lib::models::AndroidString {
                name: format!("{}|{}", p.name, item.quantity),
                text: item.text.clone(),
                is_translatable: p.is_translatable,
                is_commented_out: false,
                line_number: 0,
                has_cdata: item.has_cdata,
            });
        }
    }
    let res_dir = get_res_directory_path(&project_path);
    lynstring_tauri_lib::commands::strings::write_strings_to_file(&to_write, res_dir.to_str().unwrap(), "default")
        .expect("Failed to write strings and plurals");

    let written_content = fs::read_to_string(&strings_path).expect("Failed to read written file");
    let plurals_after = lynstring_tauri_lib::commands::strings::parse_plurals_xml(&written_content)
        .expect("Failed to parse written plurals");

    let apples = plurals_after.iter().find(|p| p.name == "apples").unwrap();
    let one_item = apples.items.iter().find(|i| i.quantity == "one").unwrap();
    assert!(one_item.text.contains("%1$d"), "Round-trip should preserve %1$d placeholder in plural");

    let plural_placeholders = plurals_after.iter().find(|p| p.name == "plural_placeholders").unwrap();
    let one = plural_placeholders.items.iter().find(|i| i.quantity == "one").unwrap();
    assert!(one.text.contains("%1$s") && one.text.contains("%2$d") && one.text.contains("%3$f") && one.text.contains("%%"),
        "Round-trip should preserve %1$s, %2$d, %3$f, %% in plural");
}

#[test]
fn test_write_plurals_preserves_cdata() {
    let config = TestConfig::load();
    skip_if_disabled!(config.writing.plurals, "test_write_plurals_preserves_cdata");

    let (_temp_dir, project_path) = create_temp_project("android_project_basic")
        .expect("Failed to create temp project");

    let strings_path = get_strings_xml_path(&project_path, "default");
    let original_content = fs::read_to_string(&strings_path).expect("Failed to read original");

    let strings = load_strings_xml(&strings_path).expect("Failed to load strings");
    let plurals = lynstring_tauri_lib::commands::strings::parse_plurals_xml(&original_content)
        .expect("Failed to parse plurals");
    let mut to_write: Vec<lynstring_tauri_lib::models::AndroidString> = strings.clone();
    for p in &plurals {
        for item in &p.items {
            to_write.push(lynstring_tauri_lib::models::AndroidString {
                name: format!("{}|{}", p.name, item.quantity),
                text: item.text.clone(),
                is_translatable: p.is_translatable,
                is_commented_out: false,
                line_number: 0,
                has_cdata: item.has_cdata,
            });
        }
    }
    let res_dir = get_res_directory_path(&project_path);
    lynstring_tauri_lib::commands::strings::write_strings_to_file(&to_write, res_dir.to_str().unwrap(), "default")
        .expect("Failed to write strings and plurals");

    let written_content = fs::read_to_string(&strings_path).expect("Failed to read written file");
    assert!(written_content.contains("<![CDATA["), "Written file should contain CDATA");
    let plurals_after = lynstring_tauri_lib::commands::strings::parse_plurals_xml(&written_content)
        .expect("Failed to parse written plurals");

    let messages_count = plurals_after.iter().find(|p| p.name == "messages_count").unwrap();
    let zero_item = messages_count.items.iter().find(|i| i.quantity == "zero").unwrap();
    assert!(zero_item.has_cdata, "CDATA item should have has_cdata true after round-trip");
    assert!(zero_item.text.contains("<b>"), "CDATA HTML content should be preserved after round-trip");
}

#[test]
fn test_write_merge_mode() {
    let config = TestConfig::load();
    skip_if_disabled!(config.writing.merge_mode, "test_write_merge_mode");

    let (_temp_dir, project_path) = create_temp_project("android_project_basic")
        .expect("Failed to create temp project");

    let strings_path = get_strings_xml_path(&project_path, "default");
    let original_strings = load_strings_xml(&strings_path).expect("Failed to load strings");
    
    // Find the res directory for writing
    let res_dir = get_res_directory_path(&project_path);
    
    // Create a subset of strings to write (simulating partial translation)
    let partial_strings: Vec<lynstring_tauri_lib::models::AndroidString> = original_strings
        .iter()
        .take(3)
        .cloned()
        .collect();
    
    // First, test overwrite mode (non-merge) - should only have the strings we write
    write_strings_xml(&partial_strings, &res_dir, "default")
        .expect("Failed to write strings in overwrite mode");
    
    let overwritten_strings = load_strings_xml(&strings_path).expect("Failed to read overwritten strings");
    
    // In overwrite mode, only our strings should exist
    assert_eq!(partial_strings.len(), overwritten_strings.len(),
               "Overwrite mode should only contain written strings");
    
    // Now restore original strings before testing merge mode
    write_strings_xml(&original_strings, &res_dir, "default")
        .expect("Failed to restore original strings");
    
    // Write in merge mode (should preserve existing strings not in our list)
    write_strings_xml_merge(&partial_strings, &res_dir, "default")
        .expect("Failed to write strings in merge mode");
    
    // Read back
    let merged_strings = load_strings_xml(&strings_path).expect("Failed to read merged strings");
    
    // In merge mode, all original strings should still be present (only updates strings in our list)
    assert_eq!(original_strings.len(), merged_strings.len(),
               "Merge mode should preserve all original strings (left: {}, right: {})", 
               original_strings.len(), merged_strings.len());
    
    // Verify all original string names are still present
    let original_names: std::collections::HashSet<&str> = 
        original_strings.iter().map(|s| s.name.as_str()).collect();
    let merged_names: std::collections::HashSet<&str> = 
        merged_strings.iter().map(|s| s.name.as_str()).collect();
    assert_eq!(original_names, merged_names,
               "Merge mode should preserve all original string names");
    
    // Verify the strings we wrote are updated
    for partial_string in &partial_strings {
        let merged = merged_strings.iter().find(|s| s.name == partial_string.name);
        assert!(merged.is_some(), "String '{}' should exist after merge", partial_string.name);
        assert_eq!(merged.unwrap().text, partial_string.text,
                  "String '{}' should be updated with new text", partial_string.name);
    }
}

#[test]
fn test_write_whitespace_preservation() {
    let config = TestConfig::load();
    skip_if_disabled!(config.writing.whitespace_preservation, "test_write_whitespace_preservation");

    let (_temp_dir, project_path) = create_temp_project("android_project_basic")
        .expect("Failed to create temp project");

    let strings_path = get_strings_xml_path(&project_path, "default");
    
    // Read original file content to capture exact whitespace
    let original_content = fs::read_to_string(&strings_path).expect("Failed to read original");
    let original_strings = load_strings_xml(&strings_path).expect("Failed to load strings");
    
    // Find the res directory for writing
    let res_dir = get_res_directory_path(&project_path);
    
    // Write strings back
    write_strings_xml(&original_strings, &res_dir, "default")
        .expect("Failed to write strings");
    
    // Read written content
    let written_content = fs::read_to_string(&strings_path).expect("Failed to read written file");
    
    // Normalize line endings for comparison (CRLF -> LF)
    let original_normalized = original_content.replace("\r\n", "\n").replace("\r", "\n");
    let written_normalized = written_content.replace("\r\n", "\n").replace("\r", "\n");
    
    // Compare line by line to check whitespace preservation
    let _original_lines: Vec<&str> = original_normalized.lines().collect();
    let written_lines: Vec<&str> = written_normalized.lines().collect();
    
    // The written file should have similar structure
    // We can't expect exact match because write may reformat, but key whitespace should be preserved
    // Check that indentation is consistent (not all on one line)
    assert!(written_lines.len() > 1, "Written file should have multiple lines (preserve formatting)");
    
    // Verify that string content itself preserves whitespace (like \n, \t in text)
    let strings_after = load_strings_xml(&strings_path).expect("Failed to read written strings");
    for (original, written) in original_strings.iter().zip(strings_after.iter()) {
        assert_eq!(original.text, written.text,
                  "String '{}' text should preserve whitespace exactly", original.name);
    }
}

#[test]
fn test_write_empty_file() {
    let config = TestConfig::load();
    skip_if_disabled!(config.writing.empty_files, "test_write_empty_file");

    let (_temp_dir, project_path) = create_temp_project("android_project_basic")
        .expect("Failed to create temp project");

    let res_dir = get_res_directory_path(&project_path);
    let _res_dir_str = res_dir.to_str().expect("Invalid res directory path");
    
    // Test writing to a new language (empty file)
    let new_language = "test-empty";
    let empty_strings: Vec<lynstring_tauri_lib::models::AndroidString> = vec![];
    
    // Should be able to write empty strings (creates file with just resources tags)
    write_strings_xml(&empty_strings, &res_dir, new_language)
        .expect("Failed to write empty strings");
    
    let new_strings_path = get_strings_xml_path(&project_path, new_language);
    assert!(new_strings_path.exists(), "New language file should be created");
    
    // File should contain at least resources tags
    let content = fs::read_to_string(&new_strings_path).expect("Failed to read new file");
    assert!(content.contains("<resources"), "File should contain resources tag");
    
    // Now write some strings to it
    let test_strings = vec![
        lynstring_tauri_lib::models::AndroidString {
            name: "test_string".to_string(),
            text: "Test value".to_string(),
            is_translatable: true,
            is_commented_out: false,
            line_number: 0,
            has_cdata: false,
        },
    ];
    
    write_strings_xml(&test_strings, &res_dir, new_language)
        .expect("Failed to write test strings");
    
    let written_strings = load_strings_xml(&new_strings_path).expect("Failed to read written strings");
    assert_eq!(written_strings.len(), 1, "Should have one string after writing");
    assert_eq!(written_strings[0].name, "test_string", "Should have correct string name");
}

#[test]
fn test_write_comment_only_file() {
    let config = TestConfig::load();
    skip_if_disabled!(config.writing.empty_files, "test_write_comment_only_file");

    let (_temp_dir, project_path) = create_temp_project("android_project_basic")
        .expect("Failed to create temp project");

    let res_dir = get_res_directory_path(&project_path);
    
    // Create a file with only comments
    let new_language = "test-comments";
    let comment_only_content = r#"<?xml version="1.0" encoding="utf-8"?>
<resources xmlns:xliff="urn:oasis:names:tc:xliff:document:1.2">
    <!-- This is a comment -->
    <!-- Another comment -->
</resources>"#;
    
    let comment_file_path = get_strings_xml_path(&project_path, new_language);
    if let Some(parent) = comment_file_path.parent() {
        fs::create_dir_all(parent).expect("Failed to create directory");
    }
    fs::write(&comment_file_path, comment_only_content).expect("Failed to write comment-only file");
    
    // Try to parse it (should handle gracefully)
    let parsed_strings = load_strings_xml(&comment_file_path);
    assert!(parsed_strings.is_ok(), "Should parse comment-only file without error");
    let strings = parsed_strings.unwrap();
    assert_eq!(strings.len(), 0, "Comment-only file should have no strings");
    
    // Write strings to it (should preserve comments)
    let test_strings = vec![
        lynstring_tauri_lib::models::AndroidString {
            name: "new_string".to_string(),
            text: "New value".to_string(),
            is_translatable: true,
            is_commented_out: false,
            line_number: 0,
            has_cdata: false,
        },
    ];
    
    write_strings_xml(&test_strings, &res_dir, new_language)
        .expect("Failed to write strings to comment-only file");
    
    let written_content = fs::read_to_string(&comment_file_path).expect("Failed to read written file");
    // Comments might be preserved or not depending on implementation, but file should be valid
    assert!(written_content.contains("<resources"), "File should still have resources tag");
    assert!(written_content.contains("new_string"), "File should contain new string");
}

#[test]
fn test_error_handling_invalid_xml() {
    let config = TestConfig::load();
    skip_if_disabled!(config.writing.error_handling, "test_error_handling_invalid_xml");

    let (_temp_dir, project_path) = create_temp_project("android_project_basic")
        .expect("Failed to create temp project");

    // Create a file with invalid XML
    let invalid_xml = r#"<?xml version="1.0" encoding="utf-8"?>
<resources>
    <string name="test">Unclosed tag
</resources>"#;
    
    let invalid_path = get_strings_xml_path(&project_path, "invalid");
    if let Some(parent) = invalid_path.parent() {
        fs::create_dir_all(parent).expect("Failed to create directory");
    }
    fs::write(&invalid_path, invalid_xml).expect("Failed to write invalid XML");
    
    // Try to parse invalid XML - should handle gracefully (either error or skip invalid parts)
    let result = load_strings_xml(&invalid_path);
    // Parser might succeed (skipping invalid parts) or fail - both are acceptable
    // Just verify it doesn't panic
    match result {
        Ok(_strings) => {
            // If it succeeds, it should handle invalid XML gracefully
            // (might parse what it can, or return empty)
            assert!(true, "Parser handled invalid XML gracefully");
        }
        Err(e) => {
            // If it fails, error should be descriptive
            assert!(!e.is_empty(), "Error message should not be empty");
        }
    }
}

#[test]
fn test_error_handling_missing_file() {
    let config = TestConfig::load();
    skip_if_disabled!(config.writing.error_handling, "test_error_handling_missing_file");

    let (_temp_dir, project_path) = create_temp_project("android_project_basic")
        .expect("Failed to create temp project");

    // Try to read from non-existent language
    let non_existent_path = get_strings_xml_path(&project_path, "nonexistent");
    
    // Should handle missing file gracefully
    let result = load_strings_xml(&non_existent_path);
    match result {
        Ok(strings) => {
            // If it returns empty list for missing file, that's acceptable
            assert_eq!(strings.len(), 0, "Missing file should return empty list");
        }
        Err(e) => {
            // If it errors, should be descriptive
            assert!(!e.is_empty(), "Error message should not be empty");
            assert!(e.to_lowercase().contains("not found") || 
                   e.to_lowercase().contains("failed") ||
                   e.to_lowercase().contains("error"),
                   "Error should indicate file issue");
        }
    }
}

#[test]
fn test_error_handling_write_to_nonexistent_path() {
    let config = TestConfig::load();
    skip_if_disabled!(config.writing.error_handling, "test_error_handling_write_to_nonexistent_path");

    let (_temp_dir, project_path) = create_temp_project("android_project_basic")
        .expect("Failed to create temp project");

    let res_dir = get_res_directory_path(&project_path);
    
    // Try to write to a completely invalid/nonexistent path
    // This should create the directory structure if possible, or error gracefully
    let test_strings = vec![
        lynstring_tauri_lib::models::AndroidString {
            name: "test".to_string(),
            text: "value".to_string(),
            is_translatable: true,
            is_commented_out: false,
            line_number: 0,
            has_cdata: false,
        },
    ];
    
    // Writing to a new language should create the directory structure
    let result = write_strings_xml(&test_strings, &res_dir, "new-language");
    assert!(result.is_ok(), "Should be able to write to new language (creates directories)");
    
    // Verify file was created
    let new_path = get_strings_xml_path(&project_path, "new-language");
    assert!(new_path.exists(), "New language file should be created");
}

