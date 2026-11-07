import type { AndroidPlural } from "../../../src-tauri/bindings/AndroidPlural";
import type { AndroidString } from "../../../src-tauri/bindings/AndroidString";

export interface LanguageDisplayInfo {
  name: string;
  emoji: string;
}

export interface AllStringsData {
  languages: string[];
  strings_by_language: Record<string, AndroidString[]>;
  /** Plurals by language (flattened as plural items in translation flow) */
  plurals_by_language?: Record<string, AndroidPlural[]>;
  /** Display names and emojis for project languages (CLDR/region-derived for composed locales) */
  language_display?: Record<string, LanguageDisplayInfo>;
}

export interface StringStatusRow {
  name: string;
  defaultValue: string;
  translations: Record<string, string>;
  translationStatus: Record<string, "translated" | "missing">;
  isTranslatable: boolean;
  isSelected?: boolean;
  /** Whether the original string used CDATA (needed for translation output format) */
  hasCdata?: boolean;
  [key: string]: string | boolean | Record<string, string> | Record<string, "translated" | "missing"> | undefined;
}

// New types for translation selection
export interface TranslationSelection {
  stringKey: string;
  defaultValue: string;
  targetLanguages: string[];
  isTranslatable: boolean;
}

export interface TranslationFilter {
  showMissing: boolean;
  showTranslated: boolean;
  showNonTranslatable: boolean;
  selectedLanguages: string[];
  searchTerm: string;
}

export interface TranslationBatch {
  selections: TranslationSelection[];
  targetLanguages: string[];
  estimatedCost?: number;
  estimatedTokens?: number;
}
