use crate::models::{get_language_name, AndroidString, Language, Project, LANGUAGES};
use crate::models::script::{check_translation_script, prompt_language_label};
use crate::commands::app_settings::{get_api_key, get_model, ANTHROPIC_MESSAGES_URL, ANTHROPIC_VERSION};
use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tauri::{Emitter, Listener};
use ts_rs::TS;
use uuid::Uuid;
use tokio::time::sleep;
use tokio::select;

#[derive(Debug, Serialize, Deserialize, TS, Clone)]
#[ts(export)]
pub struct TranslationRequest {
    pub strings: Vec<AndroidString>,
    pub target_language: String,
    pub project: Project,
    pub extra_instructions: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, TS, Clone)]
#[ts(export)]
pub struct TranslationResult {
    pub translated: Vec<AndroidString>,
    pub failed: Vec<AndroidString>,
    pub skipped: Vec<AndroidString>, // Strings skipped due to length constraints
}

#[derive(Debug, Serialize, Deserialize, TS, Clone)]
#[ts(export)]
pub struct TranslationProgress {
    pub current_language: String,
    pub total_languages: u32,
    pub languages_done: u32,
    pub current_language_strings_done: u32,
    pub current_language_total_strings: u32,
    /// Current batch/chunk being processed (1-indexed)
    pub current_batch: u32,
    /// Total number of batches/chunks for the current language
    pub total_batches: u32,
    /// Whether a batch is currently being translated (API call in progress)
    pub batch_in_progress: bool,
}

// Translation event structures
#[derive(Debug, Serialize, Deserialize, TS, Clone)]
#[ts(export)]
pub struct TranslationStartEvent {
    pub session_id: String,
    pub source_strings: Vec<AndroidString>,
    pub target_languages: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, TS, Clone)]
#[ts(export)]
pub struct TranslationProgressEvent {
    pub session_id: String,
    pub total_strings: u32,
    pub completed_strings: u32,
    pub languages_completed: u32,
    pub current_language: Option<String>,
    /// Current batch/chunk being processed (1-indexed)
    pub current_batch: u32,
    /// Total number of batches/chunks for the current language
    pub total_batches: u32,
    /// Whether a batch is currently being translated (API call in progress)
    pub batch_in_progress: bool,
}

#[derive(Debug, Serialize, Deserialize, TS, Clone)]
#[ts(export)]
pub struct TranslationCompleteEvent {
    pub session_id: String,
    pub results: HashMap<String, TranslationResult>, // language -> TranslationResult
    pub total_input_tokens: i32,
    pub total_output_tokens: i32,
}

#[derive(Debug, Serialize, Deserialize, TS, Clone)]
#[ts(export)]
pub struct TranslationErrorEvent {
    pub session_id: String,
    pub error: String,
    /// Number of translations completed before cancellation/error (0 if no progress)
    pub completed_translations: Option<u32>,
    /// Total number of translations requested
    pub total_translations: Option<u32>,
}

/// Event emitted when tokens are tracked for a cancelled request (fire-and-forget)
#[derive(Debug, Serialize, Deserialize, TS, Clone)]
#[ts(export)]
pub struct TranslationCancelledTokensEvent {
    pub input_tokens: i32,
    pub output_tokens: i32,
}

// Output budget per request; batches are ~3.5k chars of source text, so this leaves plenty of room
const MAX_OUTPUT_TOKENS: u32 = 32_000;

// Translation API limits (conservative limits for batching)
const MAX_SINGLE_TEXT_BYTES: usize = 5000; // conservative limit per string (the model can handle much more)
#[allow(dead_code)]
const MAX_REQUEST_BYTES: usize = 80_000;   // safe below ~100k per request (kept for reference)
const TARGET_BATCH_BYTES: usize = 15_000;  // target batch size for good progress granularity
const MAX_WRONG_SCRIPT_RETRIES: u32 = 2;   // re-request a batch that came back in the wrong script (e.g. Hindi for Kannada)

// Dynamic batching constants
const MAX_CHARS_PER_PROMPT: usize = 3500;  // Safe middle ground (3000-4000) for all providers
const OUTPUT_MULTIPLIER: usize = 3;        // input * 3 ≈ total output size for multi-language scenarios
const SMALL_STRING_THRESHOLD: usize = 5;   // ≤5 strings = "few strings"

lazy_static! {
    static ref PLACEHOLDER_PATTERNS: Vec<Regex> = vec![
        Regex::new(r"%\d*\$?[sdfx]").unwrap(),           // %s, %d, %1$s, %2$d, etc.
        Regex::new(r"<xliff:g[^>]*>.*?</xliff:g>").unwrap(),  // <xliff:g>...</xliff:g>
        // Note: Curly braces and other app-specific patterns are not validated here
        // as they vary by app. AI quality control should catch those issues.
    ];
    
    // Regex to detect CDATA sections
    static ref CDATA_REGEX: Regex = Regex::new(r"<!\[CDATA\[(.*?)\]\]>").unwrap();
}

/// Escapes unescaped apostrophes in Android string resources.
/// Already escaped apostrophes (\') are left unchanged.
///
/// # Examples
/// ```
/// use lynstring_tauri_lib::commands::translation::escape_android_apostrophes;
/// assert_eq!(escape_android_apostrophes("Don't"), "Don\\'t");
/// assert_eq!(escape_android_apostrophes("Don\\'t"), "Don\\'t");
/// assert_eq!(escape_android_apostrophes("It's a test"), "It\\'s a test");
/// ```
pub fn escape_android_apostrophes(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        if chars[i] == '\\' && i + 1 < chars.len() && chars[i + 1] == '\'' {
            // Already escaped apostrophe, keep as-is
            result.push('\\');
            result.push('\'');
            i += 2;
        } else if chars[i] == '\'' {
            // Unescaped apostrophe, escape it
            result.push('\\');
            result.push('\'');
            i += 1;
        } else {
            result.push(chars[i]);
            i += 1;
        }
    }

    result
}

/// Escapes unescaped double quotes in Android string resources.
/// Already escaped double quotes (\") are left unchanged.
/// Does NOT escape double quotes inside HTML/XML tags (e.g., <font color="#FFF">).
///
/// # Examples
/// ```
/// use lynstring_tauri_lib::commands::translation::escape_android_double_quotes;
/// assert_eq!(escape_android_double_quotes("Say \"hello\""), "Say \\\"hello\\\"");
/// assert_eq!(escape_android_double_quotes("Say \\\"hello\\\""), "Say \\\"hello\\\"");
/// // HTML tags with quotes are preserved
/// assert_eq!(escape_android_double_quotes("<font color=\"#FFF\">text</font>"), "<font color=\"#FFF\">text</font>");
/// ```
pub fn escape_android_double_quotes(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;
    let mut inside_tag = false;

    while i < chars.len() {
        if chars[i] == '<' {
            // Entering an HTML/XML tag
            inside_tag = true;
            result.push(chars[i]);
            i += 1;
        } else if chars[i] == '>' {
            // Exiting an HTML/XML tag
            inside_tag = false;
            result.push(chars[i]);
            i += 1;
        } else if chars[i] == '\\' && i + 1 < chars.len() && chars[i + 1] == '"' {
            // Already escaped double quote, keep as-is
            result.push('\\');
            result.push('"');
            i += 2;
        } else if chars[i] == '"' {
            if inside_tag {
                // Inside a tag, don't escape quotes (they're part of HTML attributes)
                result.push('"');
                i += 1;
            } else {
                // Outside a tag, escape unescaped double quotes
                result.push('\\');
                result.push('"');
                i += 1;
            }
        } else {
            result.push(chars[i]);
            i += 1;
        }
    }

    result
}

/// Escapes unescaped @ and ? in Android string resources.
/// In Android XML, @ starts a resource reference (e.g. @string/foo) and ? starts a theme
/// attribute reference (e.g. ?attr/colorPrimary). When translations return text starting
/// with or containing these characters literally (e.g. Twitter handles "@user", or "?"),
/// they must be escaped as \@ and \? to prevent Android from interpreting them as references.
/// Already escaped sequences (\@ and \?) are left unchanged.
///
/// # Examples
/// ```
/// use lynstring_tauri_lib::commands::translation::escape_android_resource_ref_chars;
/// assert_eq!(escape_android_resource_ref_chars("@CrookCatcherApp"), "\\@CrookCatcherApp");
/// assert_eq!(escape_android_resource_ref_chars("@user ਨੂੰ ਫਾਲੋ ਕਰੋ"), "\\@user ਨੂੰ ਫਾਲੋ ਕਰੋ");
/// assert_eq!(escape_android_resource_ref_chars("?"), "\\?");
/// assert_eq!(escape_android_resource_ref_chars("Follow \\@user"), "Follow \\@user");
/// ```
pub fn escape_android_resource_ref_chars(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        if chars[i] == '\\' && i + 1 < chars.len() && (chars[i + 1] == '@' || chars[i + 1] == '?') {
            // Already escaped @ or ?, keep as-is
            result.push('\\');
            result.push(chars[i + 1]);
            i += 2;
        } else if chars[i] == '@' || chars[i] == '?' {
            // Unescaped @ or ?, escape it
            result.push('\\');
            result.push(chars[i]);
            i += 1;
        } else {
            result.push(chars[i]);
            i += 1;
        }
    }

    result
}

