/**
 * Development Configuration
 * 
 * Centralized development flags for the entire application.
 * 
 * DEV_MODE is automatically set based on build environment:
 * - true by default (local development)
 * - false when VITE_PRODUCTION_BUILD=true (GitHub Actions release builds)
 */

// 🚨 DEVELOPMENT FLAG - Automatically set based on build environment 🚨
// Set VITE_PRODUCTION_BUILD=true in GitHub Actions to disable dev mode
export const DEV_MODE = import.meta.env.VITE_PRODUCTION_BUILD !== 'true';

// Development configuration
export const DEV_CONFIG = {
  // UI settings
  SHOW_DEV_TOOLBAR: true,
  ENABLE_CONSOLE_LOGS: true,
} as const;

// Helper functions
export function isDevMode(): boolean {
  return DEV_MODE;
}

export function logDev(message: string, ...args: any[]): void {
  if (DEV_MODE && DEV_CONFIG.ENABLE_CONSOLE_LOGS) {
    console.log(`🔧 DEV: ${message}`, ...args);
  }
}

export function warnDev(message: string, ...args: any[]): void {
  if (DEV_MODE && DEV_CONFIG.ENABLE_CONSOLE_LOGS) {
    console.warn(`🚨 DEV WARNING: ${message}`, ...args);
  }
}

export function shouldShowDevToolbar(): boolean {
  return DEV_MODE && DEV_CONFIG.SHOW_DEV_TOOLBAR;
}
