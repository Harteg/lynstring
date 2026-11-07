import { invoke, type InvokeOptions } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { writable, get } from "svelte/store";
import type { TranslationResult } from "../../../src-tauri/bindings/TranslationResult";
import type { TranslationProgress } from "../../../src-tauri/bindings/TranslationProgress";
import type { AndroidString } from "../../../src-tauri/bindings/AndroidString";
import type { Project } from "../../../src-tauri/bindings/Project";
import {
  addSession,
  updateSession,
  getSession,
  removeSession,
  type TranslationSession
} from '$lib/stores/translation-history';

export interface TranslationOptions {
  extraInstructions?: string;
  overwriteMode?: boolean;
  source?: 'update' | 'translate';
}

export interface TranslationState {
  isTranslating: boolean;
  isPreparing: boolean; // True when preparing to start translation (e.g., auto-translate detected new strings)
  progress: TranslationProgress | null;
  results: Map<string, TranslationResult>;
  error: string | null;
  currentSessionId: string | null;
  currentProjectPath: string | null; // Which project the active translation is for
  currentProjectName: string | null; // Display name of the project being translated (avoids wrong label when user switches)
  translationSource: 'update' | 'translate' | null; // Which UI triggered the translation (for spinner targeting)
  pendingSession?: {
    sourceStrings: AndroidString[];
    targetLanguages: string[];
    timestamp: number;
  }; // Temporary storage for session data until completion
}

// Create a Svelte store for translation state
const NETWORK_ERROR_PATTERNS = [
  'networkerror',
  'failed to fetch',
  'econn',
  'enet',
  'socket hang up',
  'connection reset',
  'connection aborted',
  'network connection lost',
  'offline'
];

const initialState: TranslationState = {
  isTranslating: false,
  isPreparing: false,
  progress: null,
  results: new Map(),
  error: null,
  currentSessionId: null,
  currentProjectPath: null,
  currentProjectName: null,
  translationSource: null,
  pendingSession: undefined,
};

const { subscribe, set, update } = writable<TranslationState>(initialState);

class TranslationService {
  private abortController: AbortController | null = null;
  private readonly offlineMessage = 'Translation paused because the internet connection was lost.';

  constructor() {
    this.setupProgressListener();

    if (typeof window !== 'undefined') {
      window.addEventListener('offline', this.handleNetworkOffline);
      window.addEventListener('online', this.handleNetworkOnline);
    }
  }

