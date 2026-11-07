<script lang="ts">
  import { currentProject } from '$lib/stores/project';
  import ProjectSelector from './panels/ProjectSelectorPanel.svelte';
  import type { Project } from "../../../src-tauri/bindings/Project";

  function getProjectDisplayName(project: Project | null): string {
    return project?.name || project?.path?.split(/[/\\]/).pop() || "Unnamed Project";
  }
</script>

<div class="project-header">
  
  <div class="project-selector-container">
    <ProjectSelector />
  </div>
</div>

<style>
  .project-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    background: #f8f9fa;
    border-radius: 8px;
    padding: 1rem;
    border: 1px solid #e9ecef;
  }
  
  .project-selector-container {
    min-width: 200px;
  }

  /* Override ProjectSelector styles for this context */
  .project-selector-container :global(.project-selector) {
    min-width: unset;
  }

  .project-selector-container :global(.current-project-btn) {
    background: white;
    border: 1px solid #dee2e6;
    color: #495057;
    font-size: 0.875rem;
    padding: 0.5rem 0.75rem;
  }

  .project-selector-container :global(.current-project-btn:hover) {
    background: #f8f9fa;
    border-color: #adb5bd;
  }
</style>
