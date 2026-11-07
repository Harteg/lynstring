use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::Manager;
use ts_rs::TS;

pub const DEFAULT_MODEL: &str = "claude-sonnet-5-5";
pub const ANTHROPIC_MESSAGES_URL: &str = "https://api.anthropic.com/v1/messages";
pub const ANTHROPIC_VERSION: &str = "2026-10-21";
const API_KEY_ENV: &str = "ANTHROPIC_API_KEY";

/// What is stored on disk in the app data directory (settings.json).
/// The API key is stored in plain text, readable only by the current user.
#[derive(Debug, Serialize, Deserialize, Default)]
struct StoredSettings {
    #[serde(default)]
    api_key: String,
    #[serde(default)]
    model: String,
}

/// Settings as shown to the frontend. The key itself is never sent back, only a hint.
#[derive(Debug, Serialize, TS)]
#[ts(export)]
pub struct AppSettings {
    pub has_api_key: bool,
    /// Last characters of the key, e.g. "…a1B2", for display
    pub api_key_hint: String,
    /// True when no key is saved and ANTHROPIC_API_KEY from the environment is used
    pub api_key_from_env: bool,
    pub model: String,
}

#[derive(Debug, Serialize, TS)]
#[ts(export)]
pub struct ApiKeyTestResult {
    pub success: bool,
    pub message: String,
}

fn settings_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    app.path()
        .app_data_dir()
        .map(|dir| dir.join("settings.json"))
        .map_err(|e| format!("Failed to get app data directory: {}", e))
}

fn load_stored(app: &tauri::AppHandle) -> Result<StoredSettings, String> {
    let path = settings_path(app)?;
    if !path.exists() {
        return Ok(StoredSettings::default());
    }
    let json = fs::read_to_string(&path).map_err(|e| format!("Failed to read settings: {}", e))?;
    serde_json::from_str(&json).map_err(|e| format!("Failed to parse settings: {}", e))
}

fn save_stored(app: &tauri::AppHandle, settings: &StoredSettings) -> Result<(), String> {
    let path = settings_path(app)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Failed to create settings directory: {}", e))?;
    }
    let json = serde_json::to_string_pretty(settings).map_err(|e| format!("Failed to serialize settings: {}", e))?;
    fs::write(&path, json).map_err(|e| format!("Failed to write settings: {}", e))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(&path, fs::Permissions::from_mode(0o600));
    }
    Ok(())
}

/// The API key to use: the saved one, or ANTHROPIC_API_KEY from the environment.
pub fn get_api_key(app: &tauri::AppHandle) -> Result<String, String> {
    let stored = load_stored(app)?;
    if !stored.api_key.trim().is_empty() {
        return Ok(stored.api_key.trim().to_string());
    }
    std::env::var(API_KEY_ENV)
        .ok()
        .filter(|k| !k.trim().is_empty())
        .ok_or_else(|| "No Anthropic API key set. Add one in Settings.".to_string())
}

/// The model to translate with (saved choice or the default).
pub fn get_model(app: &tauri::AppHandle) -> String {
    load_stored(app)
        .ok()
        .map(|s| s.model)
        .filter(|m| !m.trim().is_empty())
        .unwrap_or_else(|| DEFAULT_MODEL.to_string())
}

fn key_hint(key: &str) -> String {
    let chars: Vec<char> = key.chars().collect();
    let tail: String = chars[chars.len().saturating_sub(4)..].iter().collect();
    format!("…{}", tail)
}

#[tauri::command]
pub async fn get_app_settings(app: tauri::AppHandle) -> Result<AppSettings, String> {
    let stored = load_stored(&app)?;
    let saved_key = stored.api_key.trim().to_string();
    let env_key = std::env::var(API_KEY_ENV).ok().filter(|k| !k.trim().is_empty());
    let active_key = if saved_key.is_empty() { env_key.clone().unwrap_or_default() } else { saved_key.clone() };
    Ok(AppSettings {
        has_api_key: !active_key.is_empty(),
        api_key_hint: if active_key.is_empty() { String::new() } else { key_hint(&active_key) },
        api_key_from_env: saved_key.is_empty() && env_key.is_some(),
        model: get_model(&app),
    })
}

/// Save the API key. An empty string removes the saved key.
#[tauri::command]
pub async fn set_api_key(app: tauri::AppHandle, api_key: String) -> Result<(), String> {
    let mut stored = load_stored(&app)?;
    stored.api_key = api_key.trim().to_string();
    save_stored(&app, &stored)
}

#[tauri::command]
pub async fn set_model(app: tauri::AppHandle, model: String) -> Result<(), String> {
    let mut stored = load_stored(&app)?;
    stored.model = model.trim().to_string();
    save_stored(&app, &stored)
}

/// Send a tiny request to check that a key works with the chosen model.
/// Uses the given key, or the active one when `api_key` is None.
#[tauri::command]
pub async fn test_api_key(
    app: tauri::AppHandle,
    api_key: Option<String>,
    model: Option<String>,
) -> Result<ApiKeyTestResult, String> {
    let key = match api_key.filter(|k| !k.trim().is_empty()) {
        Some(k) => k.trim().to_string(),
        None => get_api_key(&app)?,
    };
    let model = model.filter(|m| !m.trim().is_empty()).unwrap_or_else(|| get_model(&app));

    let response = reqwest::Client::new()
        .post(ANTHROPIC_MESSAGES_URL)
        .header("x-api-key", &key)
        .header("anthropic-version", ANTHROPIC_VERSION)
        .json(&serde_json::json!({
            "model": model,
            "max_tokens": 16,
            "output_config": { "effort": "low" },
            "messages": [{ "role": "user", "content": "Reply with OK." }]
        }))
        .send()
        .await
        .map_err(|e| format!("Could not reach the Anthropic API: {}", e))?;

    let status = response.status();
    if status.is_success() {
        return Ok(ApiKeyTestResult { success: true, message: "API key works".to_string() });
    }
    let body: serde_json::Value = response.json().await.unwrap_or_default();
    let detail = body
        .pointer("/error/message")
        .and_then(|m| m.as_str())
        .unwrap_or("unknown error")
        .to_string();
    let message = match status.as_u16() {
        401 => "Invalid API key".to_string(),
        403 => format!("This key isn't allowed to use {}: {}", model, detail),
        404 => format!("Model {} not found", model),
        _ => format!("Anthropic API error ({}): {}", status.as_u16(), detail),
    };
    Ok(ApiKeyTestResult { success: false, message })
}
