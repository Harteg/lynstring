import { writable, get } from 'svelte/store';
import { load } from '@tauri-apps/plugin-store';

const STORE_KEY = 'unused-strings-suggestion-seen.json';
const STATE_KEY = 'seenProjectPaths';

let store: Awaited<ReturnType<typeof load>> | null = null;

const { subscribe, set, update } = writable<Set<string>>(new Set());

export const unusedStringsSuggestionSeenStore = { subscribe };

export async function initializeUnusedStringsSuggestionStore() {
	try {
		store = await load(STORE_KEY, {
			defaults: { [STATE_KEY]: [] },
			autoSave: true
		});

		const paths = await store.get<string[]>(STATE_KEY);
		if (paths?.length) {
			set(new Set(paths));
		}
	} catch (error) {
		console.error('Failed to load unused strings suggestion store:', error);
	}
}

export function hasSeenSuggestionForProject(projectPath: string): boolean {
	return get(unusedStringsSuggestionSeenStore).has(projectPath);
}

export async function markSuggestionSeenForProject(projectPath: string): Promise<void> {
	update((s) => {
		const next = new Set(s);
		next.add(projectPath);
		return next;
	});

	if (!store) return;

	try {
		const paths = Array.from(get(unusedStringsSuggestionSeenStore));
		await store.set(STATE_KEY, paths);
	} catch (error) {
		console.error('Failed to save unused strings suggestion seen:', error);
	}
}