  private setupProgressListener() {
    // Listen for translation events
    listen<{ session_id: string; source_strings: AndroidString[]; target_languages: string[] }>("translation-start", (event) => {
      console.log('Translation start event:', event.payload);
      const { session_id, source_strings, target_languages } = event.payload;
      
      // Get current project path from state (should be set by translateMultipleLanguages)
      let projectPath = '';
      subscribe(state => {
        projectPath = state.currentProjectPath || '';
      })();
      
      // Clear preparing state and set translating state
      // Preserve currentProjectPath - it's already set by translateMultipleLanguages
      // Initialize progress with total_languages count
      // Store session data temporarily (will be used to create session on success)
      update(state => ({
        ...state,
        isPreparing: false,
        isTranslating: true,
        currentSessionId: session_id,
        error: null,
        progress: {
          current_language: '',
          total_languages: target_languages.length,
          languages_done: 0,
          current_language_strings_done: 0,
          current_language_total_strings: 0,
          current_batch: 0,
          total_batches: 0,
          batch_in_progress: false
        },
        // Explicitly preserve currentProjectPath and currentProjectName (set by translateMultipleLanguages)
        currentProjectPath: state.currentProjectPath,
        currentProjectName: state.currentProjectName,
        // Store session data temporarily for history creation on success
        pendingSession: {
          sourceStrings: source_strings,
          targetLanguages: target_languages,
          timestamp: Date.now()
        }
      }));
      
      // Create session immediately when translation starts (so it exists for cancellation)
      const timestamp = Date.now();
      const mode = source_strings.length === 1 
        ? 'single_string_multi_language' 
        : 'batch_multi_language';
      
      const session: TranslationSession = {
        id: session_id,
        projectPath,
        timestamp,
        status: 'in_progress',
        sourceStrings: source_strings,
        targetLanguages: target_languages,
        results: new Map(),
        progress: {
          totalStrings: 0, // Will be updated by progress events
          completedStrings: 0,
          languagesCompleted: 0
        },
        mode
      };
      
      // Add to history immediately
      addSession(session);
    });

    listen<{ session_id: string; total_strings: number; completed_strings: number; languages_completed: number; current_language?: string; current_batch: number; total_batches: number; batch_in_progress: boolean }>("translation-progress", async (event) => {
      const { session_id, total_strings, completed_strings, languages_completed, current_language, current_batch, total_batches, batch_in_progress } = event.payload;
      console.log('Translation progress event:', { session_id, total_strings, completed_strings, languages_completed, current_language, current_batch, total_batches, batch_in_progress });
      
      // Note: Token limit enforcement is handled server-side in Edge Function
      // This check is for UX/display purposes only, not authoritative
      // The Edge Function will make the final allow/reject decision
      
      // Update history session progress - CRITICAL: This ensures cancelled translations with progress are detected
      updateSession(session_id, {
        progress: {
          totalStrings: total_strings,
          completedStrings: completed_strings,
          languagesCompleted: languages_completed,
          currentLanguage: current_language
        }
      });

      // Also update translation store progress for UI
      // The backend sends cumulative progress (total_strings and completed_strings are across all languages)
      // We show overall progress: completed_strings / total_strings
      // IMPORTANT: Ensure progress never goes backward (use Math.max to prevent regression)
      update(state => {
        // Ensure we preserve currentProjectPath and total_languages
        const preservedTotalLanguages = state.progress?.total_languages || 1;
        
        // Get current progress to prevent going backward
        const currentCompleted = state.progress?.current_language_strings_done || 0;
        const currentTotal = state.progress?.current_language_total_strings || 0;
        
        console.log('Progress update:', {
          currentCompleted,
          currentTotal,
          newCompleted: completed_strings,
          newTotal: total_strings,
          currentProjectPath: state.currentProjectPath,
          isTranslating: state.isTranslating
        });
        
        // Only update if the new values are valid and progress hasn't regressed
        // Use Math.max to ensure completed_strings never decreases
        // Also clamp to total_strings to prevent showing > 100%
        const safeCompletedStrings = Math.min(
          Math.max(currentCompleted, completed_strings),
          total_strings || currentTotal
        );
        const safeTotalStrings = total_strings > 0 ? total_strings : currentTotal;
        
        return {
          ...state,
          progress: {
            current_language: current_language || '',
            total_languages: preservedTotalLanguages,
            languages_done: languages_completed,
            // Show overall progress: completed_strings out of total_strings
            // Use safe values to prevent regression and overflow
            current_language_strings_done: safeCompletedStrings,
            current_language_total_strings: safeTotalStrings,
            // Batch/chunk progress info
            current_batch,
            total_batches,
            batch_in_progress
          } as TranslationProgress,
          isTranslating: true,
        // Explicitly preserve currentProjectPath and currentProjectName
        currentProjectPath: state.currentProjectPath,
        currentProjectName: state.currentProjectName
        };
      });
    });

    listen<{ session_id: string; results: Record<string, TranslationResult>; total_input_tokens: number; total_output_tokens: number }>("translation-complete", async (event) => {
      const { session_id, results, total_input_tokens, total_output_tokens } = event.payload;
      console.log('Translation complete event:', event.payload);
      
      // Token usage is tracked server-side in the Edge Function
      // No local tracking needed - server is authoritative source
      
      // Results are already in TranslationResult format - just convert to Map
      const resultsMap = new Map<string, TranslationResult>();
      const resultsForHistory = new Map<string, Map<string, string>>();
      
      for (const [language, result] of Object.entries(results)) {
        // Use result directly - backend already processed everything
        resultsMap.set(language, result);
        
        // Log results for debugging
        console.log(`\nTranslation results for ${language}:`);
        console.log(`- Successfully translated: ${result.translated.length} strings`);
        if (result.skipped.length > 0) {
          console.log(`- Skipped (too long): ${result.skipped.length} strings`);
        }
        if (result.failed.length > 0) {
          console.log(`- Failed translations: ${result.failed.length} strings`);
        }
        
        if (result.translated.length > 0) {
          console.log('\nNew translations:');
          result.translated.forEach((str) => {
            console.log(`${str.name}: "${str.text}"`);
          });
        }
        
        if (result.skipped.length > 0) {
          console.log('\nSkipped strings (too long):');
          result.skipped.forEach((str) => {
            console.log(`${str.name}`);
          });
        }
        
        if (result.failed.length > 0) {
          console.log('\nFailed strings:');
          result.failed.forEach((str) => {
            console.log(`${str.name}: "${str.text}"`);
          });
        }
        
        // Store results for history
        const langMap = new Map<string, string>();
        for (const translated of result.translated) {
          langMap.set(translated.name, translated.text);
        }
        resultsForHistory.set(language, langMap);
      }
      
      // Only add to history if translation was successful (has results)
      // Check if we have any successful translations
      const hasSuccessfulTranslations = Array.from(resultsForHistory.values()).some(map => map.size > 0);
      
      if (hasSuccessfulTranslations) {
        // Update existing session (created on translation-start) to completed status
        const existingSession = getSession(session_id);
        if (existingSession) {
          const completedStrings = Array.from(resultsForHistory.values()).reduce((sum, map) => sum + map.size, 0);
          
          updateSession(session_id, {
            status: 'completed',
            results: resultsForHistory,
            progress: {
              ...existingSession.progress,
              completedStrings,
              totalStrings: existingSession.progress.totalStrings || existingSession.sourceStrings.length,
              languagesCompleted: resultsForHistory.size
            },
            duration: Date.now() - existingSession.timestamp
          });
        } else {
          // Fallback: create session if it doesn't exist (shouldn't happen, but just in case)
          const currentState = get(translationStore);
          const pendingSession = currentState.pendingSession;
          const projectPath = currentState.currentProjectPath;
          
          if (pendingSession && projectPath) {
            const mode = pendingSession.sourceStrings.length === 1 
              ? 'single_string_multi_language' 
              : 'batch_multi_language';
            
            const completedStrings = Array.from(resultsForHistory.values()).reduce((sum, map) => sum + map.size, 0);
            
            const session: TranslationSession = {
              id: session_id,
              projectPath,
              timestamp: pendingSession.timestamp,
              status: 'completed',
              sourceStrings: pendingSession.sourceStrings,
              targetLanguages: pendingSession.targetLanguages,
              results: resultsForHistory,
              progress: {
                totalStrings: pendingSession.sourceStrings.length,
                completedStrings,
                languagesCompleted: resultsForHistory.size
              },
              duration: Date.now() - pendingSession.timestamp,
              mode
            };
            
            addSession(session);
          }
        }
      }
      
      // Clear pending session data
      update(state => ({
        ...state,
        pendingSession: undefined
      }));
      
      update(state => ({
        ...state,
        isTranslating: false,
        isPreparing: false,
        progress: null,
        results: resultsMap,
        currentSessionId: null,
        currentProjectPath: null,
        currentProjectName: null,
        translationSource: null,
        pendingSession: undefined,
        // Clear any stale error (e.g. from frontend timeout that fired before backend completed)
        error: null
      }));
      
      // Refresh project data to show updated translations in UI
      // Import projectDataStore dynamically to avoid circular dependency
      import('$lib/stores/project-data').then(({ projectDataStore }) => {
        import('$lib/stores/project').then(({ currentProject }) => {
          import('svelte/store').then(({ get }) => {
            const project = get(currentProject);
            if (project) {
              projectDataStore.loadProjectData(project.path);
            }
          });
        });
      });
    });

    listen<{ session_id: string; error: string; completed_translations?: number | null; total_translations?: number | null }>("translation-error", (event) => {
      const { session_id, error, completed_translations, total_translations } = event.payload;
      console.error('Translation error event:', event.payload);

      const normalizedError = this.normalizeErrorMessage(error);
      const isCancelled = error.includes('cancelled') || error.includes('Cancelled');
      const status: TranslationSession['status'] = isCancelled ? 'cancelled' : 'failed';
      
      // Update history session with error
      // Get the latest session - use getSession but it should have the latest progress from updateSession calls
      const session = getSession(session_id);
      if (session) {
        const duration = Date.now() - session.timestamp;
        
        // Check if there are any results in the session OR in the current translation state
        // (in case some languages completed before cancellation)
        const sessionHasResults = Array.from(session.results.values()).some(langMap => langMap.size > 0);
        
        // Also check current translation state for any completed results
        // (in case some languages completed before cancellation)
        const currentState = get({ subscribe });
        const stateHasResults = currentState.results.size > 0 && 
          Array.from(currentState.results.values()).some(result => result.translated.length > 0);
        
        // Also check progress - if any strings were completed, there are results
        // (even if they're only written to files and not yet in session/state)
        const progressHasResults = session.progress.completedStrings > 0 || 
                                   (currentState.progress?.current_language_strings_done || 0) > 0;
        
        // Use progress info from event if available (more accurate than session/state)
        const completedFromEvent = completed_translations ?? null;
        const totalFromEvent = total_translations ?? null;
        const hadResults = sessionHasResults || stateHasResults || progressHasResults || (completedFromEvent !== null && completedFromEvent > 0);

        console.log('Translation error/cancellation:', {
          isCancelled,
          sessionHasResults,
          stateHasResults,
          progressHasResults,
          hadResults,
          completedFromEvent,
          totalFromEvent,
          sessionResultsCount: Array.from(session.results.values()).reduce((sum, map) => sum + map.size, 0),
          stateResultsCount: Array.from(currentState.results.values()).reduce((sum, result) => sum + result.translated.length, 0),
          sessionProgressCompleted: session.progress.completedStrings,
          stateProgressCompleted: currentState.progress?.current_language_strings_done || 0,
          sessionProgressTotal: session.progress.totalStrings
        });

        if (isCancelled && !hadResults) {
          // Cancelled with no results at all - remove from history and restore previous
          console.log('Translation cancelled with no results - removing from history');
          removeSession(session_id);
        } else {
          // Cancelled but had some results, or failed - update session to keep it in history
          // If we have results in state but not in session, update session with those results
          let resultsToSave = session.results;
          if (!sessionHasResults && stateHasResults) {
            // Convert TranslationResult format to the Map format used in sessions
            const resultsMap = new Map<string, Map<string, string>>();
            for (const [language, result] of currentState.results.entries()) {
              if (result.translated.length > 0) {
                const langMap = new Map<string, string>();
                for (const translated of result.translated) {
                  langMap.set(translated.name, translated.text);
                }
                resultsMap.set(language, langMap);
              }
            }
            resultsToSave = resultsMap;
            console.log('Saving results from state to session:', {
              languages: Array.from(resultsMap.keys()),
              totalStrings: Array.from(resultsMap.values()).reduce((sum, map) => sum + map.size, 0)
            });
          }
          
          // If we have progress showing completed strings but no actual results data,
          // use progress data to ensure the session is properly marked
          // Prefer event data if available (more accurate), otherwise use session/state data
          const finalCompletedStrings = completedFromEvent !== null && completedFromEvent > 0
            ? completedFromEvent
            : Array.from(resultsToSave.values()).reduce((sum, map) => sum + map.size, 0) || session.progress.completedStrings;
          const finalTotalStrings = totalFromEvent !== null && totalFromEvent > 0
            ? totalFromEvent
            : session.progress.totalStrings;
          
          const finalProgress = {
            ...session.progress,
            completedStrings: finalCompletedStrings,
            totalStrings: finalTotalStrings,
            languagesCompleted: resultsToSave.size || session.progress.languagesCompleted
          };
          
          console.log('Updating session with cancelled status and results:', {
            status,
            resultsCount: Array.from(resultsToSave.values()).reduce((sum, map) => sum + map.size, 0),
            languagesCount: resultsToSave.size,
            progressCompletedStrings: finalProgress.completedStrings,
            progressTotalStrings: finalProgress.totalStrings,
            progressLanguagesCompleted: finalProgress.languagesCompleted
          });
          
          updateSession(session_id, {
            status,
            error: normalizedError,
            duration,
            results: resultsToSave,
            progress: finalProgress
          });
        }
      }
      
      update(state => ({
        ...state,
        isTranslating: false,
        isPreparing: false,
        progress: null,
        error: normalizedError,
        currentSessionId: null,
        translationSource: null
        // Keep currentProjectPath so error is associated with correct project
      }));
    });

    // Listen for tokens from cancelled requests (fire-and-forget background tracking)
    listen<{ input_tokens: number; output_tokens: number }>("translation-cancelled-tokens", async (event) => {
      const { input_tokens, output_tokens } = event.payload;
      console.log('📊 Cancelled request tokens received:', { input_tokens, output_tokens });
      
      // Token usage is tracked server-side in the Edge Function
      // No local tracking needed - server is authoritative source
    });
  }

