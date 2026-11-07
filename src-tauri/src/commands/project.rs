use crate::commands::storage::{get_storage_path, load_projects_storage};
use crate::models::Project;
use std::fs;
use std::path::{Path, PathBuf};
use tauri_plugin_dialog::DialogExt;
use uuid::Uuid;

/// Check if a directory is a valid Android res folder
fn is_res_folder(path: &Path) -> bool {
    if !path.is_dir() {
        return false;
    }

    // Check if it contains a 'values' folder
    let values_path = path.join("values");
    if !values_path.exists() || !values_path.is_dir() {
        return false;
    }

    // Check if it contains at least one 'values-*' folder (language-specific)
    match fs::read_dir(path) {
        Ok(mut entries) => entries.any(|entry| {
            if let Ok(entry) = entry {
                let name = entry.file_name();
                let name_str = name.to_string_lossy();
                name_str.starts_with("values-") && entry.path().is_dir()
            } else {
                false
            }
        }),
        Err(_) => false,
    }
}

/// Recursively search for a res folder starting from the given path
fn find_res_folder(start_path: &Path) -> Option<PathBuf> {
    // Check current directory
    if is_res_folder(start_path) {
        return Some(start_path.to_path_buf());
    }

    // Search subdirectories (depth-first)
    if let Ok(entries) = fs::read_dir(start_path) {
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.is_dir() {
                    // Skip hidden directories and common non-Android directories
                    let name = path.file_name().unwrap_or_default().to_string_lossy();
                    if name.starts_with('.')
                        || name == "build"
                        || name == "node_modules"
                        || name == ".git"
                    {
                        continue;
                    }

                    if let Some(res_path) = find_res_folder(path.as_path()) {
                        return Some(res_path);
                    }
                }
            }
        }
    }

    // Search parent directories (up to 3 levels up to avoid going too far)
    let mut current = start_path;
    for _ in 0..3 {
        if let Some(parent) = current.parent() {
            if is_res_folder(parent) {
                return Some(parent.to_path_buf());
            }
            current = parent;
        } else {
            break;
        }
    }

    None
}

#[tauri::command]
pub async fn pick_project_folder(app: tauri::AppHandle) -> Result<Option<String>, String> {
    let folder_path = app.dialog().file().blocking_pick_folder();
    match folder_path {
        Some(path) => {
            // Try to find the res folder automatically
            if let Some(res_path) = find_res_folder(path.as_path().unwrap()) {
                Ok(Some(res_path.to_string_lossy().to_string()))
            } else {
                // If no res folder found, return the original path
                // The user can manually specify the res folder path
                Ok(Some(path.to_string()))
            }
        }
        None => Ok(None), // User cancelled
    }
}

#[tauri::command]
pub fn create_project(
    name: String,
    path: String,
    context_prompt: String,
    // MVP: API key is optional - users cannot use their own API key in MVP
    // TODO: Make required again when allowing users to provide their own API keys
    api_key: Option<String>,
    protected_names: Vec<String>,
) -> Result<Project, String> {
    // Basic validation

    if name.trim().is_empty() {
        return Err("Project name cannot be empty".to_string());
    }

    if path.trim().is_empty() {
        return Err("Project path cannot be empty".to_string());
    }

    // TODO: Add more validation (check if path exists, is valid Android project, etc.)

    Ok(Project {
        id: Uuid::new_v4().to_string(),
        name,
        path,
        context_prompt,
        // MVP: Use empty string if API key not provided - not used in MVP but keep for future use
        // TODO: Make required again when allowing users to provide their own API keys
        api_key: api_key.unwrap_or_default(),
        protected_names,
        selected_languages: Vec::new(), // Initialize empty
    })
}

