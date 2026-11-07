use std::path::{Path, PathBuf};
use std::fs;
use std::collections::HashSet;
use tauri::AppHandle;
use notify::{Watcher, RecommendedWatcher, RecursiveMode, Event, EventKind};
use tokio::sync::mpsc;
use tokio::time::Duration;
use crate::commands::strings::parse_strings_xml;
use crate::commands::snapshot::{
    add_new_strings_to_snapshot, create_snapshot_from_strings, get_project_snapshots, load_snapshot,
    save_snapshot, update_snapshot_with_current_strings
};
use crate::commands::file_watcher::{get_watcher_state, get_file_change_sender, handle_file_change, is_strings_xml_file, is_default_strings_xml, start_polling, stop_polling, is_polling_active};
use crate::commands::file_watcher::change_detector::check_for_default_strings_changes;
use crate::commands::events::emit_strings_file_changed;
use crate::commands::auto_translate::get_auto_translate_state;

/// Get all language folders in the project directory
fn get_language_folders(project_path: &Path) -> HashSet<String> {
    let mut result = HashSet::new();
    
    if let Ok(entries) = fs::read_dir(project_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if let Some(dir_name) = path.file_name().and_then(|n| n.to_str()) {
                    if dir_name.starts_with("values-") && path.join("strings.xml").exists() {
                        result.insert(dir_name.to_string());
                    }
                }
            }
        }
    }
    
    result
}

