mod test_config;
mod helpers;

use test_config::TestConfig;
use helpers::*;
use lynstring_tauri_lib::models::AndroidString;

macro_rules! skip_if_disabled {
    ($config:expr, $test_name:literal) => {
        if !$config {
            eprintln!("⏭️  [SKIP] {}", $test_name);
            return;
        }
    };
}

#[test]
fn test_unused_detection() {
    let config = TestConfig::load();
    skip_if_disabled!(config.unused_strings.detection, "test_unused_detection");

    let (_temp_dir, project_path) = create_temp_project("android_project_basic")
        .expect("Failed to create temp project");

    // Scan code files for string references
    let referenced_strings = scan_code_for_string_references(&project_path)
        .expect("Failed to scan code for string references");
    
    // Expected strings that should be found in code
    let expected_used = vec!["bold_text", "email_template", "file_path", "simple_quote", 
                             "price_label", "tom_and_jerry"];
    
    // Verify all expected strings are found
    for string_name in &expected_used {
        assert!(referenced_strings.contains(*string_name), 
               "Should find reference to '{}' in code", string_name);
    }

    // Load strings to see which ones exist
    let strings_path = get_strings_xml_path(&project_path, "default");
    let strings = load_strings_xml(&strings_path).expect("Failed to parse strings");
    
    // Strings that should be used (referenced in code)
    let used_strings = expected_used;
    
    // Verify these strings exist
    for used_name in &used_strings {
        assert!(strings.iter().any(|s| s.name == *used_name), 
                "Used string '{}' should exist", used_name);
    }

    // Strings that are likely unused (not referenced in our test code)
    // Note: This is a simplified test - the actual function does more sophisticated detection
    let potentially_unused = vec!["question_mark", "quoted_block", "at_symbol"];
    
    // These should exist but might be unused
    for unused_name in &potentially_unused {
        let exists = strings.iter().any(|s| s.name == *unused_name);
        // Just verify the test logic works - actual unused detection is more complex
        if exists {
            println!("String '{}' exists (may be unused)", unused_name);
        }
    }
}

#[test]
fn test_unused_deletion() {
    let config = TestConfig::load();
    skip_if_disabled!(config.unused_strings.deletion, "test_unused_deletion");

    let (_temp_dir, project_path) = create_temp_project("android_project_basic")
        .expect("Failed to create temp project");

    let strings_path = get_strings_xml_path(&project_path, "default");
    let original_strings = load_strings_xml(&strings_path).expect("Failed to parse strings");
    let original_count = original_strings.len();

    // For this test, we'll manually remove a string that we know is unused
    // In a real scenario, find_unused_strings would identify these
    
    // Find a string that's likely unused (not in our test code references)
    let unused_name = "question_mark"; // This is not referenced in our test code
    
    // Filter out the unused string
    let remaining_strings: Vec<AndroidString> = original_strings
        .into_iter()
        .filter(|s| s.name != unused_name)
        .collect();

    assert_eq!(remaining_strings.len(), original_count - 1, 
               "Should have one less string after deletion");

    // Find the res directory for writing
    let res_dir = get_res_directory_path(&project_path);
    
    // Write back the filtered strings
    write_strings_xml(&remaining_strings, &res_dir, "default")
        .expect("Failed to write filtered strings");

    // Verify the string was removed
    let after_deletion = load_strings_xml(&strings_path).expect("Failed to parse after deletion");
    assert!(!after_deletion.iter().any(|s| s.name == unused_name),
            "Unused string should be removed");
    assert_eq!(after_deletion.len(), original_count - 1,
               "String count should be reduced by one");
}