/// Validates that placeholders in the translated string match the original.
/// Checks that all Android-specific placeholder types (%s, %d, %1$s, <xliff:g>) are preserved.
/// Does NOT validate app-specific patterns (like curly braces) - those should be caught by AI quality control.
///
/// # Examples
/// ```
/// use lynstring_tauri_lib::commands::translation::validate_placeholders;
/// assert!(validate_placeholders("Hello %s", "Bonjour %s"));
/// assert!(!validate_placeholders("Hello %s", "Bonjour"));
/// assert!(validate_placeholders("Item %1$s of %2$d", "Artículo %1$s de %2$d"));
/// assert!(!validate_placeholders("Item %1$s of %2$d", "Artículo %1$s de %1$d"));
/// ```
pub fn validate_placeholders(original: &str, translated: &str) -> bool {
    for re in PLACEHOLDER_PATTERNS.iter() {
        let original_matches: Vec<_> = re.find_iter(original).map(|m| m.as_str()).collect();
        let translated_matches: Vec<_> = re.find_iter(translated).map(|m| m.as_str()).collect();

        // Check that all placeholders from original exist in translation
        for placeholder in &original_matches {
            if !translated_matches.contains(placeholder) {
                return false;
            }
        }

        // Check counts match
        if original_matches.len() != translated_matches.len() {
            return false;
        }
    }

    true
}

/// Escapes invalid \u Unicode escape sequences that cause Android's "Invalid unicode
/// escape sequence" error. Android requires \uXXXX (exactly 4 hex digits). Malformed
/// sequences like \u, \u1, \u12, \u123, \u12. cause resource compilation to fail.
/// Valid \uXXXX is left unchanged. Invalid \u is fixed by escaping the backslash (\\u).
pub fn sanitize_invalid_unicode_escapes(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;

    fn is_hex(c: char) -> bool {
        c.is_ascii_hexdigit()
    }

    while i < chars.len() {
        if chars[i] == '\\' && i + 1 < chars.len() && chars[i + 1] == 'u' {
            // Check if \u is followed by exactly 4 hex digits
            let has_four_hex = i + 5 < chars.len()
                && is_hex(chars[i + 2])
                && is_hex(chars[i + 3])
                && is_hex(chars[i + 4])
                && is_hex(chars[i + 5]);
            if has_four_hex {
                // Valid \uXXXX — keep as-is
                for j in 0..6 {
                    result.push(chars[i + j]);
                }
                i += 6;
            } else {
                // Invalid \u — escape the backslash so it displays literally
                result.push('\\');
                result.push('\\');
                result.push('u');
                i += 2;
            }
        } else {
            result.push(chars[i]);
            i += 1;
        }
    }

    result
}

/// Extracts content from CDATA wrapper if present
fn extract_from_cdata(text: &str) -> String {
    if let Some(caps) = CDATA_REGEX.captures(text) {
        caps.get(1).map(|m| m.as_str()).unwrap_or(text).to_string()
    } else {
        text.to_string()
    }
}

/// Restores CDATA wrapper only when the original had one (original.has_cdata).
/// We must NOT add CDATA when the original used escape sequences instead.
/// E.g. a string with HTML links but escaped apostrophes (Gmail\'s) should stay
/// non-CDATA so that escaping is applied; CDATA skips escaping entirely.
fn restore_cdata_wrapper(translated: &str, original_had_cdata: bool) -> String {
    if original_had_cdata {
        // Original had CDATA — preserve that format. Extract in case AI added CDATA.
        let mut translated_content = extract_from_cdata(translated);
        // Fix invalid \u escape sequences (e.g. \u, \u1) that cause Android build errors
        translated_content = sanitize_invalid_unicode_escapes(&translated_content);
        // Android requires apostrophes and quotes escaped even inside CDATA (e.g. 'ਤੇ → \'ਤੇ)
        translated_content = escape_android_apostrophes(&translated_content);
        translated_content = escape_android_double_quotes(&translated_content);
        format!("<![CDATA[{}]]>", translated_content)
    } else {
        // Original did NOT have CDATA — strip CDATA if AI wrongly added it.
        // Return raw content so escaping (apostrophes, quotes, @, ?) is applied.
        extract_from_cdata(translated)
    }
}

#[tauri::command]
pub async fn get_supported_languages() -> Result<Vec<Language>, String> {
    Ok(LANGUAGES.to_vec())
}