#[tauri::command]
pub async fn start_watching_project(app: AppHandle, project_path: String) -> Result<(), String> {
    // Stop any existing watcher first
    stop_watching_project().await?;
    
    // Canonicalize the path to ensure consistent snapshot keys
    let project_path_buf = PathBuf::from(&project_path)
        .canonicalize()
        .map_err(|e| format!("Failed to canonicalize project path: {}", e))?;
    let project_path = project_path_buf.to_string_lossy().to_string();
    
    if !project_path_buf.exists() {
        return Err(format!("Project path does not exist: {}", project_path));
    }
    
    // Create a channel for file change events
    let (tx, mut rx) = mpsc::unbounded_channel::<PathBuf>();
    let sender = get_file_change_sender();
    *sender.lock().unwrap() = Some(tx);
    
    // Initialize snapshot for this project (load from disk if exists, otherwise create new)
    let snapshots = get_project_snapshots();
    let default_strings_path = project_path_buf.join("values").join("strings.xml");
    let mut snapshot_existed = false;
    
    if default_strings_path.exists() {
        // Try to load from disk first
        let snapshot = load_snapshot(&app, &project_path);
        
        if snapshot.strings.is_empty() {
            // No snapshot on disk, create new one from current file
            if let Ok(xml_content) = fs::read_to_string(&default_strings_path) {
                if let Ok(strings) = parse_strings_xml(&xml_content) {
                    let new_snapshot = create_snapshot_from_strings(&strings);
                    let mut snapshots_guard = snapshots.lock().unwrap();
                    snapshots_guard.insert(project_path.clone(), new_snapshot.clone());
                    // Save to disk
                    drop(snapshots_guard);
                    if let Err(e) = save_snapshot(&app, &project_path, &new_snapshot) {
                        eprintln!("Failed to save initial snapshot: {}", e);
                    }
                }
            }
        } else {
            // Loaded from disk, store in memory
            snapshot_existed = true;
            let mut snapshots_guard = snapshots.lock().unwrap();
            snapshots_guard.insert(project_path.clone(), snapshot);
        }
    }
    
    // Spawn a tokio task to handle debouncing and emitting events
    let app_clone = app.clone();
    let project_path_clone = project_path.clone(); // Already canonicalized above
    let project_path_for_polling = project_path_buf.clone();
    let app_for_polling = app.clone();
    
    // Spawn a separate task to periodically poll for language folder changes
    // This is a fallback for when FSEvents doesn't report folder deletions
    start_polling();
    tokio::spawn(async move {
        let mut known_languages = get_language_folders(&project_path_for_polling);
        
        while is_polling_active() {
            // Poll every 2 seconds
            tokio::time::sleep(Duration::from_secs(2)).await;
            
            if !is_polling_active() {
                break;
            }
            
            let current_languages = get_language_folders(&project_path_for_polling);
            if current_languages != known_languages {
                known_languages = current_languages;
                let empty_paths: Vec<String> = Vec::new();
                emit_strings_file_changed(&app_for_polling, &empty_paths);
            }
        }
    });
    
    tokio::spawn(async move {
        let mut pending_paths = std::collections::HashSet::new();
        let mut debounce_timer: Option<tokio::time::Sleep> = None;
        
        loop {
            tokio::select! {
                // Receive file change events
                path = rx.recv() => {
                    match path {
                        Some(path) => {
                            pending_paths.insert(path);
                            
                            // Always use debounce timer - don't emit immediately
                            // This ensures we batch all rapid file changes together
                            if debounce_timer.is_none() {
                                // Start a new debounce timer (200ms for responsiveness)
                                debounce_timer = Some(tokio::time::sleep(Duration::from_millis(200)));
                            }
                            // If timer already exists, just add to pending_paths and let timer fire
                        }
                        None => {
                            // Channel closed, emit any pending changes and exit
                            if !pending_paths.is_empty() {
                                emit_file_changes(&app_clone, &mut pending_paths, &project_path_clone);
                            }
                            break;
                        }
                    }
                }
                // Handle debounce timer
                _ = async {
                    if let Some(timer) = debounce_timer.take() {
                        timer.await;
                    } else {
                        std::future::pending().await
                    }
                }, if debounce_timer.is_some() => {
                    // Timer fired - emit all pending changes
                    emit_file_changes(&app_clone, &mut pending_paths, &project_path_clone);
                    debounce_timer = None; // Clear timer after emitting
                }
            }
        }
    });
    
    // Create a new watcher
    let watcher_state = get_watcher_state();
    
    let app_for_watcher = app.clone();
    
    // Track known language folders to detect deletions
    let known_languages = std::sync::Arc::new(std::sync::Mutex::new(
        get_language_folders(&project_path_buf)
    ));
    let known_languages_clone = known_languages.clone();
    let project_path_for_check = project_path_buf.clone();
    
    let mut watcher = RecommendedWatcher::new(
        move |result: Result<Event, notify::Error>| {
            match result {
                Ok(event) => {
                    match event.kind {
                        EventKind::Modify(_) | EventKind::Create(_) => {
                            // Process modify and create events for strings.xml files
                            for path in &event.paths {
                                if is_strings_xml_file(path) {
                                    handle_file_change(path.clone());
                                }
                            }
                            
                            // Check if new language folders were added
                            let current_languages = get_language_folders(&project_path_for_check);
                            let mut known = known_languages_clone.lock().unwrap();
                            if current_languages != *known {
                                *known = current_languages;
                                let empty_paths: Vec<String> = Vec::new();
                                emit_strings_file_changed(&app_for_watcher, &empty_paths);
                            }
                        }
                        EventKind::Remove(_) => {
                            // Check if language folders were removed
                            let current_languages = get_language_folders(&project_path_for_check);
                            let mut known = known_languages_clone.lock().unwrap();
                            if current_languages != *known {
                                *known = current_languages;
                                let empty_paths: Vec<String> = Vec::new();
                                emit_strings_file_changed(&app_for_watcher, &empty_paths);
                            }
                        }
                        _ => {
                            // For any other event, check if language folders changed
                            let current_languages = get_language_folders(&project_path_for_check);
                            let mut known = known_languages_clone.lock().unwrap();
                            if current_languages != *known {
                                *known = current_languages;
                                let empty_paths: Vec<String> = Vec::new();
                                emit_strings_file_changed(&app_for_watcher, &empty_paths);
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("File watcher error: {}", e);
                }
            }
        },
        notify::Config::default()
            .with_poll_interval(Duration::from_millis(500))
            .with_compare_contents(true),
    )
    .map_err(|e| format!("Failed to create file watcher: {}", e))?;
    
    // Watch the project directory recursively
    watcher
        .watch(&project_path_buf, RecursiveMode::Recursive)
        .map_err(|e| format!("Failed to start watching project: {}", e))?;
    
    // Store the watcher
    *watcher_state.lock().unwrap() = Some(watcher);
    
    // Check for existing changes on startup (only if snapshot already existed)
    // This ensures changes made while the app was closed are detected when the app starts
    if snapshot_existed && default_strings_path.exists() {
        // Check for changes and emit event if found
        // This will notify the frontend about any existing changes
        check_for_default_strings_changes(&app, &default_strings_path, &project_path);
    }
    
    Ok(())
}

/// Emit file changes to the frontend
fn emit_file_changes(
    app: &AppHandle,
    pending_paths: &mut std::collections::HashSet<PathBuf>,
    project_path: &str,
) {
    if !pending_paths.is_empty() {
        // Check for changes in default strings.xml (always active, not just for auto-translate)
        // Only check if the default strings.xml file is actually in the changed files
        let default_strings_path = pending_paths.iter()
            .find(|p| is_default_strings_xml(p));
        
        let had_default_strings_changes = if let Some(path) = default_strings_path {
            // Only check if default strings.xml actually changed
            check_for_default_strings_changes(app, path, project_path)
        } else {
            false // No default strings.xml in this batch, skip change detection
        };
        
        // Only emit "strings-file-changed" if:
        // 1. There were actual changes in default strings.xml, OR
        // 2. There are other files that changed (not just default strings.xml), OR
        // 3. Only default strings.xml is in the list (this indicates a language folder deletion triggered a reload)
        let has_other_files = pending_paths.iter().any(|p| !is_default_strings_xml(p));
        let only_default_strings = pending_paths.len() == 1 && default_strings_path.is_some();
        let should_emit = had_default_strings_changes || has_other_files || only_default_strings;
        
        if should_emit {
            let paths: Vec<String> = pending_paths
                .iter()
                .map(|p| p.to_string_lossy().to_string())
                .collect();
            
            emit_strings_file_changed(app, &paths);
        }
        
        pending_paths.clear();
    }
}

#[tauri::command]
pub async fn stop_watching_project() -> Result<(), String> {
    // Stop the polling task
    stop_polling();
    
    let watcher_state = get_watcher_state();
    let mut watcher_guard = watcher_state.lock().unwrap();
    
    if let Some(watcher) = watcher_guard.take() {
        drop(watcher); // Dropping the watcher stops it
    }
    
    // Close the channel (this will cause the tokio task to exit)
    let sender = get_file_change_sender();
    *sender.lock().unwrap() = None;
    
    Ok(())
}

#[tauri::command]
pub async fn enable_auto_translate_tracking(project_path: String) -> Result<(), String> {
    // Canonicalize the path to ensure consistency
    let project_path_buf = PathBuf::from(&project_path)
        .canonicalize()
        .map_err(|e| format!("Failed to canonicalize project path: {}", e))?;
    let project_path = project_path_buf.to_string_lossy().to_string();
    
    let auto_translate_state = get_auto_translate_state();
    let mut state = auto_translate_state.lock().unwrap();
    
    state.enabled = true;
    state.project_path = Some(project_path.clone());
    
    // Initialize tracked strings by reading current default strings.xml
    let default_strings_path = project_path_buf.join("values").join("strings.xml");
    
    if default_strings_path.exists() {
        let xml_content = fs::read_to_string(&default_strings_path)
            .map_err(|e| format!("Failed to read default strings.xml: {}", e))?;
        
        let strings = parse_strings_xml(&xml_content)
            .map_err(|e| format!("Failed to parse default strings.xml: {}", e))?;
        
        // Track all current translatable string names
        state.tracked_string_names = strings
            .iter()
            .filter(|s| s.is_translatable)
            .map(|s| s.name.clone())
            .collect();
    }
    
    Ok(())
}

#[tauri::command]
pub async fn disable_auto_translate_tracking() -> Result<(), String> {
    let auto_translate_state = get_auto_translate_state();
    let mut state = auto_translate_state.lock().unwrap();
    
    state.enabled = false;
    state.tracked_string_names.clear();
    state.project_path = None;
    
    Ok(())
}

/// Update the default strings snapshot.
///
/// Modes:
/// - `full` (default): Replace snapshot with current file state. Use when user acknowledges
///   changes (Update/Ignore). Includes optional ignored_strings for Ignore flow.
/// - `add_new_only`: Add strings not in snapshot without touching existing entries. Use after
///   translation so new strings become baseline while keeping modified strings detectable.
#[tauri::command]
pub async fn update_default_strings_snapshot(
    app: AppHandle,
    project_path: String,
    ignored_strings: Option<Vec<String>>,
    mode: Option<String>,
) -> Result<(), String> {
    let project_path_buf = PathBuf::from(&project_path)
        .canonicalize()
        .map_err(|e| format!("Failed to canonicalize project path: {}", e))?;
    let project_path = project_path_buf.to_string_lossy().to_string();

    let default_strings_path = project_path_buf.join("values").join("strings.xml");

    if !default_strings_path.exists() {
        return Err("Default strings.xml not found".to_string());
    }

    let xml_content = fs::read_to_string(&default_strings_path)
        .map_err(|e| format!("Failed to read default strings.xml: {}", e))?;

    let strings = parse_strings_xml(&xml_content)
        .map_err(|e| format!("Failed to parse default strings.xml: {}", e))?;

    let snapshots = get_project_snapshots();
    let snapshot = {
        let snapshots_guard = snapshots.lock().unwrap();
        snapshots_guard.get(&project_path).cloned().unwrap_or_else(|| {
            load_snapshot(&app, &project_path)
        })
    };

    let updated_snapshot = match mode.as_deref() {
        Some("add_new_only") => add_new_strings_to_snapshot(&snapshot, &strings),
        _ => update_snapshot_with_current_strings(
            &snapshot,
            &strings,
            &ignored_strings.unwrap_or_default(),
        ),
    };

    {
        let mut snapshots_guard = snapshots.lock().unwrap();
        snapshots_guard.insert(project_path.clone(), updated_snapshot.clone());
    }

    save_snapshot(&app, &project_path, &updated_snapshot)?;

    Ok(())
}

/// Check for changes in default strings.xml on startup
/// This detects changes that happened while the app was closed
#[tauri::command]
pub async fn check_default_strings_changes_on_startup(
    app: AppHandle,
    project_path: String,
) -> Result<(), String> {
    // Canonicalize the path to ensure consistent snapshot keys
    let project_path_buf = PathBuf::from(&project_path)
        .canonicalize()
        .map_err(|e| format!("Failed to canonicalize project path: {}", e))?;
    let project_path = project_path_buf.to_string_lossy().to_string();
    
    let default_strings_path = project_path_buf.join("values").join("strings.xml");
    
    if !default_strings_path.exists() {
        return Ok(()); // No default strings file, nothing to check
    }
    
    // Check if a snapshot exists (if not, this is first load and we don't want to show changes)
    let snapshot = load_snapshot(&app, &project_path);
    if snapshot.strings.is_empty() {
        return Ok(()); // No snapshot exists, this is first load
    }
    
    // Ensure snapshot is in memory
    let snapshots = get_project_snapshots();
    {
        let mut snapshots_guard = snapshots.lock().unwrap();
        if !snapshots_guard.contains_key(&project_path) {
            snapshots_guard.insert(project_path.clone(), snapshot);
        }
    }
    
    // Check for changes
    check_for_default_strings_changes(&app, &default_strings_path, &project_path);
    
    Ok(())
}

