<script lang="ts">
  import RefreshCwIcon from "@lucide/svelte/icons/refresh-cw";
  import { currentProject } from '$lib/stores/project';
  import { projectDataStore } from '$lib/stores/project-data';
  import { Button } from "$lib/components/ui/button";

  let isSpinning = $state(false);

  async function refreshProjectData(event: Event) {
    event.stopPropagation();
    if ($currentProject && !isSpinning) {
      isSpinning = true;
      await projectDataStore.loadProjectData($currentProject.path);
      // Reset spinning state after animation completes
      setTimeout(() => {
        isSpinning = false;
      }, 600); // Match the animation duration
    }
  }
</script>

<Button 
  variant="outline"
  size="icon"
  class="refresh-btn {isSpinning ? 'spinning' : ''} border border-border h-10 w-10"
  onclick={refreshProjectData}
  disabled={!$currentProject || isSpinning}
  title="Refresh project data"
>
  <RefreshCwIcon class="h-5 w-5" />
</Button>

<style>
  :global(.refresh-btn.spinning svg) {
    animation: spin 0.6s ease-in-out;
  }

  @keyframes spin {
    from {
      transform: rotate(0deg);
    }
    to {
      transform: rotate(360deg);
    }
  }
</style>
