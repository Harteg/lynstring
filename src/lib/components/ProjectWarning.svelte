<script lang="ts">
  import AlertCircle from "@lucide/svelte/icons/alert-circle";
  import FolderOpen from "@lucide/svelte/icons/folder-open";
  import { currentProject } from '$lib/stores/project';
  import { editingProject } from '$lib/stores/project';
  import { showNewProjectDialog } from '$lib/stores/dialog';

  export let errorType: 'strings_not_found' | 'general' | null;
  export let errorMessage: string;

  function editProject() {
    if ($currentProject) {
      editingProject.set($currentProject);
      showNewProjectDialog.set(true);
    }
  }
</script>

{#if errorType === 'strings_not_found'}
  <div class="bg-destructive/10 border border-destructive/20 rounded-lg p-4">
    <div class="flex items-start gap-3 mb-4">
      <div class="flex-shrink-0 mt-0.5">
        <AlertCircle class="w-5 h-5 text-destructive" />
      </div>
      <div class="flex-1">
        <h3 class="font-semibold text-destructive text-sm mb-1">Project Path Issue</h3>
        <p class="text-destructive/80 text-sm mb-2">
          This doesn't look like an Android project. Failed to find strings.xml file.
        </p>
        <p class="text-destructive/70 text-xs">
          The project folder may have been moved or the path is incorrect. Please update the project path.
        </p>
      </div>
    </div>
    <button 
      class="flex items-center gap-2 px-3 py-2 bg-destructive/20 text-destructive border border-destructive/30 rounded-md text-sm font-medium hover:bg-destructive/30 transition-colors ml-8"
      onclick={editProject}
    >
      <FolderOpen class="w-4 h-4" />
      Update Project Path
    </button>
  </div>
{:else if errorType === 'general'}
  <div class="bg-destructive/10 border border-destructive/20 rounded-lg p-4">
    <div class="flex items-start gap-3 mb-4">
      <div class="flex-shrink-0 mt-0.5">
        <AlertCircle class="w-5 h-5 text-destructive" />
      </div>
      <div class="flex-1">
        <h3 class="font-semibold text-destructive text-sm mb-1">Project Loading Error</h3>
        <p class="text-destructive/80 text-sm mb-2">
          {errorMessage}
        </p>
      </div>
    </div>
    <button 
      class="flex items-center gap-2 px-3 py-2 bg-destructive/20 text-destructive border border-destructive/30 rounded-md text-sm font-medium hover:bg-destructive/30 transition-colors ml-8"
      onclick={editProject}
    >
      <FolderOpen class="w-4 h-4" />
      Edit Project
    </button>
  </div>
{/if}
