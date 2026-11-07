<script lang="ts">
  import { translationSelectionStore } from '$lib/stores/translation-selection';
  import { projectDataStore } from '$lib/stores/project-data';
  import { currentProject } from '$lib/stores/project';
  import { getFlagEmoji, getLanguageName } from '$lib/services/languages';
  import PlusIcon from "@lucide/svelte/icons/plus";
  import Trash2 from '@lucide/svelte/icons/trash-2';
  import { Checkbox } from '$lib/components/ui/checkbox';
  import { Button } from '$lib/components/ui/button';
  import AddLanguageDialog from './AddLanguageDialog.svelte';
  import * as Dialog from '$lib/components/ui/dialog';
  import * as AlertDialog from '$lib/components/ui/alert-dialog';
  import { invoke } from '@tauri-apps/api/core';
  import { toast } from 'svelte-sonner';

  interface Props {
    open?: boolean;
  }
  let { open = $bindable() }: Props = $props();

  let showAddLanguageDialog = $state(false);
  let languageToDelete = $state<{ code: string; name: string; translatedCount: number } | null>(null);
  let deleteConfirmOpen = $state(false);

  const projectData = $derived($projectDataStore);
  const translationSelection = $derived($translationSelectionStore);

  const currentProjectLanguages = $derived.by(() => new Set(projectData.languages));

  const languageData = $derived(() => {
    if (!projectData.languages.length) return [];

    return projectData.languages
      .map((lang) => {
        let missingCount = 0;
        if (projectData.strings.length > 0) {
          for (const string of projectData.strings) {
            if (string.translationStatus[lang] === 'missing') {
              missingCount++;
            }
          }
        }

        return {
          code: lang,
          name: getLanguageName(lang),
          flag: getFlagEmoji(lang),
          missing: missingCount,
          total: projectData.strings.length,
        };
      })
      .sort((a, b) => a.name.localeCompare(b.name));
  });

  const selectedCount = $derived(translationSelection.selectedLanguages.size);
  const totalCount = $derived(projectData.languages.length);

  function toggleLanguage(languageCode: string) {
    translationSelectionStore.toggleLanguageSelection(languageCode);
  }

  function toggleAllLanguages() {
    const allLangs = languageData().map((lang) => lang.code);
    const currentSelected = Array.from(translationSelection.selectedLanguages);

    if (currentSelected.length === allLangs.length) {
      translationSelectionStore.clearLanguageSelection();
    } else {
      translationSelectionStore.selectAllLanguages();
    }
  }

  function handleAddLanguage(event?: Event) {
    event?.stopPropagation();
    showAddLanguageDialog = true;
  }

  function openDeleteConfirm(language: { code: string; name: string; missing: number; total: number }, event: MouseEvent) {
    event.stopPropagation();
    event.preventDefault();
    languageToDelete = {
      code: language.code,
      name: language.name,
      translatedCount: language.total - language.missing
    };
    deleteConfirmOpen = true;
  }

  async function confirmDelete() {
    if (!languageToDelete || !$currentProject) return;

    try {
      await invoke('delete_language_from_project', {
        projectPath: $currentProject.path,
        languageCode: languageToDelete.code
      });

      await projectDataStore.loadProjectData($currentProject.path);

      toast.success(`Deleted ${languageToDelete.name}`);
    } catch (err) {
      toast.error('Failed to delete language', { description: String(err) });
    } finally {
      deleteConfirmOpen = false;
      languageToDelete = null;
    }
  }

  function cancelDelete() {
    deleteConfirmOpen = false;
    languageToDelete = null;
  }

</script>