#[tauri::command]
pub async fn translate_strings_multi_language_single_call(
    app: tauri::AppHandle,
    window: tauri::Window,
    source_strings: Vec<AndroidString>,
    target_languages: Vec<String>,
    project: Project,
    extra_instructions: Option<String>,
    expected_total_strings: Option<u32>,
    simulate: Option<bool>,
    simulate_with_delay: Option<bool>,
) -> Result<HashMap<String, HashMap<String, String>>, String> {
    // Fail early with a clear message when no API key is configured
    if !simulate.unwrap_or(false) {
        get_api_key(&app)?;
    }
    let model = get_model(&app);
    
    // Create cancellation flag
    let cancelled = Arc::new(AtomicBool::new(false));
    let cancelled_clone = cancelled.clone();
    let cancel_listener = window.listen("cancel-translation", move |_| {
        cancelled_clone.store(true, Ordering::SeqCst);
    });
    
    // Generate unique session ID
    let session_id = Uuid::new_v4().to_string();
    
    // Emit translation-start event
    app.emit(
        "translation-start",
        TranslationStartEvent {
            session_id: session_id.clone(),
            source_strings: source_strings.clone(),
            target_languages: target_languages.clone(),
        },
    )
    .ok();
    
    println!(
        "[translation] translate_strings_multi_language_single_call requested languages: {:?}",
        target_languages
    );

    // Detect batching case
    let batching_case = detect_batching_case(source_strings.len(), target_languages.len());
    println!(
        "[translation] Detected batching case: {:?} ({} strings, {} languages)",
        batching_case,
        source_strings.len(),
        target_languages.len()
    );
    
    // Results map: language -> TranslationResult
    let mut all_results: HashMap<String, TranslationResult> = HashMap::new();
    
    // Track actual cumulative progress (not assumed)
    let mut actual_strings_completed: usize = 0;
    
    // Calculate total strings: use expected_total if provided, otherwise calculate from source_strings * languages
    // expected_total is the sum of strings per language (e.g., if ar needs 1, as needs 743, total = 1+743+...)
    let total_strings_across_all_languages = expected_total_strings
        .unwrap_or_else(|| (source_strings.len() * target_languages.len()) as u32) as usize;
    
    // Track cumulative token usage across all API calls
    let mut total_input_tokens: i32 = 0;
    let mut total_output_tokens: i32 = 0;
    
    // Route to appropriate batching strategy
    match batching_case {
        BatchingCase::CaseB => {
            // Few strings → many languages: batch by languages
            let results = translate_multi_language_batch(
                &app,
                &session_id,
                &source_strings,
                &target_languages,
                &project,
                &extra_instructions,
                cancelled.clone(),
                total_strings_across_all_languages,
                0,
                &mut total_input_tokens,
                &mut total_output_tokens,
                simulate.unwrap_or(false),
                simulate_with_delay.unwrap_or(true),
                &model,
            )
            .await?;
            
            // Convert results to TranslationResult format
            for (lang, translations) in results {
                let mut translated = Vec::new();
                let mut failed = Vec::new();
                
                for source_string in &source_strings {
                    if let Some(translated_text) = translations.get(&source_string.name) {
                        if validate_placeholders(&source_string.text, translated_text) {
                            translated.push(AndroidString {
                                name: source_string.name.clone(),
                                text: translated_text.clone(),
                                is_translatable: source_string.is_translatable,
                                is_commented_out: false,
                                line_number: 0,
                                has_cdata: false, // text is already in output format (CDATA-wrapped or raw); writer uses as-is
                            });
                        } else {
                            failed.push(source_string.clone());
                        }
                    } else {
                        failed.push(source_string.clone());
                    }
                }
                
                all_results.insert(lang, TranslationResult {
                    translated,
                    failed,
                    skipped: Vec::new(),
                });
            }
        }
        BatchingCase::CaseA | BatchingCase::CaseC => {
            // Case A: Many strings → one language (batch by strings)
            // Case C: Many strings → many languages (batch by languages first, then strings)
            // Both use the same strategy: process languages sequentially, batch strings within each language
            // Process each language sequentially (one target language per translation call)
            
            for (lang_idx, target_language) in target_languages.iter().enumerate() {
        // Check for cancellation
        if cancelled.load(Ordering::SeqCst) {
            // Emit cancellation with progress info
            app.emit(
                "translation-error",
                TranslationErrorEvent {
                    session_id: session_id.clone(),
                    error: "Translation cancelled".to_string(),
                    completed_translations: if actual_strings_completed > 0 { Some(actual_strings_completed as u32) } else { None },
                    total_translations: Some(total_strings_across_all_languages as u32),
                },
            )
            .ok();
            app.unlisten(cancel_listener);
            return Err("Translation cancelled".to_string());
        }
        
        println!(
            "[translation] Processing language {} ({}/{})",
            target_language,
            lang_idx + 1,
            target_languages.len()
        );
        
        // Chunk strings to know total batches for this language
        let batches_for_language = chunk_strings_by_size(&source_strings, TARGET_BATCH_BYTES);
        let total_batches_for_language = batches_for_language.len() as u32;
        // Global batch count: each language contributes N batches, total = languages * batches_per_language
        let total_batches_global = target_languages.len() as u32 * total_batches_for_language;
        let batches_completed_before_this_language = lang_idx as u32 * total_batches_for_language;
        
        // Emit progress using actual cumulative progress (not assumed)
        app.emit(
            "translation-progress",
            TranslationProgressEvent {
                session_id: session_id.clone(),
                total_strings: total_strings_across_all_languages as u32,
                completed_strings: actual_strings_completed as u32,
                languages_completed: lang_idx as u32,
                current_language: Some(target_language.clone()),
                current_batch: batches_completed_before_this_language,
                total_batches: total_batches_global,
                batch_in_progress: false,
            },
        )
        .ok();
        
        // Translate strings for this language (with retry logic)
        let mut strings_to_translate = source_strings.clone();
        let mut all_translations = HashMap::new();
        let mut all_skipped_strings: Vec<AndroidString> = Vec::new();
        let mut retry_count = 0;
        const MAX_RETRIES: usize = 2;
        // Only count batches from previous attempts when we got Ok with partial failures (retry with subset).
        // On Err retry we start over - don't add to batch count.
        let mut previous_attempt_completed_all_batches = false;
        
        while retry_count <= MAX_RETRIES && !strings_to_translate.is_empty() {
            // Check for cancellation before retry
            if cancelled.load(Ordering::SeqCst) {
                // Emit cancellation with progress info
                app.emit(
                    "translation-error",
                    TranslationErrorEvent {
                        session_id: session_id.clone(),
                        error: "Translation cancelled".to_string(),
                        completed_translations: if actual_strings_completed > 0 { Some(actual_strings_completed as u32) } else { None },
                        total_translations: Some(total_strings_across_all_languages as u32),
                    },
                )
                .ok();
                app.unlisten(cancel_listener);
                return Err("Translation cancelled".to_string());
            }
            
            if retry_count > 0 {
                println!(
                    "[translation] Retry {} for {} failed strings in language {}",
                    retry_count,
                    strings_to_translate.len(),
                    target_language
                );
            }
            
            // Only count batches from previous attempts when we got Ok with partial failures.
            // On Err retry we start over with full strings - don't add to batch count.
            let batches_completed_in_this_language_from_previous_attempts =
                if retry_count > 0 && previous_attempt_completed_all_batches {
                    total_batches_for_language
                } else {
                    0
                };

            match translate_language_batch(
                &app,
                &session_id,
                &strings_to_translate,
                target_language,
                &project,
                &extra_instructions,
                cancelled.clone(),
                total_strings_across_all_languages,
                actual_strings_completed,
                total_batches_global,
                batches_completed_before_this_language,
                batches_completed_in_this_language_from_previous_attempts,
                &mut total_input_tokens,
                &mut total_output_tokens,
                simulate.unwrap_or(false),
                simulate_with_delay.unwrap_or(true),
                &model,
            )
            .await
            {
                Ok((translations, skipped_strings)) => {
                    // This attempt completed all its batches (even if some strings failed)
                    previous_attempt_completed_all_batches = true;
                    // Merge skipped strings (only add unique ones)
                    for skipped in skipped_strings {
                        if !all_skipped_strings.iter().any(|s| s.name == skipped.name) {
                            all_skipped_strings.push(skipped);
                        }
                    }
                    
                    // Merge successful translations
                    all_translations.extend(translations);
                    
                    // Find strings that still failed (missing from translations)
                    // Don't count skipped strings as failed - they're already tracked
                    let skipped_names: Vec<String> = all_skipped_strings.iter().map(|s| s.name.clone()).collect();
                    let failed_string_names: Vec<String> = strings_to_translate
                        .iter()
                        .filter(|s| !all_translations.contains_key(&s.name) && !skipped_names.contains(&s.name))
                        .map(|s| s.name.clone())
                        .collect();
                    
                    if failed_string_names.is_empty() {
                        // All strings translated successfully!
                        break;
                    } else {
                        println!(
                            "[translation] {} strings failed in language {} (attempt {}): {:?}",
                            failed_string_names.len(),
                            target_language,
                            retry_count + 1,
                            failed_string_names
                        );
                        
                        // Prepare for retry
                        strings_to_translate = source_strings
                            .iter()
                            .filter(|s| failed_string_names.contains(&s.name))
                            .cloned()
                            .collect();
                        
                        retry_count += 1;
                        
                        if retry_count <= MAX_RETRIES && !strings_to_translate.is_empty() {
                            // Short delay before retry
                            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                        }
                    }
                }
                Err(e) => {
                    // Attempt failed before completing - on retry we start from batch 0
                    previous_attempt_completed_all_batches = false;
                    // Check if this is a cancellation error
                    if e.contains("cancelled") || e.contains("Cancelled") {
                        // Emit cancellation with progress info
                        app.emit(
                            "translation-error",
                            TranslationErrorEvent {
                                session_id: session_id.clone(),
                                error: "Translation cancelled".to_string(),
                                completed_translations: if actual_strings_completed > 0 { Some(actual_strings_completed as u32) } else { None },
                                total_translations: Some(total_strings_across_all_languages as u32),
                            },
                        )
                        .ok();
                        app.unlisten(cancel_listener);
                        return Err("Translation cancelled".to_string());
                    }
                    
                    eprintln!(
                        "[translation][error] Failed to translate language {} (attempt {}): {}",
                        target_language, retry_count + 1, e
                    );
                    
                    retry_count += 1;
                    
                    if retry_count <= MAX_RETRIES {
                        // Short delay before retry
                        tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
                    } else {
                        // Max retries exceeded
                        app.emit(
                            "translation-error",
                            TranslationErrorEvent {
                                session_id: session_id.clone(),
                                error: format!("Failed to translate {}: {}", target_language, e),
                                completed_translations: None,
                                total_translations: None,
                            },
                        )
                        .ok();
                        // Continue with other languages even if one fails completely
                        break;
                    }
                }
            }
        }
        
        // Build TranslationResult for this language
        let mut translated: Vec<AndroidString> = Vec::new();
        let mut failed: Vec<AndroidString> = Vec::new();
        let skipped_names: Vec<String> = all_skipped_strings.iter().map(|s| s.name.clone()).collect();
        let skipped_keys: std::collections::HashSet<String> = skipped_names.iter().cloned().collect();
        
        for source_string in &source_strings {
            // Skip strings that were skipped due to length
            if skipped_keys.contains(&source_string.name) {
                continue;
            }
            
            if let Some(translated_text) = all_translations.get(&source_string.name) {
                // Validate placeholders
                if validate_placeholders(&source_string.text, translated_text) {
                    translated.push(AndroidString {
                        name: source_string.name.clone(),
                        text: translated_text.clone(),
                        is_translatable: source_string.is_translatable,
                        is_commented_out: false,
                        line_number: 0,
                        has_cdata: false, // text is already in output format (CDATA-wrapped or raw); writer uses as-is
                    });
                } else {
                    eprintln!(
                        "[translation][warn] Placeholder mismatch for key '{}' in language '{}': original='{}', translated='{}'",
                        source_string.name, target_language, source_string.text, translated_text
                    );
                    failed.push(source_string.clone());
                }
            } else {
                // String missing from translations even after retries
                eprintln!(
                    "[translation][warn] String '{}' missing from {} translations after {} retries",
                    source_string.name, target_language, retry_count
                );
                failed.push(source_string.clone());
            }
        }
        
        // Store complete TranslationResult for this language
        all_results.insert(
            target_language.clone(),
            TranslationResult {
                translated: translated.clone(),
                failed: failed.clone(),
                skipped: all_skipped_strings.clone(),
            },
        );
        
        // Update actual cumulative progress based on what was actually translated (after validation)
        // Use the final translated count, not the intermediate batch counts, to ensure accuracy
        // This ensures progress never goes backward, even if languages have different string counts
        let final_translated_count = translated.len();
        actual_strings_completed += final_translated_count;
        
        // Emit final progress for this language to ensure UI matches final count
        let batches_completed_after_this_language = (lang_idx + 1) as u32 * total_batches_for_language;
        app.emit(
            "translation-progress",
            TranslationProgressEvent {
                session_id: session_id.clone(),
                total_strings: total_strings_across_all_languages as u32,
                completed_strings: actual_strings_completed as u32,
                languages_completed: (lang_idx + 1) as u32,
                current_language: Some(target_language.clone()),
                current_batch: batches_completed_after_this_language,
                total_batches: total_batches_global,
                batch_in_progress: false,
            },
        )
        .ok();
        
        // Note: Files are already written incrementally after each batch
        // No need to write again here
        
        // Log summary for this language
        if !failed.is_empty() || !all_skipped_strings.is_empty() {
            let mut summary_parts = vec![format!("{} successful", translated.len())];
            if !failed.is_empty() {
                summary_parts.push(format!("{} failed", failed.len()));
            }
            if !all_skipped_strings.is_empty() {
                summary_parts.push(format!("{} skipped (too long)", all_skipped_strings.len()));
            }
            println!(
                "[translation] Language {} completed: {}",
                target_language,
                summary_parts.join(", ")
            );
        } else {
            println!(
                "[translation] Language {} completed: {} strings successfully translated",
                target_language,
                translated.len()
            );
        }
            }
        }
    }
    
    // Note: Files are already written incrementally after each batch completes
    // No need to write again here - this ensures that if translation is cancelled,
    // already-completed batches remain written to file
    
    // Emit translation-complete event with token counts
    app.emit(
        "translation-complete",
        TranslationCompleteEvent {
            session_id: session_id.clone(),
            results: all_results.clone(),
            total_input_tokens,
            total_output_tokens,
        },
    )
    .ok();
    
    // Clean up the cancel listener
    app.unlisten(cancel_listener);
    
    // Return raw translations map for backward compatibility (used by some callers)
    let mut raw_results: HashMap<String, HashMap<String, String>> = HashMap::new();
    for (lang, result) in &all_results {
        let mut lang_map = HashMap::new();
        for translated in &result.translated {
            lang_map.insert(translated.name.clone(), translated.text.clone());
        }
        raw_results.insert(lang.clone(), lang_map);
    }
    
    Ok(raw_results)
}

