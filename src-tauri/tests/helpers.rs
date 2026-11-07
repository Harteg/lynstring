use lynstring_tauri_lib::commands::strings::{parse_strings_xml, write_strings_to_file};
use lynstring_tauri_lib::models::AndroidString;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

/// Load a fixture project path
pub fn fixture_path(relative_path: &str) -> PathBuf {
    PathBuf::from("tests/fixtures").join(relative_path)
}

/// Load strings.xml from a file path
pub fn load_strings_xml(path: &Path) -> Result<Vec<AndroidString>, String> {
    let content = fs::read_to_string(path)
        .map_err(|e| format!("Failed to read file {}: {}", path.display(), e))?;
    parse_strings_xml(&content)
}

/// Write strings to a file
#[allow(dead_code)] // Used by tests conditionally
pub fn write_strings_xml(strings: &[AndroidString], project_path: &Path, language: &str) -> Result<(), String> {
    let project_path_str = project_path.to_str()
        .ok_or_else(|| "Path contains invalid UTF-8".to_string())?
        .to_string();
    
    write_strings_to_file(strings, &project_path_str, language)
}

/// Write strings to a file in merge mode (preserves existing strings not in the list)
#[allow(dead_code)] // Used by tests conditionally
pub fn write_strings_xml_merge(strings: &[AndroidString], project_path: &Path, language: &str) -> Result<(), String> {
    // Import the test-only public function for merge mode
    // This function is only available when compiling tests
    #[cfg(test)]
    {
        // Access the function through the module path
        let project_path_str = project_path.to_str()
            .ok_or_else(|| "Path contains invalid UTF-8".to_string())?
            .to_string();
        lynstring_tauri_lib::commands::strings::write_strings_to_file_merge_public(strings, &project_path_str, language)
    }
    #[cfg(not(test))]
    {
        // This should never be called outside tests
        unreachable!("write_strings_xml_merge should only be used in tests")
    }
}

/// Compare two files, normalizing line endings
#[allow(dead_code)] // Used by tests conditionally (for golden file comparisons)
pub fn compare_with_golden(actual_path: &Path, golden_path: &Path) -> Result<(), String> {
    let actual = fs::read_to_string(actual_path)
        .map_err(|e| format!("Failed to read actual file {}: {}", actual_path.display(), e))?;
    let expected = fs::read_to_string(golden_path)
        .map_err(|e| format!("Failed to read golden file {}: {}", golden_path.display(), e))?;
    
    // Normalize line endings to \n
    let actual_normalized = actual.replace("\r\n", "\n").replace("\r", "\n");
    let expected_normalized = expected.replace("\r\n", "\n").replace("\r", "\n");
    
    if actual_normalized != expected_normalized {
        return Err(format!(
            "Files differ:\nActual: {}\nExpected: {}\n\nActual content:\n{}\n\nExpected content:\n{}",
            actual_path.display(),
            golden_path.display(),
            actual_normalized,
            expected_normalized
        ));
    }
    
    Ok(())
}

/// Create a temporary copy of a fixture project
#[allow(dead_code)] // Used by tests conditionally
pub fn create_temp_project(fixture_name: &str) -> Result<(TempDir, PathBuf), String> {
    let fixture_dir = fixture_path(fixture_name);
    if !fixture_dir.exists() {
        return Err(format!("Fixture directory not found: {}", fixture_dir.display()));
    }
    
    let temp_dir = TempDir::new()
        .map_err(|e| format!("Failed to create temp directory: {}", e))?;
    
    // Copy fixture to temp directory
    copy_dir_all(&fixture_dir, temp_dir.path())
        .map_err(|e| format!("Failed to copy fixture: {}", e))?;
    
    let project_path = temp_dir.path().to_path_buf();
    Ok((temp_dir, project_path))
}

/// Recursively copy a directory
#[allow(dead_code)] // Used by create_temp_project
fn copy_dir_all(src: &Path, dst: &Path) -> Result<(), String> {
    fs::create_dir_all(dst)
        .map_err(|e| format!("Failed to create destination directory: {}", e))?;
    
    for entry in fs::read_dir(src)
        .map_err(|e| format!("Failed to read source directory: {}", e))? {
        let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
        let path = entry.path();
        let name = entry.file_name();
        
        if path.is_dir() {
            copy_dir_all(&path, &dst.join(&name))?;
        } else {
            fs::copy(&path, dst.join(&name))
                .map_err(|e| format!("Failed to copy file {:?}: {}", name, e))?;
        }
    }
    
    Ok(())
}

