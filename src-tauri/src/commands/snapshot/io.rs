use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};
use crate::commands::snapshot::{ProjectSnapshot, create_empty_snapshot, project_path_hash};

/// Get snapshot file path in app data directory
pub fn get_snapshot_path(app: &AppHandle, project_path: &str) -> Result<PathBuf, String> {
    let app_data_dir = app.path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data directory: {}", e))?;
    
    let hash = project_path_hash(project_path);
    Ok(app_data_dir.join("snapshots").join(format!("{}.json", hash)))
}

/// Load snapshot from disk
pub fn load_snapshot(app: &AppHandle, project_path: &str) -> ProjectSnapshot {
    let snapshot_path = match get_snapshot_path(app, project_path) {
        Ok(path) => path,
        Err(_) => return create_empty_snapshot(),
    };
    
    if snapshot_path.exists() {
        if let Ok(content) = fs::read_to_string(&snapshot_path) {
            if let Ok(snapshot) = serde_json::from_str::<ProjectSnapshot>(&content) {
                return snapshot;
            }
        }
    }
    
    create_empty_snapshot()
}

/// Save snapshot to disk
pub fn save_snapshot(app: &AppHandle, project_path: &str, snapshot: &ProjectSnapshot) -> Result<(), String> {
    let snapshot_path = get_snapshot_path(app, project_path)?;
    
    // Create directory if it doesn't exist
    if let Some(parent) = snapshot_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create snapshot directory: {}", e))?;
    }
    
    let json = serde_json::to_string_pretty(snapshot)
        .map_err(|e| format!("Failed to serialize snapshot: {}", e))?;
    
    fs::write(&snapshot_path, json)
        .map_err(|e| format!("Failed to write snapshot: {}", e))?;
    
    Ok(())
}

