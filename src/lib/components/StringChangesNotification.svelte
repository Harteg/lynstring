<script lang="ts">
  import { stringChangesStore, type PendingChange } from '$lib/stores/string-changes';
  import { currentProject } from '$lib/stores/project';
  import { Button } from '$lib/components/ui/button';
  import { invoke } from '@tauri-apps/api/core';
  import XIcon from '@lucide/svelte/icons/x';
  import LanguagesIcon from '@lucide/svelte/icons/languages';
  import RefreshCcw from '@lucide/svelte/icons/refresh-ccw';
  import { translationService, translationStore } from '$lib/services/translation';
  import { translationSelectionStore } from '$lib/stores/translation-selection';
  import { projectDataStore } from '$lib/stores/project-data';

  // Get current project's pending changes
  const currentChanges = $derived(() => {
    if (!$currentProject) return null;
    const pending = $stringChangesStore;
    return pending.find(p => p.projectPath === $currentProject.path) || null;
  });

  const hasChanges = $derived(() => {
    const changes = currentChanges();
    if (!changes) return false;
    return changes.changes.new.length > 0 || 
           changes.changes.modified.length > 0 || 
           changes.changes.deleted.length > 0;
  });

  async function handleAcknowledge() {
    if (!currentChanges()) return;
    
    try {
      await stringChangesStore.acknowledgeChanges(currentChanges()!.id);
    } catch (error) {
      console.error('Failed to acknowledge changes:', error);
    }
  }

  async function handleDismiss() {
    if (!currentChanges()) return;
    stringChangesStore.removeChange(currentChanges()!.id);
  }

  async function handleTranslateNew() {
    if (!currentChanges() || !$currentProject) return;
    
    const newStrings = currentChanges()!.changes.new;
    if (newStrings.length === 0) return;
    
    // Set preparing state
    translationService.setPreparing(true);
    
    try {
      // TODO: Trigger translation for new strings
      // For now, just acknowledge
      await handleAcknowledge();
    } finally {
      translationService.setPreparing(false);
    }
  }

  async function handleUpdateModified() {
    if (!currentChanges() || !$currentProject) return;
    
    const modified = currentChanges()!.changes.modified;
    if (modified.length === 0) return;
    
    // Set preparing state
    translationService.setPreparing(true);
    
    try {
      // TODO: Trigger translation for modified strings
      // For now, just acknowledge
      await handleAcknowledge();
    } finally {
      translationService.setPreparing(false);
    }
  }

  const newCount = $derived(currentChanges()?.changes.new.length || 0);
  const modifiedCount = $derived(currentChanges()?.changes.modified.length || 0);
  const deletedCount = $derived(currentChanges()?.changes.deleted.length || 0);
  const isTranslatingOrPreparing = $derived(
    $translationStore.isTranslating || $translationStore.isPreparing
  );
</script>

