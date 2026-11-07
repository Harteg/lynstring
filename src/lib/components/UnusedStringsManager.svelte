<script lang="ts">
  import { currentProject } from '$lib/stores/project';
  import { invoke } from '@tauri-apps/api/core';
  import { Button } from '$lib/components/ui/button';
  import { Checkbox } from '$lib/components/ui/checkbox';
  import * as AlertDialog from '$lib/components/ui/alert-dialog';
  import { Loader2 } from '@lucide/svelte';
  import { listen } from '@tauri-apps/api/event';
  import { onDestroy } from 'svelte';
  import { projectDataStore } from '$lib/stores/project-data';
  import { unusedStringsScanRequestStore, consumeScanRequest } from '$lib/stores/unused-strings-scan';
  import { toast } from 'svelte-sonner';

  // Unused strings dialog state
  let showUnusedDialog = $state(false);
  let unusedStrings = $state<string[]>([]);
  let selectedStringsToDelete = $state<Set<string>>(new Set());
  let isDeletingUnused = $state(false);
  let isFindingUnused = $state(false);
  let currentSearchPath = $state<string>('');
  let unusedProgressListener: (() => void) | null = $state(null);
  let errorMessage = $state<string | null>(null);
  
  // Expose loading state for parent component
  export function getIsFinding() {
    return isFindingUnused;
  }
  
  // Function to close unused strings dialog
  function closeUnusedDialog() {
    showUnusedDialog = false;
    unusedStrings = [];
    selectedStringsToDelete = new Set();
    currentSearchPath = '';
    errorMessage = null;
  }
  
  // Function to toggle string selection
  function toggleStringSelection(stringName: string) {
    const newSet = new Set(selectedStringsToDelete);
    if (newSet.has(stringName)) {
      newSet.delete(stringName);
    } else {
      newSet.add(stringName);
    }
    selectedStringsToDelete = newSet;
  }
  
  // Function to select all strings
  function selectAllStrings() {
    selectedStringsToDelete = new Set(unusedStrings);
  }
  
  // Function to deselect all strings
  function deselectAllStrings() {
    selectedStringsToDelete = new Set();
  }
  
  // Computed: count of selected strings
  const selectedCount = $derived(selectedStringsToDelete.size);
  
  // Function to find unused strings
  export async function findUnusedStrings() {
    if (!$currentProject) {
      errorMessage = 'Please select a project first';
      showUnusedDialog = true;
      return;
    }
    
    // Open dialog immediately to show loading state
    showUnusedDialog = true;
    unusedStrings = [];
    selectedStringsToDelete = new Set();
    errorMessage = null;
    
    try {
      isFindingUnused = true;
      currentSearchPath = 'Initializing search...';
      
      // Set up progress listener
      const unlisten = await listen<string>('unused-strings-search-progress', (event) => {
        currentSearchPath = event.payload;
      });
      
      unusedProgressListener = unlisten;
      
      const strings = await invoke<string[]>('find_unused_strings', {
        projectPath: $currentProject.path
      });
      
      // Clean up listener
      if (unusedProgressListener) {
        unusedProgressListener();
        unusedProgressListener = null;
      }
      
      unusedStrings = strings;
      
      
      if (strings.length === 0) {
        currentSearchPath = '';
        errorMessage = 'All string keys are referenced in your project code, and all translation strings have corresponding entries in the default language.';
      } else {
        // Initialize all strings as selected by default
        selectedStringsToDelete = new Set(strings);
      }
    } catch (err) {
      if (unusedProgressListener) {
        unusedProgressListener();
        unusedProgressListener = null;
      }
      currentSearchPath = '';
      errorMessage = `Failed to find unused strings: ${String(err)}`;
    } finally {
      isFindingUnused = false;
      currentSearchPath = '';
    }
  }
  
  // Function to delete unused strings
  async function deleteUnusedStrings() {
    if (selectedStringsToDelete.size === 0) {
      errorMessage = 'Please select at least one string to delete.';
      return;
    }
    
    try {
      isDeletingUnused = true;
      errorMessage = null;
      
      if (!$currentProject) {
        errorMessage = 'Please select a project first';
        isDeletingUnused = false;
        return;
      }
      
      const stringsToDelete = Array.from(selectedStringsToDelete);
      
      await invoke('delete_unused_strings', {
        projectPath: $currentProject.path,
        stringNames: stringsToDelete
      });
      
      const deletedCount = stringsToDelete.length;
      
      
      // Reload project data
      await projectDataStore.loadProjectData($currentProject.path);

      closeUnusedDialog();
      toast.success(`Deleted ${deletedCount} unused string key${deletedCount === 1 ? '' : 's'}`);
    } catch (err) {
      errorMessage = `Failed to delete unused strings: ${String(err)}`;
    } finally {
      isDeletingUnused = false;
    }
  }
  
  // Respond to scan requests from suggestion card or Settings menu.
  // Consume the trigger immediately so project switch etc. won't re-run and reopen the dialog.
  $effect(() => {
    const trigger = $unusedStringsScanRequestStore;
    if (trigger > 0) {
      consumeScanRequest();
      findUnusedStrings();
    }
  });

  // Clean up listener on component destroy
  onDestroy(() => {
    if (unusedProgressListener) {
      unusedProgressListener();
    }
  });
</script>

