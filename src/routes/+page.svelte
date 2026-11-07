<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount, onDestroy } from "svelte";
  import NewProjectDialog from "@/components/dialogs/NewProjectDialog.svelte";
  import ProjectCard from "$lib/components/ProjectCard.svelte";
  import AppFooter from "$lib/components/AppFooter.svelte";
  import LoadingSection from "$lib/components/LoadingSection.svelte";
  import FirstProjectPrompt from "$lib/components/FirstProjectPrompt.svelte";
  import ApiKeySetup from "$lib/components/ApiKeySetup.svelte";
  import type { Project } from "../../src-tauri/bindings/Project";

  import { 
    currentProject, 
    editingProject,
    isLoadingProject,
    projectsList
  } from '$lib/stores/project';

  import { projectDataStore } from '$lib/stores/project-data';
  import { showNewProjectDialog } from '$lib/stores/dialog';
  import { initializeDevPreferencesStore } from '$lib/stores/dev-preferences';
  import { initializeTranslationHistory } from '$lib/stores/translation-history';
  import { initializeUnusedStringsSuggestionStore } from '$lib/stores/unused-strings-suggestion';
  import { appSettingsStore } from '$lib/stores/app-settings';
  import { get } from 'svelte/store';

  onMount(async () => {
    await initializeDevPreferencesStore();
    await initializeTranslationHistory();
    await initializeUnusedStringsSuggestionStore();
    await loadCurrentProject();
  });
  
  async function loadProjects() {
    try {
      const list = await invoke<Project[]>("load_projects");
      projectsList.set(list);
    } catch (err) {
      console.error("Failed to load projects:", err);
    }
  }
  
  async function loadCurrentProject() {
    try {
      isLoadingProject.set(true);
      const project = await invoke<Project | null>("get_current_project");
      
      // Load projects list first
      await loadProjects();
      
      // If no project is selected, automatically select the first available project
      if (!project) {
        const list = get(projectsList);
        if (list.length > 0) {
          const firstProject = list[0];
          await invoke("set_current_project", { projectId: firstProject.id });
          currentProject.set(firstProject);
        } else {
          currentProject.set(null);
        }
      } else {
        currentProject.set(project);
      }
      
      // Note: projectDataStore.loadProjectData() is called by the reactive statement below
    } catch (err) {
      console.error("Failed to load current project:", err);
    } finally {
      isLoadingProject.set(false);
    }
  }
  
  function openEditProjectDialog(event: CustomEvent<Project>) {
    editingProject.set(event.detail);
    showNewProjectDialog.set(true);
  }
  
  // Watch for changes to currentProject and load/reset data accordingly
  $effect(() => {
    if ($currentProject) {
      projectDataStore.loadProjectData($currentProject.path);
    } else {
      projectDataStore.reset().catch(err => {
        console.error("Failed to reset project data:", err);
      });
      editingProject.set(null);  // Clear editing state when no project is selected
    }
  });
  
  // Cleanup file watcher on unmount
  onDestroy(async () => {
    await projectDataStore.reset();
  });

  // Reload projects list when currentProject changes (reacts to project creation, deletion, etc.)
  // This follows the same pattern as ProjectSelectorPanel - reactive to store changes
  let hasInitialized = $state(false);
  $effect(() => {
    const project = $currentProject;
    
    // Skip on initial mount, then reload on any subsequent change
    if (hasInitialized) {
      loadProjects();
    } else {
      hasInitialized = true;
    }
  });

  onMount(() => {
    const allowSelector = "input, textarea, [contenteditable], .allow-context";

    const handler = (e: MouseEvent) => {
      const el = e.target as HTMLElement | null;
      if (!el) return;

      // allow context menu on text fields and anything with .allow-context
      if (el.closest(allowSelector)) return;

      e.preventDefault();
    };

    window.addEventListener("contextmenu", handler, { capture: true });

    onDestroy(() => {
      window.removeEventListener("contextmenu", handler, { capture: true });
    });
  });
</script>

<main class="app-container">
  <div class="main-content">
  
    {#if !($appSettingsStore?.has_api_key ?? false)}
      <ApiKeySetup />
    {:else if $isLoadingProject}
      <LoadingSection title="Loading project..." />
    {:else if $projectsList.length === 0}
      <FirstProjectPrompt />
    {:else}
      <ProjectCard />
    {/if}

  </div>

  {#if $appSettingsStore?.has_api_key}
    <AppFooter />
  {/if}

  <NewProjectDialog 
    bind:isOpen={$showNewProjectDialog}
  />
</main>

<style>
  :global(html, body) {
    height: 100%;
    margin: 0;
    padding: 0;
    overflow: hidden;
    -webkit-user-select: none;
    -moz-user-select: none;
    -ms-user-select: none;
    user-select: none;
    overscroll-behavior: none;
    -webkit-overflow-scrolling: auto;
    cursor: default;
  }

  /* Allow text selection in text fields and contenteditable elements */
  :global(input, textarea, [contenteditable], .selectable) {
    user-select: text !important;
  }

  :global(button){
    cursor: pointer;
  }

  :global(#svelte) {
    height: 100%;
  }

  :root {
    font-family: Inter, Avenir, Helvetica, Arial, sans-serif;
    font-size: 16px;
    line-height: 24px;
    font-weight: 400;
    font-synthesis: none;
    text-rendering: optimizeLegibility;
    -webkit-font-smoothing: antialiased;
    -moz-osx-font-smoothing: grayscale;
    -webkit-text-size-adjust: 100%;
  }

  .app-container {
    display: flex;
    flex-direction: column;
    height: 100vh;
  }

  .main-content {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    padding-top: var(--app-top-offset);
  }

</style>
