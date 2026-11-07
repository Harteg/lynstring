import { writable, derived } from 'svelte/store';
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { AllStringsData, StringStatusRow, TranslationSelection } from '../types/strings';
import { setProjectLanguageDisplay } from '$lib/services/languages';
import { currentProject } from './project';
import { translationSelectionStore } from './translation-selection';
import { selectedStringsStore } from './selected-strings';

// Store for raw AllStringsData (needed for duplicate detection)
const rawStringsDataStore = writable<AllStringsData | null>(null);

// Combined project data store
function createProjectDataStore() {
  const { subscribe, set, update } = writable<{
    languages: string[];
    strings: StringStatusRow[];
    isLoading: boolean;
    error: string | null;
    errorType: 'strings_not_found' | 'general' | null;
    currentProjectPath: string | null;
  }>({
    languages: [],
    strings: [],
    isLoading: false,
    error: null,
    errorType: null,
    currentProjectPath: null,
  });

  // Store the current project path for file watching
  let currentProjectPath: string | null = null;
  let fileChangeUnlisten: (() => void) | null = null;
  let changesUnlisten: (() => void) | null = null;
  let fileChangeDebounceTimer: ReturnType<typeof setTimeout> | null = null;

  // Define loadProjectData function first so it can be referenced
  async function loadProjectData(projectPath: string) {
      try {
        update(state => ({ ...state, isLoading: true, error: null, errorType: null }));
        
        const data: AllStringsData = await invoke("load_project_data", { projectPath });
        
        // Store raw data for duplicate detection
        rawStringsDataStore.set(data);
        
        // Update language display cache for project languages (flags/names for composed locales like af-rZA)
        setProjectLanguageDisplay(data.language_display ?? null);
        
        // Process the data into our string format
        const strings = processStringsData(data);
        
        set({
          languages: data.languages.filter(lang => lang !== "default"),
          strings,
          isLoading: false,
          error: null,
          errorType: null,
          currentProjectPath: projectPath,
        });
      
      // Setup file watcher for this project
      await setupFileWatcher(projectPath);
        
        // Update translation selection store with new data
        translationSelectionStore.updateProjectData({
          languages: data.languages.filter(lang => lang !== "default"),
          strings
        });
        
        // Track project_opened event (fires only once per session)
        const projectStringCount = strings.length;
        const missingStringsCount = strings.filter(row => 
          Object.values(row.translationStatus).some(status => status === 'missing')
        ).length;
        const existingLanguagesCount = data.languages.filter(lang => lang !== "default").length;
        
      } catch (err) {
        console.error("Failed to load project data:", err);
        
        // Determine error type based on error message
        const errorMessage = String(err);
        const errorType = errorMessage.includes('strings.xml not found') || 
                         errorMessage.includes('Failed to load default strings') ||
                         errorMessage.includes('Failed to read directory')
          ? 'strings_not_found' as const
          : 'general' as const;
        
        // Clear raw data on error
        rawStringsDataStore.set(null);
        setProjectLanguageDisplay(null);
        
        set({
          languages: [],
          strings: [],
          isLoading: false,
          error: errorMessage,
          errorType,
          currentProjectPath: null,
        });
      }
  }

  // Setup file change listener
  async function setupFileWatcher(projectPath: string) {
    // Clean up existing listeners
    if (fileChangeUnlisten) {
      fileChangeUnlisten();
      fileChangeUnlisten = null;
    }
    
    if (changesUnlisten) {
      changesUnlisten();
      changesUnlisten = null;
    }

    // Stop watching previous project
    if (currentProjectPath) {
      try {
        await invoke("stop_watching_project");
      } catch (err) {
        console.error("Failed to stop watching previous project:", err);
      }
    }

    // Start watching new project
    try {
      await invoke("start_watching_project", { projectPath });
      currentProjectPath = projectPath;
      
      // Listen for file change events - update status immediately for responsiveness
      // Only debounce when multiple files change rapidly (e.g., during translation)
      const unlisten = await listen<string[]>("strings-file-changed", (event) => {
        // Clear existing debounce timer
        if (fileChangeDebounceTimer) {
          clearTimeout(fileChangeDebounceTimer);
        }
        // Very short debounce (50ms) to batch rapid file changes while keeping status updates fast
        fileChangeDebounceTimer = setTimeout(() => {
          if (currentProjectPath) {
            loadProjectData(currentProjectPath);
          }
          fileChangeDebounceTimer = null;
        }, 50); // 50ms debounce - fast enough to feel immediate, but batches rapid changes
      });

      // Listen for default strings changes (new, modified, deleted)
      changesUnlisten = await listen("default-strings-changed", (event: any) => {
        // Import and add to changes store
        import('$lib/stores/string-changes').then(({ stringChangesStore }) => {
          if (currentProjectPath) {
            stringChangesStore.addChanges(event.payload, currentProjectPath);
          }
        });
      });
      
      // Store file change unlisten function
      fileChangeUnlisten = unlisten;
      
      // Check for changes that happened while the app was closed
      // This must be called AFTER listeners are set up so events are received
      try {
        await invoke("check_default_strings_changes_on_startup", { projectPath });
      } catch (err) {
        console.error("Failed to check for changes on startup:", err);
      }
    } catch (err) {
      console.error("Failed to start file watcher:", err);
    }
  }

  // Cleanup file watcher
  async function cleanupFileWatcher() {
    if (fileChangeDebounceTimer) {
      clearTimeout(fileChangeDebounceTimer);
      fileChangeDebounceTimer = null;
    }
    
    if (fileChangeUnlisten) {
      fileChangeUnlisten();
      fileChangeUnlisten = null;
    }
    
    if (changesUnlisten) {
      changesUnlisten();
      changesUnlisten = null;
    }
    
    if (currentProjectPath) {
      try {
        await invoke("stop_watching_project");
      } catch (err) {
        console.error("Failed to stop file watcher:", err);
      }
      currentProjectPath = null;
    }
  }

  return {
    subscribe,
    loadProjectData,
    async reset() {
      await cleanupFileWatcher();
      rawStringsDataStore.set(null);
      setProjectLanguageDisplay(null);
      set({ 
        languages: [], 
        strings: [], 
        isLoading: false, 
        error: null, 
        errorType: null,
        currentProjectPath: null,
      });
      translationSelectionStore.reset();
    }
  };
}