<Dialog.Root bind:open>
  <Dialog.Content class="dialog-standard max-w-[500px]" showClose={true}>
    <Dialog.Header class="dialog-standard-header">
      <Dialog.Title class="dialog-standard-title">Languages</Dialog.Title>
      <Dialog.Description class="dialog-standard-description">
        Choose which languages to translate
      </Dialog.Description>
    </Dialog.Header>

    <div class="dialog-body">
      {#if totalCount > 0}
      <div class="languages-list-container">
        <div class="select-all-row">
          <button
            class="select-all-btn"
            onclick={toggleAllLanguages}
          >
            <Checkbox
              checked={selectedCount === totalCount && totalCount > 0}
              indeterminate={selectedCount > 0 && selectedCount < totalCount}
              onchange={toggleAllLanguages}
            />
            <span class="select-all-text">Select all</span>
            <span class="select-all-count">{selectedCount}/{totalCount}</span>
          </button>
          <button
            class="add-lang-btn"
            onclick={handleAddLanguage}
            aria-label="Add languages"
          >
            <PlusIcon class="w-5 h-5" />
          </button>
        </div>

        <div class="language-list">
        {#each languageData() as language}
          <div
            class="language-item"
            role="button"
            tabindex="0"
            onclick={() => toggleLanguage(language.code)}
            onkeydown={(e) => e.key === 'Enter' && toggleLanguage(language.code)}
          >
            <Checkbox
              checked={translationSelection.selectedLanguages.has(language.code)}
              onchange={() => toggleLanguage(language.code)}
            />
            <span class="emoji-font">{language.flag}</span>
            <span class="lang-code">{language.code}</span>
            <span class="lang-dot">·</span>
            <span class="lang-name">{language.name}</span>
            <span class="language-status-wrap">
              <span class="language-status">
                {#if language.missing > 0}
                  <span class="lang-missing">{language.missing} missing</span>
                {:else}
                  <span class="lang-complete">Complete</span>
                {/if}
              </span>
              <span class="delete-lang-btn">
                <Button
                  variant="ghost"
                  size="sm"
                  onclick={(e) => openDeleteConfirm(language, e)}
                  aria-label="Delete {language.name}"
                >
                  <Trash2 class="w-3.5 h-3.5" />
                  Delete
                </Button>
              </span>
            </span>
          </div>
        {/each}
        </div>
      </div>
      {:else}
      <div class="empty-state">
        <p>No languages in this project yet.</p>
        <Button onclick={handleAddLanguage}>
          <PlusIcon class="w-4 h-4 mr-2" />
          Add languages
        </Button>
      </div>
      {/if}
    </div>

    <Dialog.Footer class="dialog-standard-footer">
      <Button onclick={() => (open = false)}>
        Done
      </Button>
    </Dialog.Footer>
  </Dialog.Content>
</Dialog.Root>

<AlertDialog.Root bind:open={deleteConfirmOpen}>
  <AlertDialog.Content class="dialog-standard sm:max-w-[420px]">
    <AlertDialog.Header class="dialog-standard-header">
      <AlertDialog.Title class="dialog-standard-title">Delete language?</AlertDialog.Title>
      <AlertDialog.Description class="dialog-standard-description">
        {#if languageToDelete}
          Do you want to delete {languageToDelete.name} with {languageToDelete.translatedCount} translation{languageToDelete.translatedCount === 1 ? '' : 's'}? This will remove the strings.xml file for this language.
        {/if}
      </AlertDialog.Description>
    </AlertDialog.Header>
    <AlertDialog.Footer class="dialog-standard-footer">
      <AlertDialog.Cancel onclick={cancelDelete}>Cancel</AlertDialog.Cancel>
      <AlertDialog.Action
        onclick={confirmDelete}
        class="bg-destructive text-destructive-foreground hover:bg-destructive/90"
      >
        Delete
      </AlertDialog.Action>
    </AlertDialog.Footer>
  </AlertDialog.Content>
</AlertDialog.Root>

<AddLanguageDialog
  bind:open={showAddLanguageDialog}
  currentProjectLanguages={currentProjectLanguages}
/>

<style>
  .dialog-body {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
    max-height: 400px;
    overflow: hidden;
  }

  .languages-list-container {
    margin: 0;
    border: 1px solid var(--border);
    border-radius: 8px;
    overflow: hidden;
    display: flex;
    flex-direction: column;
    flex: 1;
    min-height: 0;
  }

  .select-all-row {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.75rem;
    border-bottom: 1px solid var(--border);
  }

  .select-all-btn {
    flex: 1;
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.75rem 1rem;
    text-align: left;
    background: color-mix(in oklch, var(--primary) 10%, transparent);
    border: 1px solid color-mix(in oklch, var(--primary) 25%, transparent);
    border-radius: 10px;
    cursor: pointer;
    transition: background 0.2s, border-color 0.2s;
  }

  .select-all-btn:hover {
    background: color-mix(in oklch, var(--primary) 15%, transparent);
    border-color: color-mix(in oklch, var(--primary) 35%, transparent);
  }

  .select-all-text {
    font-weight: 600;
    font-size: 0.9375rem;
    color: var(--foreground);
  }

  .select-all-count {
    font-size: 0.8125rem;
    color: var(--muted-foreground);
    font-variant-numeric: tabular-nums;
  }

  .add-lang-btn {
    width: 44px;
    height: 44px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--primary);
    color: var(--primary-foreground);
    border: none;
    border-radius: 10px;
    cursor: pointer;
    transition: background 0.2s;
  }

  .add-lang-btn:hover {
    background: var(--primary-hover);
  }

  .language-list {
    display: flex;
    flex-direction: column;
    overflow-y: auto;
    padding-right: 0.25rem;
    flex: 1;
    min-height: 0;
  }

  .language-item {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.625rem 1rem;
    width: 100%;
    text-align: left;
    background: transparent;
    border: none;
    border-bottom: 1px solid var(--border);
    cursor: pointer;
  }

  .language-item:last-child {
    border-bottom: none;
  }

  .language-item:hover {
    background: color-mix(in oklch, var(--foreground) 4%, transparent);
  }

  .language-status-wrap {
    position: relative;
    display: flex;
    align-items: center;
    justify-content: flex-end;
    margin-left: auto;
    min-width: 5rem; /* Reserve space for Delete button to prevent jump */
  }

  .language-status {
    display: inline;
  }

  .delete-lang-btn {
    position: absolute;
    right: 0;
    top: 50%;
    transform: translateY(-50%);
    display: flex;
    align-items: center;
    opacity: 0;
    pointer-events: none;
  }

  .delete-lang-btn :global(button) {
    margin-right: -0.5rem;
    font-size: 0.75rem;
    color: var(--destructive);
  }
  .delete-lang-btn :global(button):hover {
    color: var(--destructive); /* keep consistent, let focus ring or background indicate hover if needed */
    background: color-mix(in oklch, var(--destructive) 10%, transparent); /* subtle hover */
  }

  .language-item:hover .language-status {
    opacity: 0;
    pointer-events: none;
  }

  .language-item:hover .delete-lang-btn {
    opacity: 1;
    pointer-events: auto;
  }

  .lang-code {
    font-size: 0.75rem;
    color: var(--muted-foreground);
    font-family: ui-monospace, monospace;
    min-width: 2rem;
  }

  .lang-dot {
    color: var(--muted-foreground);
  }

  .lang-name {
    flex: 1;
    font-weight: 500;
    font-size: 0.9375rem;
  }

  .lang-missing {
    font-size: 0.75rem;
    color: var(--warning);
  }

  .lang-complete {
    font-size: 0.75rem;
    color: var(--primary);
  }

  .emoji-font {
    font-family: "Segoe UI Emoji", "Segoe UI Symbol", "Apple Color Emoji",
      "Noto Color Emoji", sans-serif;
    font-size: 1.25rem;
    line-height: 1;
  }

  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 1rem;
    padding: 2rem;
    text-align: center;
    color: var(--muted-foreground);
  }
</style>
