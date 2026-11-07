<script lang="ts">
  import * as AlertDialog from '$lib/components/ui/alert-dialog';
  import { Button } from '$lib/components/ui/button';
  import { Input } from '$lib/components/ui/input';
  import { Checkbox } from '$lib/components/ui/checkbox';
  import { getLanguages, type Language } from '$lib/services/languages';
  import { invoke } from "@tauri-apps/api/core";
  import { get } from 'svelte/store';
  import { currentProject } from '$lib/stores/project';
  import { projectDataStore } from '$lib/stores/project-data';
  import { translationSelectionStore } from '$lib/stores/translation-selection';
  import { toast } from "svelte-sonner";

  interface Props {
    open: boolean;
    currentProjectLanguages: Set<string>;
    onOpenChange?: (open: boolean) => void;
  }

  let { open = $bindable(), currentProjectLanguages, onOpenChange }: Props = $props();

  let allLanguages = $state<Language[]>([]);
  let selectedNewLanguages = $state<string[]>([]);
  let languageSearchQuery = $state('');

  $effect(() => {
    if (open) {
      loadLanguages();
    }
  });

  async function loadLanguages() {
    allLanguages = await getLanguages();
    selectedNewLanguages = [];
    languageSearchQuery = '';
  }

  function closeDialog() {
    open = false;
    selectedNewLanguages = [];
    languageSearchQuery = '';
    onOpenChange?.(false);
  }

  function clearSelection() {
    selectedNewLanguages = [];
  }

  const filteredLanguages = $derived.by(() => {
    if (!languageSearchQuery.trim()) {
      return allLanguages;
    }
    const query = languageSearchQuery.toLowerCase();
    return allLanguages.filter(lang => 
      lang.name.toLowerCase().includes(query) || 
      lang.code.toLowerCase().includes(query)
    );
  });

  function toggleNewLanguage(languageCode: string) {
    if (currentProjectLanguages.has(languageCode)) return;

    const index = selectedNewLanguages.indexOf(languageCode);
    if (index !== -1) {
      selectedNewLanguages = selectedNewLanguages.filter(lang => lang !== languageCode);
    } else {
      selectedNewLanguages = [...selectedNewLanguages, languageCode];
    }
  }

  const newlySelectedLanguages = $derived.by(() => {
    return selectedNewLanguages
      .filter((lang: string) => !currentProjectLanguages.has(lang))
      .map((code: string): Language => {
        const lang = allLanguages.find(l => l.code === code);
        return lang || { code, name: code, emoji: '🌐' };
      });
  });

  async function handleAddLanguages() {
    if (newlySelectedLanguages.length === 0) return;

    if (!$currentProject) {
      toast.error("No project selected");
      return;
    }

    const translationState = get(translationSelectionStore);
    const allLanguagesWereSelected = translationState.selectedLanguages.size === currentProjectLanguages.size 
      && currentProjectLanguages.size > 0;
    const noLanguagesSelected = translationState.selectedLanguages.size === 0;

    const languagesToAdd = newlySelectedLanguages.map((lang: Language) => lang.code);

    try {
      const addedLanguages = await invoke<string[]>("add_languages_to_project", {
        projectPath: $currentProject.path,
        languageCodes: languagesToAdd
      });

      closeDialog();
      await projectDataStore.loadProjectData($currentProject.path);

      if (allLanguagesWereSelected && addedLanguages.length > 0) {
        // If all languages were selected before, select all languages (including new ones)
        translationSelectionStore.selectAllLanguages();
      } else if (noLanguagesSelected && addedLanguages.length > 0) {
        // If no languages were selected, select all newly added languages
        translationSelectionStore.selectLanguages(addedLanguages);
      }

      if (addedLanguages.length > 0) {
        toast.success(`Added ${addedLanguages.length} language${addedLanguages.length !== 1 ? 's' : ''}`);
      }
    } catch (err) {
      toast.error("Failed to add languages", { description: String(err) });
    }
  }
</script>

