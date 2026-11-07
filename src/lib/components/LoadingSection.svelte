<script lang="ts">
  import { projectDataStore } from '$lib/stores/project-data';

  export let title = "Loading project...";
</script>

<div class="loading-section">
  <p>{title}</p>
  <p class="loading-detail">
    {#if $projectDataStore.isLoading}
      Loading project data...
    {:else}
      Initializing project...
    {/if}
  </p>
</div>

<style>
  .loading-section {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 2rem;
    text-align: center;
  }

  .loading-section p {
    margin: 0.5rem 0;
    color: var(--muted-foreground);
  }

  .loading-section p:first-child {
    font-size: 1.125rem;
    font-weight: 500;
    color: var(--foreground);
  }

  .loading-detail {
    font-size: 0.875rem;
    opacity: 0.8;
  }
</style>