#[tauri::command]
pub fn detect_available_languages(project_path: String) -> Result<Vec<String>, String> {
    let project_folder = Path::new(&project_path);
    let mut result = Vec::new();

    let entries =
        fs::read_dir(project_folder).map_err(|e| format!("Failed to read directory: {}", e))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
        let path = entry.path();

        if path.is_dir() {
            if let Some(dir_name) = path.file_name().and_then(|n| n.to_str()) {
                if dir_name.starts_with("values-") && path.join("strings.xml").exists() {
                    let lang = dir_name.strip_prefix("values-").unwrap_or("");
                    // Include any valid Android locale (e.g. ja, en-rUS, ja-rJP).
                    // Display names come from get_language_name which uses CLDR fallback for unknown codes.
                    if lang.len() >= 2 && lang.len() <= 15 && lang.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
                        result.push(lang.to_string());
                    }
                }
            }
        }
    }

    result.sort();
    result.dedup();
    Ok(result)
}

#[tauri::command]
pub async fn update_project_selected_languages(
    app: tauri::AppHandle,
    project_id: String,
    selected_languages: Vec<String>,
) -> Result<(), String> {
    let storage_path = get_storage_path(&app)?;
    let mut storage = load_projects_storage(&storage_path)?;

    // Find and update the project
    if let Some(project) = storage.projects.iter_mut().find(|p| p.id == project_id) {
        project.selected_languages = selected_languages;

        // Save to disk
        let json = serde_json::to_string_pretty(&storage)
            .map_err(|e| format!("Failed to serialize projects: {}", e))?;
        fs::write(&storage_path, json)
            .map_err(|e| format!("Failed to write projects file: {}", e))?;
    } else {
        return Err(format!("Project not found: {}", project_id));
    }

    Ok(())
}

/// The standard XML preamble for Android strings.xml files
const STRINGS_XML_PREAMBLE: &str = r#"<?xml version="1.0" encoding="utf-8"?>
<resources xmlns:xliff="urn:oasis:names:tc:xliff:document:1.2">

</resources>
"#;

#[tauri::command]
pub fn add_languages_to_project(
    project_path: String,
    language_codes: Vec<String>,
) -> Result<Vec<String>, String> {
    let project_folder = Path::new(&project_path);
    
    if !project_folder.exists() {
        return Err(format!("Project path does not exist: {}", project_path));
    }

    let mut added_languages: Vec<String> = Vec::new();

    for lang_code in &language_codes {
        // Validate language code (basic check: 2-5 chars, alphanumeric with optional hyphen)
        if lang_code.len() < 2 || lang_code.len() > 10 {
            return Err(format!("Invalid language code: {}", lang_code));
        }

        let values_folder_name = format!("values-{}", lang_code);
        let values_folder_path = project_folder.join(&values_folder_name);
        let strings_xml_path = values_folder_path.join("strings.xml");

        // Check if strings.xml already exists
        if strings_xml_path.exists() {
            // Language already exists, skip it
            continue;
        }

        // Create the values-{lang} folder if it doesn't exist
        if !values_folder_path.exists() {
            fs::create_dir_all(&values_folder_path)
                .map_err(|e| format!("Failed to create folder {}: {}", values_folder_name, e))?;
        }

        // Create the strings.xml file with the preamble
        fs::write(&strings_xml_path, STRINGS_XML_PREAMBLE)
            .map_err(|e| format!("Failed to create strings.xml for {}: {}", lang_code, e))?;

        added_languages.push(lang_code.clone());
    }

    Ok(added_languages)
}

#[tauri::command]
pub fn delete_language_from_project(
    project_path: String,
    language_code: String,
) -> Result<(), String> {
    // Never allow deleting the default language
    if language_code.is_empty() || language_code == "default" {
        return Err("Cannot delete the default language".to_string());
    }

    let project_folder = Path::new(&project_path);
    if !project_folder.exists() {
        return Err(format!("Project path does not exist: {}", project_path));
    }

    let values_folder_name = format!("values-{}", language_code);
    let values_folder_path = project_folder.join(&values_folder_name);

    if !values_folder_path.exists() {
        return Err(format!(
            "Language folder does not exist: {}",
            values_folder_name
        ));
    }

    if !values_folder_path.is_dir() {
        return Err(format!("Expected directory: {}", values_folder_path.display()));
    }

    fs::remove_dir_all(&values_folder_path)
        .map_err(|e| format!("Failed to delete language folder: {}", e))?;

    Ok(())
}
