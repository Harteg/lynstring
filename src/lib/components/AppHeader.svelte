<script lang="ts">
  import SettingsDropDown from './SettingsDropDown.svelte';
  import ProjectSelector from './panels/ProjectSelectorPanel.svelte';
  import LynStringLogo from './LynStringLogo.svelte';
  import { projectsList } from '$lib/stores/project';
  import { appSettingsStore } from '$lib/stores/app-settings';
</script>

<header class="app-header" data-tauri-drag-region>
  <div class="header-left">
    <LynStringLogo size="sm" />
  </div>

  <div class="header-center">
    {#if $appSettingsStore?.has_api_key && $projectsList.length > 0}
      <ProjectSelector />
    {/if}
  </div>

  <div class="header-actions">
    <SettingsDropDown />
  </div>
</header>

<style>
  .app-header {
    width: 100%;
    height: var(--app-header-height);
    display: grid;
    grid-template-columns: 1fr auto 1fr;
    align-items: center;
    padding: 0 2rem;
    gap: 1rem;
    border-bottom: 1px solid var(--border);
    background: var(--appheader-background);
  }
  
  .header-left {
    display: flex;
    align-items: center;
    gap: 1rem;
    justify-self: start;
    /* Space for macOS traffic lights when using titleBarStyle: Overlay */
    padding-left: env(safe-area-inset-left, 0px);
  }

  .header-center {
    justify-self: center;
    min-width: 0;
  }

  .header-actions {
    display: flex;
    gap: 0.5rem;
    align-items: center;
    justify-self: end;
  }
</style>