  private normalizeErrorMessage(error: string): string {
    const lower = error.toLowerCase();
    const isOffline = typeof navigator !== 'undefined' && navigator?.onLine === false;
    
    // Don't treat server errors, token limit errors, or other specific errors as network issues
    const isSpecificError = 
      lower.includes('token limit') ||
      lower.includes('server returned error') ||
      lower.includes('server error') ||
      lower.includes('authentication failed') ||
      lower.includes('rate limit') ||
      lower.includes('invalid request') ||
      lower.includes('edge function');
    
    // Only check for network patterns if it's not a specific error type
    const looksLikeNetwork = !isSpecificError && NETWORK_ERROR_PATTERNS.some(pattern => lower.includes(pattern));

    if (isOffline || looksLikeNetwork) {
      return this.offlineMessage;
    }

    return error;
  }

  private handleNetworkOffline = () => {
    this.abortController?.abort();

    update(state => {
      if (!state.currentSessionId) {
        return state;
      }

      const sessionId = state.currentSessionId;
      const session = getSession(sessionId);
      if (session) {
        const duration = Date.now() - session.timestamp;
        updateSession(sessionId, {
          status: 'failed',
          error: this.offlineMessage,
          duration
        });
      }

      return {
        ...state,
        isTranslating: false,
        isPreparing: false,
        progress: null,
        error: this.offlineMessage,
        currentSessionId: null,
        translationSource: null
      };
    });
  };

