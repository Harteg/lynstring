use std::collections::HashMap;

use crate::models::get_language_name;

/// A writing system that can be recognised from Unicode code point ranges.
pub struct Script {
    pub name: &'static str,
    ranges: &'static [(u32, u32)],
}

impl Script {
    fn contains(&self, c: char) -> bool {
        let cp = c as u32;
        self.ranges.iter().any(|&(lo, hi)| cp >= lo && cp <= hi)
    }
}

const ARABIC: Script = Script { name: "Arabic", ranges: &[(0x0600, 0x06FF), (0x0750, 0x077F), (0x08A0, 0x08FF), (0xFB50, 0xFDFF), (0xFE70, 0xFEFF)] };
const ARMENIAN: Script = Script { name: "Armenian", ranges: &[(0x0530, 0x058F), (0xFB13, 0xFB17)] };
const BENGALI: Script = Script { name: "Bengali", ranges: &[(0x0980, 0x09FF)] };
const CJK: Script = Script { name: "Chinese characters", ranges: &[(0x3000, 0x303F), (0x3400, 0x4DBF), (0x4E00, 0x9FFF), (0xF900, 0xFAFF)] };
const CYRILLIC: Script = Script { name: "Cyrillic", ranges: &[(0x0400, 0x052F), (0x1C80, 0x1C8F)] };
const DEVANAGARI: Script = Script { name: "Devanagari", ranges: &[(0x0900, 0x097F), (0xA8E0, 0xA8FF)] };
const ETHIOPIC: Script = Script { name: "Ethiopic", ranges: &[(0x1200, 0x139F), (0x2D80, 0x2DDF)] };
const GEORGIAN: Script = Script { name: "Georgian", ranges: &[(0x10A0, 0x10FF), (0x1C90, 0x1CBF)] };
const GREEK: Script = Script { name: "Greek", ranges: &[(0x0370, 0x03FF), (0x1F00, 0x1FFF)] };
const GUJARATI: Script = Script { name: "Gujarati", ranges: &[(0x0A80, 0x0AFF)] };
const GURMUKHI: Script = Script { name: "Gurmukhi", ranges: &[(0x0A00, 0x0A7F)] };
const HANGUL: Script = Script { name: "Hangul", ranges: &[(0x1100, 0x11FF), (0x3130, 0x318F), (0xAC00, 0xD7AF), (0x3000, 0x303F), (0x4E00, 0x9FFF)] };
const HEBREW: Script = Script { name: "Hebrew", ranges: &[(0x0590, 0x05FF), (0xFB1D, 0xFB4F)] };
const JAPANESE: Script = Script { name: "Japanese kana and kanji", ranges: &[(0x3000, 0x30FF), (0x31F0, 0x31FF), (0x3400, 0x4DBF), (0x4E00, 0x9FFF), (0xF900, 0xFAFF), (0xFF66, 0xFF9F)] };
const KANNADA: Script = Script { name: "Kannada", ranges: &[(0x0C80, 0x0CFF)] };
const KHMER: Script = Script { name: "Khmer", ranges: &[(0x1780, 0x17FF), (0x19E0, 0x19FF)] };
const LAO: Script = Script { name: "Lao", ranges: &[(0x0E80, 0x0EFF)] };
const MALAYALAM: Script = Script { name: "Malayalam", ranges: &[(0x0D00, 0x0D7F)] };
const MYANMAR: Script = Script { name: "Myanmar", ranges: &[(0x1000, 0x109F), (0xAA60, 0xAA7F)] };
const ODIA: Script = Script { name: "Odia", ranges: &[(0x0B00, 0x0B7F)] };
const SINHALA: Script = Script { name: "Sinhala", ranges: &[(0x0D80, 0x0DFF)] };
const TAMIL: Script = Script { name: "Tamil", ranges: &[(0x0B80, 0x0BFF)] };
const TELUGU: Script = Script { name: "Telugu", ranges: &[(0x0C00, 0x0C7F)] };
const THAI: Script = Script { name: "Thai", ranges: &[(0x0E00, 0x0E7F)] };

/// Every script we can recognise, used to name what a wrong-script response was written in.
const ALL_SCRIPTS: &[&Script] = &[
    &ARABIC, &ARMENIAN, &BENGALI, &CYRILLIC, &DEVANAGARI, &ETHIOPIC, &GEORGIAN, &GREEK,
    &GUJARATI, &GURMUKHI, &HANGUL, &HEBREW, &JAPANESE, &CJK, &KANNADA, &KHMER, &LAO,
    &MALAYALAM, &MYANMAR, &ODIA, &SINHALA, &TAMIL, &TELUGU, &THAI,
];

/// A language written in a non-Latin script, with its name in that language.
pub struct ScriptLanguage {
    pub native_name: &'static str,
    pub script: &'static Script,
}

