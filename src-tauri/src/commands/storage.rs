use crate::models::Project;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::Manager;

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectsStorage {
    pub projects: Vec<Project>,
    pub current_project_id: Option<String>,
}

impl Default for ProjectsStorage {
    fn default() -> Self {
        Self {
            projects: Vec::new(),
            current_project_id: None,
        }
    }
}

pub fn get_storage_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    app.path()
        .app_data_dir()
        .map(|dir| dir.join("projects.json"))
        .map_err(|e| format!("Failed to get app data directory: {}", e))
}

#[tauri::command]
pub async fn save_project(app: tauri::AppHandle, project: Project) -> Result<(), String> {
    let storage_path = get_storage_path(&app)?;

    // Ensure the directory exists
    if let Some(parent) = storage_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create storage directory: {}", e))?;
    }

    // Load existing projects
    let mut storage = load_projects_storage(&storage_path)?;

    // Add or update project
    if let Some(existing) = storage.projects.iter_mut().find(|p| p.id == project.id) {
        *existing = project;
    } else {
        storage.projects.push(project);
    }

    // Save to disk
    let json = serde_json::to_string_pretty(&storage)
        .map_err(|e| format!("Failed to serialize projects: {}", e))?;

    fs::write(&storage_path, json).map_err(|e| format!("Failed to write projects file: {}", e))?;

    Ok(())
}

#[tauri::command]
pub async fn delete_project(app: tauri::AppHandle, project_id: String) -> Result<(), String> {
    let storage_path = get_storage_path(&app)?;
    let mut storage = load_projects_storage(&storage_path)?;

    // Remove the project from the list
    storage.projects.retain(|p| p.id != project_id);

    // Clear current project if it's the one being deleted
    if let Some(current_id) = &storage.current_project_id {
        if current_id == &project_id {
            storage.current_project_id = None;
        }
    }

    // Save to disk
    let json = serde_json::to_string_pretty(&storage)
        .map_err(|e| format!("Failed to serialize projects: {}", e))?;

    fs::write(&storage_path, json).map_err(|e| format!("Failed to write projects file: {}", e))?;

    Ok(())
}

#[tauri::command]
pub async fn load_projects(app: tauri::AppHandle) -> Result<Vec<Project>, String> {
    let storage_path = get_storage_path(&app)?;
    let mut storage = load_projects_storage(&storage_path)?;
    
    // Validate and fix all projects
    let mut needs_save = false;
    for project in &mut storage.projects {
        let original_id = project.id.clone();
        project.validate_and_fix();
        // If ID was fixed, we need to save
        if original_id != project.id {
            needs_save = true;
        }
    }
    
    // Save if any projects were fixed
    if needs_save {
        let json = serde_json::to_string_pretty(&storage)
            .map_err(|e| format!("Failed to serialize projects: {}", e))?;
        fs::write(&storage_path, json).map_err(|e| format!("Failed to write projects file: {}", e))?;
    }
    
    Ok(storage.projects)
}

#[tauri::command]
pub async fn set_current_project(app: tauri::AppHandle, project_id: String) -> Result<(), String> {
    let storage_path = get_storage_path(&app)?;
    let mut storage = load_projects_storage(&storage_path)?;
    storage.current_project_id = Some(project_id);

    let json = serde_json::to_string_pretty(&storage)
        .map_err(|e| format!("Failed to serialize projects: {}", e))?;

    fs::write(&storage_path, json).map_err(|e| format!("Failed to write projects file: {}", e))?;

    Ok(())
}

#[tauri::command]
pub async fn get_current_project(app: tauri::AppHandle) -> Result<Option<Project>, String> {
    let storage_path = get_storage_path(&app)?;
    let storage = load_projects_storage(&storage_path)?;

    if let Some(current_id) = storage.current_project_id {
        Ok(storage.projects.into_iter().find(|p| p.id == current_id))
    } else {
        Ok(None)
    }
}

pub fn load_projects_storage(storage_path: &PathBuf) -> Result<ProjectsStorage, String> {
    if storage_path.exists() {
        let content = fs::read_to_string(storage_path)
            .map_err(|e| format!("Failed to read projects file: {}", e))?;

        serde_json::from_str(&content).map_err(|e| format!("Failed to parse projects file: {}", e))
    } else {
        Ok(ProjectsStorage::default())
    }
}
