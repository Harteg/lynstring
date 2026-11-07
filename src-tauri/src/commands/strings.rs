use crate::commands::project::detect_available_languages;
use crate::models::{get_language_emoji, get_language_name, AndroidPlural, AndroidString, PluralItem};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use tauri::{Emitter, Manager};
use ts_rs::TS;
use walkdir::WalkDir;

#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct LanguageDisplayInfo {
    pub name: String,
    pub emoji: String,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct AllStringsData {
    pub languages: Vec<String>,
    pub strings_by_language: HashMap<String, Vec<AndroidString>>,
    pub plurals_by_language: HashMap<String, Vec<AndroidPlural>>,
    /// Display names and emojis for project languages (includes CLDR/region-derived for composed locales)
    pub language_display: HashMap<String, LanguageDisplayInfo>,
}

#[tauri::command]
pub fn read_strings_xml(
    project_path: String,
    language: String,
) -> Result<Vec<AndroidString>, String> {
    let project_folder = Path::new(&project_path);

    // Build path to strings.xml
    let values_folder = if language.is_empty() {
        "values".to_string()
    } else {
        format!("values-{}", language)
    };

    let strings_path = project_folder.join(&values_folder).join("strings.xml");

    if !strings_path.exists() {
        return Err(format!(
            "strings.xml not found at: {}",
            strings_path.display()
        ));
    }

    // Read the XML file
    let xml_content = fs::read_to_string(&strings_path)
        .map_err(|e| format!("Failed to read strings.xml: {}", e))?;

    parse_strings_xml(&xml_content)
}

/// Read both strings and plurals from strings.xml (internal use for load_all_strings).
fn read_resources_xml(project_path: &str, language: &str) -> Result<(Vec<AndroidString>, Vec<AndroidPlural>), String> {
    let project_folder = Path::new(project_path);
    let values_folder = if language.is_empty() {
        "values".to_string()
    } else {
        format!("values-{}", language)
    };
    let strings_path = project_folder.join(&values_folder).join("strings.xml");
    if !strings_path.exists() {
        return Err(format!("strings.xml not found at: {}", strings_path.display()));
    }
    let xml_content = fs::read_to_string(&strings_path)
        .map_err(|e| format!("Failed to read strings.xml: {}", e))?;
    let strings = parse_strings_xml(&xml_content)?;
    let plurals = parse_plurals_xml(&xml_content)?;
    Ok((strings, plurals))
}


#[tauri::command]
pub fn load_all_strings(
    project_path: String,
    languages: Vec<String>,
) -> Result<AllStringsData, String> {
    let mut strings_by_language = HashMap::new();
    let mut plurals_by_language = HashMap::new();
    let mut actual_languages = Vec::new();

    match read_resources_xml(&project_path, "") {
        Ok((strings, plurals)) => {
            strings_by_language.insert("default".to_string(), strings);
            plurals_by_language.insert("default".to_string(), plurals);
            actual_languages.push("default".to_string());
        }
        Err(e) => return Err(format!("Failed to load default strings: {}", e)),
    }

    for language in languages {
        if let Ok((strings, plurals)) = read_resources_xml(&project_path, &language) {
            strings_by_language.insert(language.clone(), strings);
            plurals_by_language.insert(language.clone(), plurals);
            actual_languages.push(language);
        }
    }

    Ok(AllStringsData {
        languages: actual_languages,
        strings_by_language,
        plurals_by_language,
        language_display: HashMap::new(),
    })
}