<AlertDialog.Root bind:open>
  <AlertDialog.Content class="dialog-standard max-w-[500px]">
    <AlertDialog.Header class="dialog-standard-header !pb-3">
      <AlertDialog.Title class="dialog-standard-title">Add Languages</AlertDialog.Title>
    </AlertDialog.Header>
    
    <div class="languages-list-container">
      <div class="search-container">
        <Input
          type="text"
          placeholder="Search languages..."
          bind:value={languageSearchQuery}
        />
      </div>
      <div class="languages-list">
        {#if allLanguages.length > 0}
          <ul>
            {#each filteredLanguages as language}
              {@const isCurrentProjectLanguage = currentProjectLanguages.has(language.code)}
              {@const isSelected = isCurrentProjectLanguage || selectedNewLanguages.includes(language.code)}
              <li class="language-item">
                <button
                  class="language-row"
                  onclick={() => toggleNewLanguage(language.code)}
                  disabled={isCurrentProjectLanguage}
                  type="button"
                >
                  <Checkbox
                    checked={isSelected}
                    disabled={isCurrentProjectLanguage}
                  />
                  <span class="language-flag">{language.emoji}</span>
                  <span class="language-code">{language.code}</span>
                  <span class="language-separator">·</span>
                  <span class="language-name">{language.name}</span>
                </button>
              </li>
            {/each}
          </ul>
        {:else if filteredLanguages.length === 0 && languageSearchQuery.trim()}
          <div class="empty-state">No languages found matching "{languageSearchQuery}"</div>
        {:else}
          <div class="empty-state">Loading languages...</div>
        {/if}
      </div>
    </div>
    
    <AlertDialog.Footer class="dialog-standard-footer dialog-footer">
      <div class="footer-left">
        {#if newlySelectedLanguages.length > 0}
          <Button onclick={clearSelection} variant="ghost" size="sm">
            Clear selection
          </Button>
        {/if}
      </div>
      <div class="footer-right">
        <AlertDialog.Cancel onclick={closeDialog}>Cancel</AlertDialog.Cancel>
        <Button
          onclick={handleAddLanguages}
          disabled={newlySelectedLanguages.length === 0}
        >
          Add {newlySelectedLanguages.length} {newlySelectedLanguages.length === 1 ? 'language' : 'languages'}
        </Button>
      </div>
    </AlertDialog.Footer>
  </AlertDialog.Content>
</AlertDialog.Root>

<style>
  .languages-list-container {
    margin: 0.5rem 0 1rem 0;
    border: 1px solid var(--border);
    border-radius: 8px;
    overflow: hidden;
  }

  .search-container {
    padding: 0.75rem;
    border-bottom: 1px solid var(--border);
  }

  .languages-list {
    max-height: 400px;
    overflow-y: auto;
  }

  .languages-list ul {
    list-style: none;
    padding: 0;
    margin: 0;
  }

  .language-item {
    border-bottom: 1px solid var(--border);
  }

  .language-item:last-child {
    border-bottom: none;
  }

  .language-row {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.625rem 1rem;
    width: 100%;
    background: transparent;
    border: none;
    text-align: left;
    cursor: pointer;
  }
  
  .language-row:hover:not(:disabled) {
    background: color-mix(in oklch, var(--foreground) 4%, transparent);
  }
  
  .language-row:disabled {
    cursor: not-allowed;
    opacity: 0.5;
  }

  .language-flag {
    font-size: 1.25rem;
    font-family: "Segoe UI Emoji", "Segoe UI Symbol", "Apple Color Emoji", "Noto Color Emoji", sans-serif;
    font-style: normal;
    font-variant-emoji: emoji;
    text-rendering: optimizeLegibility;
  }

  .language-code {
    font-size: 0.75rem;
    color: var(--muted-foreground);
    font-family: ui-monospace, monospace;
    min-width: 2rem;
  }

  .language-separator {
    color: var(--muted-foreground);
  }

  .language-name {
    flex: 1;
    font-size: 0.9375rem;
    font-weight: 500;
  }

  .empty-state {
    padding: 2rem;
    text-align: center;
    color: var(--muted-foreground);
  }

  :global(.dialog-footer) {
    display: flex !important;
    flex-direction: row !important;
    flex-wrap: nowrap !important;
    justify-content: space-between !important;
  }

  .footer-left {
    display: flex;
    align-items: center;
  }

  .footer-right {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }
</style>