<!-- Unused Strings Dialog -->
<AlertDialog.Root bind:open={showUnusedDialog}>
  <AlertDialog.Content class="dialog-standard sm:max-w-[600px]">
    <AlertDialog.Header class="dialog-standard-header">
      <AlertDialog.Title class="dialog-standard-title">
        {#if errorMessage && unusedStrings.length === 0 && !isFindingUnused}
          {unusedStrings.length === 0 ? 'No Unused Strings' : 'Error'}
        {:else}
          Unused String Keys
        {/if}
      </AlertDialog.Title>
      <AlertDialog.Description class="dialog-standard-description">
        {#if errorMessage && unusedStrings.length === 0 && !isFindingUnused}
          {errorMessage}
        {:else if isFindingUnused}
          Scanning project files for string references...
        {:else}
          Select strings to delete. These are not referenced in your code (.java, .kt, .xml files).
        {/if}
      </AlertDialog.Description>
    </AlertDialog.Header>
    
    {#if !(errorMessage && unusedStrings.length === 0 && !isFindingUnused)}
      <div class="strings-list-container">
        {#if !isFindingUnused && unusedStrings.length > 0}
          <div class="strings-selection-controls">
            <div class="selection-buttons">
              <Button
                onclick={selectAllStrings}
                variant="ghost"
                size="sm"
                disabled={selectedCount === unusedStrings.length || isDeletingUnused}
              >
                Select All
              </Button>
              <Button
                onclick={deselectAllStrings}
                variant="ghost"
                size="sm"
                disabled={selectedCount === 0 || isDeletingUnused}
              >
                Deselect All
              </Button>
              <span class="selection-count">
                {selectedCount} of {unusedStrings.length} selected
              </span>
            </div>
          </div>
        {/if}
        <div class="strings-list">
          {#if isFindingUnused}
            <div class="search-progress">
              <Loader2 class="spinner" />
              <p class="search-path">{currentSearchPath || 'Starting search...'}</p>
            </div>
          {:else if unusedStrings.length > 0}
            <ul>
              {#each unusedStrings as stringName}
                <li class="string-item">
                  <button
                    class="string-checkbox-label"
                    onclick={(e) => {
                      e.stopPropagation();
                      toggleStringSelection(stringName);
                    }}
                    disabled={isDeletingUnused}
                    type="button"
                  >
                    <Checkbox
                      checked={selectedStringsToDelete.has(stringName)}
                      disabled={isDeletingUnused}
                    />
                    <span class="string-name">{stringName}</span>
                  </button>
                </li>
              {/each}
            </ul>
          {/if}
        </div>
      </div>
    {/if}
    
    <AlertDialog.Footer class="dialog-standard-footer">
      <AlertDialog.Cancel onclick={closeUnusedDialog} disabled={isDeletingUnused || isFindingUnused}>
        Cancel
      </AlertDialog.Cancel>
      {#if !isFindingUnused && !errorMessage && unusedStrings.length > 0}
        <Button
          onclick={deleteUnusedStrings}
          disabled={isDeletingUnused || selectedCount === 0}
          variant="destructive"
        >
          {isDeletingUnused ? 'Deleting...' : `Delete ${selectedCount} ${selectedCount === 1 ? 'string' : 'strings'}`}
        </Button>
      {/if}
    </AlertDialog.Footer>
  </AlertDialog.Content>
</AlertDialog.Root>

<style>
  .strings-list-container {
    margin: 0.5rem 0 1rem 0;
    border: 1px solid var(--border);
    border-radius: 8px;
    overflow: hidden;
  }

  .strings-selection-controls {
    padding: 0.75rem;
    border-bottom: 1px solid var(--border);
  }

  .selection-buttons {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    flex-wrap: nowrap;
  }

  .selection-count {
    margin-left: auto;
    font-size: 0.875rem;
    color: var(--muted-foreground);
    font-weight: 500;
    padding: 0.25rem 0.5rem;
    background: var(--background);
    border-radius: 4px;
  }

  .strings-list {
    max-height: 400px;
    overflow-y: auto;
  }

  .strings-list ul {
    list-style: none;
    padding: 0;
    margin: 0;
  }

  .string-item {
    border-bottom: 1px solid var(--border);
    list-style: none;
  }

  .string-item:last-child {
    border-bottom: none;
  }

  .string-checkbox-label {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.625rem 1rem;
    cursor: pointer;
    user-select: none;
    width: 100%;
    background: transparent;
    border: none;
    text-align: left;
  }
  
  .string-checkbox-label:hover:not(:disabled) {
    background: color-mix(in oklch, var(--foreground) 4%, transparent);
  }
  
  .string-checkbox-label:disabled {
    cursor: not-allowed;
    opacity: 0.5;
  }
  
  .string-checkbox-label :global([data-slot="checkbox"]) {
    pointer-events: auto;
    flex-shrink: 0;
  }

  .string-name {
    font-family: 'SF Mono', 'Monaco', 'Inconsolata', 'Roboto Mono', monospace;
    font-size: 0.875rem;
    color: var(--foreground);
    flex: 1;
    font-weight: 500;
  }

  .search-progress {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 1rem;
    padding: 3rem 2rem;
    min-height: 200px;
    justify-content: center;
  }

  .search-progress :global(.spinner) {
    width: 32px;
    height: 32px;
    animation: spin 1s linear infinite;
    color: var(--primary);
  }

  @keyframes spin {
    from {
      transform: rotate(0deg);
    }
    to {
      transform: rotate(360deg);
    }
  }

  .search-path {
    font-size: 0.875rem;
    color: var(--muted-foreground);
    text-align: center;
    font-family: monospace;
    word-break: break-all;
    margin: 0;
  }
</style>