  private handleNetworkOnline = () => {
    update(state => {
      if (state.error === this.offlineMessage) {
        return {
          ...state,
          error: null
        };
      }
      return state;
    });
  };

  subscribe(listener: (state: TranslationState) => void) {
    return subscribe(listener);
  }

  async translateStrings(
    strings: AndroidString[],
    targetLanguage: string,
    project: Project,
    options: TranslationOptions = {}
  ): Promise<TranslationResult> {
    // Prevent parallel translations (rate limiting)
    // Note: Only check isTranslating - components set isPreparing before calling us
    const state = get(translationStore);
    if (state.isTranslating) {
      throw new Error('A translation is already in progress. Please wait for it to complete.');
    }
    // Claim the lock immediately to prevent race with concurrent requests
    update(s => ({ ...s, isTranslating: true, progress: null, error: null }));

    // Create new AbortController for this translation session
    this.abortController = new AbortController();


    try {
      const result = await invoke<TranslationResult>("translate_strings", {
        strings,
        targetLanguage,
        project,
        extraInstructions: options.extraInstructions,
      });


      // Add new strings to snapshot (preserves existing modified strings in UI)
      try {
        await invoke('update_default_strings_snapshot', {
          projectPath: project.path,
          mode: 'add_new_only'
        });
      } catch (snapErr) {
        console.warn('Failed to add new strings to snapshot after translation:', snapErr);
      }

      update(state => {
        const newResults = new Map(state.results);
        newResults.set(targetLanguage, result);
        return {
          ...state,
          results: newResults,
          isTranslating: false,
          isPreparing: false,
          progress: null
        };
      });

      return result;
    } catch (error) {
      if (error instanceof DOMException && error.name === 'AbortError') {
        update(state => ({
          ...state,
          isTranslating: false,
          progress: null,
          error: 'Translation cancelled'
        }));
        return {
          translated: [],
          failed: strings,
          skipped: []
        };
      }
      
      const errorMessage = error instanceof Error ? error.message : String(error);
      
      

      update(state => ({
        ...state,
        error: errorMessage,
        isTranslating: false,
        isPreparing: false,
        progress: null,
        translationSource: null
      }));
      throw error;
    } finally {
      this.abortController = null;
    }
  }

