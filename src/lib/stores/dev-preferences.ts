import { writable, get } from 'svelte/store';
import { load } from '@tauri-apps/plugin-store';
import { isDevMode } from '$lib/config/dev';

export interface DevPreferences {
  simulateTranslations: boolean;
  simulateWithDelay: boolean;
}

// Default dev preferences
const defaultDevPreferences: DevPreferences = {
  simulateTranslations: false,
  simulateWithDelay: true, // Default to true to maintain current behavior
};

// Create the dev preferences store
export const devPreferencesStore = writable<DevPreferences>(defaultDevPreferences);

// Store instance for persistence
let store: Awaited<ReturnType<typeof load>> | null = null;

// Initialize the dev preferences store
export async function initializeDevPreferencesStore() {
  // Only initialize in dev mode
  if (!isDevMode()) {
    return;
  }

  try {
    // Create or load the store
    store = await load('dev-preferences.json', { 
      defaults: { preferences: defaultDevPreferences },
      autoSave: true 
    });
    
    // Try to load existing preferences
    const savedPreferences = await store.get<DevPreferences>('preferences');
    
    let preferences: DevPreferences;
    if (savedPreferences) {
      // Merge with defaults to handle new fields
      preferences = { ...defaultDevPreferences, ...savedPreferences };
      devPreferencesStore.set(preferences);
      console.log('Loaded dev preferences:', preferences);
    } else {
      // Initialize with defaults
      preferences = defaultDevPreferences;
      await store.set('preferences', preferences);
      console.log('Initialized dev preferences with defaults:', preferences);
    }
  } catch (error) {
    console.error('Failed to initialize dev preferences store:', error);
    // Use defaults if loading fails
    devPreferencesStore.set(defaultDevPreferences);
  }
}

// Save dev preferences to disk
export async function saveDevPreferences(preferences: DevPreferences) {
  if (!isDevMode()) {
    return;
  }

  try {
    if (!store) {
      console.error('Dev preferences store not initialized');
      return;
    }
    
    await store.set('preferences', preferences);
    console.log('Saved dev preferences:', preferences);
  } catch (error) {
    console.error('Failed to save dev preferences:', error);
  }
}

// Update dev preferences and save
export async function updateDevPreferences(updater: (current: DevPreferences) => DevPreferences) {
  if (!isDevMode()) {
    return;
  }

  devPreferencesStore.update((current) => {
    const updated = updater(current);
    saveDevPreferences(updated).catch(console.error);
    return updated;
  });
}

