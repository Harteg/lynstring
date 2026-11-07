import { writable, derived, get } from 'svelte/store';
import type { TranslationSelection, TranslationFilter, TranslationBatch, StringStatusRow } from '../types/strings';
import { currentProject } from './project';
import { invoke } from "@tauri-apps/api/core";


// Translation selection store
function createTranslationSelectionStore() {
  // Initialize with empty selection, will be updated when project loads
  const initialLanguages = new Set<string>([]);
  console.log('TranslationSelectionStore initialized with empty languages, will load from project');

  const { subscribe, set, update } = writable<{
    selectedLanguages: Set<string>; // Set of languages selected for translation
    filter: TranslationFilter;
    isLoading: boolean;
    error: string | null;
  }>({
    selectedLanguages: initialLanguages,
    filter: {
      showMissing: true,
      showTranslated: false,
      showNonTranslatable: false,
      selectedLanguages: [],
      searchTerm: ''
    },
    isLoading: false,
    error: null
  });

  let currentProjectData: { languages: string[]; strings: StringStatusRow[] } | null = null;

  // Subscribe to current project changes to load project's selected languages
  currentProject.subscribe(async (project) => {
    if (project) {
      console.log('Project changed, loading selected languages for:', project.path);
      
      // Load project's selected languages (fallback to preferences if not set)
      const languagesFromProject = project.selected_languages && project.selected_languages.length > 0 
        ? new Set(project.selected_languages)
        : new Set<string>([]);
      
      update(state => ({
        ...state,
        selectedLanguages: languagesFromProject
      }));
      
      console.log('Loaded project language selection:', Array.from(languagesFromProject));
    } else {
      // No project selected, clear language selection
      update(state => ({
        ...state,
        selectedLanguages: new Set()
      }));
      console.log('No project selected, cleared language selection');
    }
  });

  return {
    subscribe,
    
    // Update current project data (called when project data changes)
    updateProjectData(projectData: { languages: string[]; strings: StringStatusRow[] }) {
      currentProjectData = projectData;
      
      const project = get(currentProject);
      const availableLanguages = new Set(projectData.languages);
      
      // Auto-select all languages if none are selected for this project
      if (project && (!project.selected_languages || project.selected_languages.length === 0)) {
        console.log('Auto-selecting all languages for project:', project.path);
        invoke("update_project_selected_languages", {
          projectId: project.id,
          selectedLanguages: projectData.languages
        });
      } else {
        // Clean up selected languages that no longer exist in the project
        update(state => {
          const validSelectedLanguages = new Set<string>();
          let hasInvalidLanguages = false;
          
          for (const lang of state.selectedLanguages) {
            if (availableLanguages.has(lang)) {
              validSelectedLanguages.add(lang);
            } else {
              hasInvalidLanguages = true;
              console.log('Removing non-existent language from selection:', lang);
            }
          }
          
          // Only update if we removed some languages
          if (hasInvalidLanguages) {
            // Save cleaned up selection to project
            if (project) {
              invoke("update_project_selected_languages", {
                projectId: project.id,
                selectedLanguages: Array.from(validSelectedLanguages)
              });
            }
            return { ...state, selectedLanguages: validSelectedLanguages };
          }
          
          return state;
        });
      }
      
      console.log('Project data updated');
    },
    

    // Language selection methods
    toggleLanguageSelection(language: string) {
      update(state => {
        const newSelectedLanguages = new Set(state.selectedLanguages);
        if (newSelectedLanguages.has(language)) {
          newSelectedLanguages.delete(language);
        } else {
          newSelectedLanguages.add(language);
        }
        
        // Save to project
        const project = get(currentProject);
        if (project) {
          invoke("update_project_selected_languages", {
            projectId: project.id,
            selectedLanguages: Array.from(newSelectedLanguages)
          });
        }
        
        return { ...state, selectedLanguages: newSelectedLanguages };
      });
    },

    selectAllLanguages() {
      if (!currentProjectData) return;
      
      update(state => {
        const newSelectedLanguages = new Set<string>(currentProjectData!.languages);
        
        // Save to project
        const project = get(currentProject);
        if (project) {
          invoke("update_project_selected_languages", {
            projectId: project.id,
            selectedLanguages: Array.from(newSelectedLanguages)
          });
        }
        
        return { ...state, selectedLanguages: newSelectedLanguages };
      });
    },

    selectLanguages(languages: string[]) {
      update(state => {
        const newSelectedLanguages = new Set(state.selectedLanguages);
        for (const lang of languages) {
          newSelectedLanguages.add(lang);
        }
        
        // Save to project
        const project = get(currentProject);
        if (project) {
          invoke("update_project_selected_languages", {
            projectId: project.id,
            selectedLanguages: Array.from(newSelectedLanguages)
          });
        }
        
        return { ...state, selectedLanguages: newSelectedLanguages };
      });
    },

    clearLanguageSelection() {
      update(state => {
        const newSelectedLanguages = new Set<string>();
        
        // Save to project
        const project = get(currentProject);
        if (project) {
          invoke("update_project_selected_languages", {
            projectId: project.id,
            selectedLanguages: Array.from(newSelectedLanguages)
          });
        }
        
        return { ...state, selectedLanguages: newSelectedLanguages };
      });
    },

    // Filter methods
    updateFilter(newFilter: Partial<TranslationFilter>) {
      update(state => ({
        ...state,
        filter: { ...state.filter, ...newFilter }
      }));
    },


    // Helper method to check if a row matches the current filter
    matchesFilter(row: StringStatusRow, filter: TranslationFilter): boolean {
      // Search term filter
      if (filter.searchTerm && !row.name.toLowerCase().includes(filter.searchTerm.toLowerCase()) && 
          !row.defaultValue.toLowerCase().includes(filter.searchTerm.toLowerCase())) {
        return false;
      }

      // Language filter
      if (filter.selectedLanguages.length > 0) {
        const hasSelectedLanguage = filter.selectedLanguages.some(lang => {
          const status = row.translationStatus[lang];
          if (filter.showMissing && status === 'missing') return true;
          if (filter.showTranslated && status === 'translated') return true;
          return false;
        });
        if (!hasSelectedLanguage) return false;
      }

      // Status filters
      const hasMissing = Object.values(row.translationStatus).some(status => status === 'missing');
      const hasTranslated = Object.values(row.translationStatus).some(status => status === 'translated');
      
      if (filter.showMissing && !hasMissing) return false;
      if (filter.showTranslated && !hasTranslated) return false;
      if (!filter.showNonTranslatable && !row.isTranslatable) return false;

      return true;
    },

    // Reset everything
    reset() {
      set({
        selectedLanguages: new Set(),
        filter: {
          showMissing: true,
          showTranslated: false,
          showNonTranslatable: false,
          selectedLanguages: [],
          searchTerm: ''
        },
        isLoading: false,
        error: null
      });
      currentProjectData = null;
    }
  };
}

export const translationSelectionStore = createTranslationSelectionStore();

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
