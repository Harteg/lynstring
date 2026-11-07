import { writable, derived } from 'svelte/store';
import { load } from '@tauri-apps/plugin-store';
import type { AndroidString } from '../../../src-tauri/bindings/AndroidString';
import { getTranslationSummary } from '../utils/translation-history';
import { currentProject } from './project';

export interface TranslationSession {
  id: string; // UUID
  projectPath: string; // Path to the project this translation belongs to
  timestamp: number; // Start time
  status: 'in_progress' | 'completed' | 'failed' | 'cancelled';
  sourceStrings: AndroidString[]; // Original strings
  targetLanguages: string[];
  results: Map<string, Map<string, string>>; // language -> key -> translated text
  progress: {
    totalStrings: number;
    completedStrings: number;
    languagesCompleted: number;
    currentLanguage?: string;
    currentString?: string;
  };
  duration?: number; // milliseconds
  error?: string;
  mode: 'single_string_multi_language' | 'batch_multi_language'; // Distinguish UI modes
}

export interface TranslationSessionSummary {
  id: string;
  timestamp: number;
  status: TranslationSession['status'];
  summary: string;
  mode: TranslationSession['mode'];
  sourceStringCount: number;
  targetLanguageCount: number;
  error?: string;
}

// Serializable version for storage (Maps need to be converted to objects)
interface SerializableTranslationSession {
  id: string;
  projectPath: string;
  timestamp: number;
  status: 'in_progress' | 'completed' | 'failed' | 'cancelled';
  sourceStrings: AndroidString[];
  targetLanguages: string[];
  results: Record<string, Record<string, string>>; // language -> key -> translated text
  progress: {
    totalStrings: number;
    completedStrings: number;
    languagesCompleted: number;
    currentLanguage?: string;
    currentString?: string;
  };
  duration?: number;
  error?: string;
  mode: 'single_string_multi_language' | 'batch_multi_language';
}

interface SerializableTranslationSessionSummary extends TranslationSessionSummary {}

// Per-project history structure
interface ProjectTranslationHistory {
  latestSession: SerializableTranslationSession | null;
  recentSessions: SerializableTranslationSession[]; // Keep last 2-3 full sessions instead of summaries
  autoTranslateEnabled: boolean;
}

// File structure: maps projectPath -> ProjectTranslationHistory
interface TranslationHistoryFile {
  projects: Record<string, ProjectTranslationHistory>;
}

// Per-project state
interface ProjectTranslationHistoryState {
  latestSession: TranslationSession | null;
  recentSessions: TranslationSession[]; // Keep last 2-3 full sessions
  autoTranslateEnabled: boolean;
}

// Global state: maps projectPath -> ProjectTranslationHistoryState
interface TranslationHistoryState {
  projects: Map<string, ProjectTranslationHistoryState>;
}

const MAX_RECENT_SESSIONS = 100; // Keep last 100 sessions per project (all history)

const defaultProjectState: ProjectTranslationHistoryState = {
  latestSession: null,
  recentSessions: [],
  autoTranslateEnabled: false
};

const defaultState: TranslationHistoryState = {
  projects: new Map()
};

const { subscribe, set, update } = writable<TranslationHistoryState>(defaultState);

// Store instance for persistence
let store: Awaited<ReturnType<typeof load>> | null = null;

// Convert session to serializable format
function sessionToSerializable(session: TranslationSession): SerializableTranslationSession {
  const results: Record<string, Record<string, string>> = {};
  session.results.forEach((langMap, language) => {
    const langObj: Record<string, string> = {};
    langMap.forEach((text, key) => {
      langObj[key] = text;
    });
    results[language] = langObj;
  });

  return {
    id: session.id,
    projectPath: session.projectPath,
    timestamp: session.timestamp,
    status: session.status,
    sourceStrings: session.sourceStrings,
    targetLanguages: session.targetLanguages,
    results,
    progress: session.progress,
    duration: session.duration,
    error: session.error,
    mode: session.mode
  };
}

// Convert serializable format back to session
function serializableToSession(serializable: SerializableTranslationSession): TranslationSession {
  const results = new Map<string, Map<string, string>>();
  for (const [language, langObj] of Object.entries(serializable.results)) {
    const langMap = new Map<string, string>();
    for (const [key, text] of Object.entries(langObj)) {
      langMap.set(key, text);
    }
    results.set(language, langMap);
  }

  return {
    id: serializable.id,
    projectPath: serializable.projectPath,
    timestamp: serializable.timestamp,
    status: serializable.status,
    sourceStrings: serializable.sourceStrings,
    targetLanguages: serializable.targetLanguages,
    results,
    progress: serializable.progress,
    duration: serializable.duration,
    error: serializable.error,
    mode: serializable.mode
  };
}

