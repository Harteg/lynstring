use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use ts_rs::TS;

use crate::models::locale_names_generated::{
    compose_locale_name, CLDR_LANGUAGE_NAMES, CLDR_TERRITORY_NAMES,
};

#[derive(Debug, Serialize, Deserialize, TS, Clone)]
#[ts(export)]
pub struct Language {
    pub code: &'static str,
    pub name: &'static str,
    pub emoji: &'static str,
}

// Centralized language data - matches src/lib/utils/languages.ts
pub const LANGUAGES: &[Language] = &[
    Language {
        code: "af",
        name: "Afrikaans",
        emoji: "🇿🇦",
    },
    Language {
        code: "am",
        name: "Amharic",
        emoji: "🇪🇹",
    },
    Language {
        code: "ar",
        name: "Arabic",
        emoji: "🇸🇦",
    },
    Language {
        code: "ar-rSA",
        name: "Arabic (Saudi Arabia)",
        emoji: "🇸🇦",
    },
    Language {
        code: "as",
        name: "Assamese",
        emoji: "🇮🇳",
    },
    Language {
        code: "az",
        name: "Azerbaijani",
        emoji: "🇦🇿",
    },
    Language {
        code: "be",
        name: "Belarusian",
        emoji: "🇧🇾",
    },
    Language {
        code: "bg",
        name: "Bulgarian",
        emoji: "🇧🇬",
    },
    Language {
        code: "bn",
        name: "Bengali",
        emoji: "🇧🇩",
    },
    Language {
        code: "bs",
        name: "Bosnian",
        emoji: "🇧🇦",
    },
    Language {
        code: "ca",
        name: "Catalan",
        emoji: "🇪🇸",
    },
    Language {
        code: "cs",
        name: "Czech",
        emoji: "🇨🇿",
    },
    Language {
        code: "da",
        name: "Danish",
        emoji: "🇩🇰",
    },
    Language {
        code: "de",
        name: "German",
        emoji: "🇩🇪",
    },
    Language {
        code: "de-rAT",
        name: "German (Austria)",
        emoji: "🇦🇹",
    },
    Language {
        code: "de-rCH",
        name: "German (Switzerland)",
        emoji: "🇨🇭",
    },
    Language {
        code: "el",
        name: "Greek",
        emoji: "🇬🇷",
    },
    Language {
        code: "en",
        name: "English",
        emoji: "🇬🇧",
    },
    Language {
        code: "en-rGB",
        name: "English (UK)",
        emoji: "🇬🇧",
    },
    Language {
        code: "en-rUS",
        name: "English (US)",
        emoji: "🇺🇸",
    },
    Language {
        code: "en-rAU",
        name: "English (Australia)",
        emoji: "🇦🇺",
    },
    Language {
        code: "en-rIN",
        name: "English (India)",
        emoji: "🇮🇳",
    },
    Language {
        code: "es",
        name: "Spanish",
        emoji: "🇪🇸",
    },
    Language {
        code: "es-419",
        name: "Spanish (Latin America)",
        emoji: "🌎",
    },
    Language {
        code: "es-rES",
        name: "Spanish (Spain)",
        emoji: "🇪🇸",
    },
    Language {
        code: "es-rUS",
        name: "Spanish (US)",
        emoji: "🇺🇸",
    },
    Language {
        code: "es-rMX",
        name: "Spanish (Mexico)",
        emoji: "🇲🇽",
    },
    Language {
        code: "et",
        name: "Estonian",
        emoji: "🇪🇪",
    },
    Language {
        code: "eu",
        name: "Basque",
        emoji: "🇪🇸",
    },
    Language {
        code: "fa",
        name: "Persian",
        emoji: "🇮🇷",
    },
    Language {
        code: "fi",
        name: "Finnish",
        emoji: "🇫🇮",
    },
    Language {
        code: "fil",
        name: "Filipino",
        emoji: "🇵🇭",
    },
    Language {
        code: "fr",
        name: "French",
        emoji: "🇫🇷",
    },
    Language {
        code: "fr-rCA",
        name: "French (Canada)",
        emoji: "🇨🇦",
    },
    Language {
        code: "fr-rFR",
        name: "French (France)",
        emoji: "🇫🇷",
    },
    Language {
        code: "gl",
        name: "Galician",
        emoji: "🇪🇸",
    },
    Language {
        code: "gu",
        name: "Gujarati",
        emoji: "🇮🇳",
    },
    Language {
        code: "hi",
        name: "Hindi",
        emoji: "🇮🇳",
    },
    Language {
        code: "hr",
        name: "Croatian",
        emoji: "🇭🇷",
    },
    Language {
        code: "hu",
        name: "Hungarian",
        emoji: "🇭🇺",
    },
    Language {
        code: "hy",
        name: "Armenian",
        emoji: "🇦🇲",
    },
    Language {
        code: "id",
        name: "Indonesian",
        emoji: "🇮🇩",
    },
    Language {
        code: "in",
        name: "Indonesian",
        emoji: "🇮🇩",
    },
    Language {
        code: "is",
        name: "Icelandic",
        emoji: "🇮🇸",
    },
    Language {
        code: "it",
        name: "Italian",
        emoji: "🇮🇹",
    },
    Language {
        code: "iw",
        name: "Hebrew",
        emoji: "🇮🇱",
    },
    Language {
        code: "ja",
        name: "Japanese",
        emoji: "🇯🇵",
    },
    Language {
        code: "ka",
        name: "Georgian",
        emoji: "🇬🇪",
    },
    Language {
        code: "kk",
        name: "Kazakh",
        emoji: "🇰🇿",
    },
    Language {
        code: "km",
        name: "Khmer",
        emoji: "🇰🇭",
    },
    Language {
        code: "kn",
        name: "Kannada",
        emoji: "🇮🇳",
    },
    Language {
        code: "ko",
        name: "Korean",
        emoji: "🇰🇷",
    },
    Language {
        code: "ky",
        name: "Kyrgyz",
        emoji: "🇰🇬",
    },
    Language {
        code: "lo",
        name: "Lao",
        emoji: "🇱🇦",
    },
    Language {
        code: "lt",
        name: "Lithuanian",
        emoji: "🇱🇹",
    },
    Language {
        code: "lv",
        name: "Latvian",
        emoji: "🇱🇻",
    },
    Language {
        code: "mk",
        name: "Macedonian",
        emoji: "🇲🇰",
    },
    Language {
        code: "ml",
        name: "Malayalam",
        emoji: "🇮🇳",
    },
    Language {
        code: "mn",
        name: "Mongolian",
        emoji: "🇲🇳",
    },
    Language {
        code: "mr",
        name: "Marathi",
        emoji: "🇮🇳",
    },
    Language {
        code: "ms",
        name: "Malay",
        emoji: "🇲🇾",
    },
    Language {
        code: "my",
        name: "Burmese",
        emoji: "🇲🇲",
    },
    Language {
        code: "ne",
        name: "Nepali",
        emoji: "🇳🇵",
    },
    Language {
        code: "nl",
        name: "Dutch",
        emoji: "🇳🇱",
    },
    Language {
        code: "nl-rBE",
        name: "Dutch (Belgium)",
        emoji: "🇧🇪",
    },
    Language {
        code: "no",
        name: "Norwegian",
        emoji: "🇳🇴",
    },
    Language {
        code: "nb",
        name: "Norwegian Bokmål",
        emoji: "🇳🇴",
    },
    Language {
        code: "nn",
        name: "Norwegian Nynorsk",
        emoji: "🇳🇴",
    },
    Language {
        code: "or",
        name: "Odia",
        emoji: "🇮🇳",
    },
    Language {
        code: "pa",
        name: "Punjabi",
        emoji: "🇮🇳",
    },
    Language {
        code: "pl",
        name: "Polish",
        emoji: "🇵🇱",
    },
    Language {
        code: "pt",
        name: "Portuguese",
        emoji: "🇵🇹",
    },
    Language {
        code: "pt-rBR",
        name: "Portuguese (Brazil)",
        emoji: "🇧🇷",
    },
    Language {
        code: "pt-rPT",
        name: "Portuguese (Portugal)",
        emoji: "🇵🇹",
    },
    Language {
        code: "ro",
        name: "Romanian",
        emoji: "🇷🇴",
    },
    Language {
        code: "ru",
        name: "Russian",
        emoji: "🇷🇺",
    },
    Language {
        code: "si",
        name: "Sinhala",
        emoji: "🇱🇰",
    },
    Language {
        code: "sk",
        name: "Slovak",
        emoji: "🇸🇰",
    },
    Language {
        code: "sl",
        name: "Slovenian",
        emoji: "🇸🇮",
    },
    Language {
        code: "sq",
        name: "Albanian",
        emoji: "🇦🇱",
    },
    Language {
        code: "sr",
        name: "Serbian",
        emoji: "🇷🇸",
    },
    Language {
        code: "sv",
        name: "Swedish",
        emoji: "🇸🇪",
    },
    Language {
        code: "sw",
        name: "Swahili",
        emoji: "🇰🇪",
    },
    Language {
        code: "ta",
        name: "Tamil",
        emoji: "🇮🇳",
    },
    Language {
        code: "te",
        name: "Telugu",
        emoji: "🇮🇳",
    },
    Language {
        code: "th",
        name: "Thai",
        emoji: "🇹🇭",
    },
    Language {
        code: "tr",
        name: "Turkish",
        emoji: "🇹🇷",
    },
    Language {
        code: "uk",
        name: "Ukrainian",
        emoji: "🇺🇦",
    },
    Language {
        code: "ur",
        name: "Urdu",
        emoji: "🇵🇰",
    },
    Language {
        code: "uz",
        name: "Uzbek",
        emoji: "🇺🇿",
    },
    Language {
        code: "vi",
        name: "Vietnamese",
        emoji: "🇻🇳",
    },
    Language {
        code: "zh",
        name: "Chinese",
        emoji: "🇨🇳",
    },
    Language {
        code: "zh-rCN",
        name: "Chinese (Simplified)",
        emoji: "🇨🇳",
    },
    Language {
        code: "zh-rHK",
        name: "Chinese (Hong Kong)",
        emoji: "🇭🇰",
    },
    Language {
        code: "zh-rTW",
        name: "Chinese (Traditional)",
        emoji: "🇹🇼",
    },
    Language {
        code: "zh-rSG",
        name: "Chinese (Singapore)",
        emoji: "🇸🇬",
    },
    Language {
        code: "zu",
        name: "Zulu",
        emoji: "🇿🇦",
    },
];

