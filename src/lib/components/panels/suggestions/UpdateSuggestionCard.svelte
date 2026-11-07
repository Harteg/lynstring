<script lang="ts">
  import { stringChangesStore } from '$lib/stores/string-changes';
  import { currentProject } from '$lib/stores/project';
  import { projectDataStore } from '$lib/stores/project-data';
  import { translationSelectionStore } from '$lib/stores/translation-selection';
  import { translationService, translationStore } from '$lib/services/translation';
  import { Button } from '$lib/components/ui/button';
  import Check from '@lucide/svelte/icons/check';
  import Sparkles from '@lucide/svelte/icons/sparkles';
  import type { AndroidString } from '../../../../../src-tauri/bindings/AndroidString';
  import SuggestionCard from './SuggestionCard.svelte';
  import * as Dialog from '$lib/components/ui/dialog';
  import DiffText from '$lib/components/DiffText.svelte';
  import Loader2 from '@lucide/svelte/icons/loader-2';
  import { onMount, onDestroy } from 'svelte';
  import * as Kbd from '$lib/components/ui/kbd';

  const translationSelection = $derived($translationSelectionStore);
  const projectData = $derived($projectDataStore);
  const isTranslatingOrPreparing = $derived(
    $translationStore.isTranslating || $translationStore.isPreparing
  );
  const showUpdateSpinner = $derived(
    isTranslatingOrPreparing &&
      $translationStore.translationSource === 'update' &&
      $translationStore.currentProjectPath === $currentProject?.path
  );

  const currentChanges = $derived.by(() => {
    if (!$currentProject) return null;
    return $stringChangesStore.find((p) => p.projectPath === $currentProject.path) || null;
  });

  const modifiedStrings = $derived(currentChanges?.changes.modified ?? []);
  const modifiedCount = $derived(modifiedStrings.length);
  const hasModified = $derived(modifiedCount > 0 && !!$currentProject);
  const shouldShow = $derived(!!$currentProject);
  const updateIcon = $derived(modifiedCount === 0 ? Check : Sparkles);
  const isDisabled = $derived(modifiedCount === 0);
  let detailsOpen = $state(false);

  async function handleUpdateAll() {
    if (!currentChanges || !$currentProject || !projectData.strings.length) return;
    const modified = currentChanges.changes.modified;
    if (modified.length === 0) return;

    const selectedLanguages = Array.from(translationSelection.selectedLanguages || []);
    if (selectedLanguages.length === 0) return;

    const changeId = currentChanges.id;
    translationService.setPreparing(true, 'update');

    try {
      const modifiedNames = new Set(modified.map((m) => m.name));
      const stringsToTranslate = projectData.strings.filter(
        (s) => modifiedNames.has(s.name) && s.defaultValue !== undefined
      );
      if (stringsToTranslate.length === 0) return;

      const stringsByLanguage = new Map<string, AndroidString[]>();
      for (const language of selectedLanguages) {
        stringsByLanguage.set(
          language,
          stringsToTranslate.map((s) => ({
            name: s.name,
            text: s.defaultValue,
            is_translatable: s.isTranslatable,
            is_commented_out: false,
            line_number: 0,
            has_cdata: s.hasCdata ?? false
          }))
        );
      }
      await translationService.translateMultipleLanguages(stringsByLanguage, $currentProject, { source: 'update' });
      await projectDataStore.loadProjectData($currentProject.path);
      await stringChangesStore.acknowledgeChanges(changeId, []);
    } catch (err) {
      console.error('Update failed:', err);
    } finally {
      translationService.setPreparing(false);
      detailsOpen = false;
    }
  }

  async function handleIgnoreAll() {
    if (!currentChanges) return;
    const modifiedNames = currentChanges.changes.modified.map((m) => m.name);
    try {
      await stringChangesStore.acknowledgeChanges(currentChanges.id, modifiedNames);
    } catch (err) {
      console.error('Failed to ignore:', err);
    }
    detailsOpen = false;
  }

  async function handleIgnoreString(stringName: string) {
    if (!currentChanges) return;
    try {
      await stringChangesStore.acknowledgeChanges(currentChanges.id, [stringName]);
    } catch (err) {
      console.error('Failed to ignore string:', err);
    }
  }

  function handleKeyDown(event: KeyboardEvent) {
    const target = event.target as HTMLElement;
    if (target.tagName === 'INPUT' || target.tagName === 'TEXTAREA' || target.isContentEditable) return;
    if (event.ctrlKey || event.metaKey || event.altKey) return;
    if (event.key.toLowerCase() === 'u') {
      event.preventDefault();
      if (
        hasModified &&
        translationSelection.selectedLanguages.size > 0 &&
        !isTranslatingOrPreparing
      ) {
        handleUpdateAll();
      }
    }
  }

  onMount(() => window.addEventListener('keydown', handleKeyDown));
  onDestroy(() => window.removeEventListener('keydown', handleKeyDown));
</script>

