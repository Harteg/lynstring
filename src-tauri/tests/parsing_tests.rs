mod test_config;
mod helpers;

use lynstring_tauri_lib::commands::strings::parse_strings_xml;
use test_config::TestConfig;
use helpers::*;

macro_rules! skip_if_disabled {
    ($config:expr, $test_name:literal) => {
        if !$config {
            eprintln!("⏭️  [SKIP] {}", $test_name);
            return;
        }
    };
}

#[test]
fn test_parse_basic_strings() {
    let config = TestConfig::load();
    skip_if_disabled!(config.parsing.basic, "test_parse_basic_strings");

    let strings_path = fixture_path("android_project_basic/app/src/main/res/values/strings.xml");
    let strings = load_strings_xml(&strings_path).expect("Failed to parse strings.xml");

    // Verify we parsed strings
    assert!(!strings.is_empty(), "Should parse at least some strings");

    // Check for specific strings
    let simple_quote = strings.iter().find(|s| s.name == "simple_quote");
    assert!(simple_quote.is_some(), "Should find simple_quote string");
    assert_eq!(simple_quote.unwrap().text, "It\\'s a nice day.");

    let bold_text = strings.iter().find(|s| s.name == "bold_text");
    assert!(bold_text.is_some(), "Should find bold_text string");
    assert!(bold_text.unwrap().text.contains("<b>"), "Should preserve HTML tags");
}

#[test]
fn test_parse_complex_strings() {
    let config = TestConfig::load();
    skip_if_disabled!(config.parsing.complex, "test_parse_complex_strings");

    let strings_path = fixture_path("android_project_basic/app/src/main/res/values/strings.xml");
    let strings = load_strings_xml(&strings_path).expect("Failed to parse strings.xml");

    // Test placeholder preservation
    let percent_placeholders = strings.iter().find(|s| s.name == "percent_placeholders");
    assert!(percent_placeholders.is_some(), "Should find percent_placeholders");
    assert!(percent_placeholders.unwrap().text.contains("%1$s"), "Should preserve %1$s placeholder");
    assert!(percent_placeholders.unwrap().text.contains("%2$d"), "Should preserve %2$d placeholder");

    // Test literal percent signs
    let percent_literal = strings.iter().find(|s| s.name == "percent_literal");
    assert!(percent_literal.is_some(), "Should find percent_literal");
    assert!(percent_literal.unwrap().text.contains("%%"), "Should preserve literal %%");

    // Test nested XML tags
    let link_text = strings.iter().find(|s| s.name == "link_text");
    assert!(link_text.is_some(), "Should find link_text");
    assert!(link_text.unwrap().text.contains("<a href"), "Should preserve <a> tags");

    // Test XLIFF tags
    let xliff_example = strings.iter().find(|s| s.name == "xliff_example");
    assert!(xliff_example.is_some(), "Should find xliff_example");
    assert!(xliff_example.unwrap().text.contains("<xliff:g"), "Should preserve XLIFF tags");

    // Test escaped XML
    let escaped_xml = strings.iter().find(|s| s.name == "escaped_xml");
    assert!(escaped_xml.is_some(), "Should find escaped_xml");
    assert!(escaped_xml.unwrap().text.contains("&lt;"), "Should preserve escaped XML");

    // Test emojis and unicode
    let emoji_mixed = strings.iter().find(|s| s.name == "emoji_mixed");
    assert!(emoji_mixed.is_some(), "Should find emoji_mixed");
    assert!(emoji_mixed.unwrap().text.contains("👍"), "Should preserve emojis");

    // Test newlines and tabs
    let newline_example = strings.iter().find(|s| s.name == "newline_example");
    assert!(newline_example.is_some(), "Should find newline_example");
    assert!(newline_example.unwrap().text.contains("\\n"), "Should preserve newline escapes");

    // Test empty string
    let empty_string = strings.iter().find(|s| s.name == "empty_string");
    assert!(empty_string.is_some(), "Should find empty_string");
    assert_eq!(empty_string.unwrap().text, "", "Empty string should be empty");
}

