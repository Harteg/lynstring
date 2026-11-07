<script lang="ts">
  import { Button } from '$lib/components/ui/button';
  import { showNewProjectDialog } from '$lib/stores/dialog';
  import FolderPlus from '@lucide/svelte/icons/folder-plus';
</script>

<div class="prompt-section">
  <div class="prompt-wrapper">
    <div class="prompt-icon">
      <FolderPlus />
    </div>
    <h1 class="prompt-title">
      Add a project to get started
    </h1>
    <p class="prompt-text">
      Select your Android project folder to start translating strings.xml files.
    </p>
    <Button
      size="lg"
      onclick={() => showNewProjectDialog.set(true)}
    >
      <strong>Add Project</strong>
    </Button>
  </div>
</div>

<style>
  .prompt-section {
    display: flex;
    align-items: center;
    justify-content: center;
    min-height: 100%;
    width: 100%;
    padding: 4rem 1.5rem;
    background: var(--background);
    overflow-y: auto;
  }

  .prompt-wrapper {
    width: 100%;
    max-width: 400px;
    display: flex;
    flex-direction: column;
    align-items: center;
    text-align: center;
    gap: 1.5rem;
  }

  .prompt-icon {
    color: var(--primary);
    opacity: 0.9;
  }

  .prompt-icon :global(svg) {
    width: 3.5rem;
    height: 3.5rem;
  }

  .prompt-title {
    font-size: 1.5rem;
    font-weight: 600;
    color: var(--foreground);
    margin: 0;
  }

  .prompt-text {
    font-size: 1rem;
    line-height: 1.5;
    color: var(--muted-foreground);
    margin: 0;
  }

  @media (max-width: 640px) {
    .prompt-section {
      padding: 2rem 1rem;
    }

    .prompt-title {
      font-size: 1.25rem;
    }
  }
</style>
