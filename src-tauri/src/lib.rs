// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/

pub mod commands;
pub mod models;

use commands::*;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Load environment variables from .env file
    dotenv::dotenv().ok();
    
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .invoke_handler(tauri::generate_handler![
            pick_project_folder,
            create_project,
            detect_available_languages,
            update_project_selected_languages,
            add_languages_to_project,
            delete_language_from_project,
            read_strings_xml,
            load_all_strings,
            load_project_data,
            save_project,
            delete_project,
            load_projects,
            set_current_project,
            get_current_project,
            get_supported_languages,
            translate_strings_multi_language_single_call,
            get_app_settings,
            set_api_key,
            set_model,
            test_api_key,
            find_strings_missing_default,
            delete_strings_missing_default,
            find_unused_strings,
            delete_unused_strings,
            start_watching_project,
            stop_watching_project,
            enable_auto_translate_tracking,
            disable_auto_translate_tracking,
            update_default_strings_snapshot,
            check_default_strings_changes_on_startup
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