#[test]
fn test_parse_plurals() {
    let config = TestConfig::load();
    skip_if_disabled!(config.parsing.plurals, "test_parse_plurals");

    let strings_path = fixture_path("android_project_basic/app/src/main/res/values/strings.xml");
    let content = std::fs::read_to_string(&strings_path).expect("Failed to read strings.xml");

    // Verify plurals block exists in file
    assert!(content.contains("<plurals"), "File should contain <plurals> tag");
    assert!(content.contains("name=\"apples\""), "File should contain apples plural");

    let plurals = lynstring_tauri_lib::commands::strings::parse_plurals_xml(&content)
        .expect("Failed to parse plurals");
    let apples_plural = plurals.iter().find(|p| p.name == "apples");
    assert!(apples_plural.is_some(), "Should parse apples plural");
    assert_eq!(apples_plural.unwrap().items.len(), 6, "Should have 6 quantity items");
    let one_item = apples_plural.unwrap().items.iter().find(|i| i.quantity == "one");
    assert!(one_item.is_some(), "Should have one quantity");
    assert!(one_item.unwrap().text.contains("%1$d"), "Should preserve placeholder");

    let messages_plural = plurals.iter().find(|p| p.name == "messages_count");
    assert!(messages_plural.is_some(), "Should parse messages_count plural");
    let zero_item = messages_plural.unwrap().items.iter().find(|i| i.quantity == "zero");
    assert!(zero_item.is_some(), "Should have zero quantity");
    assert!(zero_item.unwrap().has_cdata, "CDATA item should have has_cdata true");
    assert!(zero_item.unwrap().text.contains("<b>"), "CDATA content should preserve HTML");
}


#[test]
fn test_parse_plurals_placeholders() {
    let config = TestConfig::load();
    skip_if_disabled!(config.parsing.plurals, "test_parse_plurals_placeholders");

    let strings_path = fixture_path("android_project_basic/app/src/main/res/values/strings.xml");
    let content = std::fs::read_to_string(&strings_path).expect("Failed to read strings.xml");

    let plurals = lynstring_tauri_lib::commands::strings::parse_plurals_xml(&content)
        .expect("Failed to parse plurals");
    let p = plurals.iter().find(|p| p.name == "plural_placeholders")
        .expect("Should find plural_placeholders");

    let one_item = p.items.iter().find(|i| i.quantity == "one").unwrap();
    assert!(one_item.text.contains("%1$s"), "Should preserve %1$s placeholder");
    assert!(one_item.text.contains("%2$d"), "Should preserve %2$d placeholder");
    assert!(one_item.text.contains("%3$f"), "Should preserve %3$f placeholder");
    assert!(one_item.text.contains("%%"), "Should preserve %% literal");
}

#[test]
fn test_parse_plurals_translatable_false() {
    let config = TestConfig::load();
    skip_if_disabled!(config.parsing.plurals, "test_parse_plurals_translatable_false");

    let strings_path = fixture_path("android_project_basic/app/src/main/res/values/strings.xml");
    let content = std::fs::read_to_string(&strings_path).expect("Failed to read strings.xml");

    let plurals = lynstring_tauri_lib::commands::strings::parse_plurals_xml(&content)
        .expect("Failed to parse plurals");
    let p = plurals.iter().find(|p| p.name == "plural_not_translatable")
        .expect("Should find plural_not_translatable");

    assert!(!p.is_translatable, "translatable=\"false\" should set is_translatable to false");
}

#[test]
fn test_parse_plurals_empty_item() {
    let config = TestConfig::load();
    skip_if_disabled!(config.parsing.plurals, "test_parse_plurals_empty_item");

    let strings_path = fixture_path("android_project_basic/app/src/main/res/values/strings.xml");
    let content = std::fs::read_to_string(&strings_path).expect("Failed to read strings.xml");

    let plurals = lynstring_tauri_lib::commands::strings::parse_plurals_xml(&content)
        .expect("Failed to parse plurals");
    let p = plurals.iter().find(|p| p.name == "plural_empty_item")
        .expect("Should find plural_empty_item");

    let other_item = p.items.iter().find(|i| i.quantity == "other")
        .expect("Should have other quantity");
    assert_eq!(other_item.text, "", "Empty item should have empty string");
}

