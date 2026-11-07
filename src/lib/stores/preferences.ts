import { writable, get } from 'svelte/store';
import { load } from '@tauri-apps/plugin-store';

export type ThemeMode = 'dark' | 'light' | 'system';

export interface UserPreferences {
  theme: ThemeMode;
}

// Default preferences
const defaultPreferences: UserPreferences = {
  theme: 'system'
};

// Create the preferences store
export const preferencesStore = writable<UserPreferences>(defaultPreferences);

// Store instance for persistence
let store: Awaited<ReturnType<typeof load>> | null = null;

// Event listener reference for cleanup
let mediaQueryListener: ((event: MediaQueryListEvent) => void) | null = null;
let mediaQuery: MediaQueryList | null = null;

// Initialize the store
export async function initializePreferencesStore() {
  try {
    // Create or load the store
    store = await load('preferences.json', { 
      defaults: { preferences: defaultPreferences },
      autoSave: true 
    });
    
    // Try to load existing preferences
    const savedPreferences = await store.get<UserPreferences>('preferences');
    
    let preferences: UserPreferences;
    if (savedPreferences) {
      // Merge with defaults to handle new fields
      preferences = { ...defaultPreferences, ...savedPreferences };
      preferencesStore.set(preferences);
      console.log('Loaded preferences:', preferences);
    } else {
      // Initialize with defaults
      preferences = defaultPreferences;
      await store.set('preferences', preferences);
      console.log('Initialized with defaults:', preferences);
    }

    // Set up theme management
    setupThemeManagement(preferences);
  } catch (error) {
    console.error('Failed to initialize preferences store:', error);
    // Use defaults if loading fails
    preferencesStore.set(defaultPreferences);
  }
}

// Set up theme management with proper cleanup
function setupThemeManagement(preferences: UserPreferences) {
  // Clean up existing listener if any
  cleanupThemeListener();
  
  const html = document.documentElement;
  mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
  
  function updateTheme() {
    // Read current preferences from store to ensure we always have the latest value
    const currentPreferences = get(preferencesStore);
    const theme = currentPreferences.theme;
    
    const isDark = theme === 'dark' ||
      (theme === 'system' && mediaQuery!.matches);

    if (isDark) {
      html.setAttribute('data-theme', 'dark');
      html.classList.add('dark');
      html.style.colorScheme = 'dark';
    } else {
      html.setAttribute('data-theme', 'light');
      html.classList.remove('dark');
      html.style.colorScheme = 'light';
    }
  }

  // Set initial theme - use requestAnimationFrame to ensure DOM is ready
  // Also call immediately in case DOM is already ready
  updateTheme();
  requestAnimationFrame(() => {
    updateTheme();
  });

  // Listen for system theme changes only if theme is set to 'system'
  if (preferences.theme === 'system') {
    mediaQueryListener = updateTheme;
    mediaQuery.addEventListener('change', mediaQueryListener);
  }
}

// Clean up the theme listener
function cleanupThemeListener() {
  if (mediaQuery && mediaQueryListener) {
    mediaQuery.removeEventListener('change', mediaQueryListener);
    mediaQueryListener = null;
  }
}

// Save preferences to disk
export async function savePreferences(preferences: UserPreferences) {
  try {
    if (!store) {
      console.error('Store not initialized');
      return;
    }
    
    await store.set('preferences', preferences);
    console.log('Saved preferences:', preferences);
  } catch (error) {
    console.error('Failed to save preferences:', error);
  }
}

// Update preferences and save
export async function updatePreferences(updater: (current: UserPreferences) => UserPreferences) {
  preferencesStore.update((current) => {
    const updated = updater(current);
    savePreferences(updated).catch(console.error);
    
    // Update theme management with new preferences
    setupThemeManagement(updated);
    
    return updated;
  });
}

// Cleanup function to be called when the app is being destroyed
export function cleanupPreferencesStore() {
  cleanupThemeListener();
  mediaQuery = null;
}