// Helper to get or create project state
function getProjectState(state: TranslationHistoryState, projectPath: string): ProjectTranslationHistoryState {
  if (!state.projects.has(projectPath)) {
    state.projects.set(projectPath, { ...defaultProjectState });
  }
  return state.projects.get(projectPath)!;
}

// Initialize the translation history store
export async function initializeTranslationHistory() {
  try {
    // Create or load the store
    store = await load('translation-history.json', {
      defaults: { projects: {} },
      autoSave: true
    });

    const savedState = await store.get<TranslationHistoryFile | null>('state');

    if (savedState?.projects && typeof savedState.projects === 'object') {
      const projects = new Map<string, ProjectTranslationHistoryState>();

      // Load per-project history
      for (const [projectPath, projectHistory] of Object.entries(savedState.projects)) {
        const latestSession = projectHistory.latestSession
          ? serializableToSession(projectHistory.latestSession)
          : null;
        
        // Normalize in-progress sessions to cancelled
        const normalizedLatest =
          latestSession && latestSession.status === 'in_progress'
            ? { ...latestSession, status: 'cancelled' as const, error: latestSession.error ?? 'Translation interrupted because the app was closed.' }
            : latestSession;

        const recentSessions = (projectHistory.recentSessions || [])
          .map(serializableToSession)
          .slice(0, MAX_RECENT_SESSIONS);

        projects.set(projectPath, {
          latestSession: normalizedLatest,
          recentSessions,
          autoTranslateEnabled: projectHistory.autoTranslateEnabled || false
        });

        // Save normalized session if it changed
        if (normalizedLatest !== latestSession) {
          await saveProjectHistory(projectPath, {
            latestSession: normalizedLatest,
            recentSessions,
            autoTranslateEnabled: projectHistory.autoTranslateEnabled || false
          });
        }
      }

      set({ projects });
    } else {
      // Start fresh with new format
      set(defaultState);
      await store.set('state', { projects: {} });
    }
  } catch (error) {
    console.error('Failed to initialize translation history store:', error);
    set(defaultState);
  }
}

// Save history for a specific project
async function saveProjectHistory(projectPath: string, projectState: ProjectTranslationHistoryState) {
  if (!store) {
    console.warn('Translation history store not initialized');
    return;
  }

  try {
    const savedState = await store.get<TranslationHistoryFile | null>('state') || { projects: {} };
    
    savedState.projects = savedState.projects || {};
    savedState.projects[projectPath] = {
      latestSession: projectState.latestSession ? sessionToSerializable(projectState.latestSession) : null,
      recentSessions: projectState.recentSessions
        .slice(0, MAX_RECENT_SESSIONS)
        .map(sessionToSerializable),
      autoTranslateEnabled: projectState.autoTranslateEnabled
    };

    await store.set('state', savedState);
  } catch (error) {
    console.error('Failed to save project translation history:', error);
  }
}

export function addSession(session: TranslationSession) {
  update(state => {
    const projectState = getProjectState(state, session.projectPath);
    
    // Move current latestSession to recentSessions if it exists
    if (projectState.latestSession) {
      const updatedRecent = [
        projectState.latestSession,
        ...projectState.recentSessions
      ]
        .sort((a, b) => b.timestamp - a.timestamp)
        .slice(0, MAX_RECENT_SESSIONS);
      projectState.recentSessions = updatedRecent;
    }

    // Set new session as latest
    projectState.latestSession = session;

    saveProjectHistory(session.projectPath, projectState);

    return state;
  });
}

// Update an existing translation session
export function updateSession(sessionId: string, updates: Partial<TranslationSession>) {
  update(state => {
    // Find the project that contains this session
    for (const [projectPath, projectState] of state.projects.entries()) {
      if (projectState.latestSession?.id === sessionId) {
        const updatedSession = { ...projectState.latestSession, ...updates };
        projectState.latestSession = updatedSession;

        saveProjectHistory(projectPath, projectState);
        return state;
      }

      // Also check recentSessions
      const recentIndex = projectState.recentSessions.findIndex(s => s.id === sessionId);
      if (recentIndex !== -1) {
        projectState.recentSessions[recentIndex] = {
          ...projectState.recentSessions[recentIndex],
          ...updates
        };
        saveProjectHistory(projectPath, projectState);
        return state;
      }
    }

    return state;
  });
}

export function removeSession(sessionId: string) {
  update(state => {
    // Find the project that contains this session
    for (const [projectPath, projectState] of state.projects.entries()) {
      if (projectState.latestSession?.id === sessionId) {
        // Restore from recentSessions if available
        const restoredSession = projectState.recentSessions.length > 0
          ? projectState.recentSessions[0]
          : null;
        
        projectState.latestSession = restoredSession;
        if (restoredSession) {
          projectState.recentSessions = projectState.recentSessions.slice(1);
        }

        saveProjectHistory(projectPath, projectState);
        return state;
      }

      // Remove from recentSessions if found there
      const filteredRecent = projectState.recentSessions.filter(s => s.id !== sessionId);
      if (filteredRecent.length !== projectState.recentSessions.length) {
        projectState.recentSessions = filteredRecent;
        saveProjectHistory(projectPath, projectState);
        return state;
      }
    }

    return state;
  });
}