export const projectDataStore = createProjectDataStore();

// Helper function to process strings data (moved from StringsService)
function processStringsData(data: AllStringsData): StringStatusRow[] {
  const defaultStrings = data.strings_by_language["default"] || [];
  const defaultPlurals = data.plurals_by_language?.["default"] || [];
  const rows: StringStatusRow[] = [];

  // Process regular strings
  for (const defaultString of defaultStrings) {
    if (!defaultString.is_translatable) continue;
    const translations: Record<string, string> = {};
    const translationStatus: Record<string, "translated" | "missing"> = {};
    for (const language of data.languages) {
      if (language === "default") continue;
      const languageStrings = data.strings_by_language[language] || [];
      const translatedString = languageStrings.find((s) => s.name === defaultString.name);
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
      isTranslatable: defaultString.is_translatable,
      isSelected: false,
      hasCdata: defaultString.has_cdata
    });
  }

  // Flatten plurals: each plural item becomes a row with key "pluralName|quantity"
  for (const plural of defaultPlurals) {
    if (!plural.is_translatable) continue;
    for (const item of plural.items) {
      const flatKey = `${plural.name}|${item.quantity}`;
      const translations: Record<string, string> = {};
      const translationStatus: Record<string, "translated" | "missing"> = {};
      for (const language of data.languages) {
        if (language === "default") continue;
        const langPlurals = data.plurals_by_language?.[language] || [];
        const p = langPlurals.find((x) => x.name === plural.name);
        const translatedItem = p?.items.find((i) => i.quantity === item.quantity);
        if (translatedItem) {
          translations[language] = translatedItem.text;
          translationStatus[language] = "translated";
        } else {
          translations[language] = "";
          translationStatus[language] = "missing";
        }
      }
      rows.push({
        name: flatKey,
        defaultValue: item.text,
        translations,
        translationStatus,
        isTranslatable: plural.is_translatable,
        isSelected: false,
        hasCdata: item.has_cdata
      });
    }
  }

  return rows;
}