{#if hasChanges() && $currentProject}
  <div class="string-changes-notification">
    <div class="notification-content">
      <div class="notification-header">
        <LanguagesIcon class="icon" size={18} />
        <div class="notification-title">
          <span>String changes detected</span>
          <span class="change-count">
            {newCount > 0 && `${newCount} new`}
            {newCount > 0 && modifiedCount > 0 && ', '}
            {modifiedCount > 0 && `${modifiedCount} modified`}
            {deletedCount > 0 && (newCount > 0 || modifiedCount > 0 ? ', ' : '') + `${deletedCount} deleted`}
          </span>
        </div>
        <button class="dismiss-button" onclick={handleDismiss} aria-label="Dismiss">
          <XIcon size={16} />
        </button>
      </div>

      <div class="changes-list">
        {#if newCount > 0}
          <div class="change-group">
            <div class="change-group-header">
              <span class="change-type new">New strings ({newCount})</span>
              {#if $translationSelectionStore.selectedLanguages.size > 0}
                <Button 
                  variant="ghost" 
                  size="sm"
                  onclick={handleTranslateNew}
                  disabled={isTranslatingOrPreparing}
                  class="action-button"
                >
                  <LanguagesIcon size={14} />
                  Translate
                </Button>
              {/if}
            </div>
            <ul class="change-items">
              {#each currentChanges()!.changes.new as name}
                <li class="change-item">{name}</li>
              {/each}
            </ul>
          </div>
        {/if}

        {#if modifiedCount > 0}
          <div class="change-group">
            <div class="change-group-header">
              <span class="change-type modified">Modified strings ({modifiedCount})</span>
              {#if $translationSelectionStore.selectedLanguages.size > 0}
                <Button 
                  variant="ghost" 
                  size="sm"
                  onclick={handleUpdateModified}
                  disabled={isTranslatingOrPreparing}
                  class="action-button"
                >
                  <RefreshCcw size={14} />
                  Update translations
                </Button>
              {/if}
            </div>
            <ul class="change-items">
              {#each currentChanges()!.changes.modified as mod}
                <li class="change-item">
                  <span class="string-name">{mod.name}</span>
                  {#if mod.translatable_changed}
                    <span class="translatable-badge">translatable changed</span>
                  {/if}
                  <div class="value-diff">
                    <span class="old-value">"{mod.old_value}"</span>
                    <span class="arrow">→</span>
                    <span class="new-value">"{mod.new_value}"</span>
                  </div>
                </li>
              {/each}
            </ul>
          </div>
        {/if}

        {#if deletedCount > 0}
          <div class="change-group">
            <div class="change-group-header">
              <span class="change-type deleted">Deleted strings ({deletedCount})</span>
            </div>
            <ul class="change-items">
              {#each currentChanges()!.changes.deleted as name}
                <li class="change-item deleted">{name}</li>
              {/each}
            </ul>
          </div>
        {/if}
      </div>

      <div class="notification-actions">
        <Button 
          variant="default" 
          size="sm"
          onclick={handleAcknowledge}
        >
          Acknowledge
        </Button>
        <Button 
          variant="ghost" 
          size="sm"
          onclick={handleDismiss}
        >
          Dismiss
        </Button>
      </div>
    </div>
  </div>
{/if}

<style>
  .string-changes-notification {
    position: fixed;
    bottom: 1rem;
    right: 1rem;
    background: var(--background);
    border: 1px solid var(--border);
    border-radius: 8px;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
    z-index: 1000;
    width: min(90vw, 500px);
    max-height: 80vh;
    overflow-y: auto;
    animation: slide-in 0.3s ease-out;
  }

  @keyframes slide-in {
    from {
      transform: translateX(100%);
      opacity: 0;
    }
    to {
      transform: translateX(0);
      opacity: 1;
    }
  }

  .notification-content {
    padding: 1rem;
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .notification-header {
    display: flex;
    align-items: center;
    gap: 0.75rem;
  }

  .notification-title {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }

  .notification-title > span:first-child {
    font-weight: 600;
    font-size: 0.875rem;
    color: var(--foreground);
  }

  .change-count {
    font-size: 0.75rem;
    color: var(--muted-foreground);
  }

  .dismiss-button {
    background: none;
    border: none;
    cursor: pointer;
    padding: 0.25rem;
    border-radius: 4px;
    color: var(--muted-foreground);
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .dismiss-button:hover {
    background: var(--muted);
    color: var(--foreground);
  }

  .changes-list {
    display: flex;
    flex-direction: column;
    gap: 1rem;
    max-height: 400px;
    overflow-y: auto;
  }

  .change-group {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .change-group-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.5rem;
  }

  .change-type {
    font-weight: 600;
    font-size: 0.8125rem;
  }

  .change-type.new {
    color: #22c55e;
  }

  .change-type.modified {
    color: #f59e0b;
  }

  .change-type.deleted {
    color: #ef4444;
  }

  .action-button {
    height: 1.75rem;
    padding: 0 0.5rem;
    font-size: 0.75rem;
  }

  .change-items {
    list-style: none;
    padding: 0;
    margin: 0;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .change-item {
    padding: 0.5rem;
    background: color-mix(in oklch, var(--muted) 30%, transparent);
    border-radius: 4px;
    font-size: 0.8125rem;
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }

  .change-item.deleted {
    opacity: 0.6;
  }

  .string-name {
    font-weight: 600;
    color: var(--foreground);
  }

  .translatable-badge {
    font-size: 0.625rem;
    padding: 0.125rem 0.375rem;
    background: var(--muted);
    border-radius: 3px;
    color: var(--muted-foreground);
    display: inline-block;
    width: fit-content;
  }

  .value-diff {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-size: 0.75rem;
    flex-wrap: wrap;
  }

  .old-value {
    color: var(--muted-foreground);
    text-decoration: line-through;
  }

  .arrow {
    color: var(--muted-foreground);
  }

  .new-value {
    color: var(--foreground);
    font-weight: 500;
  }

  .notification-actions {
    display: flex;
    gap: 0.5rem;
    justify-content: flex-end;
  }
</style>

