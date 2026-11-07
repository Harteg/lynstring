<script lang="ts">
  import { projectDataStore } from '$lib/stores/project-data';
  import { translationSelectionStore } from '$lib/stores/translation-selection';
  import { stringChangesStore } from '$lib/stores/string-changes';
  import { duplicateKeysStore } from '$lib/stores/project-data';
  import { currentProject } from '$lib/stores/project';
  import Check from '@lucide/svelte/icons/check';
  import Plus from '@lucide/svelte/icons/plus';
  import ChevronRight from '@lucide/svelte/icons/chevron-right';
  import { Button } from '$lib/components/ui/button';
  import TranslationSelectionDialog from '$lib/components/dialogs/TranslationSelectionDialog.svelte';
  import AddLanguageDialog from '$lib/components/dialogs/AddLanguageDialog.svelte';

  const projectData = $derived($projectDataStore);
  const translationSelection = $derived($translationSelectionStore);

  let showLanguageDialog = $state(false);
  let showAddLanguageDialog = $state(false);

  const currentProjectLanguages = $derived(new Set(projectData.languages));

  // Calculate stats
  let totalMissingTranslations = $state(0);

  $effect(() => {
    if (!projectData.strings.length || !projectData.languages.length) {
      totalMissingTranslations = 0;
      return;
    }

    let totalMissing = 0;
    for (const string of projectData.strings) {
      for (const language of projectData.languages) {
        if (string.translationStatus[language] === 'missing') {
          totalMissing++;
        }
      }
    }
    totalMissingTranslations = totalMissing;
  });

  const totalStrings = $derived(projectData.strings.length);
  const totalLanguages = $derived(projectData.languages.length);
  const selectedLanguages = $derived(translationSelection.selectedLanguages.size);
  const duplicateKeys = $derived($duplicateKeysStore);
  const hasDuplicates = $derived(duplicateKeys.length > 0);
  const currentChanges = $derived.by(() =>
    $currentProject ? $stringChangesStore.find((p) => p.projectPath === $currentProject.path) : null
  );
  const hasModified = $derived((currentChanges?.changes.modified.length ?? 0) > 0);

  // Everything in order: no missing, no modified, no duplicates, and we have a project
  const isComplete = $derived(
    totalMissingTranslations === 0 &&
      !hasModified &&
      !hasDuplicates &&
      totalStrings > 0 &&
      totalLanguages > 0
  );
  const hasProject = $derived(totalStrings > 0 || totalLanguages > 0);
</script>

<div class="status-panel" class:status-panel-complete={isComplete} class:status-panel-empty={!hasProject}>
  {#if !hasProject}
    <p class="empty-hint">Select a project to see status</p>
  {:else}
    <div class="status-grid">
      <div class="stat-block">
        <span class="stat-big">{totalStrings}</span>
        <span class="stat-label">strings</span>
      </div>

      <div class="stat-divider" aria-hidden="true"></div>

      <button
        class="stat-block language-block"
        onclick={() => (showLanguageDialog = true)}
        disabled={totalLanguages === 0}
        aria-label="Select languages"
      >
        <span class="stat-big">{selectedLanguages}<span class="stat-sep">/</span>{totalLanguages}</span>
        <span class="stat-label">
          languages
          {#if totalLanguages > 0}
            <ChevronRight class="w-3.5 h-3.5 opacity-60" />
          {/if}
        </span>
      </button>
    </div>
  {/if}
</div>

<TranslationSelectionDialog bind:open={showLanguageDialog} />
<AddLanguageDialog bind:open={showAddLanguageDialog} {currentProjectLanguages} />

<style>
  .status-panel {
    display: flex;
    flex-direction: column;
    height: 150px;
    border-radius: 14px;
    border: 1px solid var(--border);
    background: var(--card);
    box-shadow: 0 1px 2px color-mix(in oklch, black 3%, transparent);
    transition: border-color 0.2s, box-shadow 0.2s;
  }

  .status-panel.status-panel-empty {
    justify-content: center;
  }

  .empty-hint {
    font-size: 0.9375rem;
    color: var(--muted-foreground);
    text-align: center;
    padding: 0.5rem 0;
  }

  .status-grid {
    flex: 1;
    min-height: 0;
    display: flex;
    align-items: stretch;
    width: 100%;
    gap: 0;
  }

  .stat-block {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    background: none;
    border: none;
    padding: 0;
    text-align: center;
    cursor: default;
  }

  .stat-divider {
    width: 1px;
    flex-shrink: 0;
    background: var(--border);
    align-self: stretch;
  }

  .stat-block.language-block {
    cursor: pointer;
    border-radius: 8px;
    padding: 0.25rem 0;
  }

  .stat-block.language-block:hover:not(:disabled) {
    background: color-mix(in oklch, var(--primary) 8%, transparent);
  }

  .stat-block.language-block:disabled {
    cursor: default;
  }

  .stat-big {
    font-family: 'Space Grotesk', system-ui, sans-serif;
    font-size: 2rem;
    font-weight: 700;
    letter-spacing: -0.03em;
    line-height: 1;
    color: var(--foreground);
  }

  .stat-sep {
    opacity: 0.5;
    font-weight: 600;
  }

  .stat-label {
    font-size: 0.8125rem;
    font-weight: 500;
    color: var(--muted-foreground);
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 0.25rem;
  }

</style>
