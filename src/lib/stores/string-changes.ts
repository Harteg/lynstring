import { writable, get } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';
import { currentProject } from './project';

export interface ModifiedString {
  name: string;
  old_value: string;
  new_value: string;
  translatable_changed: boolean;
}

export interface StringChanges {
  new: string[];
  modified: ModifiedString[];
  deleted: string[];
}

export interface PendingChange {
  id: string; // Unique ID for this change batch
  timestamp: number;
  changes: StringChanges;
  projectPath: string;
}

// Store for pending string changes (accumulates across saves)
function createStringChangesStore() {
  const { subscribe, set, update } = writable<PendingChange[]>([]);

  return {
    subscribe,
    
    // Add new changes to the queue
    addChanges(changes: StringChanges, projectPath: string) {
      update(pending => {
        // Check if we already have pending changes for this project
        const existingIndex = pending.findIndex(p => p.projectPath === projectPath);
        
        if (existingIndex >= 0) {
          // Merge with existing changes
          const existing = pending[existingIndex];
          
          // Merge logic for modified strings:
          // - Replace all existing modified strings with new ones from event
          // - Strings not in the new event were reverted/auto-undone and are removed
          // - Strings in the new event replace any existing entries with the same name
          const merged: StringChanges = {
            new: [...new Set([...existing.changes.new, ...changes.new])],
            modified: changes.modified, // Use new modified strings directly (replaces all existing)
            deleted: [...new Set([...existing.changes.deleted, ...changes.deleted])]
          };
          
          // If all changes are gone (empty arrays), remove the entry entirely
          if (merged.new.length === 0 && merged.modified.length === 0 && merged.deleted.length === 0) {
            return pending.filter((_, index) => index !== existingIndex);
          }
          
          // Update existing entry
          const updated = [...pending];
          updated[existingIndex] = {
            ...existing,
            changes: merged,
            timestamp: Date.now() // Update timestamp
          };
          return updated;
        } else {
          // Add new entry
          return [
            ...pending,
            {
              id: `${projectPath}-${Date.now()}`,
              timestamp: Date.now(),
              changes,
              projectPath
            }
          ];
        }
      });
    },
    
    // Remove specific change entry
    removeChange(changeId: string) {
      update(pending => pending.filter(p => p.id !== changeId));
    },
    
    // Acknowledge changes and optionally mark strings as ignored
    // If ignoredStringNames is provided, only those strings are removed from the UI
    // If empty array, all changes are acknowledged (for translation)
    async acknowledgeChanges(changeId: string, ignoredStringNames?: string[]) {
      const { get } = await import('svelte/store');
      const pending = get({ subscribe });
      const change = pending.find(p => p.id === changeId);
      
      if (!change) {
        return;
      }
      
      try {
        // Update snapshot in backend with ignored strings list
        // This marks specific strings as ignored so they won't show up again
        await invoke('update_default_strings_snapshot', {
          projectPath: change.projectPath,
          ignoredStrings: ignoredStringNames || []
        });
        
        // Update the store: remove acknowledged strings
        update(currentPending => {
          const changeIndex = currentPending.findIndex(p => p.id === changeId);
          if (changeIndex < 0) return currentPending;
          
          const changeToUpdate = currentPending[changeIndex];
          
          // If ignoredStringNames is provided, remove only those strings
          // If empty/undefined, remove all (acknowledged for translation)
          if (ignoredStringNames?.length) {
            const ignoredSet = new Set(ignoredStringNames);
            const remainingModified = changeToUpdate.changes.modified.filter(
              m => !ignoredSet.has(m.name)
            );
            
            // If no changes left, remove the entire entry
            if (remainingModified.length === 0 && 
                changeToUpdate.changes.new.length === 0 && 
                changeToUpdate.changes.deleted.length === 0) {
              return currentPending.filter(p => p.id !== changeId);
            }
            
            // Update with remaining changes
            const updated = [...currentPending];
            updated[changeIndex] = {
              ...changeToUpdate,
              changes: {
                ...changeToUpdate.changes,
                modified: remainingModified
              }
            };
            return updated;
          } else {
            // Empty array means acknowledge all - remove the entire entry
            return currentPending.filter(p => p.id !== changeId);
          }
        });
      } catch (error) {
        console.error('Failed to acknowledge changes:', error);
        throw error;
      }
    },
    
    // Clear all changes for a project (when project changes)
    clearForProject(projectPath: string) {
      update(pending => pending.filter(p => p.projectPath !== projectPath));
    },
    
    // Get changes for current project
    getCurrentProjectChanges(): PendingChange | null {
      const project = get(currentProject);
      if (!project) return null;
      
      const pending = get({ subscribe });
      return pending.find((p: PendingChange) => p.projectPath === project.path) || null;
    },
    
    reset() {
      set([]);
    }
  };
}

export const stringChangesStore = createStringChangesStore();