/// Find the res directory in an Android project structure
#[allow(dead_code)] // Used by get_res_directory_path
pub fn find_res_directory(project_path: &Path) -> Result<PathBuf, String> {
    // Try common Android project structures
    let possible_paths = vec![
        project_path.join("app/src/main/res"),
        project_path.join("src/main/res"),
        project_path.join("res"),
        project_path.to_path_buf(), // In case project_path already points to res
    ];
    
    for path in &possible_paths {
        if path.exists() && path.is_dir() {
            // Verify it's actually a res directory by checking for values folder
            if path.join("values").exists() || path.join("values-es").exists() || path.join("values-fr").exists() {
                return Ok(path.clone());
            }
        }
    }
    
    Err(format!("Could not find res directory in project: {}", project_path.display()))
}

/// Get the strings.xml path for a language in a project
#[allow(dead_code)] // Used by tests conditionally
pub fn get_strings_xml_path(project_path: &Path, language: &str) -> PathBuf {
    let values_folder = if language.is_empty() || language == "default" {
        "values".to_string()
    } else {
        format!("values-{}", language)
    };
    
    // Find the res directory first
    if let Ok(res_dir) = find_res_directory(project_path) {
        return res_dir.join(&values_folder).join("strings.xml");
    }
    
    // Fallback: try common Android project structures
    let possible_paths = vec![
        project_path.join("app/src/main/res").join(&values_folder).join("strings.xml"),
        project_path.join("src/main/res").join(&values_folder).join("strings.xml"),
        project_path.join("res").join(&values_folder).join("strings.xml"),
        project_path.join(&values_folder).join("strings.xml"),
    ];
    
    for path in &possible_paths {
        if path.exists() {
            return path.clone();
        }
    }
    
    // Return the first possible path (will be created if needed)
    possible_paths[0].clone()
}

/// Get the project path that points to the res directory (for use with read_strings_xml, etc.)
#[allow(dead_code)] // Used by tests conditionally
pub fn get_res_directory_path(project_path: &Path) -> PathBuf {
    find_res_directory(project_path).unwrap_or_else(|_| {
        // Fallback: assume project_path already points to res or try common structures
        if project_path.join("values").exists() {
            project_path.to_path_buf()
        } else {
            project_path.join("app/src/main/res")
        }
    })
}

/// Scan code files (Java, Kotlin, XML) for string resource references
/// Uses the same regex patterns as the actual find_unused_strings implementation
/// Returns a HashSet of string names that are referenced in the code
/// 
/// Patterns matched (same as find_unused_strings):
/// - `R.string.key_name` (Java/Kotlin) - uses word boundary pattern
/// - `@string/key_name` (XML layouts)
#[allow(dead_code)] // Used by tests conditionally
pub fn scan_code_for_string_references(project_path: &Path) -> Result<HashSet<String>, String> {
    use walkdir::WalkDir;
    
    let mut referenced_strings = HashSet::new();
    
    // Use the SAME regex patterns as find_unused_strings in strings.rs
    // Pattern for R.string.key_name in Java/Kotlin files (handles underscores, matches word boundaries better)
    let r_string_pattern = regex::Regex::new(r#"\bR\.string\.([a-zA-Z_][a-zA-Z0-9_]*)"#)
        .map_err(|e| format!("Failed to compile regex: {}", e))?;
    
    // Pattern for @string/key_name in XML files
    let xml_string_pattern = regex::Regex::new(r#"@string/([a-zA-Z_][a-zA-Z0-9_]*)"#)
        .map_err(|e| format!("Failed to compile regex: {}", e))?;
    
    // Directories to skip (same as find_unused_strings)
    let skip_dirs = ["build", ".gradle", ".idea", ".git", "gradle", "node_modules", ".cargo", "target"];
    
    // Walk through all files in the project (same approach as find_unused_strings)
    for entry in WalkDir::new(project_path)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        
        // Skip if not a file
        if !path.is_file() {
            continue;
        }
        
        // Skip if in a directory we should ignore
        let should_skip_dir = path
            .components()
            .any(|c| {
                if let std::path::Component::Normal(name) = c {
                    let name_str = name.to_string_lossy();
                    skip_dirs.iter().any(|skip| name_str == *skip)
                } else {
                    false
                }
            });
        
        if should_skip_dir {
            continue;
        }
        
        // Only search .java, .kt, and .xml files (same as find_unused_strings)
        let extension = path.extension().and_then(|ext| ext.to_str());
        let should_search = match extension {
            Some("java") | Some("kt") | Some("xml") => true,
            _ => false,
        };
        
        if !should_search {
            continue;
        }
        
        // Read file content
        let content = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => continue, // Skip files we can't read
        };
        
        // Search for R.string.key_name patterns (Java/Kotlin)
        for cap in r_string_pattern.captures_iter(&content) {
            if let Some(name) = cap.get(1) {
                referenced_strings.insert(name.as_str().to_string());
            }
        }
        
        // Search for @string/key_name patterns (XML)
        for cap in xml_string_pattern.captures_iter(&content) {
            if let Some(name) = cap.get(1) {
                referenced_strings.insert(name.as_str().to_string());
            }
        }
    }
    
    Ok(referenced_strings)
}


