import { writable } from 'svelte/store';
import type { Project } from '../../../src-tauri/bindings/Project';
import { panelStateStore } from './panel-state';

// Project state
export const currentProject = writable<Project | null>(null);
export const editingProject = writable<Project | null>(null);
export const projectsList = writable<Project[]>([]);

// Loading states
export const isLoadingProject = writable(true);

// Subscribe to project changes
currentProject.subscribe((project) => {
    // open status panel when project changes
    panelStateStore.setStatus();
});
