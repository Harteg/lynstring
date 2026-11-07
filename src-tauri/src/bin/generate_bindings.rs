use ts_rs::TS;

fn main() {
    // Export translation types
    lynstring_tauri_lib::commands::translation::TranslationRequest::export_all().unwrap();
    lynstring_tauri_lib::commands::translation::TranslationResult::export_all().unwrap();
    lynstring_tauri_lib::commands::translation::TranslationProgress::export_all().unwrap();
    lynstring_tauri_lib::commands::translation::TranslationProgressEvent::export_all().unwrap();
    lynstring_tauri_lib::commands::translation::TranslationCompleteEvent::export_all().unwrap();
    
    // Export strings types
    lynstring_tauri_lib::models::AndroidString::export_all().unwrap();
    lynstring_tauri_lib::commands::strings::AllStringsData::export_all().unwrap();
    
    // Export app settings types
    lynstring_tauri_lib::commands::app_settings::AppSettings::export_all().unwrap();
    lynstring_tauri_lib::commands::app_settings::ApiKeyTestResult::export_all().unwrap();

    println!("TypeScript bindings generated successfully!");
}
