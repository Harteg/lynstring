import { invoke } from "@tauri-apps/api/core";

export interface Language {
  code: string;
  name: string;
  emoji: string;
}

let cachedLanguages: Language[] | null = null;
let languageMap: Map<string, Language> | null = null;
/** Display info for project languages (from backend, includes composed locales like af-rZA) */
let projectLanguageDisplay: Map<string, { name: string; emoji: string }> | null = null;

// Set display info for project languages (from load_project_data). Used for composed locales like af-rZA.
export function setProjectLanguageDisplay(display: Record<string, { name: string; emoji: string }> | null) {
  projectLanguageDisplay = display ? new Map(Object.entries(display)) : null;
}

// Fetch languages from the Rust backend
export async function getLanguages(): Promise<Language[]> {
  if (cachedLanguages) {
    return cachedLanguages;
  }
  
  try {
    cachedLanguages = await invoke<Language[]>("get_supported_languages");
    // Update the map when we get new data
    languageMap = new Map(cachedLanguages.map(lang => [lang.code, lang]));
    return cachedLanguages;
  } catch (error) {
    console.error("Failed to fetch languages from backend:", error);
    // Fallback to empty array if backend fails
    cachedLanguages = [];
    languageMap = new Map();
    return cachedLanguages;
  }
}

// Initialize languages on module load (non-blocking)
getLanguages().catch(console.error);

// Utility function to get flag emoji for a language code
export function getFlagEmoji(language: string): string {
  return languageMap?.get(language)?.emoji
    ?? projectLanguageDisplay?.get(language)?.emoji
    ?? '🌐';
}

// Utility function to get language name for a language code
export function getLanguageName(language: string): string {
  return languageMap?.get(language)?.name
    ?? projectLanguageDisplay?.get(language)?.name
    ?? language;
}

// Utility function to get full language object
export function getLanguage(language: string): Language {
  const fromMap = languageMap?.get(language);
  if (fromMap) return fromMap;
  const fromProject = projectLanguageDisplay?.get(language);
  if (fromProject) return { code: language, name: fromProject.name, emoji: fromProject.emoji };
  return { code: language, name: language, emoji: '🌐' };
}

// Clear cache (useful for testing or if languages are updated)
export function clearLanguageCache() {
  cachedLanguages = null;
  languageMap = null;
  projectLanguageDisplay = null;
}