/// Base language code -> native name and expected script.
/// Only languages with their own (non-Latin) script are listed; Latin-script languages are not checked.
const SCRIPT_LANGUAGES: &[(&str, ScriptLanguage)] = &[
    ("am", ScriptLanguage { native_name: "አማርኛ", script: &ETHIOPIC }),
    ("ar", ScriptLanguage { native_name: "العربية", script: &ARABIC }),
    ("as", ScriptLanguage { native_name: "অসমীয়া", script: &BENGALI }),
    ("be", ScriptLanguage { native_name: "Беларуская", script: &CYRILLIC }),
    ("bg", ScriptLanguage { native_name: "Български", script: &CYRILLIC }),
    ("bn", ScriptLanguage { native_name: "বাংলা", script: &BENGALI }),
    ("el", ScriptLanguage { native_name: "Ελληνικά", script: &GREEK }),
    ("fa", ScriptLanguage { native_name: "فارسی", script: &ARABIC }),
    ("gu", ScriptLanguage { native_name: "ગુજરાતી", script: &GUJARATI }),
    ("he", ScriptLanguage { native_name: "עברית", script: &HEBREW }),
    ("hi", ScriptLanguage { native_name: "हिन्दी", script: &DEVANAGARI }),
    ("hy", ScriptLanguage { native_name: "Հայերեն", script: &ARMENIAN }),
    ("iw", ScriptLanguage { native_name: "עברית", script: &HEBREW }),
    ("ja", ScriptLanguage { native_name: "日本語", script: &JAPANESE }),
    ("ka", ScriptLanguage { native_name: "ქართული", script: &GEORGIAN }),
    ("kk", ScriptLanguage { native_name: "Қазақ тілі", script: &CYRILLIC }),
    ("km", ScriptLanguage { native_name: "ខ្មែរ", script: &KHMER }),
    ("kn", ScriptLanguage { native_name: "ಕನ್ನಡ", script: &KANNADA }),
    ("ko", ScriptLanguage { native_name: "한국어", script: &HANGUL }),
    ("ky", ScriptLanguage { native_name: "Кыргызча", script: &CYRILLIC }),
    ("lo", ScriptLanguage { native_name: "ລາວ", script: &LAO }),
    ("mk", ScriptLanguage { native_name: "Македонски", script: &CYRILLIC }),
    ("ml", ScriptLanguage { native_name: "മലയാളം", script: &MALAYALAM }),
    ("mn", ScriptLanguage { native_name: "Монгол", script: &CYRILLIC }),
    ("mr", ScriptLanguage { native_name: "मराठी", script: &DEVANAGARI }),
    ("my", ScriptLanguage { native_name: "မြန်မာ", script: &MYANMAR }),
    ("ne", ScriptLanguage { native_name: "नेपाली", script: &DEVANAGARI }),
    ("or", ScriptLanguage { native_name: "ଓଡ଼ିଆ", script: &ODIA }),
    ("pa", ScriptLanguage { native_name: "ਪੰਜਾਬੀ", script: &GURMUKHI }),
    ("ru", ScriptLanguage { native_name: "Русский", script: &CYRILLIC }),
    ("si", ScriptLanguage { native_name: "සිංහල", script: &SINHALA }),
    ("sr", ScriptLanguage { native_name: "Српски", script: &CYRILLIC }),
    ("ta", ScriptLanguage { native_name: "தமிழ்", script: &TAMIL }),
    ("te", ScriptLanguage { native_name: "తెలుగు", script: &TELUGU }),
    ("th", ScriptLanguage { native_name: "ไทย", script: &THAI }),
    ("uk", ScriptLanguage { native_name: "Українська", script: &CYRILLIC }),
    ("ur", ScriptLanguage { native_name: "اردو", script: &ARABIC }),
    ("zh", ScriptLanguage { native_name: "中文", script: &CJK }),
];

/// Look up the expected script for an Android locale code (e.g. "kn", "kn-rIN", "b+sr+Cyrl").
/// Returns None for Latin-script languages and for locales that override the default script
/// (e.g. "b+sr+Latn", "pa-rPK").
pub fn script_language(code: &str) -> Option<&'static ScriptLanguage> {
    let (base, region, explicit_script) = if let Some(rest) = code.strip_prefix("b+") {
        let parts: Vec<&str> = rest.split('+').collect();
        let script = parts.iter().skip(1).find(|p| p.len() == 4).copied();
        (parts[0], None, script)
    } else {
        let mut parts = code.splitn(2, '-');
        let base = parts.next().unwrap_or(code);
        let region = parts.next().map(|r| r.trim_start_matches('r'));
        (base, region, None)
    };

    // A script subtag other than the default (e.g. Latn) means we can't assume the script
    if let Some(script) = explicit_script {
        let default_matches = matches!(
            (base, script),
            ("sr", "Cyrl") | ("zh", "Hans") | ("zh", "Hant") | ("pa", "Guru")
        );
        if !default_matches {
            return None;
        }
    }
    // Punjabi in Pakistan is written in Shahmukhi (Arabic script), not Gurmukhi
    if base == "pa" && region == Some("PK") {
        return None;
    }

    SCRIPT_LANGUAGES
        .iter()
        .find(|(c, _)| c.eq_ignore_ascii_case(base))
        .map(|(_, l)| l)
}

