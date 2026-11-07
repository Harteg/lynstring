use std::collections::HashSet;
use std::sync::{Arc, Mutex};

// Auto-translate tracking state
#[derive(Clone)]
pub struct AutoTranslateState {
    pub enabled: bool,
    pub tracked_string_names: HashSet<String>,
    pub project_path: Option<String>,
}

type AutoTranslateStateType = Arc<Mutex<AutoTranslateState>>;
static AUTO_TRANSLATE_STATE: std::sync::OnceLock<AutoTranslateStateType> = std::sync::OnceLock::new();

pub fn get_auto_translate_state() -> &'static AutoTranslateStateType {
    AUTO_TRANSLATE_STATE.get_or_init(|| {
        Arc::new(Mutex::new(AutoTranslateState {
            enabled: false,
            tracked_string_names: HashSet::new(),
            project_path: None,
        }))
    })
}

