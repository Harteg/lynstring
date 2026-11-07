import { writable } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';
import type { AppSettings } from '../../../src-tauri/bindings/AppSettings';
import type { ApiKeyTestResult } from '../../../src-tauri/bindings/ApiKeyTestResult';

/** Models offered in Settings. Prices are per million input / output tokens. */
export const MODELS = [
  { id: 'claude-sonnet-5-5', name: 'Claude Sonnet 5.5', note: 'Recommended · $2 / $10' },
  { id: 'claude-haiku-5-5', name: 'Claude Haiku 5.5', note: 'Cheapest · $0.10 / $0.50' },
  { id: 'claude-opus-5-5', name: 'Claude Opus 5.5', note: 'Most careful · $4 / $20' },
] as const;

export const appSettingsStore = writable<AppSettings | null>(null);

export async function loadAppSettings(): Promise<AppSettings> {
  const settings = await invoke<AppSettings>('get_app_settings');
  appSettingsStore.set(settings);
  return settings;
}

export function testApiKey(apiKey: string | null, model: string): Promise<ApiKeyTestResult> {
  return invoke<ApiKeyTestResult>('test_api_key', { apiKey, model });
}

/** Save the key (when given) and model, then refresh the store. */
export async function saveAppSettings(apiKey: string | null, model: string): Promise<void> {
  if (apiKey !== null) {
    await invoke('set_api_key', { apiKey });
  }
  await invoke('set_model', { model });
  await loadAppSettings();
}

export async function removeApiKey(): Promise<void> {
  await invoke('set_api_key', { apiKey: '' });
  await loadAppSettings();
}