#[test]
fn test_parse_cdata() {
    let config = TestConfig::load();
    skip_if_disabled!(config.parsing.cdata, "test_parse_cdata");

    let strings_path = fixture_path("android_project_basic/app/src/main/res/values/strings.xml");
    let content = std::fs::read_to_string(&strings_path).expect("Failed to read strings.xml");

    // Verify CDATA exists in file
    assert!(content.contains("<![CDATA["), "File should contain CDATA section");
    assert!(content.contains("email_footer_text"), "File should contain email_footer_text with CDATA");

    // Parse and verify CDATA is properly extracted
    let strings = load_strings_xml(&strings_path).expect("Failed to parse strings.xml");
    let email_footer = strings.iter().find(|s| s.name == "email_footer_text");
    
    // TODO: When CDATA is implemented, this test should verify:
    // 1. String with CDATA is parsed correctly
    // 2. CDATA content is extracted (without CDATA markers)
    // 3. HTML/XML tags inside CDATA are preserved
    // 4. The full CDATA content is available in the string text
    
    assert!(email_footer.is_some(), "Should parse string with CDATA");
    let footer = email_footer.unwrap();
    
    // CDATA content should be extracted (without <![CDATA[ and ]]> markers)
    assert!(!footer.text.contains("<![CDATA["), "CDATA markers should be removed from parsed text");
    assert!(!footer.text.contains("]]>"), "CDATA closing marker should be removed");
    
    // Content inside CDATA should be preserved
    assert!(footer.text.contains("Sent by"), "CDATA content 'Sent by' should be preserved");
    assert!(footer.text.contains("Dummy App"), "CDATA content 'Dummy App' should be preserved");
    assert!(footer.text.contains("<a href"), "HTML tags inside CDATA should be preserved");
    assert!(footer.text.contains("<strong>"), "HTML tags inside CDATA should be preserved");
}

#[test]
fn test_parse_annotation() {
    let config = TestConfig::load();
    skip_if_disabled!(config.parsing.annotation, "test_parse_annotation");

    let strings_path = fixture_path("android_project_basic/app/src/main/res/values/strings.xml");
    let strings = load_strings_xml(&strings_path).expect("Failed to parse strings.xml");

    let annotation_example = strings.iter().find(|s| s.name == "annotation_example");
    assert!(annotation_example.is_some(), "Should find annotation_example string");
    
    // Annotation tags should be preserved in text content
    let text = &annotation_example.unwrap().text;
    assert!(text.contains("<annotation"), "Should preserve <annotation> tags in text");
    assert!(text.contains("annotated text"), "Should preserve annotation content");
}

#[test]
fn test_parse_commented_strings() {
    let config = TestConfig::load();
    skip_if_disabled!(config.parsing.commented, "test_parse_commented_strings");

    // Create a test XML with both regular and commented strings
    let xml_content = r#"<?xml version="1.0" encoding="utf-8"?>
<resources>
    <string name="active_string">This is an active string</string>
    <!-- <string name="commented_string">This is commented out</string> -->
    <string name="another_active">Another active string</string>
    <!--
    <string name="multiline_commented">This is a multiline comment</string>
    -->
    <string name="final_active">Final active string</string>
    <!-- Comment with other content -->
    <string name="after_comment">String after comment</string>
    <!-- Multiple commented strings in one comment block
    <string name="commented1">First commented</string>
    <string name="commented2">Second commented</string>
    -->
</resources>"#;

    let strings = parse_strings_xml(xml_content).expect("Failed to parse strings.xml");

    // Should only parse active strings, not commented ones
    assert_eq!(strings.len(), 4, "Should only parse 4 active strings");
    
    // Verify active strings are parsed
    assert!(strings.iter().any(|s| s.name == "active_string"), "Should parse active_string");
    assert!(strings.iter().any(|s| s.name == "another_active"), "Should parse another_active");
    assert!(strings.iter().any(|s| s.name == "final_active"), "Should parse final_active");
    assert!(strings.iter().any(|s| s.name == "after_comment"), "Should parse after_comment");
    
    // Verify commented strings are NOT parsed
    assert!(!strings.iter().any(|s| s.name == "commented_string"), "Should NOT parse commented_string");
    assert!(!strings.iter().any(|s| s.name == "multiline_commented"), "Should NOT parse multiline_commented");
    assert!(!strings.iter().any(|s| s.name == "commented1"), "Should NOT parse commented1");
    assert!(!strings.iter().any(|s| s.name == "commented2"), "Should NOT parse commented2");
}

