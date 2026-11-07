<script lang="ts">
  import { projectDataStore } from '$lib/stores/project-data';
  import { translationSelectionStore } from '$lib/stores/translation-selection';
  import { selectedStringsStore } from '$lib/stores/selected-strings';
  import { currentProject } from '$lib/stores/project';
  import { translationService, translationStore } from '$lib/services/translation';
  import Check from '@lucide/svelte/icons/check';
  import LanguagesIcon from '@lucide/svelte/icons/languages';
  import { Button } from '$lib/components/ui/button';
  import * as Kbd from '$lib/components/ui/kbd';
  import { onDestroy, onMount } from 'svelte';
  import type { AndroidString } from '../../../../../src-tauri/bindings/AndroidString';
  import type { StringStatusRow } from '$lib/types/strings';
  import { duplicateKeysStore } from '$lib/stores/project-data';
  import SuggestionCard from './SuggestionCard.svelte';
  import Loader2 from '@lucide/svelte/icons/loader-2';

  let { showAlert = () => {} }: { showAlert?: (title: string, message: string) => void } = $props();

  const projectData = $derived($projectDataStore);
  const translationSelection = $derived($translationSelectionStore);
  const selectedStrings = $derived($selectedStringsStore);
  const duplicateKeys = $derived($duplicateKeysStore);
  const hasDuplicates = $derived(duplicateKeys.length > 0);
  const translationState = $derived($translationStore);

  $effect(() => {
    if (
      $currentProject &&
      translationState.currentProjectPath &&
      translationState.currentProjectPath !== $currentProject.path &&
      !translationState.isTranslating &&
      !translationState.isPreparing
    ) {
      translationService.clearState();
    }
  });

  const PROGRESS_REFRESH_INTERVAL = 2000;
  let lastProgressRefresh = 0;
  $effect(() => {
    if (!translationState.progress || !$currentProject) return;
    const now = Date.now();
    if (now - lastProgressRefresh < PROGRESS_REFRESH_INTERVAL) return;
    lastProgressRefresh = now;
    projectDataStore.loadProjectData($currentProject.path);
  });
  $effect(() => {
    if (!translationState.isTranslating) lastProgressRefresh = 0;
  });

  const isTranslatingOrPreparing = $derived(
    translationState.isTranslating || translationState.isPreparing
  );
  const showTranslateSpinner = $derived(
    isTranslatingOrPreparing &&
      translationState.translationSource === 'translate' &&
      translationState.currentProjectPath === $currentProject?.path
  );

  const missingCount = $derived(() => {
    if (!projectData.strings.length) return 0;
    const selectedLanguages = Array.from(translationSelection.selectedLanguages);
    if (selectedLanguages.length === 0) return 0;
    if (selectedStrings.size > 0) return selectedStrings.size * selectedLanguages.length;
    let totalMissing = 0;
    for (const string of projectData.strings) {
      for (const language of selectedLanguages) {
        if (string.translationStatus[language] === 'missing') totalMissing++;
      }
    }
    return totalMissing;
  });

  const shouldShow = $derived(() => $currentProject && !hasDuplicates);
  const translateIcon = $derived(missingCount() === 0 ? Check : LanguagesIcon);
  const isDisabled = $derived(missingCount() === 0);

  function isOnline() {
    return navigator.onLine;
  }

  async function handleTranslateAll(event?: MouseEvent) {
    if (!$currentProject) return;
    if (hasDuplicates) return;
    if (event && isTranslatingOrPreparing) return;
    if (translationState.isTranslating) return;
    if (!isOnline()) {
      showAlert('No Internet Connection', 'Please connect to the internet to generate translations.');
      return;
    }
    const selectedLanguages = Array.from(translationSelection.selectedLanguages);
    if (selectedLanguages.length === 0) return;

    const hasManualSelection = selectedStrings.size > 0;
    const stringsToTranslate = hasManualSelection
      ? projectData.strings.filter((s) => selectedStrings.has(s.name))
      : projectData.strings.filter((s) =>
          selectedLanguages.some((lang) => s.translationStatus[lang] !== 'translated')
        );

    const stringsByLanguage = new Map<string, AndroidString[]>();
    for (const language of selectedLanguages) {
      const stringsForLanguage = stringsToTranslate
        .filter((s) =>
          hasManualSelection ? true : s.translationStatus[language] !== 'translated'
        )
        .map((s) => ({
          name: s.name,
          text: s.defaultValue,
          is_translatable: s.isTranslatable,
          is_commented_out: false,
          line_number: 0,
          has_cdata: s.hasCdata ?? false
        }));
      if (stringsForLanguage.length > 0) stringsByLanguage.set(language, stringsForLanguage);
    }

    try {
      await translationService.translateMultipleLanguages(stringsByLanguage, $currentProject, { source: 'translate' });
      await projectDataStore.loadProjectData($currentProject.path);
    } catch (err) {
      console.error('Translation failed:', err);
    }
  }

  function handleKeyDown(event: KeyboardEvent) {
    const target = event.target as HTMLElement;
    if (target.tagName === 'INPUT' || target.tagName === 'TEXTAREA' || target.isContentEditable) return;
    if (event.ctrlKey || event.metaKey || event.altKey) return;
    if (event.key.toLowerCase() === 't') {
      event.preventDefault();
      if (
        !hasDuplicates &&
        missingCount() > 0 &&
        translationSelection.selectedLanguages.size > 0 &&
        !isTranslatingOrPreparing
      ) {
        handleTranslateAll();
      }
    }
  }

  onMount(() => window.addEventListener('keydown', handleKeyDown));
  onDestroy(() => window.removeEventListener('keydown', handleKeyDown));
</script>

{#if shouldShow()}
  <div class="translate-suggestion">
    <SuggestionCard
      icon={translateIcon}
      title="{missingCount()} translations missing"
      variant={missingCount() > 0 ? 'warning' : 'default'}
    >
      {#snippet action()}
        <Button
          onclick={(e) => handleTranslateAll(e as unknown as MouseEvent)}
          disabled={hasDuplicates ||
            missingCount() === 0 ||
            translationSelection.selectedLanguages.size === 0 ||
            isTranslatingOrPreparing}
        >
          {#if showTranslateSpinner}
            <Loader2 class="h-4 w-4 animate-spin" />
          {:else}
            Translate
            <Kbd.Root class="kbd-on-primary">T</Kbd.Root>
          {/if}
        </Button>
      {/snippet}
    </SuggestionCard>
  </div>
{/if}

<style>
  :global(.kbd-on-primary) {
    margin-left: 0.5rem;
    font-size: 0.7rem;
    padding: 0.125rem 0.375rem;
    background: color-mix(in oklch, var(--primary-foreground) 25%, transparent) !important;
    color: var(--primary-foreground) !important;
    border-color: color-mix(in oklch, var(--primary-foreground) 35%, transparent) !important;
  }

  .translate-suggestion :global(button:disabled) {
    background: var(--muted) !important;
    color: var(--muted-foreground) !important;
    border-color: var(--border) !important;
    box-shadow: none !important;
    opacity: 1 !important;
  }

  .translate-suggestion :global(button:disabled .kbd-on-primary) {
    background: color-mix(in oklch, var(--muted-foreground) 25%, transparent) !important;
    color: var(--muted-foreground) !important;
    border-color: color-mix(in oklch, var(--muted-foreground) 35%, transparent) !important;
  }
</style>