{#if shouldShow}
  <div class="update-suggestion">
    <SuggestionCard
      icon={updateIcon}
      title="{modifiedCount} modified {modifiedCount === 1 ? 'string' : 'strings'}"
      description={modifiedCount > 0 ? 'Update translations' : undefined}
      variant={modifiedCount > 0 ? 'warning' : 'default'}
    >
      {#snippet action()}
        <div class="action-row">
          {#if modifiedCount > 0}
            <Button
              variant="ghost"
              size="sm"
              onclick={() => (detailsOpen = true)}
            >
              Details
            </Button>
          {/if}
          <Button
            size="sm"
            onclick={handleUpdateAll}
            disabled={modifiedCount === 0 ||
              translationSelection.selectedLanguages.size === 0 ||
              isTranslatingOrPreparing}
          >
          {#if showUpdateSpinner}
            <Loader2 class="h-4 w-4 animate-spin" />
          {:else}
            Update
            <Kbd.Root class="kbd-on-primary">U</Kbd.Root>
          {/if}
          </Button>
        </div>
      {/snippet}
    </SuggestionCard>

    <Dialog.Root bind:open={detailsOpen}>
    <Dialog.Content class="dialog-standard sm:max-w-[520px]" showClose={true}>
      <Dialog.Header class="dialog-standard-header">
        <Dialog.Title class="dialog-standard-title">Modified strings</Dialog.Title>
        <Dialog.Description class="dialog-standard-description">
          These strings changed in your default values. Update translations to keep them in sync.
        </Dialog.Description>
      </Dialog.Header>

      <div class="modified-list">
        {#each modifiedStrings as modified (modified.name)}
          <div class="modified-item">
            <div class="modified-header">
              <span class="modified-key">{modified.name}</span>
              <Button variant="ghost" size="sm" onclick={() => handleIgnoreString(modified.name)}>
                Ignore
              </Button>
            </div>
            <div class="modified-diff">
              <div class="diff-row old">
                <span class="diff-label">−</span>
                <span class="diff-content">
                  <DiffText oldText={modified.old_value} newText={modified.new_value} type="old" />
                </span>
              </div>
              <div class="diff-row new">
                <span class="diff-label">+</span>
                <span class="diff-content">
                  <DiffText oldText={modified.old_value} newText={modified.new_value} type="new" />
                </span>
              </div>
            </div>
          </div>
        {/each}
      </div>

      <Dialog.Footer class="dialog-standard-footer">
        <Button variant="ghost" onclick={handleIgnoreAll}>Ignore all</Button>
        <Button
          onclick={handleUpdateAll}
          disabled={modifiedCount === 0 ||
            translationSelection.selectedLanguages.size === 0 ||
            isTranslatingOrPreparing}
        >
          {#if showUpdateSpinner}
            <Loader2 class="h-4 w-4 animate-spin" />
          {:else}
            Update {modifiedCount} {modifiedCount === 1 ? 'translation' : 'translations'}
          {/if}
        </Button>
      </Dialog.Footer>
    </Dialog.Content>
  </Dialog.Root>
  </div>
{/if}

<style>
  .update-suggestion :global(button:disabled) {
    background: var(--muted) !important;
    color: var(--muted-foreground) !important;
    border-color: var(--border) !important;
    box-shadow: none !important;
    opacity: 1 !important;
  }

  .update-suggestion :global(button:disabled .kbd-on-primary) {
    background: color-mix(in oklch, var(--muted-foreground) 25%, transparent) !important;
    color: var(--muted-foreground) !important;
    border-color: color-mix(in oklch, var(--muted-foreground) 35%, transparent) !important;
  }

  .action-row {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }
  .modified-list {
    max-height: 60vh;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 1rem;
    padding-right: 0.25rem;
  }
  .modified-item {
    padding: 0.75rem;
    background: var(--card);
    border-radius: 8px;
    border: 1px solid var(--border);
  }
  .modified-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.75rem;
    margin-bottom: 0.5rem;
  }
  .modified-key {
    font-weight: 600;
    font-size: 0.8125rem;
    font-family: ui-monospace, monospace;
    flex: 1;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .modified-diff {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }
  .diff-row {
    display: flex;
    gap: 0.5rem;
    font-size: 0.8125rem;
    padding: 0.375rem 0.5rem;
    border-radius: 4px;
  }
  .diff-row.old {
    background: color-mix(in oklch, var(--destructive) 15%, transparent);
  }
  .diff-row.old .diff-label {
    color: var(--destructive);
  }
  .diff-row.new {
    background: color-mix(in oklch, var(--primary) 15%, transparent);
  }
  .diff-row.new .diff-label {
    color: var(--primary);
  }
  .diff-label {
    flex-shrink: 0;
    width: 1.25rem;
    text-align: center;
  }
  .diff-content {
    flex: 1;
    word-break: break-word;
  }

  :global(.kbd-on-primary) {
    margin-left: 0.5rem;
    font-size: 0.7rem;
    padding: 0.125rem 0.375rem;
    background: color-mix(in oklch, var(--primary-foreground) 25%, transparent) !important;
    color: var(--primary-foreground) !important;
    border-color: color-mix(in oklch, var(--primary-foreground) 35%, transparent) !important;
  }
</style>