/// Normalize BCP 47 / ISO codes (e.g. en-US) to Android resource folder format (e.g. en-rUS).
/// This ensures existing projects with values-en-US folders still get correct display names.
fn normalize_code_for_lookup(code: &str) -> &str {
    match code {
        "en-US" => "en-rUS",
        "en-GB" => "en-rGB",
        "en-AU" => "en-rAU",
        "en-IN" => "en-rIN",
        "es-ES" => "es-rES",
        "es-US" => "es-rUS",
        "es-MX" => "es-rMX",
        "de-AT" => "de-rAT",
        "de-CH" => "de-rCH",
        "fr-CA" => "fr-rCA",
        "fr-FR" => "fr-rFR",
        "pt-BR" => "pt-rBR",
        "pt-PT" => "pt-rPT",
        "ar-SA" => "ar-rSA",
        "nl-BE" => "nl-rBE",
        "zh-CN" => "zh-rCN",
        "zh-HK" => "zh-rHK",
        "zh-TW" => "zh-rTW",
        "zh-SG" => "zh-rSG",
        _ => code,
    }
}

/// Parse locale code into (language, region) for composition.
/// Handles Android format (en-rUS) and BCP47 numeric (es-419).
fn parse_locale_parts(code: &str) -> Option<(&str, &str)> {
    let parts: Vec<&str> = code.splitn(2, '-').collect();
    if parts.len() != 2 {
        return None;
    }
    let (lang, rest) = (parts[0], parts[1]);
    let region = if rest.starts_with('r') && rest.len() >= 2 {
        &rest[1..]
    } else if rest.chars().all(|c| c.is_ascii_digit()) {
        rest
    } else if rest.len() == 2 && rest.chars().all(|c| c.is_ascii_alphabetic()) {
        rest
    } else {
        return None;
    };
    Some((lang, region))
}

