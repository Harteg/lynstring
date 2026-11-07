<script lang="ts">
  import { currentProject } from '$lib/stores/project';
  import { projectDataStore, duplicateKeysStore } from '$lib/stores/project-data';
  import { stringChangesStore } from '$lib/stores/string-changes';
  import { requestUnusedStringsScan } from '$lib/stores/unused-strings-scan';
  import {
    unusedStringsSuggestionSeenStore,
    markSuggestionSeenForProject
  } from '$lib/stores/unused-strings-suggestion';
  import SearchIcon from '@lucide/svelte/icons/search';
  import { Button } from '$lib/components/ui/button';
  import SuggestionCard from './SuggestionCard.svelte';

  const projectData = $derived($projectDataStore);
  const duplicateKeys = $derived($duplicateKeysStore);
  const currentChanges = $derived.by(() =>
    $currentProject ? $stringChangesStore.find((p) => p.projectPath === $currentProject.path) : null
  );
  const totalMissingTranslations = $derived.by(() => {
    const { strings, languages } = projectData;
    if (!strings?.length || !languages?.length) return 0;
    let missing = 0;
    for (const str of strings) {
      for (const lang of languages) {
        if (str.translationStatus?.[lang] === 'missing') missing++;
      }
    }
    return missing;
  });
  const totalStrings = $derived(projectData.strings.length);
  const totalLanguages = $derived(projectData.languages.length);
  const hasModified = $derived((currentChanges?.changes.modified.length ?? 0) > 0);
  const hasDuplicates = $derived(duplicateKeys.length > 0);
  const isComplete = $derived(
    totalMissingTranslations === 0 &&
      !hasModified &&
      !hasDuplicates &&
      totalStrings > 0 &&
      totalLanguages > 0
  );

  /** At least one non-default language has all strings missing (new language added) */
  const hasLanguageWithAllMissing = $derived.by(() => {
    const { strings, languages } = projectData;
    if (!strings?.length || !languages?.length) return false;
    return languages.some(
      (lang) =>
        lang !== 'default' &&
        strings.every((s) => s.translationStatus?.[lang] === 'missing')
    );
  });

  const hasSeenForCurrentProject = $derived(
    $currentProject ? $unusedStringsSuggestionSeenStore.has($currentProject.path) : false
  );

  const shouldShow = $derived(
    $currentProject &&
      !isComplete &&
      hasLanguageWithAllMissing &&
      !hasSeenForCurrentProject
  );

  function handleScan() {
    requestUnusedStringsScan();
    if ($currentProject) {
      markSuggestionSeenForProject($currentProject.path);
    }
  }

  async function handleNotNow() {
    if ($currentProject) {
      await markSuggestionSeenForProject($currentProject.path);
    }
  }
</script>

{#if shouldShow}
  <SuggestionCard
    icon={SearchIcon}
    title="Scan for unused strings"
  >
    {#snippet action()}
      <div class="action-row">
        <Button variant="ghost" size="sm" onclick={handleNotNow}>
          Not now
        </Button>
        <Button size="sm" onclick={handleScan}>
          Scan
        </Button>
      </div>
    {/snippet}
  </SuggestionCard>
{/if}

<style>
  .action-row {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }
</style>
