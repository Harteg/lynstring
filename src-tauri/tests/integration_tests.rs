mod test_config;
mod helpers;

use test_config::TestConfig;
use helpers::*;
use lynstring_tauri_lib::models::AndroidString;
use lynstring_tauri_lib::commands::strings::load_all_strings;
use std::collections::HashSet;

macro_rules! skip_if_disabled {
    ($config:expr, $test_name:literal) => {
        if !$config {
            eprintln!("⏭️  [SKIP] {}", $test_name);
            return;
        }
    };
}

#[test]
fn test_integration_missing_translations() {
    let config = TestConfig::load();
    skip_if_disabled!(config.integration.missing_translations, "test_integration_missing_translations");

    let (_temp_dir, project_path) = create_temp_project("android_project_basic")
        .expect("Failed to create temp project");

    // Find the res directory - the functions expect project_path to point to res
    let res_dir = get_res_directory_path(&project_path);
    let res_dir_str = res_dir.to_str().expect("Invalid res directory path");
    let languages = vec!["es".to_string(), "fr".to_string()];

    // Load all strings
    let all_data = load_all_strings(res_dir_str.to_string(), languages.clone())
        .expect("Failed to load all strings");

    let default_strings = all_data.strings_by_language.get("default")
        .expect("Should have default strings");
    
    // Check Spanish translations
    let es_strings = all_data.strings_by_language.get("es")
        .expect("Should have Spanish strings");
    
    // Find missing translations in Spanish
    let default_names: HashSet<&str> = default_strings.iter()
        .filter(|s| s.is_translatable)
        .map(|s| s.name.as_str())
        .collect();
    let es_names: HashSet<&str> = es_strings.iter()
        .map(|s| s.name.as_str())
        .collect();
    
    let missing_in_es: Vec<&str> = default_names.difference(&es_names).copied().collect();
    
    // Spanish should be missing most strings (only has 3 translated)
    assert!(!missing_in_es.is_empty(), "Spanish should have missing translations");
    assert!(missing_in_es.len() > 3, "Should have more than 3 missing translations");

    // Check French translations
    let fr_strings = all_data.strings_by_language.get("fr")
        .expect("Should have French strings");
    let fr_names: HashSet<&str> = fr_strings.iter()
        .map(|s| s.name.as_str())
        .collect();
    
    let missing_in_fr: Vec<&str> = default_names.difference(&fr_names).copied().collect();
    
    // French should be missing most strings (only has 2 translated)
    assert!(!missing_in_fr.is_empty(), "French should have missing translations");
    assert!(missing_in_fr.len() > 2, "Should have more than 2 missing translations");
}

#[test]
fn test_integration_translate_write() {
    let config = TestConfig::load();
    skip_if_disabled!(config.integration.translate_write, "test_integration_translate_write");

    let (_temp_dir, project_path) = create_temp_project("android_project_basic")
        .expect("Failed to create temp project");

    let strings_path = get_strings_xml_path(&project_path, "default");
    let original_strings = load_strings_xml(&strings_path).expect("Failed to parse strings");

    // Take a small subset for testing
    let test_strings: Vec<AndroidString> = original_strings
        .into_iter()
        .take(3)
        .collect();

    // Write to a new language (test)
    let test_language = "test";
    let test_strings_path = get_strings_xml_path(&project_path, test_language);
    
    // Ensure directory exists
    if let Some(parent) = test_strings_path.parent() {
        std::fs::create_dir_all(parent).expect("Failed to create test language directory");
    }

    // Find the res directory for writing
    let res_dir = get_res_directory_path(&project_path);
    let _res_dir_str = res_dir.to_str().expect("Invalid res directory path");
    
    // For this test, we'll just write the strings as-is (mock translation)
    // In a real scenario, these would be translated
    write_strings_xml(&test_strings, &res_dir, test_language)
        .expect("Failed to write test strings");

    // Read back and verify - get the correct path for the test language
    let test_strings_path = get_strings_xml_path(&project_path, test_language);
    let written_strings = load_strings_xml(&test_strings_path).expect("Failed to read written strings");
    
    assert_eq!(test_strings.len(), written_strings.len(), 
               "Should preserve all strings through write cycle");
    
    // Verify string names match
    let original_names: HashSet<&str> = test_strings.iter()
        .map(|s| s.name.as_str())
        .collect();
    let written_names: HashSet<&str> = written_strings.iter()
        .map(|s| s.name.as_str())
        .collect();
    
    assert_eq!(original_names, written_names, "String names should match");
}

#[test]
fn test_integration_batch_translation() {
    let config = TestConfig::load();
    skip_if_disabled!(config.integration.batch_translation, "test_integration_batch_translation");

    let (_temp_dir, project_path) = create_temp_project("android_project_basic")
        .expect("Failed to create temp project");

    let strings_path = get_strings_xml_path(&project_path, "default");
    let all_strings = load_strings_xml(&strings_path).expect("Failed to parse strings");

    // Simulate batch translation by processing in chunks
    let chunk_size = 5;
    let chunks: Vec<&[AndroidString]> = all_strings.chunks(chunk_size).collect();
    
    assert!(!chunks.is_empty(), "Should have at least one chunk");
    
    // Verify chunks don't overlap and cover all strings
    let mut total_in_chunks = 0;
    let mut all_names = HashSet::new();
    
    for chunk in &chunks {
        total_in_chunks += chunk.len();
        for string in chunk.iter() {
            all_names.insert(string.name.as_str());
        }
    }
    
    assert_eq!(total_in_chunks, all_strings.len(), 
               "Chunks should cover all strings");
    assert_eq!(all_names.len(), all_strings.len(),
               "Should have all unique string names");
    
    // Verify no duplicates
    let mut seen = HashSet::new();
    for chunk in &chunks {
        for string in chunk.iter() {
            assert!(seen.insert(string.name.as_str()), 
                   "String '{}' should not appear in multiple chunks", string.name);
        }
    }
}