/// Batching case enum
#[derive(Debug, Clone, Copy)]
enum BatchingCase {
    CaseA, // Many strings → one language (batch by strings)
    CaseB, // Few strings → many languages (batch by languages)
    CaseC, // Many strings → many languages (batch by languages first, then strings)
}

/// Detect which batching case applies based on string count and language count
fn detect_batching_case(string_count: usize, language_count: usize) -> BatchingCase {
    if language_count == 1 {
        // One language → always Case A (batch by strings)
        BatchingCase::CaseA
    } else if string_count <= SMALL_STRING_THRESHOLD {
        // Few strings + many languages → Case B (batch by languages)
        BatchingCase::CaseB
    } else {
        // Many strings + many languages → Case C (batch by languages first, then strings)
        BatchingCase::CaseC
    }
}

/// Calculate total character count for a slice of strings
fn calculate_strings_chars(strings: &[AndroidString]) -> usize {
    strings.iter().map(|s| s.text.chars().count()).sum()
}

/// Chunk strings into batches based on byte size to respect API limits and provide progress updates
/// Returns a vector of batches, where each batch is a Vec<AndroidString>
fn chunk_strings_by_size(strings: &[AndroidString], target_batch_bytes: usize) -> Vec<Vec<AndroidString>> {
    let mut batches = Vec::new();
    let mut current_batch = Vec::new();
    let mut current_batch_size = 0usize;
    
    for string in strings {
        // Calculate size of this string as XML (rough estimate)
        let xml_size = format!(r#"<string name="{}">{}</string>"#, string.name, string.text)
            .as_bytes()
            .len();
        
        // If adding this string would exceed target, start new batch
        // (unless current batch is empty - always add at least one string per batch)
        if current_batch_size + xml_size > target_batch_bytes && !current_batch.is_empty() {
            batches.push(current_batch);
            current_batch = Vec::new();
            current_batch_size = 0;
        }
        
        current_batch.push(string.clone());
        current_batch_size += xml_size;
    }
    
    // Add remaining batch if not empty
    if !current_batch.is_empty() {
        batches.push(current_batch);
    }
    
    batches
}

/// Translate few strings to many languages using language batching (Case B)
/// Returns HashMap<language, HashMap<string_name, translated_text>>
/// Also accumulates token usage in the provided mutable references
async fn translate_multi_language_batch(
    app: &tauri::AppHandle,
    session_id: &str,
    source_strings: &[AndroidString],
    target_languages: &[String],
    project: &Project,
    extra_instructions: &Option<String>,
    cancelled: Arc<AtomicBool>,
    total_strings_across_all_languages: usize,
    strings_completed_before: usize,
    total_input_tokens: &mut i32,
    total_output_tokens: &mut i32,
    simulate: bool,
    simulate_with_delay: bool,
    model: &str,
) -> Result<HashMap<String, HashMap<String, String>>, String> {
    // Check for cancellation
    if cancelled.load(Ordering::SeqCst) {
        return Err("Translation cancelled".to_string());
    }
    
    // Calculate total input characters
    let total_chars = calculate_strings_chars(source_strings);
    
    // Calculate max languages per batch based on output multiplier
    // Formula: MAX_CHARS_PER_PROMPT / (total_chars * OUTPUT_MULTIPLIER)
    // Ensure at least 1 language per batch
    let max_languages_per_batch = if total_chars > 0 {
        (MAX_CHARS_PER_PROMPT / (total_chars * OUTPUT_MULTIPLIER)).max(1)
    } else {
        1
    };
    
    println!(
        "[translation] Case B: {} strings ({} chars) → {} languages, batching {} languages per batch",
        source_strings.len(),
        total_chars,
        target_languages.len(),
        max_languages_per_batch
    );
    
    // Split languages into batches
    let language_batches: Vec<Vec<String>> = target_languages
        .chunks(max_languages_per_batch)
        .map(|chunk| chunk.iter().cloned().collect())
        .collect();
    
    let mut all_results: HashMap<String, HashMap<String, String>> = HashMap::new();
    let mut strings_completed = strings_completed_before;
    
    // Process each language batch
    for (batch_idx, language_batch) in language_batches.iter().enumerate() {
        // Check for cancellation BEFORE processing batch
        if cancelled.load(Ordering::SeqCst) {
            return Err("Translation cancelled".to_string());
        }
        
        println!(
            "[translation] Processing language batch {}/{} ({} languages)",
            batch_idx + 1,
            language_batches.len(),
            language_batch.len()
        );
        
        // Emit progress at START of batch (batch_in_progress: true)
        app.emit(
            "translation-progress",
            TranslationProgressEvent {
                session_id: session_id.to_string(),
                total_strings: total_strings_across_all_languages as u32,
                completed_strings: strings_completed as u32,
                languages_completed: all_results.len() as u32,
                current_language: Some(format!("{} languages", language_batch.len())),
                current_batch: (batch_idx + 1) as u32,
                total_batches: language_batches.len() as u32,
                batch_in_progress: true,
            },
        )
        .ok();
        
        // Build prompt for multi-language translation
        let prompt = build_multi_language_translation_prompt(
            source_strings,
            language_batch,
            project,
            extra_instructions,
        );
        
        // Build XML strings
        let xml_strings: Vec<String> = source_strings
            .iter()
            .map(|s| format!(r#"<string name="{}">{}</string>"#, s.name, s.text))
            .collect();
        
        // Build the full text to translate
        let full_text = format!("{}\n\n{}\n\n{}", prompt, xml_strings.join("\n"), language_reminder(language_batch));

        let mut wrong_script_retries = 0;
        let batch_results = loop {
            // Call the Anthropic API with retry logic
            // Pass language_batch for multi-language formatting in simulation mode
            let api_result = call_translation_api_with_retry(&full_text, app, &cancelled, simulate, simulate_with_delay, &xml_strings, Some(language_batch), model).await?;

            // Accumulate token usage
            *total_input_tokens += api_result.token_usage.input_tokens;
            *total_output_tokens += api_result.token_usage.output_tokens;

            // Parse multi-language response
            let results = parse_multi_language_response(
                &api_result.text,
                language_batch,
                source_strings,
            )?;

            // Simulation copies the source strings, so skip the script check there
            let script_error = if simulate {
                None
            } else {
                results
                    .iter()
                    .find_map(|(lang, translations)| check_translation_script(lang, translations.values()).err())
            };
            match script_error {
                None => break results,
                Some(e) if wrong_script_retries < MAX_WRONG_SCRIPT_RETRIES => {
                    wrong_script_retries += 1;
                    eprintln!(
                        "[translation][warn] {} (language batch {}), retrying ({}/{})",
                        e, batch_idx + 1, wrong_script_retries, MAX_WRONG_SCRIPT_RETRIES
                    );
                }
                Some(e) => {
                    return Err(format!("{}. Gave up after {} attempts; please try again.", e, MAX_WRONG_SCRIPT_RETRIES + 1));
                }
            }
            if cancelled.load(Ordering::SeqCst) {
                return Err("Translation cancelled".to_string());
            }
        };

        // Check for cancellation AFTER batch completes but BEFORE writing
        // If cancelled, don't write this batch but return what we have so far
        if cancelled.load(Ordering::SeqCst) {
            // Return what we have so far (already written batches remain in file)
            return Err("Translation cancelled".to_string());
        }
        
        // Write to file after each batch completes (incremental writing)
        use crate::commands::strings::write_strings_to_file_merge;
        for (lang, translations) in &batch_results {
            // Convert translations to AndroidString format for writing
            let mut strings_to_write: Vec<AndroidString> = Vec::new();
            for (name, text) in translations {
                // Find the original string to preserve metadata
                if let Some(source_string) = source_strings.iter().find(|s| &s.name == name) {
                    strings_to_write.push(AndroidString {
                        name: name.clone(),
                        text: text.clone(),
                        is_translatable: source_string.is_translatable,
                        is_commented_out: false,
                        line_number: 0,
                        has_cdata: false, // text is already in output format (CDATA-wrapped or raw); writer uses as-is
                    });
                }
            }
            
            // Write translations to file (merge mode)
            if !strings_to_write.is_empty() {
                if let Err(e) = write_strings_to_file_merge(&strings_to_write, &project.path, lang) {
                    eprintln!("Failed to write translations for language {} after batch {}: {}", lang, batch_idx + 1, e);
                    // Continue processing even if write fails
                } else {
                    println!(
                        "[translation] Wrote {} translations to file for language {} after batch {}/{}",
                        strings_to_write.len(),
                        lang,
                        batch_idx + 1,
                        language_batches.len()
                    );
                }
            }
        }
        
        // Merge results
        for (lang, translations) in batch_results {
            all_results.insert(lang, translations);
        }
        
        // Update progress
        strings_completed += source_strings.len() * language_batch.len();
        app.emit(
            "translation-progress",
            TranslationProgressEvent {
                session_id: session_id.to_string(),
                total_strings: total_strings_across_all_languages as u32,
                completed_strings: strings_completed as u32,
                languages_completed: all_results.len() as u32,
                current_language: Some(format!("{} languages", language_batch.len())),
                current_batch: (batch_idx + 1) as u32,
                total_batches: language_batches.len() as u32,
                batch_in_progress: false,
            },
        )
        .ok();
        
        // Small delay between batches
        if batch_idx < language_batches.len() - 1 {
            tokio::time::sleep(tokio::time::Duration::from_millis(150)).await;
        }
    }
    
    Ok(all_results)
}

/// Translate all strings for a single target language using Claude API
/// Returns (translations_map, skipped_strings)
/// Writes translations to file after each batch completes (incremental writing)
async fn translate_language_batch(
    app: &tauri::AppHandle,
    session_id: &str,
    source_strings: &[AndroidString],
    target_language: &str,
    project: &Project,
    extra_instructions: &Option<String>,
    cancelled: Arc<AtomicBool>,
    total_strings_across_all_languages: usize,
    strings_completed_before_this_language: usize,
    total_batches_global: u32,
    batches_completed_before_this_language: u32,
    // Batches already completed for this language in previous retry attempts (0 on first attempt)
    batches_completed_in_this_language_from_previous_attempts: u32,
    total_input_tokens: &mut i32,
    total_output_tokens: &mut i32,
    simulate: bool,
    simulate_with_delay: bool,
    model: &str,
) -> Result<(HashMap<String, String>, Vec<AndroidString>), String> {
    use crate::commands::strings::write_strings_to_file_merge;
    // Check for cancellation
    if cancelled.load(Ordering::SeqCst) {
        return Err("Translation cancelled".to_string());
    }
    
    // Filter out strings that are too long (skip them instead of failing)
    let mut valid_strings = Vec::new();
    let mut skipped_strings = Vec::new();
    
    for s in source_strings {
        let xml_string = format!(r#"<string name="{}">{}</string>"#, s.name, s.text);
        if xml_string.as_bytes().len() > MAX_SINGLE_TEXT_BYTES {
            eprintln!(
                "[translation][warn] Skipping string '{}' in language {} - too long ({} bytes, max {} bytes)",
                s.name,
                target_language,
                xml_string.as_bytes().len(),
                MAX_SINGLE_TEXT_BYTES
            );
            skipped_strings.push(s.clone());
        } else {
            valid_strings.push(s.clone());
        }
    }
    
    // If all strings were too long, return empty result with all skipped
    if valid_strings.is_empty() {
        eprintln!(
            "[translation][warn] All {} strings were too long for language {}",
            source_strings.len(),
            target_language
        );
        return Ok((HashMap::new(), skipped_strings));
    }
    
    // Log summary if any were skipped
    if !skipped_strings.is_empty() {
        println!(
            "[translation] Skipped {} oversized strings in language {}, translating {} strings",
            skipped_strings.len(),
            target_language,
            valid_strings.len()
        );
    }
    
    // Chunk valid strings into batches for progress updates and API limits
    let batches = chunk_strings_by_size(&valid_strings, TARGET_BATCH_BYTES);
    println!(
        "[translation] Chunked {} strings into {} batches for language {}",
        valid_strings.len(),
        batches.len(),
        target_language
    );
    
    let mut all_translations = HashMap::new();
    let mut strings_completed_in_this_language = 0usize;
    
    // Process each batch
    for (batch_idx, batch) in batches.iter().enumerate() {
        // Check for cancellation before each batch
        if cancelled.load(Ordering::SeqCst) {
            return Err("Translation cancelled".to_string());
        }
        
        println!(
            "[translation] Processing batch {}/{} ({} strings) for language {}",
            batch_idx + 1,
            batches.len(),
            batch.len(),
            target_language
        );
        
        // Emit progress at START of batch (batch_in_progress: true)
        let total_strings_completed = strings_completed_before_this_language + strings_completed_in_this_language;
        let current_batch_global = batches_completed_before_this_language
            + batches_completed_in_this_language_from_previous_attempts
            + (batch_idx + 1) as u32;
        app.emit(
            "translation-progress",
            TranslationProgressEvent {
                session_id: session_id.to_string(),
                total_strings: total_strings_across_all_languages as u32,
                completed_strings: total_strings_completed as u32,
                languages_completed: 0, // Will be updated by caller
                current_language: Some(target_language.to_string()),
                current_batch: current_batch_global,
                total_batches: total_batches_global,
                batch_in_progress: true,
            },
        )
        .ok();
        
        // Build the prompt/context with XML strings for this batch
        let prompt_with_strings = build_translation_prompt(
            batch,
            target_language,
            project,
            extra_instructions,
        );
        
        // Build XML strings WITHOUT CDATA - we'll add it back after translation
        let xml_strings: Vec<String> = batch
            .iter()
            .map(|s| {
                format!(r#"<string name="{}">{}</string>"#, s.name, s.text)
            })
            .collect();
        
        // Build the full text to translate (prompt + XML strings + closing language reminder)
        let full_text = format!(
            "{}\n\n{}\n\n{}",
            prompt_with_strings,
            xml_strings.join("\n"),
            language_reminder(&[target_language.to_string()])
        );

        let mut wrong_script_retries = 0;
        let batch_translations = loop {
            // Translate using the Anthropic API with retry logic
            // Single language mode - pass None for target_languages
            let api_result = call_translation_api_with_retry(&full_text, app, &cancelled, simulate, simulate_with_delay, &xml_strings, None, model).await?;

            // Accumulate token usage
            *total_input_tokens += api_result.token_usage.input_tokens;
            *total_output_tokens += api_result.token_usage.output_tokens;

            // Parse translated XML strings from response
            let translations = extract_translations_from_response(
                &api_result.text,
                batch,
            )?;

            // Simulation copies the source strings, so skip the script check there
            let script_error = if simulate {
                None
            } else {
                check_translation_script(target_language, translations.values()).err()
            };
            match script_error {
                None => break translations,
                Some(e) if wrong_script_retries < MAX_WRONG_SCRIPT_RETRIES => {
                    wrong_script_retries += 1;
                    eprintln!(
                        "[translation][warn] {} (batch {}/{}), retrying ({}/{})",
                        e, batch_idx + 1, batches.len(), wrong_script_retries, MAX_WRONG_SCRIPT_RETRIES
                    );
                }
                Some(e) => {
                    return Err(format!("{}. Gave up after {} attempts; please try again.", e, MAX_WRONG_SCRIPT_RETRIES + 1));
                }
            }
            if cancelled.load(Ordering::SeqCst) {
                return Err("Translation cancelled".to_string());
            }
        };

        // Get count before moving the HashMap
        let actual_translations_in_batch = batch_translations.len();
        
        // Check for cancellation AFTER batch completes but BEFORE writing
        // If cancelled, don't write this batch but return what we have so far
        if cancelled.load(Ordering::SeqCst) {
            // Return what we have so far (already written batches remain in file)
            return Err("Translation cancelled".to_string());
        }
        
        // Merge translations from this batch
        all_translations.extend(batch_translations);
        
        // Update progress - count actual translations, not batch size
        // This ensures progress is accurate even if some strings in a batch fail
        strings_completed_in_this_language += actual_translations_in_batch;
        let total_strings_completed = strings_completed_before_this_language + strings_completed_in_this_language;
        
        // Write to file after each batch completes (incremental writing)
        // Convert accumulated translations to AndroidString format for writing
        let mut strings_to_write: Vec<AndroidString> = Vec::new();
        for (name, text) in &all_translations {
            // Find the original string to preserve metadata
            if let Some(source_string) = source_strings.iter().find(|s| &s.name == name) {
                strings_to_write.push(AndroidString {
                    name: name.clone(),
                    text: text.clone(),
                    is_translatable: source_string.is_translatable,
                    is_commented_out: false,
                    line_number: 0,
                    has_cdata: false, // text is already in output format (CDATA-wrapped or raw); writer uses as-is
                });
            }
        }
        
        // Write accumulated translations to file (merge mode)
        if !strings_to_write.is_empty() {
            if let Err(e) = write_strings_to_file_merge(&strings_to_write, &project.path, target_language) {
                eprintln!("Failed to write translations for language {} after batch {}: {}", target_language, batch_idx + 1, e);
                // Continue processing even if write fails
            } else {
                println!(
                    "[translation] Wrote {} translations to file for language {} after batch {}/{}",
                    strings_to_write.len(),
                    target_language,
                    batch_idx + 1,
                    batches.len()
                );
            }
        }
        
        // Emit progress at END of batch (batch_in_progress: false)
        app.emit(
            "translation-progress",
            TranslationProgressEvent {
                session_id: session_id.to_string(),
                total_strings: total_strings_across_all_languages as u32,
                completed_strings: total_strings_completed as u32,
                languages_completed: 0, // Will be updated by caller
                current_language: Some(target_language.to_string()),
                current_batch: current_batch_global,
                total_batches: total_batches_global,
                batch_in_progress: false,
            },
        )
        .ok();
        
        // Small delay between batches to avoid rate limiting (except for last batch)
        if batch_idx < batches.len() - 1 {
            tokio::time::sleep(tokio::time::Duration::from_millis(150)).await;
        }
    }
    
    // Validate that all valid strings were translated
    let mut missing_keys = Vec::new();
    for s in &valid_strings {
        if !all_translations.contains_key(&s.name) {
            missing_keys.push(s.name.clone());
        }
    }
    
    if !missing_keys.is_empty() {
        eprintln!(
            "[translation][warn] Missing translations for keys in language {}: {:?}",
            target_language, missing_keys
        );
    }
    
    Ok((all_translations, skipped_strings))
}

// Token usage result from API call
#[derive(Debug, Clone)]
struct TokenUsage {
    input_tokens: i32,
    output_tokens: i32,
}

// Result from API call with token usage
struct ApiCallResult {
    text: String,
    token_usage: TokenUsage,
}

/// Call the Anthropic Messages API with retry logic and token tracking
/// Returns both the translated text and token usage
/// If simulate is true, returns the original XML strings after a 5 second delay (for dev testing)
/// If simulate_with_delay is false, skips the delay for faster testing
/// target_languages: If Some, formats response with language markers for multi-language mode
/// model: The Claude model to use (e.g. "claude-sonnet-5-5")
async fn call_translation_api_with_retry(
    text: &str,
    app: &tauri::AppHandle,
    cancelled: &Arc<AtomicBool>,
    simulate: bool,
    simulate_with_delay: bool,
    original_xml_strings: &[String],
    target_languages: Option<&[String]>,
    model: &str,
) -> Result<ApiCallResult, String> {
    // Simulation mode: return original strings after delay (for dev testing)
    if simulate {
        println!("[translation][simulate] Simulating translation - copying default strings to target language");
        
        // Wait 5 seconds to simulate API call delay (if enabled)
        if simulate_with_delay {
            for _ in 0..50 {
                if cancelled.load(Ordering::SeqCst) {
                    return Err("Translation cancelled".to_string());
                }
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
            
            // Check for cancellation one more time
            if cancelled.load(Ordering::SeqCst) {
                return Err("Translation cancelled".to_string());
            }
        } else {
            println!("[translation][simulate] Delay disabled - returning immediately");
        }
        
        // Format response based on mode
        let simulated_response = if let Some(languages) = target_languages {
            // Multi-language mode: format with language markers
            // Format: [LANGUAGE: code]\n<string>...</string>\n...
            let mut parts = Vec::new();
            for lang in languages {
                parts.push(format!("[LANGUAGE: {}]", lang));
                parts.push(original_xml_strings.join("\n"));
            }
            parts.join("\n\n")
        } else {
            // Single language mode: just return XML strings
            original_xml_strings.join("\n")
        };
        
        println!("[translation][simulate] Returning simulated response (copied {} strings)", original_xml_strings.len());
        
        return Ok(ApiCallResult {
            text: simulated_response,
            token_usage: TokenUsage {
                input_tokens: 0,
                output_tokens: 0,
            },
        });
    }
    
    const MAX_RETRIES: u32 = 5;
    const INITIAL_RETRY_DELAY_MS: u64 = 1000; // 1 second
    
    let api_key = get_api_key(app)?;

    #[derive(Serialize, Clone)]
    struct Message {
        role: &'static str,
        content: String,
    }

    #[derive(Serialize, Clone)]
    struct OutputConfig {
        effort: &'static str,
    }

    #[derive(Serialize, Clone)]
    struct RequestBody {
        model: String,
        max_tokens: u32,
        // Translation needs little reasoning; low effort keeps it fast and cheap on every model
        output_config: OutputConfig,
        messages: Vec<Message>,
    }

    #[derive(Deserialize)]
    struct ContentBlock {
        #[serde(rename = "type")]
        kind: String,
        #[serde(default)]
        text: Option<String>,
    }

    #[derive(Deserialize)]
    struct Usage {
        #[serde(default)]
        input_tokens: Option<i32>,
        #[serde(default)]
        output_tokens: Option<i32>,
    }

    #[derive(Deserialize)]
    struct MessagesResponse {
        #[serde(default)]
        content: Vec<ContentBlock>,
        #[serde(default)]
        stop_reason: Option<String>,
        #[serde(default)]
        usage: Option<Usage>,
        #[serde(default)]
        error: Option<serde_json::Value>,
    }

    let body = RequestBody {
        model: model.to_string(),
        max_tokens: MAX_OUTPUT_TOKENS,
        output_config: OutputConfig { effort: "low" },
        messages: vec![Message { role: "user", content: text.to_string() }],
    };

    // Turn an Anthropic error response into a message for the user
    let extract_error_msg = |bytes: &[u8], status: u16| -> String {
        let detail = serde_json::from_slice::<MessagesResponse>(bytes)
            .ok()
            .and_then(|r| r.error)
            .and_then(|e| e.get("message").and_then(|m| m.as_str()).map(|s| s.to_string()))
            .unwrap_or_else(|| String::from_utf8_lossy(bytes).to_string());
        match status {
            401 => "Invalid Anthropic API key. Check it in Settings.".to_string(),
            403 => format!("Your API key isn't allowed to use {}: {}", model, detail),
            404 => format!("Model {} not found. Pick another model in Settings.", model),
            429 => format!("Rate limited by Anthropic: {}", detail),
            _ => format!("Anthropic API error ({}): {}", status, detail),
        }
    };

    // Helper for exponential backoff delay
    let backoff_delay = |attempt: u32| -> u64 {
        INITIAL_RETRY_DELAY_MS * (1 << attempt)
    };

    let mut last_error: Option<String> = None;

    for attempt in 0..=MAX_RETRIES {
        // Check for cancellation before each attempt
        if cancelled.load(Ordering::SeqCst) {
            return Err("Translation cancelled".to_string());
        }

        // Spawn the API call in a background task so we can:
        // 1. Return immediately on cancellation
        // 2. Still report tokens when the response arrives (fire-and-forget)
        let (result_tx, mut result_rx) = tokio::sync::oneshot::channel::<Result<(reqwest::StatusCode, Vec<u8>, Option<TokenUsage>), String>>();

        let body_clone = body.clone();
        let api_key_owned = api_key.clone();
        let text_len = text.len();
        let app_for_tracking = app.clone();

        tokio::spawn(async move {
            eprintln!("[translation][bg] Background task started, calling the Anthropic API...");

            let response = match reqwest::Client::new()
                .post(ANTHROPIC_MESSAGES_URL)
                .header("x-api-key", &api_key_owned)
                .header("anthropic-version", ANTHROPIC_VERSION)
                .json(&body_clone)
                .send()
                .await
            {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("[translation][bg] API request failed: {}", e);
                    let _ = result_tx.send(Err(format!("Failed to reach the Anthropic API: {}", e)));
                    return;
                }
            };

            let status = response.status();
            let bytes = response.bytes().await.unwrap_or_default().to_vec();

            let token_usage = if status.is_success() {
                serde_json::from_slice::<MessagesResponse>(&bytes)
                    .ok()
                    .and_then(|parsed| parsed.usage)
                    .map(|usage| TokenUsage {
                        input_tokens: usage.input_tokens.unwrap_or(0),
                        output_tokens: usage.output_tokens.unwrap_or(0),
                    })
            } else {
                None
            };

            if result_tx.send(Ok((status, bytes, token_usage.clone()))).is_err() {
                // Main task was cancelled - still report the tokens this request used
                let tokens = token_usage.unwrap_or(TokenUsage {
                    input_tokens: (text_len as f32 / 4.0).ceil() as i32,
                    output_tokens: 0,
                });
                app_for_tracking.emit(
                    "translation-cancelled-tokens",
                    TranslationCancelledTokensEvent {
                        input_tokens: tokens.input_tokens,
                        output_tokens: tokens.output_tokens,
                    },
                ).ok();
            }
        });

        // Wait for result with cancellation checking
        let api_result = loop {
            select! {
                biased;
                result = &mut result_rx => {
                    break result.map_err(|_| "Channel closed unexpectedly".to_string())?;
                }
                _ = sleep(Duration::from_millis(500)) => {
                    if cancelled.load(Ordering::SeqCst) {
                        return Err("Translation cancelled".to_string());
                    }
                }
            }
        };

        let (status, bytes) = match api_result {
            Ok((s, b, _)) => (s, b),
            Err(e) => {
                last_error = Some(e.clone());
                if attempt < MAX_RETRIES {
                    let delay_ms = backoff_delay(attempt);
                    eprintln!(
                        "[translation] Network error on attempt {}/{}, retrying in {}ms...",
                        attempt + 1,
                        MAX_RETRIES + 1,
                        delay_ms
                    );
                    sleep(Duration::from_millis(delay_ms)).await;
                    continue;
                }
                return Err(last_error.unwrap());
            }
        };

        if !status.is_success() {
            let error_msg = extract_error_msg(&bytes, status.as_u16());
            // 429 (rate limit), 529 (overloaded) and other 5xx are temporary; other 4xx are not
            let retryable = status == 429 || status.is_server_error() || status.as_u16() == 529;
            if retryable && attempt < MAX_RETRIES {
                let delay_ms = backoff_delay(attempt);
                eprintln!(
                    "[translation] {} on attempt {}/{}, retrying in {}ms...",
                    error_msg,
                    attempt + 1,
                    MAX_RETRIES + 1,
                    delay_ms
                );
                last_error = Some(error_msg);
                sleep(Duration::from_millis(delay_ms)).await;
                continue;
            }
            return Err(error_msg);
        }

        // Parse response
        let parsed: MessagesResponse = match serde_json::from_slice(&bytes) {
            Ok(p) => p,
            Err(e) => {
                last_error = Some(format!("Failed to parse response: {}", e));
                if attempt < MAX_RETRIES {
                    let delay_ms = backoff_delay(attempt);
                    sleep(Duration::from_millis(delay_ms)).await;
                    continue;
                }
                return Err(last_error.unwrap());
            }
        };

        match parsed.stop_reason.as_deref() {
            Some("refusal") => {
                return Err("The model declined to translate this content.".to_string());
            }
            Some("max_tokens") => {
                eprintln!("[translation][warn] Response hit max_tokens ({}); output is truncated", MAX_OUTPUT_TOKENS);
            }
            _ => {}
        }

        // Join the text blocks (thinking blocks, if any, are skipped)
        let text_result: String = parsed
            .content
            .iter()
            .filter(|b| b.kind == "text")
            .filter_map(|b| b.text.as_deref())
            .collect();
        if text_result.is_empty() {
            return Err("Empty translation response".to_string());
        }

        let token_usage = parsed
            .usage
            .map(|usage| TokenUsage {
                input_tokens: usage.input_tokens.unwrap_or(0),
                output_tokens: usage.output_tokens.unwrap_or(0),
            })
            .unwrap_or_else(|| TokenUsage {
                // Fallback: estimate tokens (rough approximation: 1 token ≈ 4 characters)
                input_tokens: (text.len() as f32 / 4.0).ceil() as i32,
                output_tokens: (text_result.len() as f32 / 4.0).ceil() as i32,
            });

        return Ok(ApiCallResult {
            text: text_result,
            token_usage,
        });
    }

    Err(last_error.unwrap_or_else(|| "Max retries exceeded".to_string()))
}

/// Extract translated XML strings from Claude API response
/// Parses <string name="...">...</string> tags and returns a map of name -> translated_text
/// Also restores CDATA wrappers if the original strings had them
fn extract_translations_from_response(
    response: &str,
    source_strings: &[AndroidString],
) -> Result<HashMap<String, String>, String> {
    lazy_static! {
        // Use (?s) flag to make . match newlines, allowing multiline content like CDATA
        static ref STRING_REGEX: Regex = Regex::new(
            r#"(?s)<string\s+name="([^"]+)">(.+?)</string>"#
        ).unwrap();
    }
    
    let mut translations = HashMap::new();
    
    for caps in STRING_REGEX.captures_iter(response) {
        let name = caps.get(1).unwrap().as_str().to_string();
        let mut text = caps.get(2).unwrap().as_str().to_string();
        
        // Find the original string to check for CDATA and restore if needed
        if let Some(original) = source_strings.iter().find(|s| s.name == name) {
            // Restore CDATA wrapper only if original had one (don't add for HTML-only strings)
            text = restore_cdata_wrapper(&text, original.has_cdata);
            
            // Escape apostrophes and double quotes that the AI may have left unescaped.
            // This is done deterministically rather than relying on the AI prompt,
            // since LLMs frequently omit escaping (especially in apostrophe-heavy
            // languages like French and Italian).
            // Skip escaping for CDATA strings — CDATA content is raw and backslash
            // escapes would be rendered literally.
            if !text.contains("<![CDATA[") {
                text = sanitize_invalid_unicode_escapes(&text);
                text = escape_android_apostrophes(&text);
                text = escape_android_double_quotes(&text);
                text = escape_android_resource_ref_chars(&text);
            }
            
            translations.insert(name, text);
        }
    }
    
    // If we didn't find any translations, log the response for debugging
    if translations.is_empty() {
        eprintln!("[translation][error] No XML strings found in response:");
        eprintln!("{}", response);
        return Err("Failed to extract translations from response. The model may have altered the XML format.".to_string());
    }
    
    Ok(translations)
}

/// Build prompt for multi-language translation (Case B)
fn build_multi_language_translation_prompt(
    _strings: &[AndroidString],
    target_languages: &[String],
    project: &Project,
    extra_instructions: &Option<String>,
) -> String {
    // Build language list with names and codes
    // Format: "Language Name (code)" to ensure the model uses the correct code;
    // non-Latin languages also get their native name and script, e.g. "Kannada (ಕನ್ನಡ, written in Kannada script) (kn)"
    let languages_with_codes: Vec<String> = target_languages
        .iter()
        .map(|code| format!("{} ({})", prompt_language_label(code), code))
        .collect();
    let languages_list = languages_with_codes.join(", ");
    
    // Include protected names only if present
    let protected_names_section = if !project.protected_names.is_empty() {
        format!(
            "Brand/terms:\n- Do NOT translate these terms: {}.\n- Translate all other content naturally and consistently with the app context.\n\n",
            project.protected_names.join(", ")
        )
    } else {
        String::new()
    };
    
    // Include extra instructions if present
    let extra_instructions_section = extra_instructions
        .as_ref()
        .filter(|s| !s.trim().is_empty())
        .map(|s| {
            format!(
                "Additional instructions (apply strictly):\n{}\n\n",
                s.trim()
            )
        })
        .unwrap_or_default();
    
    // Include project context if present
    let context_section = if !project.context_prompt.trim().is_empty() {
        format!("App context:\n{}\n\n", project.context_prompt.trim())
    } else {
        String::new()
    };
    
    format!(
        r#"Translate the Android strings from the app "{app_name}" into the following languages: {languages_list}.

{context}Requirements:
- Translate for speakers of each target language. Prefer natural phrasing while preserving meaning.
- Write each language section only in that language. Never switch to another language, even a related one.
- Keep the XML structure EXACTLY as provided. Do not add, remove, or modify any XML tags or attributes.
- Keep string names (in name="...") exactly the same. Do not translate the names.
- Preserve ALL formatting and special characters where possible:
  • Android placeholders: %s, %d, %1$s, %2$d must remain exactly as-is
  • CDATA wrappers: <![CDATA[...]]> must be preserved for HTML content
  • HTML/XML tags: <b>, <i>, <u>, <a href="...">, <xliff:g> must remain identical
  • Special characters and symbols: Try to preserve brackets, braces, and other non-linguistic symbols
- If percent symbols are used (%) they should be escaped by doubling them (%%).
- Do NOT remove, modify, translate, or reorder any tags or their attributes.
- Keep URLs, email addresses, phone numbers, and file paths unchanged.
- Keep punctuation and line breaks unless grammar requires adjustment.
- Avoid double spaces.

{protected_names}{extra}IMPORTANT: Format your response with clear language separators. For each language, use this exact format:

[LANGUAGE: <language_code>]
<string name="...">...</string>
<string name="...">...</string>
...

[LANGUAGE: <next_language_code>]
<string name="...">...</string>
...

Use the EXACT language codes specified above: {language_codes_list}

The following are Android XML strings. Translate ONLY the text content inside the tags for each language, keeping the XML structure intact:"#,
        language_codes_list = target_languages.join(", "),
        app_name = project.name,
        languages_list = languages_list,
        context = context_section,
        protected_names = protected_names_section,
        extra = extra_instructions_section
    )
}

/// Normalize language code - handles Android legacy codes and ISO standard codes
/// Maps "id" <-> "in" for Indonesian (both are valid, Android uses "in", ISO uses "id")
fn normalize_language_code(code: &str) -> String {
    let code_lower = code.to_lowercase();
    // Normalize Indonesian: both "in" and "id" map to "in" (Android standard)
    if code_lower == "id" {
        "in".to_string()
    } else {
        code_lower
    }
}

/// Parse multi-language response with language separators
/// Expected format: [LANGUAGE: es]\n<string>...</string>\n...\n[LANGUAGE: fr]\n...
fn parse_multi_language_response(
    response: &str,
    target_languages: &[String],
    source_strings: &[AndroidString],
) -> Result<HashMap<String, HashMap<String, String>>, String> {
    lazy_static! {
        // Match [LANGUAGE: code] marker
        static ref LANGUAGE_MARKER_REGEX: Regex = Regex::new(
            r#"\[LANGUAGE:\s*([^\]]+)\]"#
        ).unwrap();
        
        // Match <string name="...">...</string>
        static ref STRING_REGEX: Regex = Regex::new(
            r#"(?s)<string\s+name="([^"]+)">(.+?)</string>"#
        ).unwrap();
    }
    
    // Normalize target languages for matching (handle "in" vs "id")
    let normalized_targets: Vec<(String, String)> = target_languages
        .iter()
        .map(|l| (normalize_language_code(l), l.clone()))
        .collect();
    
    let mut results: HashMap<String, HashMap<String, String>> = HashMap::new();
    
    // Find all language markers and their positions
    let mut language_markers: Vec<(usize, String)> = Vec::new();
    for caps in LANGUAGE_MARKER_REGEX.captures_iter(response) {
        let start_pos = caps.get(0).unwrap().start();
        let language_code_from_response = caps.get(1).unwrap().as_str().trim();
        let normalized_response_code = normalize_language_code(language_code_from_response);
        
        // Check if this language matches any target (using normalized codes)
        if normalized_targets.iter().any(|(norm, _)| *norm == normalized_response_code) {
            language_markers.push((start_pos, normalized_response_code));
        }
    }
    
    if language_markers.is_empty() {
        return Err("No language markers found in response. Expected format: [LANGUAGE: code] followed by XML strings.".to_string());
    }
    
    // Sort by position
    language_markers.sort_by_key(|(pos, _)| *pos);
    
    // Extract content for each language section
    for (idx, (start_pos, normalized_response_code)) in language_markers.iter().enumerate() {
        // Find matching language from target list (using normalized codes)
        let matching_language = normalized_targets.iter()
            .find(|(norm, _)| *norm == *normalized_response_code)
            .map(|(_, original)| original.clone());
        
        if matching_language.is_none() {
            eprintln!(
                "[translation][warn] Found language '{}' in response but couldn't match to target list",
                normalized_response_code
            );
            continue;
        }
        let language_code = matching_language.unwrap();
        
        // Determine end position (start of next marker or end of string)
        let end_pos = if idx + 1 < language_markers.len() {
            language_markers[idx + 1].0
        } else {
            response.len()
        };
        
        // Extract section content (skip the marker itself)
        let marker_slice = &response[*start_pos..];
        let marker_end = start_pos + marker_slice.find(']').unwrap_or(0) + 1;
        let section_content = &response[marker_end..end_pos];
        
        // Parse strings from this section
        let mut translations = HashMap::new();
        for str_caps in STRING_REGEX.captures_iter(section_content) {
            let name = str_caps.get(1).unwrap().as_str().to_string();
            let mut text = str_caps.get(2).unwrap().as_str().to_string();
            
            // Find original string to restore CDATA if needed
            if let Some(original) = source_strings.iter().find(|s| s.name == name) {
                text = restore_cdata_wrapper(&text, original.has_cdata);
                
                // Escape apostrophes and double quotes — skip for CDATA (raw content)
                if !text.contains("<![CDATA[") {
                    text = sanitize_invalid_unicode_escapes(&text);
                    text = escape_android_apostrophes(&text);
                    text = escape_android_double_quotes(&text);
                    text = escape_android_resource_ref_chars(&text);
                }
                
                translations.insert(name, text);
            }
        }
        
        if !translations.is_empty() {
            results.insert(language_code, translations);
        }
    }
    
    if results.is_empty() {
        return Err("Failed to parse any translations from multi-language response.".to_string());
    }
    
    Ok(results)
}

/// Build prompt for single-language translation with context
/// Closing line placed after the strings, right where the model starts writing. A single mention of
/// the language at the top of a long prompt was not enough: Haiku once wrote a whole Kannada batch in Hindi.
fn language_reminder(target_languages: &[String]) -> String {
    match target_languages {
        [code] => format!("Write every translation in {}.", prompt_language_label(code)),
        codes => {
            let labels: Vec<String> = codes
                .iter()
                .map(|code| format!("{} ({})", prompt_language_label(code), code))
                .collect();
            format!("Write each language section only in that language: {}.", labels.join(", "))
        }
    }
}

fn build_translation_prompt(
    _strings: &[AndroidString],
    target_language: &str,
    project: &Project,
    extra_instructions: &Option<String>,
) -> String {
    let language_name = get_language_name(target_language);
    let language_label = prompt_language_label(target_language);

    // Include protected names only if present
    let protected_names_section = if !project.protected_names.is_empty() {
        format!(
            "Brand/terms:\n- Do NOT translate these terms: {}.\n- Translate all other content naturally and consistently with the app context.\n\n",
            project.protected_names.join(", ")
        )
    } else {
        String::new()
    };
    
    // Include extra instructions if present
    let extra_instructions_section = extra_instructions
        .as_ref()
        .filter(|s| !s.trim().is_empty())
        .map(|s| {
            format!(
                "Additional instructions (apply strictly):\n{}\n\n",
                s.trim()
            )
        })
        .unwrap_or_default();
    
    // Include project context if present
    let context_section = if !project.context_prompt.trim().is_empty() {
        format!("App context:\n{}\n\n", project.context_prompt.trim())
    } else {
        String::new()
    };
    
    format!(
        r#"Translate the Android strings from the app "{app_name}" into {language_label}.

{context}Requirements:
- Translate for speakers of {language_name}. Prefer natural phrasing while preserving meaning.
- Write every string in {language_name} only. Never switch to another language, even a related one.
- Keep the XML structure EXACTLY as provided. Do not add, remove, or modify any XML tags or attributes.
- Keep string names (in name="...") exactly the same. Do not translate the names.
- Preserve ALL formatting and special characters where possible:
  • Android placeholders: %s, %d, %1$s, %2$d must remain exactly as-is
  • CDATA wrappers: <![CDATA[...]]> must be preserved for HTML content
  • HTML/XML tags: <b>, <i>, <u>, <a href="...">, <xliff:g> must remain identical
  • Special characters and symbols: Try to preserve brackets, braces, and other non-linguistic symbols
- If percent symbols are used (%) they should be escaped by doubling them (%%).
- Do NOT remove, modify, translate, or reorder any tags or their attributes.
- Keep URLs, email addresses, phone numbers, and file paths unchanged.
- Keep punctuation and line breaks unless grammar requires adjustment.
- Avoid double spaces.

{protected_names}{extra}The following are Android XML strings. Translate ONLY the text content inside the tags, keeping the XML structure intact:"#,
        app_name = project.name,
        language_label = language_label,
        language_name = language_name,
        context = context_section,
        protected_names = protected_names_section,
        extra = extra_instructions_section
    )
}