/// Language name for prompts, including the native name and script when the language has its own
/// script, e.g. "Kannada (ಕನ್ನಡ, written in Kannada script)".
pub fn prompt_language_label(code: &str) -> String {
    let name = get_language_name(code);
    match script_language(code) {
        Some(lang) => format!("{} ({}, written in {} script)", name, lang.native_name, lang.script.name),
        None => name.to_string(),
    }
}

fn is_latin(c: char) -> bool {
    c.is_ascii_alphabetic() || ('\u{00C0}'..='\u{024F}').contains(&c) || ('\u{1E00}'..='\u{1EFF}').contains(&c)
}

/// Minimum number of wrong-script letters before a response is rejected (avoids flagging stray symbols).
const MIN_FOREIGN_LETTERS: usize = 10;
/// Reject when more than this share of non-Latin letters are in the wrong script.
const MAX_FOREIGN_SHARE: f64 = 0.10;
/// Reject an all-Latin response for a non-Latin language once it has this many Latin letters.
const MIN_LATIN_LETTERS_FOR_UNTRANSLATED: usize = 100;

/// Check that translated texts are written in the script expected for `code`.
/// Latin letters are ignored (brand names, URLs, placeholders and markup are Latin in any language),
/// except when a response contains no letters of the expected script at all.
/// Returns Err with a human-readable reason when the texts are mostly in another script.
pub fn check_translation_script<'a, I>(code: &str, texts: I) -> Result<(), String>
where
    I: IntoIterator<Item = &'a String>,
{
    let Some(lang) = script_language(code) else {
        return Ok(());
    };

    let mut expected = 0usize;
    let mut latin = 0usize;
    let mut foreign: HashMap<&'static str, usize> = HashMap::new();
    for text in texts {
        for c in text.chars().filter(|c| c.is_alphabetic()) {
            if lang.script.contains(c) {
                expected += 1;
            } else if is_latin(c) {
                latin += 1;
            } else {
                let script = ALL_SCRIPTS.iter().find(|s| s.contains(c)).map_or("another script", |s| s.name);
                *foreign.entry(script).or_insert(0) += 1;
            }
        }
    }

    let foreign_total: usize = foreign.values().sum();
    let language_name = get_language_name(code);
    if foreign_total >= MIN_FOREIGN_LETTERS
        && foreign_total as f64 > MAX_FOREIGN_SHARE * (expected + foreign_total) as f64
    {
        let (main_script, _) = foreign.iter().max_by_key(|(_, n)| **n).unwrap();
        let share = 100.0 * foreign_total as f64 / (expected + foreign_total) as f64;
        return Err(format!(
            "{} translation is {:.0}% in {} script instead of {} script",
            language_name, share, main_script, lang.script.name
        ));
    }
    if expected == 0 && foreign_total == 0 && latin >= MIN_LATIN_LETTERS_FOR_UNTRANSLATED {
        return Err(format!(
            "{} translation contains no {} script (it looks untranslated)",
            language_name, lang.script.name
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn texts(items: &[&str]) -> Vec<String> {
        items.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn accepts_kannada_with_brand_names() {
        let t = texts(&["CrookCatcher ತೆರೆಯಲಾಗಿದೆ", "www.crookcatcher.app ಗೆ ಭೇಟಿ ನೀಡಿ", "ನಿಖರತೆ %1$s"]);
        assert!(check_translation_script("kn", &t).is_ok());
    }

    #[test]
    fn rejects_hindi_for_kannada() {
        let t = texts(&["वर्तमान योजना", "बिलिंग समर्थित नहीं", "भुगतान विफल", "ನಿಖರತೆ"]);
        let err = check_translation_script("kn", &t).unwrap_err();
        assert!(err.contains("Devanagari"), "{}", err);
        assert!(err.contains("Kannada"), "{}", err);
    }

    #[test]
    fn rejects_untranslated_english_for_thai() {
        let t = texts(&[
            "Open the app to finish setting up your account and start protecting your phone today",
            "Recordings are uploaded to Google Drive when you are connected to the internet",
        ]);
        assert!(check_translation_script("th", &t).is_err());
    }

    #[test]
    fn ignores_latin_script_languages() {
        let t = texts(&["वर्तमान योजना वर्तमान योजना"]);
        assert!(check_translation_script("es", &t).is_ok());
    }

    #[test]
    fn handles_android_locale_forms() {
        assert_eq!(script_language("kn-rIN").unwrap().script.name, "Kannada");
        assert_eq!(script_language("b+sr+Cyrl").unwrap().script.name, "Cyrillic");
        assert!(script_language("b+sr+Latn").is_none());
        assert!(script_language("pa-rPK").is_none());
        assert_eq!(script_language("zh-rTW").unwrap().script.name, "Chinese characters");
    }

    #[test]
    fn japanese_kana_and_kanji_are_accepted() {
        let t = texts(&["設定を完了してください", "カメラ"]);
        assert!(check_translation_script("ja", &t).is_ok());
    }

    #[test]
    fn prompt_label_includes_native_name() {
        assert_eq!(prompt_language_label("kn"), "Kannada (ಕನ್ನಡ, written in Kannada script)");
        assert_eq!(prompt_language_label("es"), get_language_name("es"));
    }
}