/// Get language name by code, falling back to CLDR composition for unknown locales.
pub fn get_language_name(code: &str) -> Cow<'static, str> {
    let normalized = normalize_code_for_lookup(code);

    // 1. Check curated LANGUAGES first
    if let Some(lang) = LANGUAGES.iter().find(|l| l.code == normalized) {
        return Cow::Borrowed(lang.name);
    }

    // 2. Direct lookup in CLDR (includes compound codes like en-US, es-419 and Android aliases)
    if let Some(name) = CLDR_LANGUAGE_NAMES.get(normalized) {
        return Cow::Borrowed(*name);
    }

    // 3. Try composing from language + region (e.g. en-rAS -> "English (American Samoa)")
    if let Some((lang, region)) = parse_locale_parts(normalized) {
        if let Some(composed) = compose_locale_name(lang, region) {
            return Cow::Owned(composed);
        }
    }

    Cow::Owned(code.to_string())
}

/// Convert a 2-letter ISO 3166-1 region code to its flag emoji (e.g. "US" -> "🇺🇸")
fn region_to_flag_emoji(region: &str) -> Option<String> {
    if region.len() != 2 {
        return None;
    }
    let upper = region.to_uppercase();
    let mut chars = upper.chars();
    let c1 = chars.next()?;
    let c2 = chars.next()?;
    if !c1.is_ascii_alphabetic() || !c2.is_ascii_alphabetic() {
        return None;
    }
    // Regional Indicator Symbol Letter A = U+1F1E6
    let base = '\u{1F1E6}' as u32 - 'A' as u32;
    let r1 = char::from_u32(base + c1 as u32)?;
    let r2 = char::from_u32(base + c2 as u32)?;
    Some(format!("{r1}{r2}"))
}

/// Get language emoji by code. Uses curated list first, then derives flag from region for composed locales.
pub fn get_language_emoji(code: &str) -> Cow<'static, str> {
    let normalized = normalize_code_for_lookup(code);

    // 1. Check curated LANGUAGES first
    if let Some(lang) = LANGUAGES.iter().find(|l| l.code == normalized) {
        return Cow::Borrowed(lang.emoji);
    }

    // 2. Derive flag from region for composed locales (e.g. af-rZA -> 🇿🇦)
    // Only for known territories - invalid codes like "SP" (Serbia is RS) produce broken glyphs
    if let Some((_, region)) = parse_locale_parts(normalized) {
        if region.len() == 2 && region.chars().all(|c| c.is_ascii_alphabetic()) {
            let region_upper = region.to_uppercase();
            if CLDR_TERRITORY_NAMES.contains_key(region_upper.as_str()) {
                if let Some(flag) = region_to_flag_emoji(region) {
                    return Cow::Owned(flag);
                }
            }
        }
    }

    Cow::Borrowed("🌐")
}

/// Get full language object by code
pub fn get_language(code: &str) -> Option<&Language> {
    let normalized = normalize_code_for_lookup(code);
    LANGUAGES.iter().find(|lang| lang.code == normalized)
}