/// Parse <plurals name="..."><item quantity="...">...</item></plurals> from XML.
pub fn parse_plurals_xml(xml_content: &str) -> Result<Vec<AndroidPlural>, String> {
    let mut plurals = Vec::new();
    let comment_re = Regex::new(r#"(?s)<!--.*?-->"#)
        .map_err(|e| format!("Failed to compile comment regex: {}", e))?;
    let mut comment_ranges: Vec<(usize, usize)> = Vec::new();
    for cap in comment_re.captures_iter(xml_content) {
        let full_match = cap.get(0).unwrap();
        comment_ranges.push((full_match.start(), full_match.end()));
    }
    let is_inside_comment = |pos: usize| -> bool {
        comment_ranges.iter().any(|(start, end)| pos >= *start && pos < *end)
    };
    let plurals_tag_re = Regex::new(r#"<plurals\s+([^>]*)>"#)
        .map_err(|e| format!("Failed to compile plurals regex: {}", e))?;
    let name_re = Regex::new(r#"name="([^"]+)""#)
        .map_err(|e| format!("Failed to compile name regex: {}", e))?;
    let item_tag_re = Regex::new(r#"<item\s+quantity="([^"]+)"\s*>"#)
        .map_err(|e| format!("Failed to compile item regex: {}", e))?;
    for cap in plurals_tag_re.captures_iter(xml_content) {
        let full_match = cap.get(0).unwrap();
        if is_inside_comment(full_match.start()) { continue; }
        let attrs = cap.get(1).unwrap().as_str();
        let name = name_re.captures(attrs).and_then(|c| c.get(1)).map(|m| m.as_str().to_string());
        let name = match name { Some(n) => n, None => continue };
        let is_translatable = !attrs.contains("translatable=\"false\"");
        let tag_end = full_match.end();
        let content_end = xml_content[tag_end..].find("</plurals>").map(|o| tag_end + o);
        let content = match content_end { Some(e) => &xml_content[tag_end..e], None => continue };
        let mut items = Vec::new();
        for ic in item_tag_re.captures_iter(content) {
            let qty = ic.get(1).unwrap().as_str().to_string();
            let ie = ic.get(0).unwrap().end();
            let end = content[ie..].find("</item>").map(|o| ie + o);
            if let Some(end) = end {
                let mut text = content[ie..end].to_string();
                let mut has_cdata = false;
                if text.trim_start().starts_with("<![CDATA[") {
                    if let Some(s) = text.find("<![CDATA[") {
                        let a = s + 9;
                        if let Some(e) = text[a..].find("]]>") { text = text[a..a+e].to_string(); has_cdata = true; }
                    }
                }
                items.push(PluralItem { quantity: qty, text, has_cdata });
            }
        }
        if !items.is_empty() { plurals.push(AndroidPlural { name, items, is_translatable, is_commented_out: false }); }
    }
    Ok(plurals)
}


#[tauri::command]
pub fn load_project_data(project_path: String) -> Result<AllStringsData, String> {
    // First, detect available languages
    let languages = detect_available_languages(project_path.clone())?;

    let mut data = load_all_strings(project_path.clone(), languages)?;

    // Build display info (name + emoji) for each language, including CLDR/region-derived for composed locales
    let mut language_display = HashMap::new();
    for lang in &data.languages {
        if lang != "default" {
            language_display.insert(
                lang.clone(),
                LanguageDisplayInfo {
                    name: get_language_name(lang).into_owned(),
                    emoji: get_language_emoji(lang).into_owned(),
                },
            );
        }
    }
    data.language_display = language_display;

    Ok(data)
}

#[tauri::command]
pub fn find_strings_missing_default(project_path: String) -> Result<Vec<String>, String> {
    // Load all strings for all languages
    let all_data = load_project_data(project_path)?;
    
    // Get default strings
    let default_strings = all_data
        .strings_by_language
        .get("default")
        .ok_or("Default strings not found")?;
    
    // Create a set of default string names for fast lookup
    let default_names: std::collections::HashSet<&String> = default_strings
        .iter()
        .map(|s| &s.name)
        .collect();
    
    // Find strings that exist in non-default languages but not in default
    let mut missing_default = Vec::new();
    
    for (language, strings) in &all_data.strings_by_language {
        if language == "default" {
            continue;
        }
        
        for string in strings {
            if !default_names.contains(&string.name) {
                // This string exists in a non-default language but not in default
                if !missing_default.contains(&string.name) {
                    missing_default.push(string.name.clone());
                }
            }
        }
    }
    
    Ok(missing_default)
}

#[tauri::command]
pub fn delete_strings_missing_default(
    project_path: String,
    string_names: Vec<String>,
) -> Result<(), String> {
    if string_names.is_empty() {
        return Ok(());
    }

    let project_folder = Path::new(&project_path);

    // Safety check: filter out any names that actually exist in the default strings.xml
    // so we never accidentally delete a string that belongs there
    let default_path = project_folder.join("values").join("strings.xml");
    let default_content = fs::read_to_string(&default_path)
        .map_err(|e| format!("Failed to read default strings.xml: {}", e))?;
    let names_in_default: HashSet<String> = extract_names_from_xml(&default_content);

    let names_to_delete: HashSet<&str> = string_names
        .iter()
        .filter(|name| !names_in_default.contains(*name))
        .map(|s| s.as_str())
        .collect();

    if names_to_delete.is_empty() {
        return Ok(());
    }

    // Delete from translation language files only (never the default)
    let languages = detect_available_languages(project_path.clone())?;
    for language in &languages {
        let lang_path = project_folder
            .join(format!("values-{}", language))
            .join("strings.xml");
        delete_strings_from_file(&lang_path, &names_to_delete)?;
    }

    Ok(())
}

/// Remove specific `<string>` entries from a file purely by line filtering.
/// Every kept line is passed through byte-for-byte — no parsing or re-serialization occurs,
/// so CDATA, multi-line strings, comments, and any other formatting are preserved exactly.
fn delete_strings_from_file(
    file_path: &Path,
    names_to_delete: &HashSet<&str>,
) -> Result<(), String> {
    if !file_path.exists() {
        return Ok(());
    }

    let content = fs::read_to_string(file_path)
        .map_err(|e| format!("Failed to read {}: {}", file_path.display(), e))?;

    let lines: Vec<&str> = content.lines().collect();
    let mut new_lines: Vec<&str> = Vec::with_capacity(lines.len());
    let mut idx = 0;

    while idx < lines.len() {
        let line = lines[idx];
        let trimmed = line.trim();

        if trimmed.starts_with("<string") && trimmed.contains("name=\"") {
            if let Some(name) = extract_name_from_line(trimmed) {
                if names_to_delete.contains(name.as_str()) {
                    if trimmed.contains("</string>") {
                        // Single-line string: skip just this one line
                        idx += 1;
                    } else {
                        // Multi-line string: skip until the closing tag (inclusive)
                        idx += 1;
                        while idx < lines.len() {
                            if lines[idx].contains("</string>") {
                                idx += 1;
                                break;
                            }
                            idx += 1;
                        }
                    }
                    continue;
                }
            }
        }

        new_lines.push(line);
        idx += 1;
    }

    let mut result = new_lines.join("\n");
    // Preserve a trailing newline if the original file had one
    if content.ends_with('\n') {
        result.push('\n');
    }

    fs::write(file_path, result)
        .map_err(|e| format!("Failed to write {}: {}", file_path.display(), e))?;

    Ok(())
}

/// Extract all `name` attribute values from `<string>` tags in raw XML content.
/// Used for quick membership checks without a full parse.
fn extract_names_from_xml(content: &str) -> HashSet<String> {
    let mut names = HashSet::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("<string") && trimmed.contains("name=\"") {
            if let Some(name) = extract_name_from_line(trimmed) {
                names.insert(name);
            }
        }
    }
    names
}

pub fn write_strings_to_file(
    strings: &[AndroidString],
    project_path: &str,
    language: &str,
) -> Result<(), String> {
    write_strings_to_file_internal(strings, project_path, language, false)
}

pub(crate) fn write_strings_to_file_merge(
    strings: &[AndroidString],
    project_path: &str,
    language: &str,
) -> Result<(), String> {
    write_strings_to_file_internal(strings, project_path, language, true)
}

// Public version for testing
pub fn write_strings_to_file_merge_public(
    strings: &[AndroidString],
    project_path: &str,
    language: &str,
) -> Result<(), String> {
    write_strings_to_file_internal(strings, project_path, language, true)
}

fn write_strings_to_file_internal(
    strings: &[AndroidString],
    project_path: &str,
    language: &str,
    merge_mode: bool,
) -> Result<(), String> {
    let project_folder = Path::new(project_path);
    
    // Determine the strings.xml path based on language
    let strings_path = if language.is_empty() || language == "default" {
        project_folder.join("values").join("strings.xml")
    } else {
        let values_folder = format!("values-{}", language);
        project_folder.join(&values_folder).join("strings.xml")
    };

    // For default language, file must exist
    if (language.is_empty() || language == "default") && !strings_path.exists() {
        return Err("Default strings.xml not found".to_string());
    }

    // Ensure directory exists
    if let Some(parent) = strings_path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Failed to create directory: {}", e))?;
    }

    // Read existing file or create new one
    let existing_lines = if strings_path.exists() {
        fs::read_to_string(&strings_path)
            .map_err(|e| format!("Failed to read existing file: {}", e))?
            .lines()
            .map(|s| s.to_string())
            .collect::<Vec<_>>()
    } else {
        vec!["<resources>".to_string(), "</resources>".to_string()]
    };

    // Split strings into regular strings and flattened plural items (name format: "pluralName|quantity")
    let (regular_strings, plural_items): (Vec<_>, Vec<_>) = strings
        .iter()
        .partition(|s| !s.name.contains('|'));
    let mut strings_map: HashMap<&str, &AndroidString> = HashMap::new();
    for string in regular_strings {
        strings_map.insert(string.name.as_str(), string);
    }
    let plurals_map: HashMap<String, AndroidPlural> = {
        let mut by_name: HashMap<String, Vec<PluralItem>> = HashMap::new();
        for s in &plural_items {
            if let Some((name, qty)) = s.name.split_once('|') {
                by_name.entry(name.to_string()).or_default().push(PluralItem {
                    quantity: qty.to_string(),
                    text: s.text.clone(),
                    has_cdata: s.has_cdata,
                });
            }
        }
        by_name.into_iter().map(|(name, items)| {
            let first = plural_items.iter().find(|s| s.name.starts_with(&format!("{}|", name))).unwrap();
            (name.clone(), AndroidPlural {
                name: name.clone(),
                items,
                is_translatable: first.is_translatable,
                is_commented_out: false,
            })
        }).collect()
    };
    let mut plurals_written: HashSet<String> = HashSet::new();

    // Track which strings we've written (to know which new ones to add at the end)
    let mut strings_written: HashSet<String> = HashSet::new();

    // Create new lines vector that preserves exact order
    let mut new_lines = Vec::new();
    
    // Track where the closing </resources> tag should be
    let mut resources_close_index = existing_lines.len();
    if existing_lines.last().map_or(false, |line| line.trim() == "</resources>") {
        resources_close_index = existing_lines.len() - 1;
    }

    // Process existing lines, preserving order and formatting
    let mut idx = 0;
    while idx < existing_lines.len() {
        let line = &existing_lines[idx];
        
        // Skip the closing tag - we'll add it at the end
        if idx == resources_close_index && line.trim() == "</resources>" {
            idx += 1;
            continue;
        }

        let trimmed = line.trim();
        // Check if this line is a plurals entry that should be updated or deleted
        if trimmed.starts_with("<plurals") && trimmed.contains("name=\"") {
            if let Some(plural_name) = extract_name_from_line(trimmed) {
                if let Some(updated_plural) = plurals_map.get(&plural_name) {
                    let mut plurals_end_idx = idx;
                    let mut found_closing = false;
                    for search_idx in idx..existing_lines.len() {
                        if existing_lines[search_idx].contains("</plurals>") {
                            plurals_end_idx = search_idx;
                            found_closing = true;
                            break;
                        }
                    }
                    if found_closing {
                        let indent = line.chars().take_while(|c| c.is_whitespace()).collect::<String>();
                        let translatable_attr = if !updated_plural.is_translatable {
                            " translatable=\"false\""
                        } else { "" };
                        let mut plural_lines = vec![format!("{}<plurals name=\"{}\"{}>", indent, updated_plural.name, translatable_attr)];
                        for item in &updated_plural.items {
                            let text_content = if item.has_cdata {
                                format!("<![CDATA[{}]]>", item.text)
                            } else { item.text.clone() };
                            plural_lines.push(format!(r#"{}    <item quantity="{}">{}</item>"#, indent, item.quantity, text_content));
                        }
                        plural_lines.push(format!("{}</plurals>", indent));
                        for pl in plural_lines { new_lines.push(pl); }
                        plurals_written.insert(plural_name.clone());
                        let mut next_idx = plurals_end_idx + 1;
                        while next_idx < existing_lines.len() {
                            let next_trimmed = existing_lines[next_idx].trim();
                            if next_trimmed.starts_with("<!--") || next_trimmed.is_empty() {
                                new_lines.push(existing_lines[next_idx].clone());
                                next_idx += 1;
                            } else { break; }
                        }
                        idx = next_idx;
                        continue;
                    }
                } else if !merge_mode {
                    let mut end_idx = idx;
                    for search_idx in idx..existing_lines.len() {
                        if existing_lines[search_idx].contains("</plurals>") {
                            end_idx = search_idx;
                            break;
                        }
                    }
                    idx = end_idx + 1;
                    continue;
                }
            }
        }

        
        // Check if this line is a string entry that should be updated or deleted
        if trimmed.starts_with("<string") && trimmed.contains("name=\"") {
            if let Some(name) = extract_name_from_line(trimmed) {
                // If this string is in our map, replace it with the updated version
                if let Some(updated_string) = strings_map.get(name.as_str()) {
                    // Find the closing </string> tag (handle multi-line strings)
                    let mut string_end_idx = idx;
                    let mut found_closing = false;
                    for search_idx in idx..existing_lines.len() {
                        if existing_lines[search_idx].contains("</string>") {
                            string_end_idx = search_idx;
                            found_closing = true;
                            break;
                        }
                    }
                    
                    if found_closing {
                        // Format the updated string entry (preserve indentation from original line)
                        let indent = line.chars().take_while(|c| c.is_whitespace()).collect::<String>();
                        let translatable_attr = if !updated_string.is_translatable {
                            " translatable=\"false\""
                        } else {
                            ""
                        };
                        
                        // Check if there's a comment or trailing content on the closing tag line
                        let closing_line = &existing_lines[string_end_idx];
                        let closing_tag_pos = closing_line.find("</string>");
                        let trailing_content = if let Some(pos) = closing_tag_pos {
                            let after_closing = &closing_line[pos + 9..]; // 9 = length of "</string>"
                            // Preserve trailing content (comments, whitespace, etc.)
                            after_closing.to_string()
                        } else {
                            String::new()
                        };
                        
                        let text_content = if updated_string.has_cdata {
                            format!("<![CDATA[{}]]>", updated_string.text)
                        } else {
                            updated_string.text.clone()
                        };
                        let new_line = format!(
                            "{}<string name=\"{}\"{}>{}</string>{}",
                            indent, updated_string.name, translatable_attr, text_content, trailing_content
                        );
                        new_lines.push(new_line);
                        strings_written.insert(name.clone());
                        
                        // Preserve any comment lines that follow the string (before next string or end)
                        // Check if there are comment lines immediately after the closing tag
                        let mut next_idx = string_end_idx + 1;
                        while next_idx < existing_lines.len() {
                            let next_line = &existing_lines[next_idx];
                            let next_trimmed = next_line.trim();
                            
                            // Stop if we hit another string tag
                            if next_trimmed.starts_with("<string") && next_trimmed.contains("name=\"") {
                                break;
                            }
                            
                            // If it's a comment line, preserve it
                            if next_trimmed.starts_with("<!--") {
                                new_lines.push(next_line.clone());
                                next_idx += 1;
                            } else if next_trimmed.is_empty() {
                                // Preserve empty lines between strings and comments
                                new_lines.push(next_line.clone());
                                next_idx += 1;
                            } else if next_trimmed == "</resources>" {
                                // Stop at closing resources tag
                                break;
                            } else {
                                // Stop at first other non-comment, non-empty line
                                break;
                            }
                        }
                        
                        // Skip all lines of the old string (including closing tag and any preserved comments)
                        idx = next_idx;
                        continue;
                    }
                } else if !merge_mode {
                    // String is not in our map, and we're in replace mode, so delete it
                    // Find the closing </string> tag (handle multi-line strings)
                    let mut string_end_idx = idx;
                    let mut found_closing = false;
                    for search_idx in idx..existing_lines.len() {
                        if existing_lines[search_idx].contains("</string>") {
                            string_end_idx = search_idx;
                            found_closing = true;
                            break;
                        }
                    }
                    
                    if found_closing {
                        // Skip all lines of this string (including closing tag) - delete it
                        idx = string_end_idx + 1;
                        continue;
                    }
                }
                // In merge mode, if string is not in our map, keep it as-is (don't delete)
            }
        }

        // Keep the line as-is (preserves original formatting, indentation, order, etc.)
        new_lines.push(line.clone());
        idx += 1;
    }

    // Add any new strings that weren't in the existing file (skip flattened plural items)
    for string in strings_map.values() {
        if !strings_written.contains(&string.name) {
            let translatable_attr = if !string.is_translatable {
                " translatable=\"false\""
            } else {
                ""
            };
            let text_content = if string.has_cdata {
                format!("<![CDATA[{}]]>", string.text)
            } else {
                string.text.clone()
            };
            let new_line = format!(
                "\t<string name=\"{}\"{}>{}</string>",
                string.name, translatable_attr, text_content
            );
            new_lines.push(new_line);
        }
    }

    // Add any new plurals that weren't in the existing file
    for (name, plural) in &plurals_map {
        if !plurals_written.contains(name) {
            let translatable_attr = if !plural.is_translatable {
                " translatable=\"false\""
            } else {
                ""
            };
            new_lines.push(format!("\t<plurals name=\"{}\"{}>", plural.name, translatable_attr));
            for item in &plural.items {
                let text_content = if item.has_cdata {
                    format!("<![CDATA[{}]]>", item.text)
                } else {
                    item.text.clone()
                };
                new_lines.push(format!(
                    "\t    <item quantity=\"{}\">{}</item>",
                    item.quantity, text_content
                ));
            }
            new_lines.push("\t</plurals>".to_string());
        }
    }

    // Add closing tag
    new_lines.push("</resources>".to_string());

    // Write back to file
    fs::write(&strings_path, new_lines.join("\n"))
        .map_err(|e| format!("Failed to write file: {}", e))?;

    Ok(())
}

fn extract_name_from_line(line: &str) -> Option<String> {
    let name_start = line.find("name=\"")? + 6;
    let name_end = line[name_start..].find("\"")?;
    Some(line[name_start..name_start + name_end].to_string())
}

pub fn parse_strings_xml(xml_content: &str) -> Result<Vec<AndroidString>, String> {
    let mut strings = Vec::new();

    // First, find all XML comment blocks and their positions
    // XML comments are in the format: <!-- ... -->
    // Use (?s) flag to make . match newlines (multiline comments)
    let comment_re = Regex::new(r#"(?s)<!--.*?-->"#)
        .map_err(|e| format!("Failed to compile comment regex: {}", e))?;
    
    let mut comment_ranges: Vec<(usize, usize)> = Vec::new();
    for cap in comment_re.captures_iter(xml_content) {
        let full_match = cap.get(0).unwrap();
        comment_ranges.push((full_match.start(), full_match.end()));
    }

    // Helper function to check if a position is inside any comment
    let is_inside_comment = |pos: usize| -> bool {
        comment_ranges.iter().any(|(start, end)| pos >= *start && pos < *end)
    };

    // Regex to find <string> opening tags with name attribute
    // Handles attributes in any order (e.g., <string translatable="false" name="key"> or <string name="key" translatable="false">)
    let string_tag_re = Regex::new(r#"<string\s+([^>]*)>"#)
        .map_err(|e| format!("Failed to compile regex: {}", e))?;
    
    // Regex to extract name from attributes string
    let name_re = Regex::new(r#"name="([^"]+)""#)
        .map_err(|e| format!("Failed to compile name regex: {}", e))?;

    // Use captures_iter to get all matches with capture groups
    for cap in string_tag_re.captures_iter(xml_content) {
        let full_match = cap.get(0).unwrap();
        let tag_start = full_match.start();
        
        // Skip if this string tag is inside a comment
        if is_inside_comment(tag_start) {
            continue;
        }
        
        let attributes_match = cap.get(1).unwrap();
        let attributes = attributes_match.as_str();
        
        // Extract name from attributes (handles any attribute order)
        let name = if let Some(name_cap) = name_re.captures(attributes) {
            name_cap.get(1).unwrap().as_str().to_string()
        } else {
            continue; // Skip if no name attribute found
        };
        
        let tag_end = full_match.end();
        
        // Check if translatable="false"
        let is_translatable = !attributes.contains("translatable=\"false\"");

        // Find the matching closing </string> tag
        // Note: We don't need to worry about nested <string> tags (they shouldn't exist)
        // We just need to find the first </string> that closes this opening tag
        let content_end = xml_content[tag_end..]
            .find("</string>")
            .map(|offset| tag_end + offset);

        if let Some(end_pos) = content_end {
            // Extract the content between the opening and closing tags
            // This preserves nested XML tags like <font> inside the string content
            let mut text = xml_content[tag_end..end_pos].to_string();
            
            // Handle CDATA sections: remove CDATA markers but preserve content
            // CDATA format: <![CDATA[content]]>
            let mut has_cdata = false;
            if text.trim_start().starts_with("<![CDATA[") {
                // Find the CDATA start and end markers
                if let Some(cdata_start) = text.find("<![CDATA[") {
                    let after_cdata_start = cdata_start + 9; // Length of "<![CDATA["
                    if let Some(cdata_end) = text[after_cdata_start..].find("]]>") {
                        // Extract content between CDATA markers
                        let cdata_content = text[after_cdata_start..after_cdata_start + cdata_end].to_string();
                        text = cdata_content;
                        has_cdata = true;
                    }
                }
            }
            
            strings.push(AndroidString {
                name,
                text,
                is_translatable,
                is_commented_out: false,
                line_number: 0,
                has_cdata,
            });
        }
    }

    Ok(strings)
}

/// Find the Android project root directory
/// The project_path might point to res/ or src/main/res/
/// We need to find the actual project root that contains both res and java/kotlin source
fn find_android_project_root(project_path: &str) -> Result<PathBuf, String> {
    let mut current_path = PathBuf::from(project_path);
    
    // Normalize the path (resolve any . or .. components)
    current_path = current_path.canonicalize()
        .map_err(|e| format!("Failed to canonicalize path: {}", e))?;
    
    // If we're in a res directory, go up until we find a directory that could be the project root
    // Look for indicators like: build.gradle, settings.gradle, AndroidManifest.xml, or src/ directory
    
    let max_levels = 5; // Don't go up more than 5 levels
    let mut levels_up = 0;
    
    while levels_up < max_levels {
        // Check if current directory looks like an Android project root
        let has_build_gradle = current_path.join("build.gradle").exists() || 
                              current_path.join("build.gradle.kts").exists();
        let has_settings_gradle = current_path.join("settings.gradle").exists() ||
                                  current_path.join("settings.gradle.kts").exists();
        let has_android_manifest = current_path.join("src/main/AndroidManifest.xml").exists() ||
                                   current_path.join("app/src/main/AndroidManifest.xml").exists();
        
        // Check if there's a src directory with java/kotlin source
        let has_java_source = current_path.join("src/main/java").exists() ||
                             current_path.join("app/src/main/java").exists() ||
                             current_path.join("src/main/kotlin").exists() ||
                             current_path.join("app/src/main/kotlin").exists();
        
        // Check if there's a res directory (to confirm we're in the right place)
        let has_res = current_path.join("src/main/res").exists() ||
                     current_path.join("app/src/main/res").exists() ||
                     current_path.join("res").exists();
        
        // If we find markers of an Android project, this is likely the root
        if (has_build_gradle || has_settings_gradle || has_android_manifest) && has_res {
            eprintln!("Found Android project root at: {:?}", current_path);
            return Ok(current_path);
        }
        
        // If we find java/kotlin source and res, this is likely the project root
        if has_java_source && has_res {
            eprintln!("Found Android project root (with source) at: {:?}", current_path);
            return Ok(current_path);
        }
        
        // Go up one level
        if let Some(parent) = current_path.parent() {
            current_path = parent.to_path_buf();
            levels_up += 1;
        } else {
            break;
        }
    }
    
    // If we couldn't find a clear project root, try going up to src/main/ or app/src/main/
    // and then to the parent
    let mut fallback_path = PathBuf::from(project_path);
    fallback_path = fallback_path.canonicalize()
        .map_err(|e| format!("Failed to canonicalize path: {}", e))?;
    
    // If path contains "src/main/res", go up 2 levels to get to the module root
    if fallback_path.to_string_lossy().contains("src/main/res") {
        if let Some(_) = fallback_path.parent() {
            if let Some(parent) = fallback_path.parent().and_then(|p| p.parent()) {
                eprintln!("Using fallback project root: {:?}", parent);
                return Ok(parent.to_path_buf());
            }
        }
    }
    
    // If path contains "app/src/main/res", go up 3 levels
    if fallback_path.to_string_lossy().contains("app/src/main/res") {
        if let Some(_) = fallback_path.parent() {
            if let Some(_) = fallback_path.parent().and_then(|p| p.parent()) {
                if let Some(parent) = fallback_path.parent().and_then(|p| p.parent()).and_then(|p| p.parent()) {
                    eprintln!("Using fallback project root (app module): {:?}", parent);
                    return Ok(parent.to_path_buf());
                }
            }
        }
    }
    
    // Last resort: if path ends with "res", go up one level
    if fallback_path.ends_with("res") {
        if let Some(parent) = fallback_path.parent() {
            eprintln!("Using last resort project root: {:?}", parent);
            return Ok(parent.to_path_buf());
        }
    }
    
    // If all else fails, use the original path but warn
    eprintln!("WARNING: Could not determine Android project root, using: {:?}", fallback_path);
    Ok(fallback_path)
}

#[tauri::command]
pub fn find_unused_strings(
    project_path: String,
    app_handle: tauri::AppHandle,
) -> Result<Vec<String>, String> {
    // Load default strings to get all string keys
    let all_data = load_project_data(project_path.clone())?;
    
    let default_strings = all_data
        .strings_by_language
        .get("default")
        .ok_or("Default strings not found")?;
    
    // Create a set of default string names for fast lookup
    let default_names: std::collections::HashSet<&String> = default_strings
        .iter()
        .map(|s| &s.name)
        .collect();
    
    // Collect all string keys from default language (for checking usage)
    let mut string_keys: Vec<String> = default_strings.iter().map(|s| s.name.clone()).collect();
    
    // Also collect strings that exist in translation files but NOT in default
    // These are strings missing default values and should be considered unused
    let mut missing_default_strings = std::collections::HashSet::new();
    
    for (language, strings) in &all_data.strings_by_language {
        if language == "default" {
            continue;
        }
        
        for string in strings {
            if !default_names.contains(&string.name) {
                // This string exists in a non-default language but not in default
                missing_default_strings.insert(string.name.clone());
            }
        }
    }
    
    // Add strings missing default to the list of strings to check
    // (They're already considered unused since they can't be referenced)
    for missing_string in &missing_default_strings {
        if !string_keys.contains(missing_string) {
            string_keys.push(missing_string.clone());
        }
    }
    
    if string_keys.is_empty() {
        return Ok(vec![]);
    }
    
    // Find the actual Android project root (might need to go up from res directory)
    let search_root = find_android_project_root(&project_path)?;
    eprintln!("Searching from project root: {:?}", search_root);
    
    // Create a set of string keys for fast lookup
    let string_keys_set: HashSet<&String> = string_keys.iter().collect();
    
    // Create a set to track which strings are used
    let mut used_strings: HashSet<String> = HashSet::new();
    
    // Regex patterns for finding string references
    // Pattern for R.string.key_name in Java/Kotlin files (handles underscores, matches word boundaries better)
    // Matches: R.string.key_name, R.string.key_name), etc.
    let r_string_pattern = Regex::new(r#"\bR\.string\.([a-zA-Z_][a-zA-Z0-9_]*)"#)
        .map_err(|e| format!("Failed to compile regex: {}", e))?;
    
    // Pattern for @string/key_name in XML files
    // Matches: "@string/key_name", '@string/key_name', android:text="@string/key_name", etc.
    let xml_string_pattern = Regex::new(r#"@string/([a-zA-Z_][a-zA-Z0-9_]*)"#)
        .map_err(|e| format!("Failed to compile regex: {}", e))?;
    
    let project_path_buf = Path::new(&project_path);
    
    // Directories to skip (build outputs, dependencies, etc.)
    let skip_dirs = ["build", ".gradle", ".idea", ".git", "gradle", "node_modules", ".cargo", "target", ".git"];
    
    // Counters for debugging
    let mut files_searched = 0;
    let mut strings_found = 0;
    let mut java_files = 0;
    let mut kt_files = 0;
    let mut xml_files = 0;
    let mut skipped_dirs = std::collections::HashSet::new();
    
    // Walk through all files in the project root (not just the res directory)
    for entry in WalkDir::new(&search_root)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        
        // Skip if not a file
        if !path.is_file() {
            continue;
        }
        
        // Skip if in a directory we should ignore (build outputs, dependencies, etc.)
        let should_skip_dir = path
            .components()
            .any(|c| {
                if let std::path::Component::Normal(name) = c {
                    let name_str = name.to_string_lossy();
                    // Skip build/dependency directories (but NOT values directories)
                    if skip_dirs.iter().any(|skip| name_str == *skip) {
                        skipped_dirs.insert(name_str.to_string());
                        return true;
                    }
                }
                false
            });
        
        if should_skip_dir {
            continue;
        }
        
        // Get file extension
        let extension = path.extension().and_then(|ext| ext.to_str());
        
        // Search ALL .xml files including:
        // - arrays.xml (contains @string/ references in <item> tags)
        // - plurals.xml (contains @string/ references)
        // - strings.xml (can contain @string/ references to other strings)
        // - All other XML files (layouts, etc.)
        // The regex @string/pattern will only match actual references, not definitions
        // (definitions use name="key" not @string/key)
        
        // Only search .java, .kt, and .xml files
        let should_search = match extension {
            Some("java") | Some("kt") | Some("xml") => true,
            _ => false,
        };
        
        if !should_search {
            continue;
        }
        
        // Count file types for debugging
        match extension {
            Some("java") => java_files += 1,
            Some("kt") => kt_files += 1,
            Some("xml") => xml_files += 1,
            _ => {}
        }
        
        files_searched += 1;
        
        // Log first few files of each type for debugging
        if (extension == Some("java") && java_files <= 3) || 
           (extension == Some("kt") && kt_files <= 3) {
            let relative_path = path
                .strip_prefix(&project_path_buf)
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|_| path.to_string_lossy().to_string());
            eprintln!("Found {} file: {}", extension.unwrap(), relative_path);
        }
        
        // Emit progress event with current path being searched
        let relative_path = path
            .strip_prefix(&search_root)
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| path.to_string_lossy().to_string());
        
        // Try to emit to the main window
        if let Some(window) = app_handle.get_webview_window("main") {
            let _ = window.emit("unused-strings-search-progress", relative_path.clone());
        }
        
        // Read file content
        let content = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => {
                eprintln!("Warning: Failed to read file: {:?}", path);
                continue; // Skip files we can't read
            }
        };
        
        // Search for R.string.key_name patterns (Java/Kotlin)
        if extension == Some("java") || extension == Some("kt") {
            for cap in r_string_pattern.captures_iter(&content) {
                if let Some(key_match) = cap.get(1) {
                    let key = key_match.as_str().to_string();
                    if string_keys_set.contains(&key) {
                        if !used_strings.contains(&key) {
                            used_strings.insert(key.clone());
                            strings_found += 1;
                            eprintln!("Found usage: {} in {:?}", key, relative_path);
                        }
                    }
                }
            }
        }
        
        // Search for @string/key_name patterns (XML)
        if extension == Some("xml") {
            for cap in xml_string_pattern.captures_iter(&content) {
                if let Some(key_match) = cap.get(1) {
                    let key = key_match.as_str().to_string();
                    if string_keys_set.contains(&key) {
                        if !used_strings.contains(&key) {
                            used_strings.insert(key.clone());
                            strings_found += 1;
                            eprintln!("Found usage: {} in {:?}", key, relative_path);
                        }
                    }
                }
            }
        }
    }
    
    eprintln!("Search summary:");
    eprintln!("  Files searched: {} (Java: {}, Kotlin: {}, XML: {})", 
              files_searched, java_files, kt_files, xml_files);
    eprintln!("  String usages found: {}", strings_found);
    eprintln!("  Total strings to check: {}", string_keys.len());
    eprintln!("  Skipped directories: {:?}", skipped_dirs);
    
    if java_files == 0 && kt_files == 0 {
        eprintln!("WARNING: No Java or Kotlin files found! Searching in: {}", project_path);
        eprintln!("Consider checking if source files are in src/main/java or app/src/main/java");
    }
    
    // Find strings that are not used
    // This includes:
    // 1. Strings in default that are not referenced in code
    // 2. Strings missing from default that are also not referenced in code
    // Note: If a string missing from default IS referenced in code, we don't delete it
    // (even though it would cause a build error, the user should fix that manually)
    let mut unused_strings: Vec<String> = string_keys
        .into_iter()
        .filter(|key| {
            // First check if it's used in code - if it is, keep it regardless of default status
            if used_strings.contains(key) {
                return false; // Used in code, so not unused
            }
            // If it's not used in code, check if it's missing from default
            // Strings missing from default that aren't used in code are safe to delete
            if missing_default_strings.contains(key) {
                return true; // Missing from default AND not used in code
            }
            // Otherwise, it's in default but not used in code
            true // Not used in code
        })
        .collect();
    
    // Sort for consistent output
    unused_strings.sort();
    
    Ok(unused_strings)
}

#[tauri::command]
pub fn delete_unused_strings(
    project_path: String,
    string_names: Vec<String>,
) -> Result<(), String> {
    if string_names.is_empty() {
        return Ok(());
    }

    let names_to_delete: HashSet<&str> = string_names.iter().map(|s| s.as_str()).collect();
    let project_folder = Path::new(&project_path);

    // Delete from the default strings.xml
    let default_path = project_folder.join("values").join("strings.xml");
    delete_strings_from_file(&default_path, &names_to_delete)?;

    // Delete from all translation language files
    let languages = detect_available_languages(project_path.clone())?;
    for language in &languages {
        let lang_path = project_folder
            .join(format!("values-{}", language))
            .join("strings.xml");
        delete_strings_from_file(&lang_path, &names_to_delete)?;
    }

    Ok(())
}