// Derived store that combines project data with selection state
export const stringsWithSelection = derived(
  [projectDataStore, selectedStringsStore],
  ([$projectData, $selectedStrings]) => {
    return $projectData.strings.map(row => ({
      ...row,
      isSelected: $selectedStrings.has(row.name)
    }));
  }
);

// Derived stores for filtered data
export const filteredStrings = derived(
  [projectDataStore, translationSelectionStore],
  ([$projectData, $translationSelection]) => {
    // Get the store instance to access the matchesFilter method
    const store = translationSelectionStore;
    const filtered = $projectData.strings.filter(row => 
      store.matchesFilter(row, $translationSelection.filter)
    );
    return filtered;
  }
);

export const selectedStringsData = derived(
  [projectDataStore, selectedStringsStore, translationSelectionStore],
  ([$projectData, $selectedStrings, $translationSelection]) => {
    const selectedStrings = $projectData.strings.filter(row => 
      $selectedStrings.has(row.name)
    );

    return selectedStrings.map(row => ({
      stringKey: row.name,
      defaultValue: row.defaultValue,
      targetLanguages: Array.from($translationSelection.selectedLanguages),
      isTranslatable: row.isTranslatable
    }));
  }
);

export const translationBatch = derived(
  [selectedStringsData, translationSelectionStore],
  ([$selectedStringsData, $translationSelection]) => {
    const targetLanguages = Array.from($translationSelection.selectedLanguages);
    
    // Filter selections to only include translatable strings
    const validSelections = $selectedStringsData.filter(selection => 
      selection.isTranslatable && selection.targetLanguages.length > 0
    );

    return {
      selections: validSelections,
      targetLanguages,
      estimatedTokens: calculateEstimatedTokens(validSelections),
      estimatedCost: calculateEstimatedCost(validSelections)
    };
  }
);

// Helper functions for cost estimation
function calculateEstimatedTokens(selections: TranslationSelection[]): number {
  // Rough estimation: 1 token ≈ 4 characters for English
  let totalTokens = 0;
  for (const selection of selections) {
    // Source text tokens
    totalTokens += Math.ceil(selection.defaultValue.length / 4);
    // Target text tokens (assuming similar length)
    totalTokens += Math.ceil(selection.defaultValue.length / 4) * selection.targetLanguages.length;
  }
  return totalTokens;
}

function calculateEstimatedCost(selections: TranslationSelection[]): number {
  // Rough estimation: $0.002 per 1K tokens
  const tokens = calculateEstimatedTokens(selections);
  return (tokens / 1000) * 0.002;
}

// Type for duplicate key information
export interface DuplicateKeyInfo {
  key: string;
  languages: string[];
}

// Function to detect duplicate string keys across all languages
function detectDuplicateKeys(data: AllStringsData | null): DuplicateKeyInfo[] {
  if (!data) return [];
  
  const duplicates: DuplicateKeyInfo[] = [];
  
  // Check each language for duplicate keys
  for (const language of data.languages) {
    const strings = data.strings_by_language[language] || [];
    const keyCounts = new Map<string, number>();
    
    // Count occurrences of each key
    for (const str of strings) {
      const count = keyCounts.get(str.name) || 0;
      keyCounts.set(str.name, count + 1);
    }
    
    // Find keys that appear more than once
    for (const [key, count] of keyCounts.entries()) {
      if (count > 1) {
        // Check if we already have this key in duplicates
        const existing = duplicates.find(d => d.key === key);
        if (existing) {
          // Add this language to the existing entry if not already there
          if (!existing.languages.includes(language)) {
            existing.languages.push(language);
          }
        } else {
          // Create new entry
          duplicates.push({
            key,
            languages: [language]
          });
        }
      }
    }
  }
  
  // Sort by key name for consistent display
  duplicates.sort((a, b) => a.key.localeCompare(b.key));
  
  return duplicates;
}

// Derived store for duplicate keys
export const duplicateKeysStore = derived(
  rawStringsDataStore,
  ($rawData) => detectDuplicateKeys($rawData)
);
