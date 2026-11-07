use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestConfig {
    pub parsing: ParsingConfig,
    pub translation: TranslationConfig,
    pub writing: WritingConfig,
    pub unused_strings: UnusedStringsConfig,
    pub integration: IntegrationConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsingConfig {
    pub basic: bool,
    pub complex: bool,
    pub plurals: bool,      // OFF by default (feature not supported)
    pub cdata: bool,        // OFF by default (feature not supported)
    pub annotation: bool,   // OFF by default (feature not supported)
    pub commented: bool,   // Test that commented strings are ignored
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationConfig {
    pub roundtrip: bool,
    pub placeholder_preservation: bool,
    pub special_chars: bool,
    pub real_ai: bool,      // Uses ANTHROPIC_API_KEY from .env
    pub comprehensive_edge_cases: bool,  // Comprehensive edge case validation
    pub escaping: bool,
    pub plurals: bool,     // Translation + write of plural items (flattened name|quantity)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WritingConfig {
    pub golden_comparison: bool,
    pub idempotence: bool,
    pub plurals: bool,      // OFF by default (feature not supported)
    pub merge_mode: bool,
    pub whitespace_preservation: bool,
    pub empty_files: bool,
    pub error_handling: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnusedStringsConfig {
    pub detection: bool,
    pub deletion: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationConfig {
    pub missing_translations: bool,
    pub translate_write: bool,
    pub batch_translation: bool,
    pub change_detection: bool,
}

impl Default for TestConfig {
    fn default() -> Self {
        Self {
            parsing: ParsingConfig {
                basic: true,
                complex: true,
                plurals: false,
                cdata: false,
                annotation: false,
                commented: true,
            },
            translation: TranslationConfig {
                roundtrip: true,
                placeholder_preservation: true,
                special_chars: true,
                real_ai: true,
                comprehensive_edge_cases: true,
                escaping: true,
                plurals: true,
            },
            writing: WritingConfig {
                golden_comparison: true,
                idempotence: true,
                plurals: false,
                merge_mode: true,
                whitespace_preservation: true,
                empty_files: true,
                error_handling: true,
            },
            unused_strings: UnusedStringsConfig {
                detection: true,
                deletion: true,
            },
            integration: IntegrationConfig {
                missing_translations: true,
                translate_write: true,
                batch_translation: true,
                change_detection: true,
            },
        }
    }
}

impl TestConfig {
    pub fn load() -> Self {
        // Try to load from test_config.toml, fall back to defaults
        let config_path = "tests/test_config.toml";
        if let Ok(content) = fs::read_to_string(config_path) {
            toml::from_str(&content).unwrap_or_else(|e| {
                eprintln!("Warning: Failed to parse test_config.toml: {}. Using defaults.", e);
                Self::default()
            })
        } else {
            eprintln!("Warning: test_config.toml not found. Using defaults.");
            Self::default()
        }
    }
}

