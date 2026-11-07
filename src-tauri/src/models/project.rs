use serde::{Deserialize, Serialize};
use ts_rs::TS;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Project {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub path: String,
    #[serde(default)]
    pub context_prompt: String,
    #[serde(default)]
    pub api_key: String,
    #[serde(default)]
    pub protected_names: Vec<String>,
    #[serde(default)]
    pub selected_languages: Vec<String>,
}

impl Project {
    /// Validates and fixes the project if needed. Returns true if the project is valid after fixes.
    pub fn validate_and_fix(&mut self) -> bool {
        // Auto-generate ID if missing
        if self.id.is_empty() {
            self.id = Uuid::new_v4().to_string();
        }
        
        // Return true if now valid (project has all required fields)
        self.is_valid()
    }
    
    /// Checks if the project has the minimum required fields to be usable
    pub fn is_valid(&self) -> bool {
        // MVP: API key is no longer required - users cannot use their own API key in MVP
        // TODO: Add back API key requirement when allowing users to provide their own API keys
        // Required: id and (name OR path)
        !self.id.is_empty() 
            && (!self.name.is_empty() || !self.path.is_empty())
            // Removed: && !self.api_key.is_empty()
    }
}