  async translateMultipleLanguages(
    stringsByLanguage: Map<string, AndroidString[]>,
    project: Project,
    options: TranslationOptions = {}
  ): Promise<Map<string, TranslationResult>> {
    // Prevent parallel translations (rate limiting)
    // Note: Only check isTranslating - components set isPreparing before calling us
    const state = get(translationStore);
    if (state.isTranslating) {
      throw new Error('A translation is already in progress. Please wait for it to complete.');
    }
    // Claim the lock immediately to prevent race with concurrent requests
    update(s => ({
      ...s,
      isTranslating: true,
      isPreparing: false,
      progress: null,
      error: null,
      currentProjectPath: project.path,
      currentProjectName: project.name,
      translationSource: options.source ?? s.translationSource
    }));

    // Note: Token limit enforcement is handled server-side in Edge Function
    // This check is for UX/display purposes only, not authoritative
    // The Edge Function will make the final allow/reject decision
    // Continue with translation - Edge Function will reject if limit exceeded

    // Create new AbortController for this translation session
    this.abortController = new AbortController();

    const languages = Array.from(stringsByLanguage.keys());
    const totalStrings = Array.from(stringsByLanguage.values()).reduce((sum, strings) => sum + strings.length, 0);
    
    // Get all unique source strings (union of all strings across all languages)
    // This ensures we translate all strings that need translation, even if different languages need different strings
    const allUniqueStrings = new Map<string, AndroidString>();
    for (const strings of stringsByLanguage.values()) {
      for (const str of strings) {
        if (!allUniqueStrings.has(str.name)) {
          allUniqueStrings.set(str.name, str);
        }
      }
    }
    const sourceStrings = Array.from(allUniqueStrings.values());
    
    // Get all unique strings being translated (for word counting)
    const allStrings = new Set<string>();
    const missingStrings: Array<{ text: string }> = [];
    sourceStrings.forEach((str) => {
      if (!allStrings.has(str.name)) {
        allStrings.add(str.name);
        missingStrings.push({ text: str.text });
      }
    });
    
    
    

    console.log('Starting translation for languages:', languages);
    update(s => ({
      ...s,
      results: new Map()
    }));

    try {
      // Set up promise to wait for completion BEFORE calling invoke
      // This ensures we don't miss the completion event if translation is very fast
      const completionPromise = new Promise<Map<string, TranslationResult>>((resolve, reject) => {
        let timeout: ReturnType<typeof setTimeout> | null = null;
        let unsubscribe: (() => void) | null = null;
        
        // Set up subscription
        unsubscribe = subscribe((state) => {
          // Translation completed successfully (even if all strings were skipped)
          // Check that we're not translating and there's no error
          if (!state.isTranslating && !state.isPreparing && !state.error) {
            if (timeout) clearTimeout(timeout);
            if (unsubscribe) {
              unsubscribe();
              unsubscribe = null;
            }
            resolve(state.results);
            return;
          }
          // Translation failed
          if (!state.isTranslating && !state.isPreparing && state.error && state.error !== 'Translation cancelled') {
            if (timeout) clearTimeout(timeout);
            if (unsubscribe) {
              unsubscribe();
              unsubscribe = null;
            }
            reject(new Error(state.error));
            return;
          }
        });
        
        // Dynamic timeout: 5 min base + 2 min per 1000 strings, max 30 min
        // (e.g. 5194 strings → ~15 min; large jobs need more time)
        const timeoutMs = Math.min(
          30 * 60 * 1000,
          5 * 60 * 1000 + Math.ceil(totalStrings / 1000) * 2 * 60 * 1000
        );
        timeout = setTimeout(() => {
          if (unsubscribe) {
            unsubscribe();
            unsubscribe = null;
          }
          reject(new Error('Translation timeout'));
        }, timeoutMs);
      });

      // Get simulate settings from dev preferences
      const { devPreferencesStore } = await import('$lib/stores/dev-preferences');
      const { get } = await import('svelte/store');
      const devPrefs = get(devPreferencesStore);
      const simulate = devPrefs?.simulateTranslations ?? false;
      const simulateWithDelay = devPrefs?.simulateWithDelay ?? true;

      // Call translation command - results will be processed by the global event listener
      // Pass expected_total_strings (sum of strings per language) for accurate progress tracking
      await invoke<Record<string, Record<string, string>>>("translate_strings_multi_language_single_call", {
        sourceStrings,
        targetLanguages: languages,
        project,
        extraInstructions: options.extraInstructions,
        expectedTotalStrings: totalStrings, // Sum of strings per language (e.g., 1+743+743+... = 1506)
        simulate,
        simulateWithDelay,
      });

      // Wait for completion and get results from store
      const resultsMap = await completionPromise;
      
      // Add new strings to snapshot (preserves existing modified strings in UI)
      try {
        await invoke('update_default_strings_snapshot', {
          projectPath: project.path,
          mode: 'add_new_only'
        });
      } catch (snapErr) {
        console.warn('Failed to add new strings to snapshot after translation:', snapErr);
      }

      // Note: isTranslating will be set to false by the completion event listener
      // But we update results here in case the event hasn't fired yet
      update(state => ({
        ...state,
        results: resultsMap
      }));

      return resultsMap;
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : String(error);
      
      if (error instanceof DOMException && error.name === 'AbortError') {
        update(state => ({
          ...state,
          isTranslating: false,
          isPreparing: false,
          progress: null,
          error: 'Translation cancelled',
          translationSource: null
        }));
        return new Map();
      }
      
      // Check if this is a cancellation error from the backend
      if (error instanceof Error && error.message === 'Translation cancelled') {
        update(state => ({
          ...state,
          isTranslating: false,
          isPreparing: false,
          progress: null,
          error: 'Translation cancelled',
          translationSource: null
        }));
        return new Map();
      }
      
      

      update(state => ({
        ...state,
        error: errorMessage,
        isTranslating: false,
        isPreparing: false,
        progress: null,
        translationSource: null
      }));
      throw error;
    } finally {
      this.abortController = null;
    }
  }

