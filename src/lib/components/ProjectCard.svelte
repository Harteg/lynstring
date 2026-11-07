<script lang="ts">
  import { projectDataStore, duplicateKeysStore } from '$lib/stores/project-data';
  import { stringChangesStore } from '$lib/stores/string-changes';
  import { currentProject } from '$lib/stores/project';
  import ProjectStatusPanel from './panels/ProjectStatusPanel.svelte';
  import TranslateSuggestionCard from './panels/suggestions/TranslateSuggestionCard.svelte';
  import UpdateSuggestionCard from './panels/suggestions/UpdateSuggestionCard.svelte';
  import DuplicateStringsSuggestionCard from './panels/suggestions/DuplicateStringsSuggestionCard.svelte';
  import UnusedStringsSuggestionCard from './panels/suggestions/UnusedStringsSuggestionCard.svelte';
  import ProjectWarning from './ProjectWarning.svelte';

  const projectData = $derived($projectDataStore);
  const duplicateKeys = $derived($duplicateKeysStore);
  const currentChanges = $derived.by(() =>
    $currentProject ? $stringChangesStore.find((p) => p.projectPath === $currentProject.path) : null
  );
  const hasDuplicates = $derived(duplicateKeys.length > 0);

  function showAlert(title: string, message: string) {
    console.log(`${title}: ${message}`);
  }
</script>

<div class="project-card">
  <div class="panels-container">
    {#if projectData.error && projectData.errorType}
      <ProjectWarning
        errorType={projectData.errorType}
        errorMessage={projectData.error}
      />
    {/if}

    <div class="pt-0"></div>


    <ProjectStatusPanel />

    <div class="suggestions-section">
      <DuplicateStringsSuggestionCard />
      <TranslateSuggestionCard {showAlert} />
      <UpdateSuggestionCard />
      <UnusedStringsSuggestionCard />
    </div>

  </div>
</div>

<style>
  .project-card {
    display: flex;
    flex-direction: column;
    width: 100%;
    height: 100%;
    align-items: center;
    justify-content: center;
    overflow-y: scroll;
    box-sizing: border-box;
    scrollbar-gutter: stable;
    padding-bottom: 32px;
    padding-left: 16px;
    padding-right: 16px;
  }

  .panels-container {
    display: flex;
    flex-direction: column;
    width: 100%;
    max-width: 600px;
    gap: 1rem;
    padding-top: 8px;
    padding-right: 13px;
    padding-left: 22px;
  }

  .suggestions-section {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  ::-webkit-scrollbar {
    width: 8px;
    position: absolute;
  }

  ::-webkit-scrollbar-track {
    background: var(--background);
  }

  ::-webkit-scrollbar-thumb {
    background: var(--muted);
    border-radius: 4px;
  }

</style>