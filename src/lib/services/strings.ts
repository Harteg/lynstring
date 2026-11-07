import { invoke } from "@tauri-apps/api/core";
import type { AllStringsData, StringStatusRow } from "../types/strings";

export class StringsService {
  static async loadAllStrings(projectPath: string, languages: string[]): Promise<AllStringsData> {
    return await invoke<AllStringsData>("load_all_strings", {
      projectPath,
      languages
    });
  }

  static processStringsData(data: AllStringsData): StringStatusRow[] {
    const defaultStrings = data.strings_by_language["default"] || [];
    const rows: StringStatusRow[] = [];

    for (const defaultString of defaultStrings) {
      const translations: Record<string, string> = {};
      const translationStatus: Record<string, "translated" | "missing"> = {};

      // Check each language for this string
      for (const language of data.languages) {
        if (language === "default") continue;

        const languageStrings = data.strings_by_language[language] || [];
        const translatedString = languageStrings.find(
          (s) => s.name === defaultString.name
        );

        if (translatedString) {
          translations[language] = translatedString.text;
          translationStatus[language] = "translated";
        } else {
          translations[language] = "";
          translationStatus[language] = "missing";
        }
      }

      rows.push({
        name: defaultString.name,
        defaultValue: defaultString.text,
        translations,
        translationStatus,
        isTranslatable: defaultString.is_translatable
      });
    }

    return rows;
  }
}