  async cancelTranslation() {
    console.log('Cancelling translation...');
    try {
      // Emit cancel event
      const { emit } = await import('@tauri-apps/api/event');
      await emit('cancel-translation');
      
      // Don't update state here - let the main translation flow handle it
      // This ensures isCanceling stays true until the final error message
    } catch (error) {
      console.error('Error cancelling translation:', error);
    }
  }

  getState(): TranslationState {
    let currentState: TranslationState;
    subscribe(state => currentState = state)();
    return currentState!;
  }

  clearResults() {
    update(state => ({
      ...state,
      results: new Map(),
      error: null
    }));
  }

  setPreparing(preparing: boolean, source?: 'update' | 'translate') {
    update(state => ({
      ...state,
      isPreparing: preparing,
      ...(source !== undefined && { translationSource: source })
    }));
  }

  /** Dismiss the current error (clears error only, keeps other state) */
  dismissError() {
    update(state => ({ ...state, error: null }));
  }

  // Clear translation state (useful when switching projects)
  clearState() {
    update(state => ({
      ...state,
      isTranslating: false,
      isPreparing: false,
      progress: null,
      results: new Map(),
      error: null,
      currentSessionId: null,
      currentProjectPath: null,
      currentProjectName: null,
      translationSource: null,
      pendingSession: undefined
    }));
  }
}

export const translationService = new TranslationService();

// Export the store for direct access in components
export const translationStore = { subscribe };