// Old non-reactive functions removed - use reactive stores instead:
// - currentLatestSession (instead of getLatestSession)
// - currentRecentSessions (instead of getRecentSessions)
// - currentAllSessions (instead of getAllSessions)

// Clear history for a specific project
export function clearProjectHistory(projectPath: string) {
  update(state => {
    state.projects.delete(projectPath);
    const storeRef = store; // Capture for closure
    if (storeRef) {
      storeRef.get<TranslationHistoryFile | null>('state').then(savedState => {
        const stateToSave = savedState || { projects: {} };
        delete stateToSave.projects?.[projectPath];
        if (storeRef) {
          storeRef.set('state', stateToSave).catch(err => {
            console.error('Failed to clear project translation history:', err);
          });
        }
      }).catch(err => {
        console.error('Failed to get state for clearing project history:', err);
      });
    }
    return state;
  });
}

// Clear all history
export function clearHistory() {
  set(defaultState);
  if (store) {
    store.set('state', { projects: {} }).catch(err => {
      console.error('Failed to clear translation history:', err);
    });
  }
}

// Get a specific session by ID (searches all projects)
export function getSession(sessionId: string): TranslationSession | null {
  let found: TranslationSession | null = null;
  subscribe(state => {
    for (const projectState of state.projects.values()) {
      if (projectState.latestSession?.id === sessionId) {
        found = projectState.latestSession;
        return;
      }
      const recent = projectState.recentSessions.find(s => s.id === sessionId);
      if (recent) {
        found = recent;
        return;
      }
    }
  })();
  return found;
}

// Get auto-translate enabled state for a project
export function getAutoTranslateEnabled(projectPath: string): boolean {
  let enabled = false;
  subscribe(state => {
    const projectState = state.projects.get(projectPath);
    enabled = projectState?.autoTranslateEnabled || false;
  })();
  return enabled;
}

// Set auto-translate enabled state for a project
export function setAutoTranslateEnabled(projectPath: string, enabled: boolean) {
  update(state => {
    const projectState = getProjectState(state, projectPath);
    projectState.autoTranslateEnabled = enabled;
    saveProjectHistory(projectPath, projectState);
    return state;
  });
}

// Removed sessionToSummary - no longer needed

// Export the store for direct access in components
export const translationHistoryStore = { subscribe };

// ===== REACTIVE STORES =====
// These derived stores automatically update when the history or current project changes

// Derived store for current project's history (FULLY REACTIVE)
export const currentProjectHistory = derived(
  [currentProject, { subscribe }],
  ([$currentProject, historyState]) => {
    if (!$currentProject) {
      return defaultProjectState;
    }
    return historyState.projects.get($currentProject.path) || defaultProjectState;
  }
);

// Derived store for latest session of current project
export const currentLatestSession = derived(
  currentProjectHistory,
  ($history) => {
    const current = $history.latestSession;
    // If current is finalized, show it
    if (current && current.status !== 'in_progress') {
      return current;
    }
    // If current is in_progress, show most recent finalized from recentSessions
    if (current && current.status === 'in_progress') {
      const finalized = $history.recentSessions.find(s => s.status !== 'in_progress');
      return finalized || null;
    }
    return null;
  }
);

// Derived store for all sessions of current project (latest + recent, sorted)
// Includes completed and cancelled sessions (with progress)
export const currentAllSessions = derived(
  currentProjectHistory,
  ($history) => {
    const all = [
      ...($history.latestSession && ($history.latestSession.status === 'completed' || 
          ($history.latestSession.status === 'cancelled' && $history.latestSession.progress.completedStrings > 0)) 
        ? [$history.latestSession] : []),
      ...$history.recentSessions.filter(s => s.status === 'completed' || 
        (s.status === 'cancelled' && s.progress.completedStrings > 0))
    ].sort((a, b) => b.timestamp - a.timestamp);
    return all;
  }
);

// Derived store for recent sessions (excluding latest)
// Returns last 3 translations (completed or cancelled with progress) - used by footer preview
export const currentRecentSessions = derived(
  currentAllSessions,
  ($all) => $all.slice(0, 3)
);

// All activity for current project - every session we have stored (excluding in_progress)
// Used for the activity history dialog
export const currentAllActivitySessions = derived(
  currentProjectHistory,
  ($history) => {
    const sessions: TranslationSession[] = [];
    if ($history.latestSession && $history.latestSession.status !== 'in_progress') {
      sessions.push($history.latestSession);
    }
    sessions.push(...$history.recentSessions.filter((s) => s.status !== 'in_progress'));
    return sessions.sort((a, b) => b.timestamp - a.timestamp);
  }
);
